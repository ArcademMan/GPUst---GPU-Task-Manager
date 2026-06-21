export interface Theme {
  id: string;
  name: string;
  /** swatch colors for the picker: [bg, accent, text] */
  swatch: [string, string, string];
}

export const THEMES: Theme[] = [
  { id: "midnight", name: "Midnight", swatch: ["#0b1020", "#5b8cff", "#e6ebff"] },
  { id: "graphite", name: "Graphite", swatch: ["#16181d", "#8b9cff", "#eceef2"] },
  { id: "nord", name: "Nord", swatch: ["#2e3440", "#88c0d0", "#eceff4"] },
  { id: "dracula", name: "Dracula", swatch: ["#282a36", "#bd93f9", "#f8f8f2"] },
  { id: "light", name: "Light", swatch: ["#f4f6fb", "#3b6cff", "#1a2138"] },
];

const STORAGE_KEY = "gpust.theme";

export function loadTheme(): string {
  return localStorage.getItem(STORAGE_KEY) ?? "midnight";
}

export function applyTheme(id: string) {
  document.documentElement.setAttribute("data-theme", id);
  localStorage.setItem(STORAGE_KEY, id);
}
