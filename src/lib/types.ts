export type CapabilityState = "supported" | "unsupported" | "requires admin" | "failed";

export interface GPUInfo {
  index: number;
  name: string;
  driver_version: string;
  memory_total_mb: number;
}

export interface GpuTelemetry {
  gpu_index: number;
  temperature_c: number;
  gpu_utilization_percent: number;
  memory_utilization_percent: number;
  core_clock_mhz: number;
  memory_clock_mhz: number;
  power_draw_watts: number;
  power_limit_watts: number;
  memory_used_mb: number;
  memory_total_mb: number;
  timestamp: string;
}

export interface GpuCapabilities {
  can_read_telemetry: CapabilityState;
  can_set_power_limit: CapabilityState;
  can_set_core_clock: CapabilityState;
  can_set_memory_clock: CapabilityState;
  can_set_core_clock_offset: CapabilityState;
  can_set_memory_clock_offset: CapabilityState;
  power_limit_min_watts: number;
  power_limit_max_watts: number;
  core_clock_min_mhz: number;
  core_clock_max_mhz: number;
  memory_clock_min_mhz: number;
  memory_clock_max_mhz: number;
  supported_core_clocks_mhz: number[];
  supported_memory_clocks_mhz: number[];
  core_clock_offset_min_mhz: number;
  core_clock_offset_max_mhz: number;
  memory_clock_offset_min_mhz: number;
  memory_clock_offset_max_mhz: number;
  notes: string[];
}

export interface TuningCommand {
  gpu_index: number;
  power_limit_watts: number | null;
  core_clock_mhz: number | null;
  memory_clock_mhz: number | null;
  core_clock_offset_mhz: number | null;
  memory_clock_offset_mhz: number | null;
}

export interface TuningResult {
  success: boolean;
  applied_power_limit: boolean;
  applied_core_clock: boolean;
  applied_memory_clock: boolean;
  applied_core_clock_offset: boolean;
  applied_memory_clock_offset: boolean;
  messages: string[];
}

export interface Profile {
  name: string;
  gpu_index: number;
  power_limit_watts: number | null;
  core_clock_mhz: number | null;
  memory_clock_mhz: number | null;
  core_clock_offset_mhz: number | null;
  memory_clock_offset_mhz: number | null;
  created_at: string;
  updated_at: string;
}

export interface LogEntry {
  timestamp: string;
  level: string;
  action: string;
  success: boolean;
  code: string | null;
  message: string;
}
