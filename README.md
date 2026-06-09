# GPU Tuner

<img src="assets/gpu-tuner-icon.png" alt="GPU Tuner icon" width="180" />

GPU Tuner is a Windows desktop application by **oscar666123** for monitoring NVIDIA GPUs and applying guarded tuning settings.

中文文档: [README.zh-CN.md](README.zh-CN.md)

## Features

- Lists NVIDIA GPUs, GPU names, driver version, and total memory
- Realtime telemetry for temperature, GPU utilization, memory utilization, core clock, memory clock, power draw, and power limit
- Power Limit range detection and guarded write support
- Locked Core Clock through NVML locked clocks, similar to `nvidia-smi -lgc`
- Locked Memory Clock through NVML locked clocks, similar to `nvidia-smi -lmc`
- Core Clock Offset / Memory Clock Offset through NVML clock offset APIs first, legacy NVML VF offset APIs as fallback, and NVAPI Pstates20 as final fallback
- Per-setting enable checkboxes, so Apply only writes selected settings
- Local JSON profiles with power limit, locked clocks, and clock offsets
- Temperature protection threshold with automatic reset attempt
- Local API logs with success/failure and error messages
- Windows EXE / MSI / NSIS packaging

## Explicitly Out Of Scope

- Voltage control
- Fan control
- Fan curves
- Undervolting
- RGB, OSD, or game overlays
- Kernel drivers
- NVIDIA driver bypasses

## Requirements

- Windows 10 or Windows 11
- NVIDIA GPU
- Recent NVIDIA Driver
- `nvml.dll`, usually installed with the NVIDIA Driver
- `nvapi64.dll`, usually installed with the NVIDIA Driver
- Administrator privileges for tuning writes

Development also requires:

- Node.js 18+
- Rust stable toolchain
- Microsoft Visual Studio Build Tools with the Desktop development with C++ workload

## Install Dependencies

```powershell
npm install
```

## Development

```powershell
npm run tauri dev
```

## Build Standalone EXE

```powershell
npm run "tauri build:exe"
```

Output:

```text
src-tauri\target\release\gpu-tuner.exe
```

## Build Installers

```powershell
npm run tauri build
```

Output:

```text
src-tauri\target\release\bundle\nsis
src-tauri\target\release\bundle\msi
```

## Safety

GPU tuning can cause instability, driver resets, or application crashes. Start with monitoring only, then apply small changes one setting at a time. The app validates ranges before writing, while the NVIDIA driver and GPU ultimately decide whether a setting is accepted.

## Limitations

- Some GeForce GPUs or driver versions may reject clock offset writes.
- Locked clocks are rounded or quantized by the driver to supported frequency steps.
- NVAPI Pstates20 writes are a fallback path and may return `NVAPI_NOT_SUPPORTED`.
- NVML/NVAPI capabilities vary across NVIDIA Driver branches.

## Author

Created by **oscar666123**.
