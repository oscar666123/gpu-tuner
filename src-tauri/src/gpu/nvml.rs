use super::models::{GPUInfo, GpuTelemetry};
use chrono::Utc;
use libloading::Library;
use std::ffi::{c_char, c_int, c_uint, c_ulonglong, c_void, CStr};
use std::mem::MaybeUninit;
use std::ptr;

const NVML_TEMPERATURE_GPU: c_uint = 0;
const NVML_CLOCK_GRAPHICS: c_uint = 0;
const NVML_CLOCK_MEM: c_uint = 2;
const NVML_CLOCK_OFFSET_VERSION_1: c_uint = 0x1000018;
const NVML_PSTATE_0: c_uint = 0;

type NvmlReturn = c_uint;
type NvmlDevice = *mut c_void;

#[repr(C)]
struct NvmlMemory {
    total: c_ulonglong,
    free: c_ulonglong,
    used: c_ulonglong,
}

#[repr(C)]
struct NvmlUtilization {
    gpu: c_uint,
    memory: c_uint,
}

#[derive(Debug, Clone)]
pub struct NvmlOffsetCapabilities {
    pub core_supported: bool,
    pub memory_supported: bool,
    pub core_current_mhz: i32,
    pub core_min_mhz: i32,
    pub core_max_mhz: i32,
    pub memory_current_mhz: i32,
    pub memory_min_mhz: i32,
    pub memory_max_mhz: i32,
    pub source: String,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct NvmlClockOffset {
    version: c_uint,
    clock_type: c_uint,
    pstate: c_uint,
    clock_offset_mhz: c_int,
    min_clock_offset_mhz: c_int,
    max_clock_offset_mhz: c_int,
}

pub struct Nvml {
    lib: Library,
}

impl Nvml {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let lib = Library::new("nvml.dll").map_err(|error| {
                format!("nvml.dll was not found or could not be loaded: {error}")
            })?;
            let code = {
                let init: libloading::Symbol<unsafe extern "C" fn() -> NvmlReturn> = lib
                    .get(b"nvmlInit_v2\0")
                    .or_else(|_| lib.get(b"nvmlInit\0"))
                    .map_err(|error| format!("NVML init entry point was not found: {error}"))?;
                init()
            };
            if code != 0 {
                return Err(format!(
                    "NVML initialization failed: {}",
                    error_string(&lib, code)
                ));
            }
            Ok(Self { lib })
        }
    }

    pub fn gpu_list(&self) -> Result<Vec<GPUInfo>, String> {
        let count = self.device_count()?;
        (0..count).map(|index| self.gpu_info(index)).collect()
    }

    pub fn gpu_info(&self, index: u32) -> Result<GPUInfo, String> {
        let device = self.device(index)?;
        let name = self.device_name(device)?;
        let driver_version = self
            .driver_version()
            .unwrap_or_else(|error| format!("Unknown ({error})"));
        let memory = self.memory_info(device)?;
        Ok(GPUInfo {
            index,
            name,
            driver_version,
            memory_total_mb: bytes_to_mb(memory.total),
        })
    }

    pub fn telemetry(&self, index: u32) -> Result<GpuTelemetry, String> {
        let device = self.device(index)?;
        let memory = self.memory_info(device)?;
        let utilization = self.utilization(device)?;
        Ok(GpuTelemetry {
            gpu_index: index,
            temperature_c: self.temperature(device)?,
            gpu_utilization_percent: utilization.gpu,
            memory_utilization_percent: utilization.memory,
            core_clock_mhz: self.clock_info(device, NVML_CLOCK_GRAPHICS)?,
            memory_clock_mhz: self.clock_info(device, NVML_CLOCK_MEM)?,
            power_draw_watts: self.power_usage(device).unwrap_or(0.0),
            power_limit_watts: self.power_limit(device).unwrap_or(0.0),
            memory_used_mb: bytes_to_mb(memory.used),
            memory_total_mb: bytes_to_mb(memory.total),
            timestamp: Utc::now(),
        })
    }

    pub fn power_limit_constraints(&self, index: u32) -> Result<(f64, f64), String> {
        let device = self.device(index)?;
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, *mut c_uint, *mut c_uint) -> NvmlReturn,
            > = self
                .lib
                .get(b"nvmlDeviceGetPowerManagementLimitConstraints\0")
                .map_err(|error| format!("Power limit constraints are unavailable: {error}"))?;
            let mut min_mw = 0;
            let mut max_mw = 0;
            self.check(
                func(device, &mut min_mw, &mut max_mw),
                "get power limit constraints",
            )?;
            Ok((mw_to_watts(min_mw), mw_to_watts(max_mw)))
        }
    }

    pub fn supported_memory_clocks(&self, index: u32) -> Result<Vec<u32>, String> {
        let device = self.device(index)?;
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, *mut c_uint, *mut c_uint) -> NvmlReturn,
            > = self
                .lib
                .get(b"nvmlDeviceGetSupportedMemoryClocks\0")
                .map_err(|error| format!("Supported memory clocks are unavailable: {error}"))?;
            let mut count = 0;
            let first = func(device, &mut count, ptr::null_mut());
            if first != 0 && count == 0 {
                return Err(format!(
                    "get supported memory clocks failed: {}",
                    error_string(&self.lib, first)
                ));
            }
            let mut clocks = vec![0; count as usize];
            self.check(
                func(device, &mut count, clocks.as_mut_ptr()),
                "get supported memory clocks",
            )?;
            clocks.truncate(count as usize);
            clocks.sort_unstable();
            clocks.dedup();
            Ok(clocks)
        }
    }

    pub fn supported_graphics_clocks(
        &self,
        index: u32,
        memory_clock_mhz: u32,
    ) -> Result<Vec<u32>, String> {
        let device = self.device(index)?;
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, c_uint, *mut c_uint, *mut c_uint) -> NvmlReturn,
            > = self
                .lib
                .get(b"nvmlDeviceGetSupportedGraphicsClocks\0")
                .map_err(|error| format!("Supported graphics clocks are unavailable: {error}"))?;
            let mut count = 0;
            let first = func(device, memory_clock_mhz, &mut count, ptr::null_mut());
            if first != 0 && count == 0 {
                return Err(format!(
                    "get supported graphics clocks failed: {}",
                    error_string(&self.lib, first)
                ));
            }
            let mut clocks = vec![0; count as usize];
            self.check(
                func(device, memory_clock_mhz, &mut count, clocks.as_mut_ptr()),
                "get supported graphics clocks",
            )?;
            clocks.truncate(count as usize);
            clocks.sort_unstable();
            clocks.dedup();
            Ok(clocks)
        }
    }

    pub fn locked_clock_ranges(&self, index: u32) -> Result<(Vec<u32>, Vec<u32>), String> {
        let memory_clocks = self.supported_memory_clocks(index)?;
        let mut graphics_clocks = Vec::new();
        for memory_clock in &memory_clocks {
            if let Ok(mut clocks) = self.supported_graphics_clocks(index, *memory_clock) {
                graphics_clocks.append(&mut clocks);
            }
        }
        graphics_clocks.sort_unstable();
        graphics_clocks.dedup();
        if memory_clocks.is_empty() || graphics_clocks.is_empty() {
            return Err("Locked clock range list is empty.".to_string());
        }
        Ok((memory_clocks, graphics_clocks))
    }

    pub fn set_gpu_locked_clocks(
        &self,
        index: u32,
        min_clock_mhz: u32,
        max_clock_mhz: u32,
    ) -> Result<(), String> {
        let device = self.device(index)?;
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, c_uint, c_uint) -> NvmlReturn,
            > = self
                .lib
                .get(b"nvmlDeviceSetGpuLockedClocks\0")
                .map_err(|error| format!("GPU locked clocks are unavailable: {error}"))?;
            self.check(
                func(device, min_clock_mhz, max_clock_mhz),
                "set GPU locked clocks",
            )
        }
    }

    pub fn reset_gpu_locked_clocks(&self, index: u32) -> Result<(), String> {
        let device = self.device(index)?;
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(NvmlDevice) -> NvmlReturn> = self
                .lib
                .get(b"nvmlDeviceResetGpuLockedClocks\0")
                .map_err(|error| format!("GPU locked clock reset is unavailable: {error}"))?;
            self.check(func(device), "reset GPU locked clocks")
        }
    }

    pub fn set_memory_locked_clocks(
        &self,
        index: u32,
        min_clock_mhz: u32,
        max_clock_mhz: u32,
    ) -> Result<(), String> {
        let device = self.device(index)?;
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, c_uint, c_uint) -> NvmlReturn,
            > = self
                .lib
                .get(b"nvmlDeviceSetMemoryLockedClocks\0")
                .map_err(|error| format!("Memory locked clocks are unavailable: {error}"))?;
            self.check(
                func(device, min_clock_mhz, max_clock_mhz),
                "set memory locked clocks",
            )
        }
    }

    pub fn reset_memory_locked_clocks(&self, index: u32) -> Result<(), String> {
        let device = self.device(index)?;
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(NvmlDevice) -> NvmlReturn> = self
                .lib
                .get(b"nvmlDeviceResetMemoryLockedClocks\0")
                .map_err(|error| format!("Memory locked clock reset is unavailable: {error}"))?;
            self.check(func(device), "reset memory locked clocks")
        }
    }

    pub fn offset_capabilities(&self, index: u32) -> Result<NvmlOffsetCapabilities, String> {
        match (
            self.clock_offset_info(index, NVML_CLOCK_GRAPHICS),
            self.clock_offset_info(index, NVML_CLOCK_MEM),
        ) {
            (Ok(core), Ok(memory)) => Ok(NvmlOffsetCapabilities {
                core_supported: true,
                memory_supported: true,
                core_current_mhz: core.clock_offset_mhz,
                core_min_mhz: core.min_clock_offset_mhz,
                core_max_mhz: core.max_clock_offset_mhz,
                memory_current_mhz: memory.clock_offset_mhz,
                memory_min_mhz: memory.min_clock_offset_mhz,
                memory_max_mhz: memory.max_clock_offset_mhz,
                source: "NVML clock offsets".to_string(),
            }),
            (core_result, memory_result) => {
                let legacy = self.legacy_offset_capabilities(index)?;
                let mut notes = Vec::new();
                if let Err(error) = core_result {
                    notes.push(format!("core: {error}"));
                }
                if let Err(error) = memory_result {
                    notes.push(format!("memory: {error}"));
                }
                Ok(NvmlOffsetCapabilities {
                    source: format!(
                        "legacy NVML VF offsets after nvmlDeviceGetClockOffsets failed ({})",
                        notes.join("; ")
                    ),
                    ..legacy
                })
            }
        }
    }

    pub fn set_core_clock_offset(&self, index: u32, offset_mhz: i32) -> Result<(), String> {
        self.set_clock_offset(index, NVML_CLOCK_GRAPHICS, offset_mhz)
            .or_else(|new_error| {
                self.set_legacy_gpc_offset(index, offset_mhz)
                    .map_err(|legacy_error| {
                        format!("{new_error}; legacy GPC offset failed: {legacy_error}")
                    })
            })
    }

    pub fn set_memory_clock_offset(&self, index: u32, offset_mhz: i32) -> Result<(), String> {
        self.set_clock_offset(index, NVML_CLOCK_MEM, offset_mhz)
            .or_else(|new_error| {
                self.set_legacy_mem_offset(index, offset_mhz)
                    .map_err(|legacy_error| {
                        format!("{new_error}; legacy memory offset failed: {legacy_error}")
                    })
            })
    }

    pub fn reset_clock_offsets(&self, index: u32) -> Result<(), String> {
        let core_result = self.set_core_clock_offset(index, 0);
        let memory_result = self.set_memory_clock_offset(index, 0);
        match (core_result, memory_result) {
            (Ok(_), Ok(_)) => Ok(()),
            (Err(core), Ok(_)) => Err(format!("Core offset reset failed: {core}")),
            (Ok(_), Err(memory)) => Err(format!("Memory offset reset failed: {memory}")),
            (Err(core), Err(memory)) => Err(format!(
                "Core offset reset failed: {core}; memory offset reset failed: {memory}"
            )),
        }
    }

    pub fn set_power_limit(&self, index: u32, watts: f64) -> Result<(), String> {
        let device = self.device(index)?;
        let milliwatts = (watts * 1000.0).round() as c_uint;
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(NvmlDevice, c_uint) -> NvmlReturn> =
                self.lib
                    .get(b"nvmlDeviceSetPowerManagementLimit\0")
                    .map_err(|error| format!("Power limit write is unavailable: {error}"))?;
            self.check(func(device, milliwatts), "set power limit")
        }
    }

    fn device_count(&self) -> Result<u32, String> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_uint) -> NvmlReturn> = self
                .lib
                .get(b"nvmlDeviceGetCount_v2\0")
                .or_else(|_| self.lib.get(b"nvmlDeviceGetCount\0"))
                .map_err(|error| format!("NVML device count entry point was not found: {error}"))?;
            let mut count = 0;
            self.check(func(&mut count), "get GPU count")?;
            Ok(count)
        }
    }

    fn device(&self, index: u32) -> Result<NvmlDevice, String> {
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(c_uint, *mut NvmlDevice) -> NvmlReturn,
            > = self
                .lib
                .get(b"nvmlDeviceGetHandleByIndex_v2\0")
                .or_else(|_| self.lib.get(b"nvmlDeviceGetHandleByIndex\0"))
                .map_err(|error| {
                    format!("NVML device handle entry point was not found: {error}")
                })?;
            let mut device: NvmlDevice = ptr::null_mut();
            self.check(func(index, &mut device), "get GPU handle")?;
            Ok(device)
        }
    }

    fn driver_version(&self) -> Result<String, String> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*mut c_char, c_uint) -> NvmlReturn> =
                self.lib
                    .get(b"nvmlSystemGetDriverVersion\0")
                    .map_err(|error| format!("Driver version query is unavailable: {error}"))?;
            let mut buffer = [0 as c_char; 96];
            self.check(
                func(buffer.as_mut_ptr(), buffer.len() as c_uint),
                "get driver version",
            )?;
            Ok(c_string(buffer.as_ptr()))
        }
    }

    fn device_name(&self, device: NvmlDevice) -> Result<String, String> {
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, *mut c_char, c_uint) -> NvmlReturn,
            > = self
                .lib
                .get(b"nvmlDeviceGetName\0")
                .map_err(|error| format!("GPU name query is unavailable: {error}"))?;
            let mut buffer = [0 as c_char; 96];
            self.check(
                func(device, buffer.as_mut_ptr(), buffer.len() as c_uint),
                "get GPU name",
            )?;
            Ok(c_string(buffer.as_ptr()))
        }
    }

    fn memory_info(&self, device: NvmlDevice) -> Result<NvmlMemory, String> {
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, *mut NvmlMemory) -> NvmlReturn,
            > = self
                .lib
                .get(b"nvmlDeviceGetMemoryInfo\0")
                .map_err(|error| format!("Memory query is unavailable: {error}"))?;
            let mut memory = MaybeUninit::<NvmlMemory>::zeroed();
            self.check(func(device, memory.as_mut_ptr()), "get memory info")?;
            Ok(memory.assume_init())
        }
    }

    fn utilization(&self, device: NvmlDevice) -> Result<NvmlUtilization, String> {
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, *mut NvmlUtilization) -> NvmlReturn,
            > = self
                .lib
                .get(b"nvmlDeviceGetUtilizationRates\0")
                .map_err(|error| format!("Utilization query is unavailable: {error}"))?;
            let mut utilization = MaybeUninit::<NvmlUtilization>::zeroed();
            self.check(func(device, utilization.as_mut_ptr()), "get utilization")?;
            Ok(utilization.assume_init())
        }
    }

    fn temperature(&self, device: NvmlDevice) -> Result<u32, String> {
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, c_uint, *mut c_uint) -> NvmlReturn,
            > = self
                .lib
                .get(b"nvmlDeviceGetTemperature\0")
                .map_err(|error| format!("Temperature query is unavailable: {error}"))?;
            let mut value = 0;
            self.check(
                func(device, NVML_TEMPERATURE_GPU, &mut value),
                "get temperature",
            )?;
            Ok(value)
        }
    }

    fn clock_info(&self, device: NvmlDevice, clock_type: c_uint) -> Result<u32, String> {
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, c_uint, *mut c_uint) -> NvmlReturn,
            > = self
                .lib
                .get(b"nvmlDeviceGetClockInfo\0")
                .map_err(|error| format!("Clock query is unavailable: {error}"))?;
            let mut value = 0;
            self.check(func(device, clock_type, &mut value), "get clock info")?;
            Ok(value)
        }
    }

    fn power_usage(&self, device: NvmlDevice) -> Result<f64, String> {
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, *mut c_uint) -> NvmlReturn,
            > = self
                .lib
                .get(b"nvmlDeviceGetPowerUsage\0")
                .map_err(|error| format!("Power usage query is unavailable: {error}"))?;
            let mut value = 0;
            self.check(func(device, &mut value), "get power usage")?;
            Ok(mw_to_watts(value))
        }
    }

    fn power_limit(&self, device: NvmlDevice) -> Result<f64, String> {
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, *mut c_uint) -> NvmlReturn,
            > = self
                .lib
                .get(b"nvmlDeviceGetPowerManagementLimit\0")
                .map_err(|error| format!("Power limit query is unavailable: {error}"))?;
            let mut value = 0;
            self.check(func(device, &mut value), "get power limit")?;
            Ok(mw_to_watts(value))
        }
    }

    fn clock_offset_info(&self, index: u32, clock_type: c_uint) -> Result<NvmlClockOffset, String> {
        let device = self.device(index)?;
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, *mut NvmlClockOffset) -> NvmlReturn,
            > = self
                .lib
                .get(b"nvmlDeviceGetClockOffsets\0")
                .map_err(|error| format!("nvmlDeviceGetClockOffsets is unavailable: {error}"))?;
            let mut info = NvmlClockOffset {
                version: NVML_CLOCK_OFFSET_VERSION_1,
                clock_type,
                pstate: NVML_PSTATE_0,
                ..Default::default()
            };
            self.check(func(device, &mut info), "get clock offset")?;
            Ok(info)
        }
    }

    fn set_clock_offset(
        &self,
        index: u32,
        clock_type: c_uint,
        offset_mhz: i32,
    ) -> Result<(), String> {
        let device = self.device(index)?;
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, *mut NvmlClockOffset) -> NvmlReturn,
            > = self
                .lib
                .get(b"nvmlDeviceSetClockOffsets\0")
                .map_err(|error| format!("nvmlDeviceSetClockOffsets is unavailable: {error}"))?;
            let mut info = NvmlClockOffset {
                version: NVML_CLOCK_OFFSET_VERSION_1,
                clock_type,
                pstate: NVML_PSTATE_0,
                clock_offset_mhz: offset_mhz,
                ..Default::default()
            };
            self.check(func(device, &mut info), "set clock offset")
        }
    }

    fn legacy_offset_capabilities(&self, index: u32) -> Result<NvmlOffsetCapabilities, String> {
        let core_current = self.legacy_gpc_offset(index).unwrap_or(0);
        let memory_current = self.legacy_mem_offset(index).unwrap_or(0);
        let (core_min, core_max) = self.legacy_gpc_offset_range(index)?;
        let (memory_min, memory_max) = self.legacy_mem_offset_range(index)?;
        Ok(NvmlOffsetCapabilities {
            core_supported: true,
            memory_supported: true,
            core_current_mhz: core_current,
            core_min_mhz: core_min,
            core_max_mhz: core_max,
            memory_current_mhz: memory_current,
            memory_min_mhz: memory_min,
            memory_max_mhz: memory_max,
            source: "legacy NVML VF offsets".to_string(),
        })
    }

    fn legacy_gpc_offset(&self, index: u32) -> Result<i32, String> {
        self.legacy_get_offset(
            index,
            b"nvmlDeviceGetGpcClkVfOffset\0",
            "get legacy GPC offset",
        )
    }

    fn legacy_mem_offset(&self, index: u32) -> Result<i32, String> {
        self.legacy_get_offset(
            index,
            b"nvmlDeviceGetMemClkVfOffset\0",
            "get legacy memory offset",
        )
    }

    fn legacy_gpc_offset_range(&self, index: u32) -> Result<(i32, i32), String> {
        self.legacy_get_offset_range(
            index,
            b"nvmlDeviceGetGpcClkMinMaxVfOffset\0",
            "get legacy GPC offset range",
        )
    }

    fn legacy_mem_offset_range(&self, index: u32) -> Result<(i32, i32), String> {
        self.legacy_get_offset_range(
            index,
            b"nvmlDeviceGetMemClkMinMaxVfOffset\0",
            "get legacy memory offset range",
        )
    }

    fn set_legacy_gpc_offset(&self, index: u32, offset_mhz: i32) -> Result<(), String> {
        self.legacy_set_offset(
            index,
            b"nvmlDeviceSetGpcClkVfOffset\0",
            offset_mhz,
            "set legacy GPC offset",
        )
    }

    fn set_legacy_mem_offset(&self, index: u32, offset_mhz: i32) -> Result<(), String> {
        self.legacy_set_offset(
            index,
            b"nvmlDeviceSetMemClkVfOffset\0",
            offset_mhz,
            "set legacy memory offset",
        )
    }

    fn legacy_get_offset(&self, index: u32, symbol: &[u8], action: &str) -> Result<i32, String> {
        let device = self.device(index)?;
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, *mut c_int) -> NvmlReturn,
            > = self
                .lib
                .get(symbol)
                .map_err(|error| format!("{action} is unavailable: {error}"))?;
            let mut offset = 0;
            self.check(func(device, &mut offset), action)?;
            Ok(offset)
        }
    }

    fn legacy_get_offset_range(
        &self,
        index: u32,
        symbol: &[u8],
        action: &str,
    ) -> Result<(i32, i32), String> {
        let device = self.device(index)?;
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(NvmlDevice, *mut c_int, *mut c_int) -> NvmlReturn,
            > = self
                .lib
                .get(symbol)
                .map_err(|error| format!("{action} is unavailable: {error}"))?;
            let mut min = 0;
            let mut max = 0;
            self.check(func(device, &mut min, &mut max), action)?;
            Ok((min, max))
        }
    }

    fn legacy_set_offset(
        &self,
        index: u32,
        symbol: &[u8],
        offset_mhz: i32,
        action: &str,
    ) -> Result<(), String> {
        let device = self.device(index)?;
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(NvmlDevice, c_int) -> NvmlReturn> =
                self.lib
                    .get(symbol)
                    .map_err(|error| format!("{action} is unavailable: {error}"))?;
            self.check(func(device, offset_mhz), action)
        }
    }

    fn check(&self, code: NvmlReturn, action: &str) -> Result<(), String> {
        if code == 0 {
            Ok(())
        } else {
            Err(format!(
                "{action} failed: {}",
                error_string(&self.lib, code)
            ))
        }
    }
}

impl Drop for Nvml {
    fn drop(&mut self) {
        unsafe {
            if let Ok(shutdown) = self
                .lib
                .get::<unsafe extern "C" fn() -> NvmlReturn>(b"nvmlShutdown\0")
            {
                let _ = shutdown();
            }
        }
    }
}

fn error_string(lib: &Library, code: NvmlReturn) -> String {
    unsafe {
        match lib.get::<unsafe extern "C" fn(NvmlReturn) -> *const c_char>(b"nvmlErrorString\0") {
            Ok(func) => {
                let ptr = func(code);
                if ptr.is_null() {
                    format!("NVML error code {code}")
                } else {
                    CStr::from_ptr(ptr).to_string_lossy().into_owned()
                }
            }
            Err(_) => format!("NVML error code {code}"),
        }
    }
}

fn c_string(ptr: *const c_char) -> String {
    unsafe {
        CStr::from_ptr(ptr)
            .to_string_lossy()
            .trim_end_matches('\0')
            .to_string()
    }
}

fn bytes_to_mb(bytes: c_ulonglong) -> u64 {
    bytes / 1024 / 1024
}

fn mw_to_watts(value: c_uint) -> f64 {
    value as f64 / 1000.0
}
