# GPU Tuner

## 中文

GPU Tuner 是一款 Windows 桌面应用，用于查看 NVIDIA 显卡状态并安全地应用常见调参项。应用使用 Tauri + React + TypeScript 构建界面，Rust 负责后端和 NVIDIA API 调用。

### 功能

- 显示 NVIDIA GPU 列表、GPU 名称、驱动版本和显存容量
- 实时监控温度、GPU 使用率、显存使用率、核心频率、显存频率、功耗和功耗限制
- 支持 Power Limit 范围检测和写入
- 支持 Locked Core Clock，使用 NVML locked clocks，语义接近 `nvidia-smi -lgc`
- 支持 Locked Memory Clock，使用 NVML locked clocks，语义接近 `nvidia-smi -lmc`
- 支持 Core Clock Offset / Memory Clock Offset，优先使用 NVML clock offset API，旧驱动回退到 legacy NVML VF offset API，最后回退到 NVAPI Pstates20
- 每个调参项都有独立启用开关，Apply 只提交勾选项
- 支持本地 JSON profiles
- 支持温度保护阈值，超温时自动尝试 reset
- 支持本地日志，记录 API 调用成功/失败和错误信息
- 支持 Windows EXE / MSI / NSIS 安装包

### 明确不包含

- 电压控制
- 风扇控制
- 风扇曲线
- 降压功能
- RGB、OSD、游戏叠加层
- 内核驱动
- 绕过 NVIDIA 驱动限制

### 系统要求

- Windows 10 或 Windows 11
- NVIDIA GPU
- 较新的 NVIDIA Driver
- `nvml.dll`，通常随 NVIDIA Driver 安装
- `nvapi64.dll`，通常随 NVIDIA Driver 安装
- 管理员权限运行

开发环境还需要：

- Node.js 18+
- Rust stable toolchain
- Microsoft Visual Studio Build Tools，安装 Desktop development with C++ workload

### 安装依赖

```powershell
npm install
```

### 开发运行

```powershell
npm run tauri dev
```

### 只构建独立 EXE

```powershell
npm run "tauri build:exe"
```

输出位置：

```text
src-tauri\target\release\gpu-tuner.exe
```

### 构建安装包

```powershell
npm run tauri build
```

输出位置：

```text
src-tauri\target\release\bundle\nsis
src-tauri\target\release\bundle\msi
```

### 安全说明

GPU 调参可能导致系统不稳定、驱动重启或应用崩溃。建议先只读取监控数据，再逐项勾选并应用小幅度调整。应用会做范围校验，但最终是否接受设置由 NVIDIA 驱动和当前 GPU 决定。

### 限制

- 部分 GeForce GPU 或驱动版本会拒绝 clock offset 写入。
- Locked clock 会被驱动量化到支持的频率档位，输入值可能被固定到相邻档位。
- NVAPI Pstates20 写入是 fallback 路径，可能返回 `NVAPI_NOT_SUPPORTED`。
- 不同 NVIDIA Driver 分支暴露的 NVML/NVAPI 能力可能不同。

## English

GPU Tuner is a Windows desktop application for monitoring NVIDIA GPUs and applying guarded tuning settings. The UI is built with Tauri + React + TypeScript, while the backend uses Rust and dynamically loaded NVIDIA APIs.

### Features

- Lists NVIDIA GPUs, GPU names, driver version, and total memory
- Realtime telemetry for temperature, GPU utilization, memory utilization, core clock, memory clock, power draw, and power limit
- Power Limit range detection and guarded write support
- Locked Core Clock through NVML locked clocks, similar to `nvidia-smi -lgc`
- Locked Memory Clock through NVML locked clocks, similar to `nvidia-smi -lmc`
- Core Clock Offset / Memory Clock Offset through NVML clock offset APIs first, legacy NVML VF offset APIs as fallback, and NVAPI Pstates20 as final fallback
- Per-setting enable checkboxes, so Apply only writes selected settings
- Local JSON profiles
- Temperature protection threshold with automatic reset attempt
- Local API logs with success/failure and error messages
- Windows EXE / MSI / NSIS packaging

### Explicitly Out Of Scope

- Voltage control
- Fan control
- Fan curves
- Undervolting
- RGB, OSD, or game overlays
- Kernel drivers
- NVIDIA driver bypasses

### Requirements

- Windows 10 or Windows 11
- NVIDIA GPU
- Recent NVIDIA Driver
- `nvml.dll`, usually installed with the NVIDIA Driver
- `nvapi64.dll`, usually installed with the NVIDIA Driver
- Administrator privileges

Development also requires:

- Node.js 18+
- Rust stable toolchain
- Microsoft Visual Studio Build Tools with the Desktop development with C++ workload

### Install Dependencies

```powershell
npm install
```

### Development

```powershell
npm run tauri dev
```

### Build Standalone EXE

```powershell
npm run "tauri build:exe"
```

Output:

```text
src-tauri\target\release\gpu-tuner.exe
```

### Build Installers

```powershell
npm run tauri build
```

Output:

```text
src-tauri\target\release\bundle\nsis
src-tauri\target\release\bundle\msi
```

### Safety

GPU tuning can cause instability, driver resets, or application crashes. Start with monitoring only, then apply small changes one setting at a time. The app validates ranges before writing, while the NVIDIA driver and GPU ultimately decide whether a setting is accepted.

### Limitations

- Some GeForce GPUs or driver versions may reject clock offset writes.
- Locked clocks are rounded or quantized by the driver to supported frequency steps.
- NVAPI Pstates20 writes are a fallback path and may return `NVAPI_NOT_SUPPORTED`.
- NVML/NVAPI capabilities vary across NVIDIA Driver branches.
