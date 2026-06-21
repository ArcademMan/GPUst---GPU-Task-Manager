<script setup lang="ts">
defineProps<{ page: string }>();
const emit = defineEmits<{ (e: "navigate", p: string): void }>();

const items = [
  { id: "tasks", label: "Processi", icon: "▦" },
  { id: "gpu", label: "GPU", icon: "⬡" },
  { id: "themes", label: "Temi", icon: "◑" },
  { id: "settings", label: "Impostazioni", icon: "⚙" },
];
</script>

<template>
  <nav class="sidebar">
    <div class="brand">
      <img src="/icon.png" alt="GPUst" class="brand-logo" />
      <div class="brand-text">
        <div class="brand-name">GPUst</div>
        <div class="brand-sub">GPU Task Manager</div>
      </div>
    </div>

    <div class="nav">
      <button
        v-for="it in items"
        :key="it.id"
        class="nav-item"
        :class="{ active: page === it.id }"
        @click="emit('navigate', it.id)"
      >
        <span class="nav-icon">{{ it.icon }}</span>
        <span class="nav-label">{{ it.label }}</span>
      </button>
    </div>

    <div class="foot">v0.1</div>
  </nav>
</template>

<style scoped>
.sidebar {
  width: 196px;
  flex-shrink: 0;
  height: 100vh;
  background: var(--bg-elev);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  padding: 16px 12px;
  gap: 18px;
}
.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 4px 6px;
}
.brand-logo {
  width: 34px;
  height: 34px;
  border-radius: 9px;
  object-fit: contain;
}
.brand-name {
  font-weight: 700;
  font-size: 15px;
  letter-spacing: 0.2px;
}
.brand-sub {
  font-size: 10.5px;
  color: var(--text-faint);
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.nav-item {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 10px 12px;
  border: none;
  border-radius: 9px;
  background: transparent;
  color: var(--text-dim);
  font-size: 13.5px;
  font-weight: 500;
  text-align: left;
  transition: all 0.12s ease;
}
.nav-item:hover {
  background: var(--bg-elev-2);
  color: var(--text);
}
.nav-item.active {
  background: var(--accent-soft);
  color: var(--accent);
  font-weight: 600;
}
.nav-icon {
  width: 18px;
  text-align: center;
  font-size: 15px;
}
.foot {
  margin-top: auto;
  padding: 8px 12px;
  font-size: 11px;
  color: var(--text-faint);
}
</style>
