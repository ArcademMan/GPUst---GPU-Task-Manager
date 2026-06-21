<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Sidebar from "./components/Sidebar.vue";
import TaskManagerView from "./views/TaskManagerView.vue";
import GpuView from "./views/GpuView.vue";
import ThemesView from "./views/ThemesView.vue";
import SettingsView from "./views/SettingsView.vue";
import { useGpu } from "./gpu";
import { applyTheme, loadTheme } from "./themes";
import { settings } from "./useSettings";

type Page = "tasks" | "gpu" | "themes" | "settings";
const page = ref<Page>("tasks");

const { procs, devices, history, error, start, stop, setIntervalMs } = useGpu();

watch(
  () => settings.refreshMs,
  (ms) => setIntervalMs(ms)
);

onMounted(async () => {
  applyTheme(loadTheme());
  start(settings.refreshMs);
  if (settings.startInTray) {
    try {
      await getCurrentWindow().hide();
    } catch {
      /* in dev potrebbe non essere disponibile */
    }
  }
});
onUnmounted(stop);
</script>

<template>
  <div class="shell">
    <Sidebar :page="page" @navigate="page = $event as Page" />
    <main class="content">
      <TaskManagerView
        v-show="page === 'tasks'"
        :procs="procs"
        :history="history"
        :error="error"
      />
      <GpuView v-show="page === 'gpu'" :devices="devices" />
      <ThemesView v-show="page === 'themes'" />
      <SettingsView v-show="page === 'settings'" />
    </main>
  </div>
</template>

<style scoped>
.shell {
  display: flex;
  height: 100vh;
  overflow: hidden;
}
.content {
  flex: 1;
  min-width: 0;
  position: relative;
}
</style>
