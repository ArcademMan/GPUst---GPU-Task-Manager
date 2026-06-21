# GPUst — GPU Task Manager

A lightweight **GPU-focused task manager for Windows**: see *which processes are
using your GPU* and how much, in real time — with a clean UI, device telemetry,
and one-click process termination.

Built with **Tauri 2 + Rust + Vue 3**. Small binary, low overhead, lives in the
system tray.

> Why it exists: the standard Task Manager makes it awkward to spot what keeps the
> GPU busy at idle. (The original culprit: Discord's hardware acceleration holding
> the GPU at ~10–18% while idle.)

![GPUst screenshot](docs/screenshot.png)
<!-- Add a screenshot at docs/screenshot.png -->

## Features

- **Per-process GPU usage** — utilization, VRAM, and a per-engine breakdown
  (3D / Video Decode / Video Encode / Copy), sortable, with a live trend sparkline.
- **Device dashboard** — per-GPU view of utilization, VRAM used/total, temperature,
  clocks, power draw, fan speed, and encoder/decoder activity.
- **Process icons** — real `.exe` icons extracted from disk.
- **Kill processes** — with a safety guard protecting critical system processes
  (`dwm`, `csrss`, `System`, …) and an optional advanced override.
- **Themes** — multiple built-in themes (Midnight, Graphite, Nord, Dracula, Light),
  remembered across launches.
- **Tray + autostart** — minimize-to-tray, launch on Windows startup, start hidden
  in tray.
- **Adjustable refresh rate.**

## How it works

Three data sources, layered so the app stays **portable across GPU vendors**:

- **PDH (all vendors)** — Windows Performance Counters (`GPU Engine`,
  `GPU Process Memory`) provide per-process utilization and dedicated VRAM. PIDs are
  parsed from the counter instance names and aggregated per process, with a moving
  average to smooth the noisy raw samples.
- **DXGI (all vendors)** — enumerates every adapter with its name and total VRAM.
- **NVML (NVIDIA only, optional)** — adds temperature, clocks, power, fan, and
  encoder/decoder telemetry. If absent (AMD/Intel, or no driver), the app degrades
  gracefully and still shows everything PDH + DXGI provide.

Per-process GPU % is reported as the **busiest engine** (like Windows Task Manager),
so it's comparable with the device-level number.

## Requirements

- **Windows 10/11** (the GPU performance counters are Windows-only)
- [**WebView2 Runtime**](https://developer.microsoft.com/microsoft-edge/webview2/)
  — preinstalled on up-to-date Windows; the installer fetches it if missing
- For development:
  - [Rust](https://rustup.rs/) (MSVC toolchain) + Visual Studio C++ Build Tools
  - [Node.js](https://nodejs.org/) 18+

## Getting started (development)

```bash
git clone https://github.com/ArcademMan/gpust.git
cd gpust/gpust
npm install
npm run tauri dev
```

> The app lives in the `gpust/` subfolder of the repository.

## Building an installer

```bash
cd gpust
npm run tauri build
```

Output lands in `gpust/src-tauri/target/release/bundle/`:

- `nsis/…-setup.exe` — NSIS installer (recommended)
- `msi/….msi` — MSI installer

The release binary is also at `gpust/src-tauri/target/release/gpust.exe`.

> Installers are **unsigned**, so Windows SmartScreen will show an "Unknown
> publisher" prompt — choose *More info → Run anyway*. Code signing requires a
> paid certificate and is optional.

## Project structure

```
.
├─ PROGETTO.md            # original project brief (Italian)
├─ LICENSE                # MIT
└─ gpust/                 # the Tauri app
   ├─ src/                # Vue 3 + TypeScript frontend
   └─ src-tauri/src/      # Rust backend
      ├─ gpu.rs           # PDH per-process sampling + moving average
      ├─ devices.rs       # device telemetry: DXGI + NVML
      ├─ icons.rs         # extract .exe icons → PNG
      ├─ commands.rs      # Tauri commands (get_gpu_usage, get_devices, kill_process)
      └─ lib.rs           # app setup, tray, autostart
```

## Roadmap

- Idle-hog detector + per-app Windows GPU preference (the original Discord use case)
- Live tray icon (current usage) + threshold alerts/notifications
- Longer history & charts ("what used my GPU while I was away")
- AMD/Intel device telemetry where available

## Contributing

Issues and pull requests are welcome. The project targets Windows only.

## License

[MIT](LICENSE) © ArcademMan
