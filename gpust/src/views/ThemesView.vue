<script setup lang="ts">
import { ref } from "vue";
import { THEMES, applyTheme, loadTheme } from "../themes";

const current = ref(loadTheme());
function pick(id: string) {
  current.value = id;
  applyTheme(id);
}
</script>

<template>
  <div class="page">
    <header class="page-head">
      <h1>Temi</h1>
      <p>Scegli l'aspetto di GPUst. La scelta viene ricordata al prossimo avvio.</p>
    </header>

    <div class="grid">
      <button
        v-for="t in THEMES"
        :key="t.id"
        class="card"
        :class="{ active: current === t.id }"
        @click="pick(t.id)"
      >
        <div class="preview" :style="{ background: t.swatch[0] }">
          <div class="p-bar" :style="{ background: t.swatch[1] }" />
          <div class="p-line" :style="{ background: t.swatch[2], opacity: 0.85 }" />
          <div class="p-line short" :style="{ background: t.swatch[2], opacity: 0.5 }" />
          <div class="p-dot" :style="{ background: t.swatch[1] }" />
        </div>
        <div class="card-foot">
          <span class="name">{{ t.name }}</span>
          <span v-if="current === t.id" class="check">✓</span>
        </div>
      </button>
    </div>
  </div>
</template>

<style scoped>
.page {
  height: 100vh;
  overflow-y: auto;
  padding: 28px 32px;
}
.page-head h1 {
  margin: 0;
  font-size: 22px;
  font-weight: 800;
}
.page-head p {
  margin: 6px 0 0;
  color: var(--text-dim);
  font-size: 13px;
}
.grid {
  margin-top: 24px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px;
}
.card {
  border: 2px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-elev);
  padding: 0;
  overflow: hidden;
  transition: all 0.14s ease;
  text-align: left;
}
.card:hover {
  transform: translateY(-2px);
  border-color: var(--text-faint);
}
.card.active {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-soft);
}
.preview {
  height: 110px;
  padding: 16px;
  position: relative;
}
.p-bar {
  height: 8px;
  width: 70%;
  border-radius: 6px;
  margin-bottom: 12px;
}
.p-line {
  height: 6px;
  width: 85%;
  border-radius: 6px;
  margin-bottom: 8px;
}
.p-line.short {
  width: 55%;
}
.p-dot {
  position: absolute;
  right: 16px;
  bottom: 16px;
  width: 22px;
  height: 22px;
  border-radius: 50%;
}
.card-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
}
.name {
  font-weight: 600;
  font-size: 13.5px;
}
.check {
  color: var(--accent);
  font-weight: 800;
}
</style>
