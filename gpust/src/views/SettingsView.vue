<script setup lang="ts">
import { onMounted, ref } from "vue";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import { settings, REFRESH_OPTIONS } from "../useSettings";

const autostart = ref(false);
const autostartBusy = ref(false);

onMounted(async () => {
  try {
    autostart.value = await isEnabled();
  } catch {
    /* plugin non pronto */
  }
});

async function toggleAutostart() {
  if (autostartBusy.value) return;
  autostartBusy.value = true;
  try {
    if (autostart.value) {
      await disable();
      autostart.value = false;
    } else {
      await enable();
      autostart.value = true;
    }
  } catch (e) {
    console.error("autostart:", e);
  } finally {
    autostartBusy.value = false;
  }
}
</script>

<template>
  <div class="page">
    <header class="page-head">
      <h1>Impostazioni</h1>
      <p>Comportamento di aggiornamento e di avvio dell'applicazione.</p>
    </header>

    <section class="card">
      <div class="sec-title">Aggiornamento</div>
      <div class="field">
        <div class="field-text">
          <div class="field-label">Frequenza di aggiornamento</div>
          <div class="field-hint">
            Ogni quanto la tabella viene aggiornata. La media mobile resta su ~5 secondi.
          </div>
        </div>
        <select v-model.number="settings.refreshMs" class="select">
          <option v-for="o in REFRESH_OPTIONS" :key="o.ms" :value="o.ms">{{ o.label }}</option>
        </select>
      </div>
    </section>

    <section class="card">
      <div class="sec-title">Avvio</div>

      <label class="field toggle-field">
        <div class="field-text">
          <div class="field-label">Avvia all'avvio di Windows</div>
          <div class="field-hint">GPUst parte automaticamente all'accensione del PC.</div>
        </div>
        <input type="checkbox" :checked="autostart" :disabled="autostartBusy" @change="toggleAutostart" />
        <span class="track"><span class="thumb" /></span>
      </label>

      <label class="field toggle-field">
        <div class="field-text">
          <div class="field-label">Avvia ridotto in tray</div>
          <div class="field-hint">
            All'avvio la finestra resta nascosta nella system tray (utile con l'avvio automatico).
          </div>
        </div>
        <input type="checkbox" v-model="settings.startInTray" />
        <span class="track"><span class="thumb" /></span>
      </label>
    </section>

    <section class="card danger">
      <div class="sec-title">Avanzate</div>
      <label class="field toggle-field">
        <div class="field-text">
          <div class="field-label">Consenti chiusura processi di sistema</div>
          <div class="field-hint">
            Rimuove la protezione su processi come <b>dwm</b>, <b>csrss</b>, <b>System</b>. Sconsigliato:
            terminarli può bloccare la sessione o causare un riavvio. Windows ne protegge comunque
            alcuni a livello di kernel.
          </div>
        </div>
        <input type="checkbox" v-model="settings.allowKillSystem" />
        <span class="track danger-track"><span class="thumb" /></span>
      </label>
    </section>
  </div>
</template>

<style scoped>
.page {
  height: 100vh;
  overflow-y: auto;
  padding: 28px 32px;
  max-width: 720px;
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
.card {
  margin-top: 20px;
  background: var(--bg-elev);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 18px 20px;
}
.card.danger {
  border-color: color-mix(in srgb, var(--danger) 40%, var(--border));
}
.sec-title {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.6px;
  color: var(--text-faint);
  margin-bottom: 14px;
}
.field {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 10px 0;
}
.field + .field {
  border-top: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
}
.field-text {
  flex: 1;
  min-width: 0;
}
.field-label {
  font-weight: 600;
  font-size: 13.5px;
}
.field-hint {
  font-size: 11.5px;
  color: var(--text-faint);
  margin-top: 3px;
  line-height: 1.45;
}
.select {
  flex-shrink: 0;
  background: var(--bg);
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 10px;
  font-family: inherit;
  font-size: 13px;
  cursor: pointer;
}
.select:hover {
  border-color: var(--accent);
}

/* toggle */
.toggle-field {
  cursor: pointer;
}
.toggle-field input {
  display: none;
}
.track {
  flex-shrink: 0;
  width: 40px;
  height: 23px;
  border-radius: 23px;
  background: var(--bar-track);
  position: relative;
  transition: background 0.15s ease;
}
.thumb {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 17px;
  height: 17px;
  border-radius: 50%;
  background: var(--text-faint);
  transition: all 0.15s ease;
}
.toggle-field input:checked + .track {
  background: var(--accent-soft);
}
.toggle-field input:checked + .track .thumb {
  left: 20px;
  background: var(--accent);
}
.toggle-field input:checked + .danger-track {
  background: var(--danger-soft);
}
.toggle-field input:checked + .danger-track .thumb {
  background: var(--danger);
}
.toggle-field input:disabled + .track {
  opacity: 0.5;
}
</style>
