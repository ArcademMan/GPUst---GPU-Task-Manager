<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    data: number[];
    width?: number;
    height?: number;
    max?: number;
    fill?: boolean;
  }>(),
  { width: 120, height: 28, max: 0, fill: true }
);

const path = computed(() => {
  const d = props.data;
  if (d.length < 2) return { line: "", area: "" };
  const w = props.width;
  const h = props.height;
  const pad = 1.5;
  const peak = props.max > 0 ? props.max : Math.max(5, ...d);
  const stepX = (w - pad * 2) / (d.length - 1);
  const pts = d.map((v, i) => {
    const x = pad + i * stepX;
    const y = h - pad - (Math.min(v, peak) / peak) * (h - pad * 2);
    return [x, y] as const;
  });
  const line = pts.map(([x, y], i) => `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`).join(" ");
  const area = `${line} L${pts[pts.length - 1][0].toFixed(1)},${h} L${pts[0][0].toFixed(1)},${h} Z`;
  return { line, area };
});
</script>

<template>
  <svg
    :viewBox="`0 0 ${width} ${height}`"
    preserveAspectRatio="none"
    class="spark"
    :style="{ height: height + 'px' }"
  >
    <path v-if="fill" :d="path.area" class="spark-area" />
    <path :d="path.line" class="spark-line" vector-effect="non-scaling-stroke" />
  </svg>
</template>

<style scoped>
.spark {
  display: block;
  width: 100%;
  max-width: 100%;
}
.spark-line {
  fill: none;
  stroke: var(--accent);
  stroke-width: 1.6;
  stroke-linejoin: round;
  stroke-linecap: round;
}
.spark-area {
  fill: var(--accent-soft);
  stroke: none;
}
</style>
