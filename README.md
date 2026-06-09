# GPU Tuner

<img src="assets/gpu-tuner-icon.png" alt="GPU Tuner icon" width="180" />

GPU Tuner is a Windows app by **oscar666123** for monitoring NVIDIA GPUs and applying guarded tuning settings.

中文文档: [README.zh-CN.md](README.zh-CN.md)

## Features

- Realtime NVIDIA GPU telemetry: temperature, utilization, clocks, power, and memory
- Guarded tuning for power limit, locked core clock, locked memory clock, and clock offsets
- Per-setting checkboxes, so Apply only writes selected values
- Local JSON profiles for saving and applying tuning values
- Temperature safety reset and local API logs
- Windows EXE, NSIS, and MSI builds

## Requirements

- Windows 10 or Windows 11
- NVIDIA GPU and recent NVIDIA Driver
- `nvml.dll` and `nvapi64.dll`
- Administrator privileges for tuning writes

Development requires Node.js 18+, Rust stable, and Visual Studio Build Tools with the Desktop C++ workload.

## Usage

```powershell
npm install
npm run tauri dev
```

Build standalone EXE:

```powershell
npm run "tauri build:exe"
```

Build installers:

```powershell
npm run tauri build
```

Outputs:

```text
src-tauri\target\release\gpu-tuner.exe
src-tauri\target\release\bundle\nsis
src-tauri\target\release\bundle\msi
```

## Safety

GPU tuning can cause instability, driver resets, or crashes. Start with monitoring, then apply small changes one setting at a time. NVIDIA driver support varies by GPU and driver version.

## Limits

- Clock offset writes may be rejected on some GeForce GPUs or drivers.
- Locked clocks may be rounded to supported NVIDIA frequency steps.
- NVAPI Pstates20 is a fallback path and may return `NVAPI_NOT_SUPPORTED`.

## Author

Created by **oscar666123**.
