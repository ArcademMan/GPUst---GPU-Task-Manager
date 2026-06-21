//! Telemetria a livello di *dispositivo* (la scheda, non i processi).
//!
//! Due livelli, per restare portabile su qualunque PC:
//!  - **DXGI** (vendor-agnostic): elenca tutte le GPU con nome e VRAM
//!    usata/totale. Funziona su NVIDIA, AMD, Intel.
//!  - **NVML** (solo NVIDIA, opzionale): aggiunge temperatura, clock,
//!    consumo, ventola, util ed encoder/decoder. Se assente, si degrada.

use serde::Serialize;

use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};
use nvml_wrapper::Nvml;

use crate::gpu::GpuState;
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
};

const MB: f64 = 1024.0 * 1024.0;

/// Una GPU con i dati disponibili. I campi opzionali esistono solo dove
/// il vendor li espone (oggi: NVIDIA via NVML).
#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    pub name: String,
    pub vendor: String,
    pub vram_used_mb: f64,
    pub vram_total_mb: f64,
    // Extra NVIDIA (None se non disponibili)
    pub util_gpu: Option<u32>,
    pub util_mem: Option<u32>,
    pub temp_c: Option<u32>,
    pub power_w: Option<f64>,
    pub power_limit_w: Option<f64>,
    pub clock_graphics: Option<u32>,
    pub clock_mem: Option<u32>,
    pub fan_pct: Option<u32>,
    pub encoder_pct: Option<u32>,
    pub decoder_pct: Option<u32>,
}

fn vendor_name(vendor_id: u32) -> &'static str {
    match vendor_id {
        0x10DE => "NVIDIA",
        0x1002 | 0x1022 => "AMD",
        0x8086 => "Intel",
        0x1414 => "Microsoft",
        _ => "GPU",
    }
}

/// Inizializza NVML (None se nessuna NVIDIA / libreria assente).
pub fn init_nvml() -> Option<Nvml> {
    Nvml::init().ok()
}

/// Lettura combinata DXGI + NVML + PDH (VRAM usata per adapter).
pub fn read(nvml: Option<&Nvml>, gpu: &GpuState) -> Vec<DeviceInfo> {
    let nvml_readings = nvml.map(read_nvml).unwrap_or_default();
    let mut used = vec![false; nvml_readings.len()];

    let mut out = Vec::new();

    unsafe {
        let factory: IDXGIFactory1 = match CreateDXGIFactory1() {
            Ok(f) => f,
            Err(_) => return fallback_from_nvml(nvml_readings),
        };

        let mut i = 0u32;
        loop {
            let adapter = match factory.EnumAdapters1(i) {
                Ok(a) => a,
                Err(_) => break, // DXGI_ERROR_NOT_FOUND: fine elenco
            };
            i += 1;

            let desc = match adapter.GetDesc1() {
                Ok(d) => d,
                Err(_) => continue,
            };

            // Salta l'adapter software (WARP / Basic Render Driver).
            if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0 {
                continue;
            }

            let name = utf16_to_string(&desc.Description);
            let vendor = vendor_name(desc.VendorId);
            let vram_total_mb = desc.DedicatedVideoMemory as f64 / MB;

            // VRAM usata (vendor-agnostic): somma PDH per LUID di questo adapter.
            let luid_key = format!(
                "0x{:08X}_0x{:08X}",
                desc.AdapterLuid.HighPart as u32, desc.AdapterLuid.LowPart
            )
            .to_uppercase();
            let vram_used_mb = gpu.vram_for_luid(&luid_key).unwrap_or(0.0);

            let mut dev = DeviceInfo {
                name: name.clone(),
                vendor: vendor.to_string(),
                vram_used_mb,
                vram_total_mb,
                util_gpu: None,
                util_mem: None,
                temp_c: None,
                power_w: None,
                power_limit_w: None,
                clock_graphics: None,
                clock_mem: None,
                fan_pct: None,
                encoder_pct: None,
                decoder_pct: None,
            };

            // Arricchimento NVIDIA: abbina la lettura NVML per nome,
            // altrimenti la prima ancora libera.
            if vendor == "NVIDIA" && !nvml_readings.is_empty() {
                let mut chosen = nvml_readings
                    .iter()
                    .enumerate()
                    .find(|(k, r)| !used[*k] && r.name == name)
                    .map(|(k, _)| k);
                if chosen.is_none() {
                    chosen = used.iter().position(|u| !*u);
                }
                if let Some(k) = chosen {
                    used[k] = true;
                    let r = &nvml_readings[k];
                    dev.util_gpu = Some(r.util_gpu);
                    dev.util_mem = Some(r.util_mem);
                    dev.temp_c = r.temp_c;
                    dev.power_w = r.power_w;
                    dev.power_limit_w = r.power_limit_w;
                    dev.clock_graphics = r.clock_graphics;
                    dev.clock_mem = r.clock_mem;
                    dev.fan_pct = r.fan_pct;
                    dev.encoder_pct = r.encoder_pct;
                    dev.decoder_pct = r.decoder_pct;
                    // VRAM da NVML: globale e più accurata (include il driver).
                    if r.vram_used_mb > 0.0 {
                        dev.vram_used_mb = r.vram_used_mb;
                    }
                    if r.vram_total_mb > 0.0 {
                        dev.vram_total_mb = r.vram_total_mb;
                    }
                }
            }

            out.push(dev);
        }
    }

    // Se per qualche motivo DXGI non ha prodotto nulla ma NVML sì.
    if out.is_empty() {
        return fallback_from_nvml(nvml_readings);
    }

    out
}

#[derive(Clone)]
struct NvmlReading {
    name: String,
    vram_total_mb: f64,
    vram_used_mb: f64,
    util_gpu: u32,
    util_mem: u32,
    temp_c: Option<u32>,
    power_w: Option<f64>,
    power_limit_w: Option<f64>,
    clock_graphics: Option<u32>,
    clock_mem: Option<u32>,
    fan_pct: Option<u32>,
    encoder_pct: Option<u32>,
    decoder_pct: Option<u32>,
}

fn read_nvml(nvml: &Nvml) -> Vec<NvmlReading> {
    let mut out = Vec::new();
    let count = nvml.device_count().unwrap_or(0);
    for idx in 0..count {
        let Ok(dev) = nvml.device_by_index(idx) else {
            continue;
        };
        let util = dev.utilization_rates().ok();
        let mem = dev.memory_info().ok();
        out.push(NvmlReading {
            name: dev.name().unwrap_or_else(|_| "NVIDIA GPU".into()),
            vram_total_mb: mem.as_ref().map(|m| m.total as f64 / MB).unwrap_or(0.0),
            vram_used_mb: mem.as_ref().map(|m| m.used as f64 / MB).unwrap_or(0.0),
            util_gpu: util.as_ref().map(|u| u.gpu).unwrap_or(0),
            util_mem: util.as_ref().map(|u| u.memory).unwrap_or(0),
            temp_c: dev.temperature(TemperatureSensor::Gpu).ok(),
            power_w: dev.power_usage().ok().map(|mw| mw as f64 / 1000.0),
            power_limit_w: dev.enforced_power_limit().ok().map(|mw| mw as f64 / 1000.0),
            clock_graphics: dev.clock_info(Clock::Graphics).ok(),
            clock_mem: dev.clock_info(Clock::Memory).ok(),
            fan_pct: dev.fan_speed(0).ok(),
            encoder_pct: dev.encoder_utilization().ok().map(|e| e.utilization),
            decoder_pct: dev.decoder_utilization().ok().map(|d| d.utilization),
        });
    }
    out
}

/// Costruisce DeviceInfo solo dalle letture NVML (se DXGI non disponibile).
fn fallback_from_nvml(readings: Vec<NvmlReading>) -> Vec<DeviceInfo> {
    readings
        .into_iter()
        .map(|r| DeviceInfo {
            name: r.name,
            vendor: "NVIDIA".to_string(),
            vram_used_mb: r.vram_used_mb,
            vram_total_mb: r.vram_total_mb,
            util_gpu: Some(r.util_gpu),
            util_mem: Some(r.util_mem),
            temp_c: r.temp_c,
            power_w: r.power_w,
            power_limit_w: r.power_limit_w,
            clock_graphics: r.clock_graphics,
            clock_mem: r.clock_mem,
            fan_pct: r.fan_pct,
            encoder_pct: r.encoder_pct,
            decoder_pct: r.decoder_pct,
        })
        .collect()
}

fn utf16_to_string(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}
