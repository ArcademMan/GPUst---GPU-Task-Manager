<script setup lang="ts">
import DeviceDashboard from "../components/DeviceDashboard.vue";
import type { DeviceInfo } from "../gpu";

defineProps<{ devices: DeviceInfo[] }>();
</script>

<template>
  <div class="page">
    <header class="page-head">
      <h1>GPU</h1>
      <p>
        Stato delle schede video installate. VRAM e nome disponibili per tutte; temperatura, clock,
        consumo e ventole dove il driver li espone (NVIDIA).
      </p>
    </header>

    <DeviceDashboard v-if="devices.length" :devices="devices" />
    <div v-else class="empty">Nessuna GPU rilevata, oppure i dati non sono ancora disponibili…</div>
  </div>
</template>

<style scoped>
.page {
  height: 100vh;
  overflow-y: auto;
  padding: 28px 32px;
}
.page-head {
  margin-bottom: 22px;
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
  max-width: 640px;
  line-height: 1.5;
}
.empty {
  color: var(--text-faint);
  text-align: center;
  margin-top: 60px;
  font-size: 13px;
}
</style>
