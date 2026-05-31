# GPU Tuner

GPU Tuner is a Windows desktop app for reading NVIDIA GPU telemetry, applying supported power limits, managing local profiles, and recording tuning logs.

## Requirements

- Windows 10 or Windows 11
- NVIDIA GPU with a recent NVIDIA driver
- `nvml.dll`, usually installed with the NVIDIA driver
- `nvapi64.dll`, usually installed with the NVIDIA driver
- Node.js 18+
- Rust stable toolchain
- Microsoft Visual Studio Build Tools with the Desktop C++ workload

## Install

```powershell
npm install
```

## Development

```powershell
npm run tauri dev
```

The app is configured to request administrator privileges because NVIDIA driver write operations often require elevation.

## Build EXE / Installer

```powershell
npm run tauri build
```

The generated installer and executable are written under `src-tauri\target\release\bundle`.

## Scope

Implemented:

- GPU list, GPU info, and telemetry through dynamically loaded NVML
- Power limit range detection and guarded write through NVML
- NVAPI dynamic loading and capability probing foundation
- Profiles stored in local JSON
- Local logs for API calls and failures
- Temperature safety threshold with automatic reset command path
- Tauri Windows bundle config and administrator manifest

Limits:

- Core clock offset and memory clock offset write operations return `unsupported` unless a stable supported public API path is added.
- Application clock reset is attempted through NVML when available.
- Voltage control, fan control, fan curves, RGB, OSD, overlays, kernel drivers, and undocumented driver bypasses are outside this app.

## Safety

GPU tuning can affect system stability. Start with read-only monitoring, apply small changes, and keep the temperature threshold enabled. Power and frequency values are range-checked before write operations.
