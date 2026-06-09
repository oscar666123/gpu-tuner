#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gpu;
mod logs;
mod profiles;
mod safety;

use gpu::models::{GPUInfo, GpuCapabilities, GpuTelemetry, TuningCommand, TuningResult};
use logs::{LogEntry, LogStore};
use profiles::ProfileStore;
use tauri::State;

struct AppState {
    profiles: ProfileStore,
    logs: LogStore,
}

#[tauri::command]
fn get_gpu_list(state: State<AppState>) -> Result<Vec<GPUInfo>, String> {
    run_read(&state, "get_gpu_list", gpu::get_gpu_list)
}

#[tauri::command]
fn get_gpu_info(state: State<AppState>, gpu_index: u32) -> Result<GPUInfo, String> {
    run_read(&state, "get_gpu_info", || gpu::get_gpu_info(gpu_index))
}

#[tauri::command]
fn get_telemetry(state: State<AppState>, gpu_index: u32) -> Result<GpuTelemetry, String> {
    run_read(&state, "get_telemetry", || gpu::get_telemetry(gpu_index))
}

#[tauri::command]
fn get_capabilities(state: State<AppState>, gpu_index: u32) -> Result<GpuCapabilities, String> {
    run_read(&state, "get_capabilities", || {
        gpu::get_capabilities(gpu_index)
    })
}

#[tauri::command]
fn apply_tuning(state: State<AppState>, command: TuningCommand) -> Result<TuningResult, String> {
    let result = gpu::apply_tuning(command);
    match &result {
        Ok(value) => {
            state.logs.write(
                "info",
                "apply_tuning",
                value.success,
                None,
                &value.messages.join("; "),
            )?;
        }
        Err(error) => {
            state
                .logs
                .write("error", "apply_tuning", false, Some("APPLY_FAILED"), error)?;
        }
    }
    result
}

#[tauri::command]
fn set_power_limit(state: State<AppState>, gpu_index: u32, watts: f64) -> Result<bool, String> {
    run_write(&state, "set_power_limit", || {
        gpu::set_power_limit(gpu_index, watts)
    })
}

#[tauri::command]
fn set_core_clock_offset(
    state: State<AppState>,
    gpu_index: u32,
    offset_mhz: i32,
) -> Result<bool, String> {
    run_write(&state, "set_core_clock_offset", || {
        gpu::set_core_clock_offset(gpu_index, offset_mhz)
    })
}

#[tauri::command]
fn set_memory_clock_offset(
    state: State<AppState>,
    gpu_index: u32,
    offset_mhz: i32,
) -> Result<bool, String> {
    run_write(&state, "set_memory_clock_offset", || {
        gpu::set_memory_clock_offset(gpu_index, offset_mhz)
    })
}

#[tauri::command]
fn reset_gpu_settings(state: State<AppState>, gpu_index: u32) -> Result<TuningResult, String> {
    let result = gpu::reset_gpu_settings(gpu_index);
    match &result {
        Ok(value) => {
            state.logs.write(
                "info",
                "reset_gpu_settings",
                value.success,
                None,
                &value.messages.join("; "),
            )?;
        }
        Err(error) => {
            state.logs.write(
                "error",
                "reset_gpu_settings",
                false,
                Some("RESET_FAILED"),
                error,
            )?;
        }
    }
    result
}

#[tauri::command]
fn save_profile(state: State<AppState>, profile: profiles::Profile) -> Result<bool, String> {
    state.profiles.save(profile)?;
    state
        .logs
        .write("info", "save_profile", true, None, "Profile saved")?;
    Ok(true)
}

#[tauri::command]
fn load_profile(state: State<AppState>, name: String) -> Result<profiles::Profile, String> {
    state.profiles.load(&name)
}

#[tauri::command]
fn list_profiles(state: State<AppState>) -> Result<Vec<profiles::Profile>, String> {
    state.profiles.list()
}

#[tauri::command]
fn delete_profile(state: State<AppState>, name: String) -> Result<bool, String> {
    state.profiles.delete(&name)?;
    state
        .logs
        .write("info", "delete_profile", true, None, "Profile deleted")?;
    Ok(true)
}

#[tauri::command]
fn get_logs(state: State<AppState>) -> Result<Vec<LogEntry>, String> {
    state.logs.read()
}

#[tauri::command]
fn clear_logs(state: State<AppState>) -> Result<bool, String> {
    state.logs.clear()?;
    Ok(true)
}

fn run_read<T, F>(state: &State<AppState>, action: &str, op: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    let result = op();
    if let Err(error) = &result {
        let _ = state
            .logs
            .write("error", action, false, Some("READ_FAILED"), error);
    }
    result
}

fn run_write<F>(state: &State<AppState>, action: &str, op: F) -> Result<bool, String>
where
    F: FnOnce() -> Result<bool, String>,
{
    let result = op();
    match &result {
        Ok(_) => state
            .logs
            .write("info", action, true, None, "Write succeeded")?,
        Err(error) => state
            .logs
            .write("error", action, false, Some("WRITE_FAILED"), error)?,
    }
    result
}

fn main() {
    let state = AppState {
        profiles: ProfileStore::new("GPU Tuner"),
        logs: LogStore::new("GPU Tuner"),
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_gpu_list,
            get_gpu_info,
            get_telemetry,
            get_capabilities,
            apply_tuning,
            set_power_limit,
            set_core_clock_offset,
            set_memory_clock_offset,
            reset_gpu_settings,
            save_profile,
            load_profile,
            list_profiles,
            delete_profile,
            get_logs,
            clear_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running GPU Tuner");
}
