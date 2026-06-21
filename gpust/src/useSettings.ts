import { reactive, watch } from "vue";

export interface Settings {
  /** intervallo di refresh della UI in ms */
  refreshMs: number;
  /** avvia l'app direttamente nascosta in tray */
  startInTray: boolean;
  /** modalità avanzata: consente di tentare la chiusura dei processi di sistema */
  allowKillSystem: boolean;
}

const KEY = "gpust.settings";

const defaults: Settings = {
  refreshMs: 1000,
  startInTray: false,
  allowKillSystem: false,
};

function load(): Settings {
  try {
    return { ...defaults, ...JSON.parse(localStorage.getItem(KEY) ?? "{}") };
  } catch {
    return { ...defaults };
  }
}

export const settings = reactive<Settings>(load());

watch(
  settings,
  (s) => localStorage.setItem(KEY, JSON.stringify(s)),
  { deep: true }
);

export const REFRESH_OPTIONS = [
  { ms: 500, label: "0,5 s — molto reattivo" },
  { ms: 1000, label: "1 s — consigliato" },
  { ms: 2000, label: "2 s — leggero" },
  { ms: 5000, label: "5 s — minimo consumo" },
];
