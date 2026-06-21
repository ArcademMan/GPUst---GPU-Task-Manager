<script setup lang="ts">
import { computed } from "vue";
import type { DeviceInfo } from "../gpu";

const props = defineProps<{ devices: DeviceInfo[] }>();

const list = computed(() => props.devices);

function gb(mb: number): string {
  if (mb <= 0) return "0";
  return mb >= 1024 ? `${(mb / 1024).toFixed(1)} GB` : `${Math.round(mb)} MB`;
}
function vramPct(d: DeviceInfo): number {
  return d.vram_total_mb > 0 ? (d.vram_used_mb / d.vram_total_mb) * 100 : 0;
}
function tempClass(t: number | null): string {
  if (t === null) return "";
  if (t >= 83) return "hot";
  if (t >= 70) return "warm";
  return "cool";
}

// gauge geometry
const R = 30;
const C = 2 * Math.PI * R;
function dash(pct: number | null): string {
  const p = Math.max(0, Math.min(100, pct ?? 0));
  return `${(p / 100) * C} ${C}`;
}

function vendorClass(v: string): string {
  return v.toLowerCase();
}
</script>

<template>
  <div v-if="list.length" class="dash">
    <div v-for="(d, i) in list" :key="i" class="dev" :class="{ wide: d.util_gpu !== null }">
      <div class="head">
        <span class="badge" :class="vendorClass(d.vendor)">{{ d.vendor }}</span>
        <span class="dname" :title="d.name">{{ d.name }}</span>
      </div>

      <div class="body">
        <!-- Gauge util (solo dove disponibile, es. NVIDIA) -->
        <div v-if="d.util_gpu !== null" class="gauge">
          <svg viewBox="0 0 72 72">
            <circle class="g-track" cx="36" cy="36" :r="R" />
            <circle
              class="g-fill"
              cx="36"
              cy="36"
              :r="R"
              :stroke-dasharray="dash(d.util_gpu)"
              transform="rotate(-90 36 36)"
            />
          </svg>
          <div class="g-label">
            <div class="g-val">{{ d.util_gpu }}<span>%</span></div>
            <div class="g-sub">GPU</div>
          </div>
        </div>

        <div class="meters">
          <!-- VRAM (sempre presente) -->
          <div class="meter">
            <div class="m-top">
              <span>VRAM</span>
              <span class="m-num">{{ gb(d.vram_used_mb) }} / {{ gb(d.vram_total_mb) }}</span>
            </div>
            <div class="bar"><div class="fill vram" :style="{ width: vramPct(d) + '%' }" /></div>
          </div>

          <!-- Chip telemetria (solo i campi disponibili) -->
          <div class="chips">
            <div v-if="d.temp_c !== null" class="chip" :class="tempClass(d.temp_c)">
              <span class="c-ico">🌡</span>{{ d.temp_c }}°C
            </div>
            <div
              v-if="d.power_w !== null"
              class="chip"
              title="Assorbimento attuale / limite di potenza (TDP) imposto alla scheda"
            >
              <span class="c-ico">⚡</span>{{ Math.round(d.power_w)
              }}<template v-if="d.power_limit_w"> / {{ Math.round(d.power_limit_w) }}</template> W
            </div>
            <div v-if="d.clock_graphics !== null" class="chip">
              <span class="c-ico">⏱</span>{{ d.clock_graphics }} MHz
            </div>
            <div v-if="d.fan_pct !== null" class="chip">
              <span class="c-ico">❄</span>{{ d.fan_pct }}%
            </div>
            <div v-if="d.encoder_pct !== null" class="chip">ENC {{ d.encoder_pct }}%</div>
            <div v-if="d.decoder_pct !== null" class="chip">DEC {{ d.decoder_pct }}%</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dash {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
}
.dev {
  flex: 1 1 320px;
  min-width: 280px;
  background: var(--bg-elev);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px;
}
.dev.wide {
  flex: 1 1 420px;
}
.head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}
.badge {
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.4px;
  padding: 3px 7px;
  border-radius: 6px;
  background: var(--bg-elev-2);
  color: var(--text-dim);
  border: 1px solid var(--border);
}
.badge.nvidia {
  color: #76b900;
  border-color: color-mix(in srgb, #76b900 45%, var(--border));
}
.badge.amd {
  color: #ed1c24;
  border-color: color-mix(in srgb, #ed1c24 45%, var(--border));
}
.badge.intel {
  color: #0071c5;
  border-color: color-mix(in srgb, #0071c5 45%, var(--border));
}
.dname {
  font-weight: 700;
  font-size: 13.5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.body {
  display: flex;
  align-items: center;
  gap: 16px;
}
.gauge {
  position: relative;
  width: 86px;
  height: 86px;
  flex-shrink: 0;
}
.gauge svg {
  width: 86px;
  height: 86px;
}
.g-track {
  fill: none;
  stroke: var(--bar-track);
  stroke-width: 8;
}
.g-fill {
  fill: none;
  stroke: var(--accent);
  stroke-width: 8;
  stroke-linecap: round;
  transition: stroke-dasharray 0.5s ease;
}
.g-label {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}
.g-val {
  font-size: 20px;
  font-weight: 800;
  font-variant-numeric: tabular-nums;
  line-height: 1;
}
.g-val span {
  font-size: 11px;
  color: var(--text-dim);
}
.g-sub {
  font-size: 10px;
  color: var(--text-faint);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.meters {
  flex: 1;
  min-width: 0;
}
.meter {
  margin-bottom: 10px;
}
.m-top {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  color: var(--text-dim);
  margin-bottom: 5px;
}
.m-num {
  font-variant-numeric: tabular-nums;
  color: var(--text);
  font-weight: 600;
}
.bar {
  height: 8px;
  border-radius: 6px;
  background: var(--bar-track);
  overflow: hidden;
}
.fill {
  height: 100%;
  border-radius: 6px;
  transition: width 0.5s ease;
}
.fill.vram {
  background: linear-gradient(90deg, var(--accent), var(--ok));
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.chip {
  font-size: 11.5px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  padding: 4px 8px;
  border-radius: 6px;
  background: var(--bg);
  border: 1px solid var(--border);
  color: var(--text-dim);
  display: flex;
  align-items: center;
  gap: 4px;
}
.c-ico {
  font-size: 11px;
}
.chip.cool {
  color: var(--ok);
}
.chip.warm {
  color: var(--warn);
}
.chip.hot {
  color: var(--danger);
  border-color: color-mix(in srgb, var(--danger) 45%, var(--border));
}
</style>
