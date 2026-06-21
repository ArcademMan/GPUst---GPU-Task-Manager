//! Cuore del progetto: lettura dell'uso GPU per-processo tramite i
//! Performance Counters di Windows (PDH), categoria `GPU Engine` +
//! `GPU Process Memory`.
//!
//! Strategia (leggera e "vera"): si apre UNA query PDH persistente e si
//! campiona su un thread in background ~1s. I valori grezzi sono ballerini,
//! quindi si tiene una finestra mobile di N campioni e si espone la media.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::Serialize;
use windows::core::w;
use windows::Win32::System::Performance::{
    PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW,
    PdhOpenQueryW, PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY,
    PDH_MORE_DATA,
};

/// Numero di campioni nella finestra mobile (~5s a 1 campione/s).
const WINDOW: usize = 5;
/// Intervallo di campionamento.
const SAMPLE_INTERVAL: Duration = Duration::from_millis(1000);

/// Breakdown per tipo di engine GPU.
#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct Engines {
    pub d3d: f64,
    pub copy: f64,
    pub video_decode: f64,
    pub video_encode: f64,
    pub other: f64,
}

/// Riga di un processo con uso GPU (valori mediati).
#[derive(Debug, Clone, Serialize)]
pub struct ProcSample {
    pub pid: u32,
    pub gpu_pct: f64,
    pub vram_mb: f64,
    pub engines: Engines,
}

/// Snapshot grezzo di un singolo campione: pid -> accumulo.
type RawSnapshot = HashMap<u32, RawProc>;

#[derive(Debug, Default, Clone)]
struct RawProc {
    vram_mb: f64,
    engines: Engines,
}

impl Engines {
    /// % GPU "stile Task Manager": il motore più attivo, non la somma.
    fn max(&self) -> f64 {
        self.d3d
            .max(self.copy)
            .max(self.video_decode)
            .max(self.video_encode)
            .max(self.other)
    }
}

/// Stato condiviso: ultima media calcolata.
#[derive(Default)]
pub struct GpuState {
    pub samples: Mutex<Vec<ProcSample>>,
    /// VRAM dedicata usata per adapter, chiave = LUID normalizzato
    /// (es. "0X00000000_0X0000D1E5"). Somma dei processi su quella scheda.
    pub vram_by_luid: Mutex<HashMap<String, f64>>,
}

impl GpuState {
    pub fn snapshot(&self) -> Vec<ProcSample> {
        self.samples.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// VRAM usata (MB) per un dato LUID, se nota dai contatori PDH.
    pub fn vram_for_luid(&self, luid: &str) -> Option<f64> {
        self.vram_by_luid
            .lock()
            .ok()
            .and_then(|m| m.get(luid).copied())
    }
}

/// Avvia il thread di campionamento PDH. Restituisce lo stato condiviso.
pub fn start(state: Arc<GpuState>) {
    thread::spawn(move || {
        if let Err(e) = sampler_loop(state) {
            eprintln!("[gpu] sampler terminato: {e}");
        }
    });
}

fn sampler_loop(state: Arc<GpuState>) -> windows::core::Result<()> {
    unsafe {
        let mut query = PDH_HQUERY::default();
        check(PdhOpenQueryW(None, 0, &mut query), "PdhOpenQueryW")?;

        let mut util = PDH_HCOUNTER::default();
        check(
            PdhAddEnglishCounterW(
                query,
                w!("\\GPU Engine(*)\\Utilization Percentage"),
                0,
                &mut util,
            ),
            "PdhAddEnglishCounterW(util)",
        )?;

        let mut mem = PDH_HCOUNTER::default();
        check(
            PdhAddEnglishCounterW(
                query,
                w!("\\GPU Process Memory(*)\\Dedicated Usage"),
                0,
                &mut mem,
            ),
            "PdhAddEnglishCounterW(mem)",
        )?;

        // Priming: il primo collect serve a fissare la baseline.
        let _ = PdhCollectQueryData(query);

        let mut history: Vec<RawSnapshot> = Vec::with_capacity(WINDOW);

        loop {
            thread::sleep(SAMPLE_INTERVAL);

            if PdhCollectQueryData(query) != 0 {
                continue;
            }

            let mut snap: RawSnapshot = HashMap::new();

            // Utilization Percentage per engine.
            for (instance, value) in read_array(util)? {
                if let Some(pid) = parse_pid(&instance) {
                    let entry = snap.entry(pid).or_default();
                    match parse_engtype(&instance) {
                        EngType::D3d => entry.engines.d3d += value,
                        EngType::Copy => entry.engines.copy += value,
                        EngType::VideoDecode => entry.engines.video_decode += value,
                        EngType::VideoEncode => entry.engines.video_encode += value,
                        EngType::Other => entry.engines.other += value,
                    }
                }
            }

            // Dedicated Usage (bytes) per processo e per adapter (LUID).
            let mut vram_luid: HashMap<String, f64> = HashMap::new();
            for (instance, value) in read_array(mem)? {
                let mb = value / (1024.0 * 1024.0);
                if let Some(pid) = parse_pid(&instance) {
                    snap.entry(pid).or_default().vram_mb += mb;
                }
                if let Some(luid) = parse_luid(&instance) {
                    *vram_luid.entry(luid).or_default() += mb;
                }
            }
            if let Ok(mut guard) = state.vram_by_luid.lock() {
                *guard = vram_luid;
            }

            // Finestra mobile.
            history.push(snap);
            if history.len() > WINDOW {
                history.remove(0);
            }

            let averaged = average(&history);
            if let Ok(mut guard) = state.samples.lock() {
                *guard = averaged;
            }
        }
    }
}

/// Media dei valori sulla finestra. VRAM e nome dal campione più recente,
/// percentuali ed engine mediati sul numero di campioni.
fn average(history: &[RawSnapshot]) -> Vec<ProcSample> {
    let n = history.len().max(1) as f64;
    let mut acc: HashMap<u32, RawProc> = HashMap::new();

    for snap in history {
        for (&pid, raw) in snap {
            let e = acc.entry(pid).or_default();
            e.engines.d3d += raw.engines.d3d;
            e.engines.copy += raw.engines.copy;
            e.engines.video_decode += raw.engines.video_decode;
            e.engines.video_encode += raw.engines.video_encode;
            e.engines.other += raw.engines.other;
        }
    }

    // VRAM: prendi l'ultimo valore noto (non si media bene, è uno stato).
    if let Some(last) = history.last() {
        for (&pid, raw) in last {
            if let Some(e) = acc.get_mut(&pid) {
                e.vram_mb = raw.vram_mb;
            }
        }
    }

    let mut out: Vec<ProcSample> = acc
        .into_iter()
        .map(|(pid, r)| {
            let engines = Engines {
                d3d: r.engines.d3d / n,
                copy: r.engines.copy / n,
                video_decode: r.engines.video_decode / n,
                video_encode: r.engines.video_encode / n,
                other: r.engines.other / n,
            };
            ProcSample {
                pid,
                // % GPU = engine più attivo (comparabile con la pagina GPU).
                gpu_pct: engines.max(),
                vram_mb: r.vram_mb,
                engines,
            }
        })
        .collect();

    out.sort_by(|a, b| b.gpu_pct.partial_cmp(&a.gpu_pct).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// Legge un counter formattato come array (instance name -> valore double).
unsafe fn read_array(counter: PDH_HCOUNTER) -> windows::core::Result<Vec<(String, f64)>> {
    let mut buffer_size: u32 = 0;
    let mut item_count: u32 = 0;

    let status = PdhGetFormattedCounterArrayW(
        counter,
        PDH_FMT_DOUBLE,
        &mut buffer_size,
        &mut item_count,
        None,
    );
    if status != PDH_MORE_DATA {
        // Nessun dato disponibile in questo ciclo: non è fatale.
        return Ok(Vec::new());
    }

    let mut buf: Vec<u8> = vec![0u8; buffer_size as usize];
    let items_ptr = buf.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;

    let status = PdhGetFormattedCounterArrayW(
        counter,
        PDH_FMT_DOUBLE,
        &mut buffer_size,
        &mut item_count,
        Some(items_ptr),
    );
    if status != 0 {
        return Ok(Vec::new());
    }

    let items = std::slice::from_raw_parts(items_ptr, item_count as usize);
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        // CStatus valido == 0; salta i NaN/invalidi.
        let value = item.FmtValue.Anonymous.doubleValue;
        if !value.is_finite() {
            continue;
        }
        let name = item.szName.to_string().unwrap_or_default();
        out.push((name, value));
    }
    Ok(out)
}

fn check(status: u32, what: &str) -> windows::core::Result<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(windows::core::Error::new(
            windows::core::HRESULT(status as i32),
            format!("{what} fallito (0x{status:08X})"),
        ))
    }
}

/// Estrae il PID da un instance name tipo `pid_1234_luid_..._engtype_3D`.
fn parse_pid(instance: &str) -> Option<u32> {
    let idx = instance.find("pid_")? + 4;
    let rest = &instance[idx..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

/// Estrae il LUID normalizzato da un instance name del contatore memoria,
/// es. `pid_..._luid_0x00000000_0x0000D1E5_phys_0` -> `0X00000000_0X0000D1E5`.
fn parse_luid(instance: &str) -> Option<String> {
    let idx = instance.find("luid_")? + 5;
    let rest = &instance[idx..];
    let end = rest.find("_phys").unwrap_or(rest.len());
    Some(rest[..end].to_uppercase())
}

enum EngType {
    D3d,
    Copy,
    VideoDecode,
    VideoEncode,
    Other,
}

fn parse_engtype(instance: &str) -> EngType {
    match instance.rsplit("engtype_").next() {
        Some(t) if t.eq_ignore_ascii_case("3D") => EngType::D3d,
        Some(t) if t.eq_ignore_ascii_case("Copy") => EngType::Copy,
        Some(t) if t.eq_ignore_ascii_case("VideoDecode") => EngType::VideoDecode,
        Some(t) if t.eq_ignore_ascii_case("VideoEncode") => EngType::VideoEncode,
        _ => EngType::Other,
    }
}

/// Helper: chiude la query (non usato nel loop infinito, ma utile per test).
#[allow(dead_code)]
unsafe fn close(query: PDH_HQUERY) {
    let _ = PdhCloseQuery(query);
}
