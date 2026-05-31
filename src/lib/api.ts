import { invoke } from "@tauri-apps/api/core";
import type {
  GPUInfo,
  GpuCapabilities,
  GpuTelemetry,
  LogEntry,
  Profile,
  TuningCommand,
  TuningResult
} from "./types";

export const api = {
  getGpuList: () => invoke<GPUInfo[]>("get_gpu_list"),
  getGpuInfo: (gpuIndex: number) => invoke<GPUInfo>("get_gpu_info", { gpuIndex }),
  getTelemetry: (gpuIndex: number) => invoke<GpuTelemetry>("get_telemetry", { gpuIndex }),
  getCapabilities: (gpuIndex: number) => invoke<GpuCapabilities>("get_capabilities", { gpuIndex }),
  applyTuning: (command: TuningCommand) => invoke<TuningResult>("apply_tuning", { command }),
  setPowerLimit: (gpuIndex: number, watts: number) => invoke<boolean>("set_power_limit", { gpuIndex, watts }),
  setCoreClockOffset: (gpuIndex: number, offsetMhz: number) =>
    invoke<boolean>("set_core_clock_offset", { gpuIndex, offsetMhz }),
  setMemoryClockOffset: (gpuIndex: number, offsetMhz: number) =>
    invoke<boolean>("set_memory_clock_offset", { gpuIndex, offsetMhz }),
  resetGpuSettings: (gpuIndex: number) => invoke<TuningResult>("reset_gpu_settings", { gpuIndex }),
  saveProfile: (profile: Profile) => invoke<boolean>("save_profile", { profile }),
  loadProfile: (name: string) => invoke<Profile>("load_profile", { name }),
  listProfiles: () => invoke<Profile[]>("list_profiles"),
  deleteProfile: (name: string) => invoke<boolean>("delete_profile", { name }),
  getLogs: () => invoke<LogEntry[]>("get_logs"),
  clearLogs: () => invoke<boolean>("clear_logs")
};
