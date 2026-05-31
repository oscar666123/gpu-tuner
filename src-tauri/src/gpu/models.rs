use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUInfo {
    pub index: u32,
    pub name: String,
    pub driver_version: String,
    pub memory_total_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuTelemetry {
    pub gpu_index: u32,
    pub temperature_c: u32,
    pub gpu_utilization_percent: u32,
    pub memory_utilization_percent: u32,
    pub core_clock_mhz: u32,
    pub memory_clock_mhz: u32,
    pub power_draw_watts: f64,
    pub power_limit_watts: f64,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityState {
    #[serde(rename = "supported")]
    Supported,
    #[serde(rename = "unsupported")]
    Unsupported,
    #[serde(rename = "requires admin")]
    RequiresAdmin,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuCapabilities {
    pub can_read_telemetry: CapabilityState,
    pub can_set_power_limit: CapabilityState,
    pub can_set_core_clock: CapabilityState,
    pub can_set_memory_clock: CapabilityState,
    pub can_set_core_clock_offset: CapabilityState,
    pub can_set_memory_clock_offset: CapabilityState,
    pub power_limit_min_watts: f64,
    pub power_limit_max_watts: f64,
    pub core_clock_min_mhz: u32,
    pub core_clock_max_mhz: u32,
    pub memory_clock_min_mhz: u32,
    pub memory_clock_max_mhz: u32,
    pub supported_core_clocks_mhz: Vec<u32>,
    pub supported_memory_clocks_mhz: Vec<u32>,
    pub core_clock_offset_min_mhz: i32,
    pub core_clock_offset_max_mhz: i32,
    pub memory_clock_offset_min_mhz: i32,
    pub memory_clock_offset_max_mhz: i32,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningCommand {
    pub gpu_index: u32,
    pub power_limit_watts: Option<f64>,
    pub core_clock_mhz: Option<u32>,
    pub memory_clock_mhz: Option<u32>,
    pub core_clock_offset_mhz: Option<i32>,
    pub memory_clock_offset_mhz: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningResult {
    pub success: bool,
    pub applied_power_limit: bool,
    pub applied_core_clock: bool,
    pub applied_memory_clock: bool,
    pub applied_core_clock_offset: bool,
    pub applied_memory_clock_offset: bool,
    pub messages: Vec<String>,
}
