//! Comandi esposti al frontend (Tauri `invoke`).

use std::sync::{Arc, Mutex};

use serde::Serialize;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use tauri::State;

use crate::devices::{self, DeviceInfo};
use crate::gpu::{Engines, GpuState};
use nvml_wrapper::Nvml;

/// Processi di sistema critici da NON terminare mai (guardia di sicurezza).
const PROTECTED: &[&str] = &[
    "dwm", "csrss", "system", "idle", "wininit", "winlogon", "services", "smss",
    "lsass", "registry", "memory compression", "system idle process",
];

/// Riga inviata al frontend.
#[derive(Debug, Clone, Serialize)]
pub struct ProcInfo {
    pub pid: u32,
    pub name: String,
    pub exe: Option<String>,
    pub gpu_pct: f64,
    pub vram_mb: f64,
    pub engines: Engines,
}

/// Stato applicativo condiviso.
pub struct AppState {
    pub gpu: Arc<GpuState>,
    pub sys: Mutex<System>,
    /// NVML è opzionale: presente solo con GPU NVIDIA + driver.
    pub nvml: Mutex<Option<Nvml>>,
}

/// Telemetria a livello di dispositivo (tutte le GPU; extra NVIDIA dove c'è).
#[tauri::command]
pub fn get_devices(state: State<AppState>) -> Vec<DeviceInfo> {
    let nvml = state.nvml.lock().expect("nvml mutex");
    devices::read(nvml.as_ref(), &state.gpu)
}

/// Ritorna i processi con uso GPU (valori già mediati lato backend),
/// arricchiti con nome e path eseguibile.
#[tauri::command]
pub fn get_gpu_usage(state: State<AppState>) -> Vec<ProcInfo> {
    let samples = state.gpu.snapshot();

    let mut sys = state.sys.lock().expect("sys mutex");
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
    );

    samples
        .into_iter()
        .filter(|s| s.gpu_pct > 0.0 || s.vram_mb > 0.0)
        .map(|s| {
            let proc = sys.process(Pid::from_u32(s.pid));
            let name = proc
                .map(|p| p.name().to_string_lossy().to_string())
                .unwrap_or_else(|| format!("PID {}", s.pid));
            let exe = proc
                .and_then(|p| p.exe())
                .map(|p| p.to_string_lossy().to_string());
            ProcInfo {
                pid: s.pid,
                name,
                exe,
                gpu_pct: s.gpu_pct,
                vram_mb: s.vram_mb,
                engines: s.engines,
            }
        })
        .collect()
}

/// Ritorna l'icona di un eseguibile come data URL PNG (con cache lato Rust).
#[tauri::command]
pub fn get_process_icon(exe: String) -> Option<String> {
    crate::icons::icon_for(&exe)
}

/// Termina un processo con guardia di sicurezza sui processi di sistema.
/// `force` (modalità avanzata) rimuove la protezione applicativa; Windows
/// può comunque rifiutare la chiusura di alcuni processi a livello kernel.
#[tauri::command]
pub fn kill_process(state: State<AppState>, pid: u32, force: bool) -> Result<(), String> {
    let sys = state.sys.lock().expect("sys mutex");
    let proc = sys
        .process(Pid::from_u32(pid))
        .ok_or_else(|| format!("Processo {pid} non trovato"))?;

    let name = proc.name().to_string_lossy().to_lowercase();
    let stem = name.strip_suffix(".exe").unwrap_or(&name);
    if !force && PROTECTED.iter().any(|p| *p == stem || *p == name) {
        return Err(format!(
            "'{}' è un processo di sistema protetto. Abilita la modalità avanzata nelle impostazioni per forzarne la chiusura (sconsigliato).",
            proc.name().to_string_lossy()
        ));
    }

    if proc.kill() {
        Ok(())
    } else {
        Err(format!("Impossibile terminare il processo {pid}"))
    }
}
