pub mod models;
mod nvapi;
mod nvml;

use models::{
    CapabilityState, GPUInfo, GpuCapabilities, GpuTelemetry, TuningCommand, TuningResult,
};

const DEFAULT_CORE_OFFSET_MIN: i32 = -200;
const DEFAULT_CORE_OFFSET_MAX: i32 = 300;
const DEFAULT_MEMORY_OFFSET_MIN: i32 = -500;
const DEFAULT_MEMORY_OFFSET_MAX: i32 = 1500;

pub fn get_gpu_list() -> Result<Vec<GPUInfo>, String> {
    let nvml = nvml::Nvml::new()?;
    nvml.gpu_list()
}

pub fn get_gpu_info(gpu_index: u32) -> Result<GPUInfo, String> {
    let nvml = nvml::Nvml::new()?;
    nvml.gpu_info(gpu_index)
}

pub fn get_telemetry(gpu_index: u32) -> Result<GpuTelemetry, String> {
    let nvml = nvml::Nvml::new()?;
    nvml.telemetry(gpu_index)
}

pub fn get_capabilities(gpu_index: u32) -> Result<GpuCapabilities, String> {
    let nvml = nvml::Nvml::new()?;
    let mut notes = Vec::new();
    let power_range = nvml.power_limit_constraints(gpu_index);
    let locked_clock_range = nvml.locked_clock_ranges(gpu_index);
    let telemetry_state = match nvml.telemetry(gpu_index) {
        Ok(_) => CapabilityState::Supported,
        Err(error) => {
            notes.push(format!("Telemetry read failed: {error}"));
            CapabilityState::Failed
        }
    };

    let (power_state, power_min, power_max) = match power_range {
        Ok((min, max)) => (CapabilityState::Supported, min, max),
        Err(error) => {
            notes.push(format!("Power limit write unavailable: {error}"));
            (CapabilityState::Unsupported, 0.0, 0.0)
        }
    };

    let (clock_state, memory_clocks, core_clocks) = match locked_clock_range {
        Ok((memory_clocks, core_clocks)) => {
            notes.push("Fixed clocks use NVML locked clocks, equivalent to nvidia-smi -lgc/-lmc style locking.".to_string());
            (CapabilityState::Supported, memory_clocks, core_clocks)
        }
        Err(error) => {
            notes.push(format!("Locked clock range query unavailable: {error}"));
            (CapabilityState::Unsupported, Vec::new(), Vec::new())
        }
    };

    let mut core_offset_state = CapabilityState::Unsupported;
    let mut memory_offset_state = CapabilityState::Unsupported;
    let mut core_offset_min = DEFAULT_CORE_OFFSET_MIN;
    let mut core_offset_max = DEFAULT_CORE_OFFSET_MAX;
    let mut memory_offset_min = DEFAULT_MEMORY_OFFSET_MIN;
    let mut memory_offset_max = DEFAULT_MEMORY_OFFSET_MAX;

    match nvml.offset_capabilities(gpu_index) {
        Ok(offsets) => {
            if offsets.core_supported {
                core_offset_state = CapabilityState::Supported;
                core_offset_min = offsets.core_min_mhz;
                core_offset_max = offsets.core_max_mhz;
            }
            if offsets.memory_supported {
                memory_offset_state = CapabilityState::Supported;
                memory_offset_min = offsets.memory_min_mhz;
                memory_offset_max = offsets.memory_max_mhz;
            }
            notes.push(format!(
                "Clock offset uses {}. Current core offset {} MHz, memory offset {} MHz.",
                offsets.source, offsets.core_current_mhz, offsets.memory_current_mhz
            ));
        }
        Err(error) => {
            notes.push(format!(
                "NVML clock offset capability detection failed: {error}"
            ));
        }
    }

    match nvapi::Nvapi::new() {
        Ok(api) => {
            if let Ok(version) = api.driver_version() {
                notes.push(format!("NVAPI driver version: {version}"));
            }
            if !matches!(core_offset_state, CapabilityState::Supported)
                || !matches!(memory_offset_state, CapabilityState::Supported)
            {
                match api.offset_capabilities(gpu_index) {
                    Ok(offsets) => {
                        if offsets.core_supported
                            && !matches!(core_offset_state, CapabilityState::Supported)
                        {
                            core_offset_state = CapabilityState::Supported;
                            core_offset_min = offsets.core_min_mhz;
                            core_offset_max = offsets.core_max_mhz;
                        } else {
                            notes.push(
                                "Core clock offset is not editable in NVAPI P0 state.".to_string(),
                            );
                        }
                        if offsets.memory_supported
                            && !matches!(memory_offset_state, CapabilityState::Supported)
                        {
                            memory_offset_state = CapabilityState::Supported;
                            memory_offset_min = offsets.memory_min_mhz;
                            memory_offset_max = offsets.memory_max_mhz;
                        } else {
                            notes.push(
                                "Memory clock offset is not editable in NVAPI P0 state."
                                    .to_string(),
                            );
                        }
                        notes.push("NVAPI Pstates20 offset is available as fallback.".to_string());
                    }
                    Err(error) => {
                        notes.push(format!(
                            "NVAPI clock offset capability detection failed: {error}"
                        ));
                    }
                }
            }
        }
        Err(error) => {
            notes.push(format!("NVAPI unavailable: {error}"));
        }
    }

    Ok(GpuCapabilities {
        can_read_telemetry: telemetry_state,
        can_set_power_limit: power_state,
        can_set_core_clock: clock_state.clone(),
        can_set_memory_clock: clock_state,
        can_set_core_clock_offset: core_offset_state,
        can_set_memory_clock_offset: memory_offset_state,
        power_limit_min_watts: power_min,
        power_limit_max_watts: power_max,
        core_clock_min_mhz: *core_clocks.first().unwrap_or(&0),
        core_clock_max_mhz: *core_clocks.last().unwrap_or(&0),
        memory_clock_min_mhz: *memory_clocks.first().unwrap_or(&0),
        memory_clock_max_mhz: *memory_clocks.last().unwrap_or(&0),
        supported_core_clocks_mhz: core_clocks,
        supported_memory_clocks_mhz: memory_clocks,
        core_clock_offset_min_mhz: core_offset_min,
        core_clock_offset_max_mhz: core_offset_max,
        memory_clock_offset_min_mhz: memory_offset_min,
        memory_clock_offset_max_mhz: memory_offset_max,
        notes,
    })
}

pub fn apply_tuning(command: TuningCommand) -> Result<TuningResult, String> {
    let capabilities = get_capabilities(command.gpu_index)?;
    let mut result = TuningResult {
        success: true,
        applied_power_limit: false,
        applied_core_clock: false,
        applied_memory_clock: false,
        applied_core_clock_offset: false,
        applied_memory_clock_offset: false,
        messages: Vec::new(),
    };

    if let Some(watts) = command.power_limit_watts {
        if matches!(capabilities.can_set_power_limit, CapabilityState::Supported) {
            match set_power_limit(command.gpu_index, watts) {
                Ok(_) => {
                    result.applied_power_limit = true;
                    result
                        .messages
                        .push(format!("Power limit set to {watts:.1} W"));
                }
                Err(error) => {
                    result.success = false;
                    result.messages.push(format!("Power limit failed: {error}"));
                }
            }
        } else {
            result.success = false;
            result
                .messages
                .push("Power limit is unsupported on this GPU or driver.".to_string());
        }
    }

    if command.core_clock_mhz.is_some() || command.memory_clock_mhz.is_some() {
        if let Some(core_clock) = command.core_clock_mhz {
            if matches!(capabilities.can_set_core_clock, CapabilityState::Supported) {
                match set_core_locked_clock(command.gpu_index, core_clock) {
                    Ok(_) => {
                        result.applied_core_clock = true;
                        result
                            .messages
                            .push(format!("Core clock locked to {core_clock} MHz"));
                    }
                    Err(error) => {
                        result.success = false;
                        result
                            .messages
                            .push(format!("Core clock lock failed: {error}"));
                    }
                }
            } else {
                result.success = false;
                result
                    .messages
                    .push("Core clock lock is unsupported on this GPU or driver.".to_string());
            }
        }

        if let Some(memory_clock) = command.memory_clock_mhz {
            if matches!(
                capabilities.can_set_memory_clock,
                CapabilityState::Supported
            ) {
                match set_memory_locked_clock(command.gpu_index, memory_clock) {
                    Ok(_) => {
                        result.applied_memory_clock = true;
                        result
                            .messages
                            .push(format!("Memory clock locked to {memory_clock} MHz"));
                    }
                    Err(error) => {
                        result.success = false;
                        result
                            .messages
                            .push(format!("Memory clock lock failed: {error}"));
                    }
                }
            } else {
                result.success = false;
                result
                    .messages
                    .push("Memory clock lock is unsupported on this GPU or driver.".to_string());
            }
        }

        if result.applied_core_clock || result.applied_memory_clock {
            result.messages.push(
                "Locked clocks use NVML locked-clock APIs, not application clock pairs."
                    .to_string(),
            );
        } else {
            result.success = false;
        }
    }

    if let Some(offset) = command.core_clock_offset_mhz {
        if offset < capabilities.core_clock_offset_min_mhz
            || offset > capabilities.core_clock_offset_max_mhz
        {
            result.success = false;
            result.messages.push(format!(
                "Core clock offset {offset} MHz is outside {}..{} MHz",
                capabilities.core_clock_offset_min_mhz, capabilities.core_clock_offset_max_mhz
            ));
        } else if matches!(
            capabilities.can_set_core_clock_offset,
            CapabilityState::Supported
        ) {
            match set_core_clock_offset(command.gpu_index, offset) {
                Ok(_) => {
                    result.applied_core_clock_offset = true;
                    result
                        .messages
                        .push(format!("Core clock offset set to {offset} MHz"));
                }
                Err(error) => {
                    result.success = false;
                    result
                        .messages
                        .push(format!("Core clock offset failed: {error}"));
                }
            }
        } else {
            result.success = false;
            result
                .messages
                .push("Core clock offset is unsupported on this GPU or driver.".to_string());
        }
    }

    if let Some(offset) = command.memory_clock_offset_mhz {
        if offset < capabilities.memory_clock_offset_min_mhz
            || offset > capabilities.memory_clock_offset_max_mhz
        {
            result.success = false;
            result.messages.push(format!(
                "Memory clock offset {offset} MHz is outside {}..{} MHz",
                capabilities.memory_clock_offset_min_mhz, capabilities.memory_clock_offset_max_mhz
            ));
        } else if matches!(
            capabilities.can_set_memory_clock_offset,
            CapabilityState::Supported
        ) {
            match set_memory_clock_offset(command.gpu_index, offset) {
                Ok(_) => {
                    result.applied_memory_clock_offset = true;
                    result
                        .messages
                        .push(format!("Memory clock offset set to {offset} MHz"));
                }
                Err(error) => {
                    result.success = false;
                    result
                        .messages
                        .push(format!("Memory clock offset failed: {error}"));
                }
            }
        } else {
            result.success = false;
            result
                .messages
                .push("Memory clock offset is unsupported on this GPU or driver.".to_string());
        }
    }

    if result.messages.is_empty() {
        result
            .messages
            .push("No tuning values were requested.".to_string());
    }

    Ok(result)
}

pub fn set_power_limit(gpu_index: u32, watts: f64) -> Result<bool, String> {
    let nvml = nvml::Nvml::new()?;
    let (min, max) = nvml.power_limit_constraints(gpu_index)?;
    crate::safety::validate_power_limit(watts, min, max)?;
    nvml.set_power_limit(gpu_index, watts)?;
    Ok(true)
}

pub fn set_core_locked_clock(gpu_index: u32, core_clock_mhz: u32) -> Result<bool, String> {
    let capabilities = get_capabilities(gpu_index)?;
    if core_clock_mhz < capabilities.core_clock_min_mhz
        || core_clock_mhz > capabilities.core_clock_max_mhz
    {
        return Err(format!(
            "Core clock {core_clock_mhz} MHz is outside {}..{} MHz",
            capabilities.core_clock_min_mhz, capabilities.core_clock_max_mhz
        ));
    }
    let nvml = nvml::Nvml::new()?;
    nvml.set_gpu_locked_clocks(gpu_index, core_clock_mhz, core_clock_mhz)?;
    Ok(true)
}

pub fn set_memory_locked_clock(gpu_index: u32, memory_clock_mhz: u32) -> Result<bool, String> {
    let capabilities = get_capabilities(gpu_index)?;
    if memory_clock_mhz < capabilities.memory_clock_min_mhz
        || memory_clock_mhz > capabilities.memory_clock_max_mhz
    {
        return Err(format!(
            "Memory clock {memory_clock_mhz} MHz is outside {}..{} MHz",
            capabilities.memory_clock_min_mhz, capabilities.memory_clock_max_mhz
        ));
    }
    let nvml = nvml::Nvml::new()?;
    nvml.set_memory_locked_clocks(gpu_index, memory_clock_mhz, memory_clock_mhz)?;
    Ok(true)
}

pub fn set_core_clock_offset(gpu_index: u32, offset_mhz: i32) -> Result<bool, String> {
    let capabilities = get_capabilities(gpu_index)?;
    crate::safety::validate_clock_offset(
        offset_mhz,
        capabilities.core_clock_offset_min_mhz,
        capabilities.core_clock_offset_max_mhz,
        "core clock",
    )?;
    if !matches!(
        capabilities.can_set_core_clock_offset,
        CapabilityState::Supported
    ) {
        return Err("Core clock offset is not editable on this GPU or driver.".to_string());
    }
    nvml::Nvml::new()
        .and_then(|nvml| nvml.set_core_clock_offset(gpu_index, offset_mhz))
        .or_else(|nvml_error| {
            nvapi::Nvapi::new()
                .and_then(|nvapi| nvapi.set_core_clock_offset(gpu_index, offset_mhz))
                .map_err(|nvapi_error| format!("NVML core offset failed: {nvml_error}; NVAPI fallback failed: {nvapi_error}"))
        })?;
    Ok(true)
}

pub fn set_memory_clock_offset(gpu_index: u32, offset_mhz: i32) -> Result<bool, String> {
    let capabilities = get_capabilities(gpu_index)?;
    crate::safety::validate_clock_offset(
        offset_mhz,
        capabilities.memory_clock_offset_min_mhz,
        capabilities.memory_clock_offset_max_mhz,
        "memory clock",
    )?;
    if !matches!(
        capabilities.can_set_memory_clock_offset,
        CapabilityState::Supported
    ) {
        return Err("Memory clock offset is not editable on this GPU or driver.".to_string());
    }
    nvml::Nvml::new()
        .and_then(|nvml| nvml.set_memory_clock_offset(gpu_index, offset_mhz))
        .or_else(|nvml_error| {
            nvapi::Nvapi::new()
                .and_then(|nvapi| nvapi.set_memory_clock_offset(gpu_index, offset_mhz))
                .map_err(|nvapi_error| format!("NVML memory offset failed: {nvml_error}; NVAPI fallback failed: {nvapi_error}"))
        })?;
    Ok(true)
}

pub fn reset_gpu_settings(gpu_index: u32) -> Result<TuningResult, String> {
    let mut messages = Vec::new();
    let mut reset_count = 0;
    let mut power_limit_reset = false;
    let mut core_clock_reset = false;
    let mut memory_clock_reset = false;
    let mut offset_reset = false;

    match nvml::Nvml::new().and_then(|nvml| {
        let default_power_limit = nvml.default_power_limit(gpu_index)?;
        nvml.set_power_limit(gpu_index, default_power_limit)?;
        Ok(default_power_limit)
    }) {
        Ok(default_power_limit) => {
            reset_count += 1;
            power_limit_reset = true;
            messages.push(format!(
                "Power limit restored to default {default_power_limit:.1} W."
            ));
        }
        Err(error) => {
            messages.push(format!("Power limit default restore failed: {error}"));
        }
    }

    match nvml::Nvml::new().and_then(|nvml| nvml.reset_gpu_locked_clocks(gpu_index)) {
        Ok(_) => {
            reset_count += 1;
            core_clock_reset = true;
            messages.push("GPU locked clocks reset through NVML.".to_string());
        }
        Err(error) => {
            messages.push(format!("GPU locked clock reset failed: {error}"));
        }
    }

    match nvml::Nvml::new().and_then(|nvml| nvml.reset_memory_locked_clocks(gpu_index)) {
        Ok(_) => {
            reset_count += 1;
            memory_clock_reset = true;
            messages.push("Memory locked clocks reset through NVML.".to_string());
        }
        Err(error) => {
            messages.push(format!("Memory locked clock reset failed: {error}"));
        }
    }

    match nvml::Nvml::new()
        .and_then(|nvml| nvml.reset_clock_offsets(gpu_index))
        .or_else(|_| nvapi::Nvapi::new().and_then(|nvapi| nvapi.reset_clock_offsets(gpu_index)))
    {
        Ok(_) => {
            reset_count += 1;
            offset_reset = true;
            messages.push("Clock offsets reset.".to_string());
        }
        Err(error) => {
            messages.push(format!("Clock offset reset failed: {error}"));
        }
    }

    Ok(TuningResult {
        success: reset_count > 0,
        applied_power_limit: power_limit_reset,
        applied_core_clock: core_clock_reset,
        applied_memory_clock: memory_clock_reset,
        applied_core_clock_offset: offset_reset,
        applied_memory_clock_offset: offset_reset,
        messages,
    })
}
