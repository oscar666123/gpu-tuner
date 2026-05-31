use libloading::Library;
use std::ffi::{c_char, c_int, c_void, CStr};
use std::mem;

type NvapiStatus = c_int;
type NvPhysicalGpuHandle = *mut c_void;
type QueryInterface = unsafe extern "C" fn(u32) -> *const c_void;

const NVAPI_OK: NvapiStatus = 0;
const NVAPI_INITIALIZE_ID: u32 = 0x0150E828;
const NVAPI_SYS_GET_DRIVER_AND_BRANCH_VERSION_ID: u32 = 0x2926AAAD;
const NVAPI_GET_ERROR_MESSAGE_ID: u32 = 0x6C2D048C;
const NVAPI_ENUM_PHYSICAL_GPUS_ID: u32 = 0xE5AC921F;
const NVAPI_GPU_GET_PSTATES20_ID: u32 = 0x6FF81213;
const NVAPI_GPU_SET_PSTATES20_ID: u32 = 0x0F4DAE6B;
const NVAPI_MAX_PHYSICAL_GPUS: usize = 64;
const NVAPI_MAX_GPU_PSTATE20_PSTATES: usize = 16;
const NVAPI_MAX_GPU_PSTATE20_CLOCKS: usize = 8;
const NVAPI_MAX_GPU_PSTATE20_BASE_VOLTAGES: usize = 4;
const NVAPI_GPU_PERF_PSTATE_P0: u32 = 0;
const NVAPI_GPU_PUBLIC_CLOCK_GRAPHICS: u32 = 0;
const NVAPI_GPU_PUBLIC_CLOCK_MEMORY: u32 = 4;

#[derive(Debug, Clone)]
pub struct OffsetCapabilities {
    pub core_supported: bool,
    pub memory_supported: bool,
    pub core_min_mhz: i32,
    pub core_max_mhz: i32,
    pub memory_min_mhz: i32,
    pub memory_max_mhz: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct NvDelta {
    value: i32,
    min: i32,
    max: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct NvClockRange {
    min_freq_khz: u32,
    max_freq_khz: u32,
    domain_id: u32,
    min_voltage_uv: u32,
    max_voltage_uv: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct NvClockEntry {
    domain_id: u32,
    type_id: u32,
    is_editable: u32,
    freq_delta_khz: NvDelta,
    data: NvClockRange,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct NvVoltageEntry {
    domain_id: u32,
    is_editable: u32,
    volt_uv: u32,
    volt_delta_uv: NvDelta,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct NvPstate {
    pstate_id: u32,
    is_editable: u32,
    clocks: [NvClockEntry; NVAPI_MAX_GPU_PSTATE20_CLOCKS],
    base_voltages: [NvVoltageEntry; NVAPI_MAX_GPU_PSTATE20_BASE_VOLTAGES],
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct NvPstates20Info {
    version: u32,
    is_editable: u32,
    num_pstates: u32,
    num_clocks: u32,
    num_base_voltages: u32,
    pstates: [NvPstate; NVAPI_MAX_GPU_PSTATE20_PSTATES],
    num_voltages: u32,
    voltages: [NvVoltageEntry; NVAPI_MAX_GPU_PSTATE20_BASE_VOLTAGES],
}

pub struct Nvapi {
    lib: Library,
    query: QueryInterface,
}

impl Nvapi {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let lib = Library::new("nvapi64.dll").map_err(|error| {
                format!("nvapi64.dll was not found or could not be loaded: {error}")
            })?;
            let query = {
                let query_symbol: libloading::Symbol<QueryInterface> = lib
                    .get(b"nvapi_QueryInterface\0")
                    .map_err(|error| format!("NVAPI query interface was not found: {error}"))?;
                *query_symbol
            };
            let initialize_ptr = query(NVAPI_INITIALIZE_ID);
            if initialize_ptr.is_null() {
                return Err("NVAPI initialize function was not found.".to_string());
            }
            let initialize: unsafe extern "C" fn() -> NvapiStatus =
                std::mem::transmute(initialize_ptr);
            let status = initialize();
            if status != NVAPI_OK {
                return Err(format!("NVAPI initialization failed with status {status}"));
            }
            Ok(Self { lib, query })
        }
    }

    pub fn driver_version(&self) -> Result<String, String> {
        unsafe {
            let ptr = (self.query)(NVAPI_SYS_GET_DRIVER_AND_BRANCH_VERSION_ID);
            if ptr.is_null() {
                return Err("NVAPI driver version function was not found.".to_string());
            }
            let func: unsafe extern "C" fn(*mut i32, *mut c_char) -> NvapiStatus =
                std::mem::transmute(ptr);
            let mut version = 0_i32;
            let mut branch = [0 as c_char; 64];
            let status = func(&mut version, branch.as_mut_ptr());
            if status != NVAPI_OK {
                return Err(format!(
                    "NVAPI driver version query failed with status {status}"
                ));
            }
            let branch_name = CStr::from_ptr(branch.as_ptr())
                .to_string_lossy()
                .into_owned();
            Ok(format!("{version} {branch_name}"))
        }
    }

    pub fn offset_capabilities(&self, gpu_index: u32) -> Result<OffsetCapabilities, String> {
        let mut info = self.pstates20(gpu_index)?;
        let pstate = p0_mut(&mut info)
            .ok_or_else(|| "P0 performance state was not reported by NVAPI.".to_string())?;
        let core = clock_entry(pstate, NVAPI_GPU_PUBLIC_CLOCK_GRAPHICS);
        let memory = clock_entry(pstate, NVAPI_GPU_PUBLIC_CLOCK_MEMORY);

        Ok(OffsetCapabilities {
            core_supported: core.map(is_editable).unwrap_or(false),
            memory_supported: memory.map(is_editable).unwrap_or(false),
            core_min_mhz: core
                .map(|entry| khz_to_mhz(entry.freq_delta_khz.min))
                .unwrap_or(0),
            core_max_mhz: core
                .map(|entry| khz_to_mhz(entry.freq_delta_khz.max))
                .unwrap_or(0),
            memory_min_mhz: memory
                .map(|entry| khz_to_mhz(entry.freq_delta_khz.min))
                .unwrap_or(0),
            memory_max_mhz: memory
                .map(|entry| khz_to_mhz(entry.freq_delta_khz.max))
                .unwrap_or(0),
        })
    }

    pub fn set_core_clock_offset(&self, gpu_index: u32, offset_mhz: i32) -> Result<(), String> {
        self.set_clock_offset(
            gpu_index,
            NVAPI_GPU_PUBLIC_CLOCK_GRAPHICS,
            offset_mhz,
            "core",
        )
    }

    pub fn set_memory_clock_offset(&self, gpu_index: u32, offset_mhz: i32) -> Result<(), String> {
        self.set_clock_offset(
            gpu_index,
            NVAPI_GPU_PUBLIC_CLOCK_MEMORY,
            offset_mhz,
            "memory",
        )
    }

    pub fn reset_clock_offsets(&self, gpu_index: u32) -> Result<(), String> {
        let mut info = self.pstates20(gpu_index)?;
        let pstate = p0_mut(&mut info)
            .ok_or_else(|| "P0 performance state was not reported by NVAPI.".to_string())?;
        let mut changed = false;
        for domain in [
            NVAPI_GPU_PUBLIC_CLOCK_GRAPHICS,
            NVAPI_GPU_PUBLIC_CLOCK_MEMORY,
        ] {
            if let Some(entry) = clock_entry_mut(pstate, domain) {
                if is_editable(entry) {
                    entry.freq_delta_khz.value = 0;
                    changed = true;
                }
            }
        }
        if changed {
            self.set_pstates20(gpu_index, &info)
        } else {
            Err("No editable P0 core or memory offset entry was reported by NVAPI.".to_string())
        }
    }

    fn set_clock_offset(
        &self,
        gpu_index: u32,
        domain: u32,
        offset_mhz: i32,
        label: &str,
    ) -> Result<(), String> {
        let mut info = self.pstates20(gpu_index)?;
        let pstate = p0_mut(&mut info)
            .ok_or_else(|| "P0 performance state was not reported by NVAPI.".to_string())?;
        let entry = clock_entry_mut(pstate, domain)
            .ok_or_else(|| format!("P0 {label} clock entry was not reported by NVAPI."))?;
        if !is_editable(entry) {
            return Err(format!(
                "P0 {label} clock offset is not editable on this GPU or driver."
            ));
        }
        let offset_khz = mhz_to_khz(offset_mhz);
        if offset_khz < entry.freq_delta_khz.min || offset_khz > entry.freq_delta_khz.max {
            return Err(format!(
                "{label} clock offset {offset_mhz} MHz is outside {}..{} MHz",
                khz_to_mhz(entry.freq_delta_khz.min),
                khz_to_mhz(entry.freq_delta_khz.max)
            ));
        }
        entry.freq_delta_khz.value = offset_khz;
        self.set_pstates20(gpu_index, &info)
    }

    fn pstates20(&self, gpu_index: u32) -> Result<NvPstates20Info, String> {
        let handle = self.physical_gpu(gpu_index)?;
        unsafe {
            let ptr = (self.query)(NVAPI_GPU_GET_PSTATES20_ID);
            if ptr.is_null() {
                return Err("NvAPI_GPU_GetPstates20 was not found.".to_string());
            }
            let func: unsafe extern "C" fn(
                NvPhysicalGpuHandle,
                *mut NvPstates20Info,
            ) -> NvapiStatus = mem::transmute(ptr);
            let mut info = NvPstates20Info {
                version: make_nvapi_version::<NvPstates20Info>(3),
                ..Default::default()
            };
            let status = func(handle, &mut info);
            self.check(status, "NvAPI_GPU_GetPstates20")?;
            Ok(info)
        }
    }

    fn set_pstates20(&self, gpu_index: u32, info: &NvPstates20Info) -> Result<(), String> {
        let handle = self.physical_gpu(gpu_index)?;
        unsafe {
            let ptr = (self.query)(NVAPI_GPU_SET_PSTATES20_ID);
            if ptr.is_null() {
                return Err("NvAPI_GPU_SetPstates20 was not found in this driver.".to_string());
            }
            let func: unsafe extern "C" fn(
                NvPhysicalGpuHandle,
                *const NvPstates20Info,
            ) -> NvapiStatus = mem::transmute(ptr);
            self.check(func(handle, info), "NvAPI_GPU_SetPstates20")
        }
    }

    fn physical_gpu(&self, index: u32) -> Result<NvPhysicalGpuHandle, String> {
        unsafe {
            let ptr = (self.query)(NVAPI_ENUM_PHYSICAL_GPUS_ID);
            if ptr.is_null() {
                return Err("NvAPI_EnumPhysicalGPUs was not found.".to_string());
            }
            let func: unsafe extern "C" fn(*mut NvPhysicalGpuHandle, *mut i32) -> NvapiStatus =
                mem::transmute(ptr);
            let mut handles = [std::ptr::null_mut(); NVAPI_MAX_PHYSICAL_GPUS];
            let mut count = 0_i32;
            self.check(
                func(handles.as_mut_ptr(), &mut count),
                "NvAPI_EnumPhysicalGPUs",
            )?;
            if index >= count as u32 {
                return Err(format!(
                    "NVAPI physical GPU index {index} is outside 0..{}.",
                    count.saturating_sub(1)
                ));
            }
            Ok(handles[index as usize])
        }
    }

    fn check(&self, status: NvapiStatus, action: &str) -> Result<(), String> {
        if status == NVAPI_OK {
            Ok(())
        } else {
            Err(format!("{action} failed: {}", self.error_message(status)))
        }
    }

    fn error_message(&self, status: NvapiStatus) -> String {
        unsafe {
            let ptr = (self.query)(NVAPI_GET_ERROR_MESSAGE_ID);
            if ptr.is_null() {
                return format!("NVAPI status {status}");
            }
            let func: unsafe extern "C" fn(NvapiStatus, *mut c_char) -> NvapiStatus =
                mem::transmute(ptr);
            let mut message = [0 as c_char; 64];
            if func(status, message.as_mut_ptr()) == NVAPI_OK {
                CStr::from_ptr(message.as_ptr())
                    .to_string_lossy()
                    .into_owned()
            } else {
                format!("NVAPI status {status}")
            }
        }
    }
}

impl Drop for Nvapi {
    fn drop(&mut self) {
        let _ = &self.lib;
    }
}

fn p0_mut(info: &mut NvPstates20Info) -> Option<&mut NvPstate> {
    let count = (info.num_pstates as usize).min(NVAPI_MAX_GPU_PSTATE20_PSTATES);
    info.pstates[..count]
        .iter_mut()
        .find(|pstate| pstate.pstate_id == NVAPI_GPU_PERF_PSTATE_P0)
}

fn clock_entry(pstate: &NvPstate, domain: u32) -> Option<&NvClockEntry> {
    pstate.clocks.iter().find(|entry| entry.domain_id == domain)
}

fn clock_entry_mut(pstate: &mut NvPstate, domain: u32) -> Option<&mut NvClockEntry> {
    pstate
        .clocks
        .iter_mut()
        .find(|entry| entry.domain_id == domain)
}

fn is_editable(entry: &NvClockEntry) -> bool {
    entry.is_editable & 1 == 1
}

fn make_nvapi_version<T>(version: u32) -> u32 {
    mem::size_of::<T>() as u32 | (version << 16)
}

fn khz_to_mhz(value: i32) -> i32 {
    value / 1000
}

fn mhz_to_khz(value: i32) -> i32 {
    value.saturating_mul(1000)
}
