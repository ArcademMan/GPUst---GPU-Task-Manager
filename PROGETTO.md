# GPUst — GPU Task Manager (Windows) · Tauri + Rust

> Brief di avvio del progetto. Documento autosufficiente per iniziare anche
> senza il contesto della conversazione in cui è nato.

## Obiettivo
Un piccolo "Task Manager focalizzato sulla GPU": mostra **quali processi stanno
usando la GPU** e quanto, in tempo reale, con UI gradevole e possibilità di
**terminare i processi**. Nasce perché sul Task Manager standard è scomodo
isolare chi tiene la GPU occupata a riposo (caso reale che ha innescato il
progetto: Discord teneva la GPU al ~10–18% per via dell'accelerazione hardware;
disattivandola si torna al ~3%).

## Stack
- **Tauri 2.x** (binario leggero, no Electron)
- **Backend: Rust** — legge i contatori prestazioni di Windows
- **Frontend: Vue 3 + TypeScript** (Vite)
- **Target: solo Windows** (i contatori PDH sono Windows-only; nessun obbligo
  cross-platform in v1)
- **IDE consigliato: VS Code** (rust-analyzer + estensione Tauri + Volar). In
  alternativa RustRover per il lato Rust.
- **Disco: SSD obbligato di fatto** — Rust + `target/` (diversi GB di file
  piccoli) + `node_modules` rendono un HDD meccanico frustrante.

## Come si leggono i dati (CUORE del progetto — GIÀ VALIDATO)
I dati vengono dai **Performance Counters di Windows**, categoria `GPU Engine`,
contatore `Utilization Percentage`. Le istanze hanno nome tipo
`pid_1234_luid_0x..._engtype_3D`, quindi si estrae il **PID** con regex e si
**somma per processo** (un processo ha più engine: 3D, Copy, VideoDecode,
VideoEncode...).

Logica di riferimento, **testata e funzionante** in PowerShell su questa
macchina (da reimplementare in Rust):

```powershell
$samples = Get-Counter '\GPU Engine(*)\Utilization Percentage' -SampleInterval 1 -MaxSamples 6
# per ogni sample: per ogni CounterSample con CookedValue > 0:
#   estrai pid con  InstanceName -match 'pid_(\d+)'
#   accumula la somma per pid, poi media sul numero di sample
# risolvi PID -> nome processo, ordina desc
```

Output reale ottenuto (mediato su 6s):
`Discord 10.5`, `dwm 6.4`, `claude 6.2`, `Code 0.9`, `System 1`.

**Punti chiave appresi:**
- Lo **snapshot singolo è ballerino**: serve una **media mobile** su N campioni
  (es. 5–6s), non un singolo `Get-Counter`.
- Bisogna **sommare tutti gli engine** dello stesso PID per l'uso reale.
- Contatore parallelo utile: **`GPU Process Memory \ Dedicated Usage`** → VRAM
  dedicata per processo.

## Implementazione Rust (backend)
Due strade per leggere i contatori:
1. **PDH API native** via crate `windows`
   (`Windows.Win32.System.Performance`): `PdhOpenQuery` /
   `PdhAddEnglishCounter('\GPU Engine(*)\Utilization Percentage')` /
   `PdhCollectQueryData` ×2 con attesa / `PdhGetFormattedCounterArray`.
   Via "pulita" e veloce → versione finale.
2. **Fallback rapido**: invocare `Get-Counter`/`typeperf` come processo e
   parsare l'output. Più semplice da prototipare.

Consiglio: prototipare con la via 2 per validare la UI, poi passare a PDH
nativo (via 1) per la versione vera.

Nomi processo / kill: crate **`sysinfo`** (`process.kill()`, nome, path).

## Comandi Tauri (Rust ↔ frontend)
- `get_gpu_usage()` → `Vec<ProcInfo>` con
  `{ pid, name, gpu_pct, vram_mb, engines: { d3d, copy, video_decode, video_encode } }`
- `kill_process(pid)` → con **guardia di sicurezza**: rifiutare/avvisare su
  processi di sistema critici (`dwm`, `csrss`, `System`, `Idle`, `wininit`,
  `winlogon`, `services`, `smss`). Conferma esplicita nel frontend.
- (opzionale) `get_history(pid)` per il grafico.

## Frontend (Vue) — funzionalità v1
- Tabella processi ordinata per uso GPU (refresh ~1s, valori **mediati** lato
  backend per non far sfarfallare).
- Breakdown per **engine** (3D / Video Decode / Encode / Copy) — aiuta a capire
  *perché* un processo usa la GPU.
- Colonna **VRAM** dedicata.
- **Mini-grafico temporale** (sparkline) per il processo selezionato.
- Pulsante **Termina processo** con conferma; voci di sistema protette.
- (nice-to-have) icona in **system tray** + alert se un processo supera una
  soglia per X secondi.

## Gotcha da ricordare
- Solo Windows (PDH). Niente illusioni cross-platform in v1.
- Mai killare processi di sistema → whitelist di protezione obbligatoria.
- Mediare i valori, mai mostrare lo snapshot grezzo.
- Tauri/WebView consumano un filo di GPU (è il compositore `dwm`): normale.
  La nostra app deve restare leggera (no animazioni pesanti — ironia della sorte).
- Progetto su SSD.

## Primi passi operativi
1. Verificare prerequisiti: **Rust toolchain**, **Node**, e i prerequisiti
   **Tauri su Windows** (WebView2 + Build Tools / MSVC).
2. `npm create tauri-app@latest` → template **Vue + TypeScript**.
3. Aggiungere crate Rust: `windows` (feature Performance) **oppure** approccio
   `Get-Counter`, più `sysinfo`, `serde`.
4. Implementare `get_gpu_usage()` (prima versione anche via `Get-Counter` per
   sbloccare la UI).
5. Tabella base nel frontend che chiama il comando ogni secondo.
6. Iterare: media mobile → engine breakdown → VRAM → kill → grafico → tray.

## Nome
**GPUst** = GPU + Rust. (cartella: `GPUst - GPU Task Manager`)
