# GPU Tuner

<img src="assets/gpu-tuner-icon.png" alt="GPU Tuner 图标" width="180" />

GPU Tuner 是 **oscar666123** 开发的 Windows 应用，用于监控 NVIDIA GPU 并安全应用常见调参项。

English documentation: [README.md](README.md)

## 功能

- 实时监控 NVIDIA GPU：温度、使用率、频率、功耗和显存
- 安全写入 Power Limit、核心定频、显存定频和 clock offset
- 每个调参项都有独立勾选框，Apply 只写入选中的项目
- 本地 JSON profiles，用于保存和应用调参值
- 温度保护 reset 和本地 API 日志
- 支持 Windows EXE、NSIS、MSI 构建

## 系统要求

- Windows 10 或 Windows 11
- NVIDIA GPU 和较新的 NVIDIA Driver
- `nvml.dll` 和 `nvapi64.dll`
- 写入调参项需要管理员权限

开发环境需要 Node.js 18+、Rust stable、Visual Studio Build Tools，并安装 Desktop C++ workload。

## 使用

```powershell
npm install
npm run tauri dev
```

构建独立 EXE：

```powershell
npm run "tauri build:exe"
```

构建安装包：

```powershell
npm run tauri build
```

输出位置：

```text
src-tauri\target\release\gpu-tuner.exe
src-tauri\target\release\bundle\nsis
src-tauri\target\release\bundle\msi
```

## 安全说明

GPU 调参可能导致系统不稳定、驱动重启或崩溃。建议先监控，再逐项小幅调整。NVIDIA 驱动是否接受设置取决于 GPU 和驱动版本。

## 限制

- 部分 GeForce GPU 或驱动会拒绝 clock offset 写入。
- Locked clock 可能被量化到 NVIDIA 支持的频率档位。
- NVAPI Pstates20 是 fallback 路径，可能返回 `NVAPI_NOT_SUPPORTED`。

## 作者

由 **oscar666123** 创建。
