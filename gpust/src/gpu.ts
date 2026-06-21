import { ref, type Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface Engines {
  d3d: number;
  copy: number;
  video_decode: number;
  video_encode: number;
  other: number;
}

export interface ProcInfo {
  pid: number;
  name: string;
  exe: string | null;
  gpu_pct: number;
  vram_mb: number;
  engines: Engines;
}

export interface DeviceInfo {
  name: string;
  vendor: string;
  vram_used_mb: number;
  vram_total_mb: number;
  util_gpu: number | null;
  util_mem: number | null;
  temp_c: number | null;
  power_w: number | null;
  power_limit_w: number | null;
  clock_graphics: number | null;
  clock_mem: number | null;
  fan_pct: number | null;
  encoder_pct: number | null;
  decoder_pct: number | null;
}

const HISTORY_LEN = 60;

export interface GpuFeed {
  procs: Ref<ProcInfo[]>;
  devices: Ref<DeviceInfo[]>;
  /** reattivo: ogni tick rimpiazza Map e array, così le sparkline si aggiornano */
  history: Ref<Map<number, number[]>>;
  error: Ref<string | null>;
  start: (ms: number) => void;
  stop: () => void;
  setIntervalMs: (ms: number) => void;
}

export function useGpu(): GpuFeed {
  const procs = ref<ProcInfo[]>([]);
  const devices = ref<DeviceInfo[]>([]);
  const error = ref<string | null>(null);
  const history = ref<Map<number, number[]>>(new Map());
  let timer: number | null = null;
  let curMs = 1000;

  async function tick() {
    try {
      const [data, devs] = await Promise.all([
        invoke<ProcInfo[]>("get_gpu_usage"),
        invoke<DeviceInfo[]>("get_devices"),
      ]);
      procs.value = data;
      devices.value = devs;
      error.value = null;

      const next = new Map(history.value);
      const seen = new Set<number>();
      for (const p of data) {
        seen.add(p.pid);
        const prev = next.get(p.pid) ?? [];
        // nuovo array (reference diversa) => le sparkline ricalcolano
        const arr = prev.concat(p.gpu_pct);
        if (arr.length > HISTORY_LEN) arr.shift();
        next.set(p.pid, arr);
      }
      for (const pid of [...next.keys()]) {
        if (!seen.has(pid)) next.delete(pid);
      }
      history.value = next;
    } catch (e) {
      error.value = String(e);
    }
  }

  function start(ms: number) {
    curMs = ms;
    if (timer !== null) clearInterval(timer);
    tick();
    timer = window.setInterval(tick, curMs);
  }

  function stop() {
    if (timer !== null) {
      clearInterval(timer);
      timer = null;
    }
  }

  function setIntervalMs(ms: number) {
    if (ms !== curMs) start(ms);
  }

  return { procs, devices, history, error, start, stop, setIntervalMs };
}
