<p align="center">
  <img src="./gpust.png" alt="GPUst" width="180">
</p>

<h1 align="center">GPUst</h1>

<p align="center">
  <strong>GPU task manager for Windows</strong><br>
  Per-process GPU usage, per-engine breakdown, VRAM, device telemetry, themes, tray.
</p>

<p align="center">

  <img src="https://img.shields.io/badge/platform-Windows-blue" alt="Platform">
  <img src="https://img.shields.io/badge/tauri-2.x-orange" alt="Tauri">
  <img src="https://img.shields.io/badge/backend-Rust-brown" alt="Rust">
  <img src="https://img.shields.io/badge/frontend-Vue%203-green" alt="Vue">
  <img src="https://img.shields.io/badge/license-MIT-lightgrey" alt="License">
</p>

---

See **which processes are using your GPU** and how much, in real time. Born because
the standard Task Manager makes it awkward to spot what keeps the GPU busy at idle
(the original culprit: Discord's hardware acceleration sitting at ~10–18%).

## Features

| Page | Description |
|------|-------------|
| **Processes** | Per-process GPU usage as the busiest engine (Task Manager style), per-engine breakdown (3D / Video Decode / Video Encode / Copy), dedicated VRAM, live trend sparkline, real `.exe` icons, sortable columns |
| **GPU** | Device dashboard per adapter: utilization, VRAM used/total, temperature, core/memory clocks, power draw + limit, fan speed, encoder/decoder activity |
| **Kill** | Terminate a process with a confirmation, guarded against critical system processes (`dwm`, `csrss`, `System`, ...) — with an optional advanced override |
| **Themes** | Built-in themes (Midnight, Graphite, Nord, Dracula, Light), remembered across launches |
| **Settings** | Adjustable refresh rate, launch on Windows startup, start hidden in tray |

## Installation

### From source

```bash
git clone https://github.com/ArcademMan/GPUst---GPU-Task-Manager.git
cd GPUst---GPU-Task-Manager/gpust
npm install
npm run tauri dev
```

> The app lives in the `gpust/` subfolder of the repository.

### From release

1. Download the latest `gpust_x.y.z_x64-setup.exe` from [Releases](https://github.com/ArcademMan/GPUst---GPU-Task-Manager/releases)
2. Run the installer
3. Launch **GPUst**

> Installers are unsigned, so Windows SmartScreen may warn ("Unknown publisher") —
> choose *More info → Run anyway*. Terminating some processes may require running
> GPUst as **Administrator**.

## Build

To produce the Windows installers:

```bash
cd gpust
npm run tauri build
```

Output: `gpust/src-tauri/target/release/bundle/` (`nsis/…-setup.exe` and `msi/….msi`).

## How it works

Three data sources, layered so the app stays **portable across GPU vendors**:

- **PDH** *(all vendors)* — Windows Performance Counters give per-process GPU
  utilization and dedicated VRAM, aggregated per process with a moving average.
- **DXGI** *(all vendors)* — enumerates every adapter with its name and total VRAM.
- **NVML** *(NVIDIA only, optional)* — adds temperature, clocks, power, fan and
  encoder/decoder telemetry. Absent on AMD/Intel? The app degrades gracefully.

## License

[MIT](LICENSE) © ArcademMan
