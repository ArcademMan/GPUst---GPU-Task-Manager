import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

// Cache condivisa a livello di modulo: persiste tra cambi pagina.
const cache = ref<Map<string, string | null>>(new Map());

export function useIcons() {
  async function ensure(exe: string | null) {
    if (!exe || cache.value.has(exe)) return;
    cache.value.set(exe, null); // pending
    try {
      const icon = await invoke<string | null>("get_process_icon", { exe });
      if (icon) {
        const next = new Map(cache.value);
        next.set(exe, icon);
        cache.value = next;
      }
    } catch {
      /* ignora */
    }
  }

  function iconFor(exe: string | null): string | null {
    return exe ? cache.value.get(exe) ?? null : null;
  }

  return { ensure, iconFor };
}
