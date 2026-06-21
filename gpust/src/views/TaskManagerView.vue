<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Sparkline from "../components/Sparkline.vue";
import type { ProcInfo } from "../gpu";
import { useIcons } from "../useIcons";
import { settings } from "../useSettings";

const props = defineProps<{
  procs: ProcInfo[];
  history: Map<number, number[]>;
  error: string | null;
}>();

const { ensure, iconFor } = useIcons();
watch(
  () => props.procs,
  (list) => {
    for (const p of list) ensure(p.exe);
  },
  { immediate: true }
);

// --- Ordinamento ---
type SortKey = "name" | "gpu" | "vram";
const sortKey = ref<SortKey>("gpu");
const sortDir = ref<1 | -1>(-1);
function setSort(k: SortKey) {
  if (sortKey.value === k) {
    sortDir.value = (sortDir.value * -1) as 1 | -1;
  } else {
    sortKey.value = k;
    sortDir.value = k === "name" ? 1 : -1;
  }
}
function arrow(k: SortKey): string {
  if (sortKey.value !== k) return "";
  return sortDir.value === 1 ? "▲" : "▼";
}

const sorted = computed(() => {
  const list = [...props.procs];
  const dir = sortDir.value;
  list.sort((a, b) => {
    let r = 0;
    if (sortKey.value === "name") r = a.name.localeCompare(b.name);
    else if (sortKey.value === "gpu") r = a.gpu_pct - b.gpu_pct;
    else r = a.vram_mb - b.vram_mb;
    return r * dir;
  });
  return list;
});

// --- Selezione / dettaglio ---
const selectedPid = ref<number | null>(null);
const selected = computed(
  () => props.procs.find((p) => p.pid === selectedPid.value) ?? null
);
const selectedHistory = computed(() =>
  selectedPid.value !== null ? props.history.get(selectedPid.value) ?? [] : []
);
function selectRow(pid: number) {
  selectedPid.value = pid;
  confirmKill.value = false;
  killError.value = null;
}
function closeDetail() {
  selectedPid.value = null;
}

// --- Kill ---
const confirmKill = ref(false);
const killError = ref<string | null>(null);
async function doKill(pid: number) {
  killError.value = null;
  try {
    await invoke("kill_process", { pid, force: settings.allowKillSystem });
    confirmKill.value = false;
    selectedPid.value = null;
  } catch (e) {
    killError.value = String(e);
  }
}

// --- Helper formato ---
function fmtPct(v: number): string {
  return v < 0.05 ? "0" : v.toFixed(1);
}
function fmtVram(mb: number): string {
  if (mb <= 0) return "—";
  return mb >= 1024 ? `${(mb / 1024).toFixed(1)} GB` : `${Math.round(mb)} MB`;
}
function initials(p: ProcInfo): string {
  return p.name.replace(/\.exe$/i, "").slice(0, 2).toUpperCase();
}

const ENGINE_INFO: Record<string, string> = {
  "3D": "Motore grafico 3D: rendering e calcoli generali sulla GPU (giochi, UI accelerata, compute).",
  "Video Decode": "Decodifica video hardware: riproduzione di video/stream senza caricare la CPU.",
  "Video Encode": "Codifica video hardware: registrazione e streaming (es. OBS, screen sharing).",
  Copy: "Motore di copia (DMA): trasferimenti di memoria da/verso la GPU.",
  Altro: "Altri motori GPU (Compute, VideoProcessing, ecc.).",
};

const engineRows = computed(() => {
  if (!selected.value) return [];
  const e = selected.value.engines;
  return [
    { label: "3D", value: e.d3d, color: "var(--accent)" },
    { label: "Video Decode", value: e.video_decode, color: "var(--ok)" },
    { label: "Video Encode", value: e.video_encode, color: "var(--warn)" },
    { label: "Copy", value: e.copy, color: "var(--text-dim)" },
    { label: "Altro", value: e.other, color: "var(--text-faint)" },
  ].filter((r) => r.value > 0.05);
});
</script>

<template>
  <div class="tm">
    <div class="table-wrap">
      <div class="table">
        <div class="thead">
          <button class="th col-name sortable" @click="setSort('name')">
            Processo <span class="ar">{{ arrow("name") }}</span>
          </button>
          <span class="th col-trend">Andamento</span>
          <button class="th col-vram sortable" @click="setSort('vram')">
            VRAM <span class="ar">{{ arrow("vram") }}</span>
          </button>
          <button class="th col-gpu sortable" @click="setSort('gpu')">
            GPU <span class="ar">{{ arrow("gpu") }}</span>
          </button>
        </div>

        <div class="rows">
          <div v-if="error" class="empty err">{{ error }}</div>
          <div v-else-if="procs.length === 0" class="empty">
            Nessun processo sta usando la GPU in questo momento…
          </div>
          <button
            v-for="p in sorted"
            :key="p.pid"
            class="row"
            :class="{ active: p.pid === selectedPid }"
            @click="selectRow(p.pid)"
          >
            <span class="col-name">
              <span class="avatar">
                <img v-if="iconFor(p.exe)" :src="iconFor(p.exe)!" class="pico" alt="" />
                <template v-else>{{ initials(p) }}</template>
              </span>
              <span class="pname">
                <span class="pname-main">{{ p.name }}</span>
                <span class="pname-pid">PID {{ p.pid }}</span>
              </span>
            </span>
            <span class="col-trend">
              <Sparkline :data="history.get(p.pid) ?? []" :width="120" :height="26" />
            </span>
            <span class="col-vram">{{ fmtVram(p.vram_mb) }}</span>
            <span class="col-gpu">
              <span class="gpu-val">{{ fmtPct(p.gpu_pct) }}%</span>
              <span class="gpu-bar"
                ><span class="gpu-bar-fill" :style="{ width: Math.min(p.gpu_pct, 100) + '%' }"
              /></span>
            </span>
          </button>
        </div>
      </div>
    </div>

    <!-- Slide-over dettaglio -->
    <transition name="slide">
      <aside v-if="selected" class="detail">
        <div class="detail-head">
          <span class="avatar lg">
            <img v-if="iconFor(selected.exe)" :src="iconFor(selected.exe)!" class="pico" alt="" />
            <template v-else>{{ initials(selected) }}</template>
          </span>
          <div class="detail-title">
            <div class="dt-name">{{ selected.name }}</div>
            <div class="dt-pid">PID {{ selected.pid }}</div>
          </div>
          <button class="icon-btn" title="Chiudi" @click="closeDetail">✕</button>
        </div>

        <div class="exe" v-if="selected.exe">{{ selected.exe }}</div>

        <div class="metric-big">
          <div>
            <div class="mb-val">{{ fmtPct(selected.gpu_pct) }}%</div>
            <div class="mb-lab">GPU (media 5s)</div>
          </div>
          <div>
            <div class="mb-val">{{ fmtVram(selected.vram_mb) }}</div>
            <div class="mb-lab">VRAM dedicata</div>
          </div>
        </div>

        <div class="chart-card">
          <Sparkline :data="selectedHistory" :width="280" :height="70" />
        </div>

        <div class="engines" v-if="engineRows.length">
          <div class="sec-title">Engine</div>
          <div v-for="r in engineRows" :key="r.label" class="eng-row" :title="ENGINE_INFO[r.label]">
            <span class="eng-label">{{ r.label }}</span>
            <span class="eng-bar">
              <span class="eng-fill" :style="{ width: Math.min(r.value, 100) + '%', background: r.color }" />
            </span>
            <span class="eng-val">{{ fmtPct(r.value) }}%</span>
          </div>
        </div>

        <div class="kill-zone">
          <div v-if="killError" class="kill-err">{{ killError }}</div>
          <template v-if="!confirmKill">
            <button class="btn-danger" @click="confirmKill = true">Termina processo</button>
          </template>
          <template v-else>
            <div class="confirm">
              <span>Terminare <b>{{ selected.name }}</b>?</span>
              <div class="confirm-actions">
                <button class="btn-ghost" @click="confirmKill = false">Annulla</button>
                <button class="btn-danger" @click="doKill(selected.pid)">Sì, termina</button>
              </div>
            </div>
          </template>
        </div>
      </aside>
    </transition>
  </div>
</template>

<style scoped>
.tm {
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}
.table-wrap {
  flex: 1;
  min-height: 0;
  overflow: auto;
}
.table {
  min-width: 540px;
}
.thead,
.row {
  display: grid;
  grid-template-columns: minmax(180px, 2fr) minmax(96px, 1fr) 96px 104px;
  align-items: center;
  gap: 12px;
  padding: 0 18px;
}
.thead {
  position: sticky;
  top: 0;
  z-index: 2;
  height: 38px;
  background: var(--bg);
  border-bottom: 1px solid var(--border);
}
.th {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-faint);
  background: transparent;
  border: none;
  text-align: left;
  display: flex;
  align-items: center;
  gap: 4px;
  height: 100%;
  padding: 0;
}
.th.sortable {
  cursor: pointer;
}
.th.sortable:hover {
  color: var(--text);
}
.col-vram,
.col-gpu {
  justify-content: flex-end;
  text-align: right;
  justify-self: end;
}
.ar {
  font-size: 8px;
  color: var(--accent);
}

.row {
  width: 100%;
  height: 56px;
  border: none;
  border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
  background: transparent;
  color: var(--text);
  text-align: left;
  transition: background 0.1s ease;
}
.row:hover {
  background: var(--bg-elev);
}
.row.active {
  background: var(--accent-soft);
  box-shadow: inset 3px 0 0 var(--accent);
}
.col-name {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}
.avatar {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  flex-shrink: 0;
  display: grid;
  place-items: center;
  font-size: 11px;
  font-weight: 700;
  color: var(--text);
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
  overflow: hidden;
}
.avatar.lg {
  width: 42px;
  height: 42px;
  font-size: 14px;
}
.pico {
  width: 20px;
  height: 20px;
  object-fit: contain;
}
.avatar.lg .pico {
  width: 28px;
  height: 28px;
}
.pname {
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.pname-main {
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.pname-pid {
  font-size: 11px;
  color: var(--text-faint);
}
.col-vram {
  font-variant-numeric: tabular-nums;
  color: var(--text-dim);
  font-size: 12.5px;
}
.col-gpu {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
}
.gpu-val {
  font-variant-numeric: tabular-nums;
  font-weight: 700;
}
.gpu-bar {
  width: 96px;
  height: 5px;
  border-radius: 4px;
  background: var(--bar-track);
  overflow: hidden;
}
.gpu-bar-fill {
  display: block;
  height: 100%;
  background: var(--accent);
  border-radius: 4px;
  transition: width 0.4s ease;
}
.empty {
  padding: 40px 18px;
  color: var(--text-faint);
  text-align: center;
}
.empty.err {
  color: var(--danger);
}

/* Slide-over */
.detail {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: 330px;
  z-index: 10;
  border-left: 1px solid var(--border);
  background: var(--bg-elev);
  padding: 18px;
  overflow-y: auto;
  box-shadow: var(--shadow);
}
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.22s ease, opacity 0.22s ease;
}
.slide-enter-from,
.slide-leave-to {
  transform: translateX(20px);
  opacity: 0;
}
.detail-head {
  display: flex;
  align-items: center;
  gap: 12px;
}
.detail-title {
  flex: 1;
  min-width: 0;
}
.dt-name {
  font-weight: 700;
  font-size: 15px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.dt-pid {
  font-size: 12px;
  color: var(--text-faint);
}
.icon-btn {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--bg-elev-2);
  color: var(--text-dim);
  font-size: 14px;
}
.icon-btn:hover {
  color: var(--text);
  border-color: var(--accent);
}
.exe {
  margin-top: 10px;
  font-size: 11px;
  color: var(--text-faint);
  word-break: break-all;
  background: var(--bg);
  padding: 6px 8px;
  border-radius: 6px;
}
.metric-big {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  margin: 16px 0;
}
.metric-big > div {
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 12px;
}
.mb-val {
  font-size: 22px;
  font-weight: 800;
  font-variant-numeric: tabular-nums;
}
.mb-lab {
  font-size: 11px;
  color: var(--text-faint);
  margin-top: 2px;
}
.chart-card {
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 12px;
  margin-bottom: 16px;
  overflow: hidden;
}
.sec-title {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-faint);
  margin-bottom: 8px;
}
.eng-row {
  display: grid;
  grid-template-columns: 92px 1fr 48px;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
  font-size: 12px;
  cursor: help;
}
.eng-label {
  color: var(--text-dim);
}
.eng-bar {
  height: 6px;
  border-radius: 4px;
  background: var(--bar-track);
  overflow: hidden;
}
.eng-fill {
  display: block;
  height: 100%;
  border-radius: 4px;
  transition: width 0.4s ease;
}
.eng-val {
  text-align: right;
  font-variant-numeric: tabular-nums;
  color: var(--text-dim);
}
.kill-zone {
  margin-top: 20px;
  border-top: 1px solid var(--border);
  padding-top: 16px;
}
.kill-err {
  color: var(--danger);
  font-size: 12px;
  margin-bottom: 10px;
  background: var(--danger-soft);
  padding: 8px 10px;
  border-radius: 6px;
}
.btn-danger {
  width: 100%;
  padding: 10px;
  border-radius: 8px;
  border: 1px solid var(--danger);
  background: var(--danger-soft);
  color: var(--danger);
  font-weight: 700;
  transition: all 0.12s ease;
}
.btn-danger:hover {
  background: var(--danger);
  color: #fff;
}
.confirm {
  display: flex;
  flex-direction: column;
  gap: 10px;
  font-size: 13px;
}
.confirm-actions {
  display: flex;
  gap: 8px;
}
.confirm-actions .btn-danger {
  width: auto;
  flex: 1;
}
.btn-ghost {
  flex: 1;
  padding: 10px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-dim);
  font-weight: 600;
}
.btn-ghost:hover {
  color: var(--text);
  border-color: var(--text-faint);
}
</style>
