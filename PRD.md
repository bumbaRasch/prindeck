# PRD: prindeck

Control panel for the Brother MFC-L2710DW on Linux — an equivalent of Brother iPrint&Scan (Windows/Android/iOS), which does not officially exist for Linux. Tauri 2: native window, tray, autostart.

- Status: M0 ✅ 26.10.2026 · M1 ✅ · M2 ✅ 27.10.2026 · M2.5 ✅ 27.10.2026
  (live device acceptance pending, §14)
- Author: bumbarasch
- Date: 2026-10-26 (revision 27.10.2026)
- Currency: device facts — M0 spike 26.10.2026; stack versions
  re-checked 27.10.2026 (§0)
- Repository: `~/Documents/projects/prindeck/`
- Name: **prindeck** — print + deck; no vendor TM; free on GitHub/crates/npm
  (checked via API 26.10.2026: GitHub 0 repos, crates.io 404, npm 404)

---

## 0. Currency as of 26.10.2026

| What | State as of 26.10.2026 | Conclusion for the project |
|---|---|---|
| iPrint&Scan on Linux | Still Windows/macOS/Android/iOS only; for Linux Brother provides only drivers (brscan4/5, brscan-skey) | Problem §1 is not closed by the vendor — the project is still relevant |
| brscan5 on Fedora | Official deb/rpm break on Fedora 40+ (GTK2 dependencies, FAQ at support.brother.com) | An additional argument for the driverless path without Brother binary drivers |
| eSCL / Mopria | The spec is public, the protocol is stable; sane-airscan in fresh repos — 0.99.38 | A mature protocol, the risk of a format change is minimal |
| Node.js / Rust | Node 26 — Latest LTS since 20.10.2026; Rust — a stable mainstream choice for Tauri | Tauri 2 — the current mainstream of desktop applications |
| Tauri 2 | Crate 2.12.5, tauri-build 2.7.4, create-tauri-app 4.8.0 (checked 27.10.2026) | The stack is alive; versions in §10 updated |
| SNMP crate | `snmp2` 0.5.x (roboplc): dependency-free, v1/v2c/v3, sync/async, ~1.3M downloads — maintained; `rasn-snmp` — a heavy ASN.1 framework | Taking `snmp2` (the overkill alternative is recorded) |
| Driverless printing | IPP Everywhere — the default CUPS path in current distributions | The queue is already set up; we do not touch system printing |

Policy: before starting each stage (M1+) — a quick re-check of the versions from this
table; on discrepancy — update the table and the stack item §10.

## 1. Problem

Brother iPrint&Scan is available only on Windows/macOS/Android/iOS. On Linux Brother
provides drivers (CUPS/LPR, brscan4/5, brscan-skey), but not an application with a UI.
Control comes down to scattered tools: the printer web panel in a browser, `simple-scan`,
the `lp`/`scanimage` CLI. There is no single window: status + toner + scan + print + history.

## 2. Goals and non-goals

### Goals (v1)

1. One native window (Tauri 2): device state dashboard, scan, print;
   tray icon, autostart, system notifications.
2. All device interaction over open protocols (eSCL, SNMP, CUPS) —
   **without Brother binary drivers**.
3. Minimum dependencies: the crate list is fixed in §10, a new crate =
   justification in the commit.
4. Works offline, bound to one machine/printer, no cloud.
5. Readiness for publication on GitHub: README with a disclaimer of non-affiliation
   with Brother, license, CI build (v1 — local bundler: deb/rpm/AppImage).

### Non-goals (v1)

- We do not replace system printing — CUPS remains the print engine (the queue is already configured).
- We do not support multiple printers (not forbidden architecturally, but not tested).
- We do not emulate the «Scan to PC» button on the device body (v2, see §12).
- We do not do OCR/document search (v2, tesseract as an optional add-on).
- Fax — no (the device has the module, but it is not needed).
- Windows/macOS builds — no (Tauri allows it, but we do not test it).
- From the iPrint&Scan reference (27.10.2026, reverse of the Windows installer;
  local notes in `docs/ref/`, not published): cloud destinations
  (Drive/OneDrive/Dropbox/SharePoint/Evernote), in-app Office conversion and OCR
  (the vendor ships commercial Kofax OmniPage + Aspose; our OCR stays a v2
  tesseract add-on), the USB bridge (prindeck is network-only), the vendor
  account/marketing layer.

## 3. Device and environment (fixed facts)

| Fact | Value |
|---|---|
| Device | Brother MFC-L2710DW (mono laser, ADF 50 sheets simplex, auto-duplex printing) |
| IP | `192.168.187.36` |
| Web panel | `http://192.168.187.36/` (responds, title «Brother MFC-L2710DW series») |
| eSCL endpoint | `http://192.168.187.36:80/eSCL/` (confirmed by `airscan-discover`, WSD also present) |
| CUPS queue | `Brother_MFC_L2710DW_series` (driver brlaser, PPD installed) |
| SANE | `sane-airscan` installed, sees the device |
| OS | Ubuntu 26.04.1 LTS (checked at M0; the PRD previously said «Fedora» erroneously); WebKitGTK runtime 2.52.6 and dev headers 2.52.6, Rust 1.99.1 — installed 27.10.2026, see `docs/m1-prerequisites.md` |

## 4. Functional requirements

### F1. Dashboard (main screen)

- Device status: idle / printing / scanning / busy / error+error text / offline.
- Black toner (%) and drum level — bars with a number.
- Printed page counter (total).
- Link "Open printer web panel", via tauri-plugin-opener.
- Refresh: manual refresh + auto-polling with a configurable interval (default 30 s).
- Data is cached: the last response is shown instantly, stale data
  is marked with a time ("updated 12 s ago"), device offline → grey badge.

### F2. Scanning (core value)

- Source: platen (Platen) / feeder (Feeder/ADF).
- Resolution: only from ScannerCapabilities. On the MFC-L2710DW — 100/200/300/600;
  150 is absent (the device silently substitutes 300 for it, checked at M0).
- Color: `RGB24` / `Grayscale8` / `BlackAndWhite1` — exact names from capabilities.
  The pair `BlackAndWhite1` + JPEG on the device yields `409` — validate before sending.
- Output format: JPEG (per page, `scan-…-pN.jpg`) / PDF (multi-page — the
  firmware assembles the whole ADF stack into one document, §13.3).
- ADF: all sheets from the feeder with one button → one PDF (JPEG — one file
  per sheet).
- Save location: default folder (setting), file name = `scan-YYYYMMDD-HHMMSS`.
- Progress: indication of job phases (created → scanning → N pages received → done).
- Result preview and an "Open folder" action after completion.
- Cancel: DELETE scan-job on the device.

### F3. Printing

- File selection (PDF/images/text) via the native Tauri dialog or drag-and-drop.
- Parameters: copies (1–99), duplex (off / long-edge / short-edge), quality
  (draft/normal), resolution/toner saving (from the queue's PPD/IPP attributes).
- Sending via CUPS (`lp`): job status visible in the UI (processing/completed/error).
- Active queue: list of unfinished jobs + a `cancel` action.

### F4. Monitoring and history

- Background SNMP polling (toner, status) with a configurable interval (default 5 min).
- System notifications (tauri-plugin-notification) on: toner < 15 %, drum < 10 %,
  device error, offline → online transition (no more than 1 notification per event).
- Tray icon: status color + menu ("Show window", "Scan", "Open panel",
  "Quit"); on Linux the tray icon requires a menu to be set (a Tauri peculiarity).
- History of toner levels/page counter: local SQLite (90-day chart).
- "State report": once a week/month take a snapshot to a file (optional, off).

### F5. From the iPrint&Scan reference (27.10.2026)

A portable subset distilled from the reverse of the Windows iPrint&Scan
17.0.20.2 installer (working notes — `docs/ref/`, local only, not published).
Candidates R1–R7 plus a deferred R9; milestones are preliminary. R1/R7 are
gated by a live check of the EWS paths on our device (§13.10).

| # | Feature | Home | Notes / milestone |
|---|---|---|---|
| R1 | Deep links into the device web panel: public pages (status, maintenance info, sleep, auto power-off, replace toner) + authenticated pages (date/time, SNTP, firmware update, network status, wired/wireless TCP-IP details) | «Обзор»/«Настройки» | M3, with the settings screen; every link verified live 27.10.2026 (`docs/ews-l2710dw.md`); authenticated links open the panel's login form first |
| R2 | Scan result preview with page operations before saving: rotate, delete/insert a page, reorder, crop | «Скан» | M3, after the ADF merge; fully client-side |
| R3 | Continuous ADF: after a pass ask "add pages?" and append to the same PDF | «Скан» | M3; on eSCL — a new ScanJobs POST + merge, the device is free between passes |
| R4 | Blank-page skip with a sensitivity setting (share of near-white pixels) | «Скан» | M3; applies to ADF passes |
| R5 | Human-readable state from the already parsed blobs: jams → "check the paper path", drum pages vs 12 000 → "drum end soon" | «Обзор» | M4, with notifications; message wording referenced from the vendor status dictionary (docs/ref) |
| R6 | Toner remaining-pages estimate next to the %: cartridge ISO yield minus the page counter | «Обзор» | M4, needs history (SQLite); the installed cartridge model/yield must be verified against the web panel first — do not hardcode |
| R7 | Firmware: a menu item/notification opening the device firmware page — no flashing from prindeck | «Обзор»/tray | M4; `/admin/firmwareupdate.html` verified live behind the login (`docs/ews-l2710dw.md`); "outdated" detection only if a trustworthy version source appears |
| R9 | PC Copy: scan → print N copies via CUPS | «Скан» | deferred — both halves exist after M3; revisit at v2 planning |

## 5. Non-functional requirements

1. **Security**: no secrets (SNMP community `public` — configurable);
   no telemetry, everything local; eSCL/SNMP traffic — only to the printer's LAN address.
2. **Crate discipline**: the list is fixed in §10; frontend —
   TypeScript strict without build surprises (Vite); a minimal DOM layer.
3. **Reliability**: all requests to the device with a timeout (5 s) and limited
   retries; the device may stay silent — the UI must not hang; polling in
   `tokio::task` with correct cancellation when the window closes.
4. **Resources**: when idle, polling must not consume CPU (one SNMP batch every 5 min),
   a tray window does not render; memory < 100 MB.
5. **Validation at boundaries**: the config (`config.json`) is validated at startup,
   error → ConfigError dialog; internal types are trusted.
6. **Resilience to firmware changes**: everything we parse (XML capabilities, SNMP)
   must not fail on unfamiliar fields — unknown → ignore + log (tracing).
7. **GitHub hygiene**: no keys/private data in the repo; README disclaimer
   "not affiliated with Brother Industries"; license (MIT or Apache-2.0 — §13).

## 6. Architecture

```
prindeck (Tauri 2)
├─ src-tauri/src/
│  ├─ main.rs          thin entry point → lib.rs::run()
│  ├─ lib.rs           builder: window, tray (+menu), single-instance, setup
│  ├─ commands/        #[tauri::command] — IPC boundary (see §9), input validation
│  │  ├─ mod.rs        AppState, scan events (scan-progress/scan-done/scan-error)
│  │  ├─ caps.rs       get_capabilities + capabilities cache
│  │  ├─ scan.rs       start_scan / cancel_scan, job-id validation
│  │  ├─ scan_errors.rs classified scan failures → user-worded messages
│  │  ├─ scan_files.rs scan file naming, magic-byte checks
│  │  ├─ scan_jobs.rs  scan-job runner (POST job, fetch loop, cancel, partial keep)
│  │  ├─ scan_pages.rs per-page fetch loop, PagesOutcome (PDF = one whole-stack
│  │  │                document, JPEG = one document per page)
│  │  ├─ print.rs      print_file / get_queue / cancel_print IPC
│  │  └─ status.rs     get_status / refresh_status / open_printer_panel
│  ├─ escl.rs          reqwest + quick-xml: capabilities, status, scan-job lifecycle
│  ├─ snmp/            snmp2: one batched GET of the Brother OIDs (§7.2)
│  │  ├─ mod.rs        SnmpCounters, blob records → toner/drum/pages fields
│  │  ├─ blob.rs       OctetString record parser (<id> 01 04 <uint32 BE>, 0xff end)
│  │  └─ client.rs     SyncSession get_many — 3 varbinds in one PDU
│  ├─ status/          SNMP → DeviceStatus; cache + polling (tokio interval)
│  │  ├─ mod.rs        DeviceStatus, StatusCache (cached first paint), `status` event
│  │  └─ poll.rs       interval from config pollSeconds (default 300 s), one retry,
│  │  │                silent history::try_record after a successful poll
│  ├─ tray.rs          tray icon + menu ("Show window" / "Quit")
│  ├─ print/           CUPS subprocesses: lp / lpstat / cancel (LC_ALL=C) + parsing
│  ├─ history/         rusqlite (bundled): snapshots (ts, toner, drum, pages)
│  └─ config/          config.json (device.host, scanDir, cupsQueue, snmp.community, pollSeconds)
│     ├─ mod.rs        Config, defaults, validation
│     ├─ store.rs      load/save of config.json
│     ├─ error.rs      ConfigError
│     └─ host.rs       host/address validation
└─ ui/                 Vite + TypeScript strict
   ├─ index.html       SPA shell: sidebar + three destinations (Скан, Обзор, Печать)
   └─ src/             no framework — vanilla DOM modules (§13.7)
      ├─ main.ts       init, wiring, event listeners
      ├─ api.ts        invoke() wrappers
      ├─ state.ts      AppState + derivations (connectivity, tones, counters)
      ├─ shell.ts      sidebar: reachability dot, banners (startup/caps/connection)
      ├─ router.ts     section switching, roving tabindex
      ├─ dom.ts        DOM helpers
      ├─ caps.ts       capabilities fetch/apply
      ├─ labels.ts     Russian UI strings
      ├─ types.ts      IPC/domain types
      ├─ screens/      scan.ts + scan_result.ts (Скан), overview.ts (Обзор),
      │                print*.ts (Печать: form, file picker/drop, queue)
      └─ styles/       base, shell, content, scan, scan_result, overview, print
```

Data flows: UI → `invoke()` → Rust commands; background processes (status polling,
scan progress) → `emit()` → UI `listen()`. Scanning — a background `tokio::task`,
writes files directly (folder dialog — via the tauri dialog plugin / setting).

Each module is isolated: escl/snmp/print do not know about each other. `status/`
merges SNMP only — the eSCL capabilities state is not part of `DeviceStatus`;
the shell combines reachability from the eSCL capabilities state and the status
poll. Data flows in one direction — "network → DTO mapping → domain → IPC".

## 7. Protocols

### 7.1 Scanning — eSCL (M0 facts 26.10.2026; ADF — M2.5 spike 27.10.2026)

RESTful HTTP+XML (originated at HP, published by Mopria; reference — `sane-airscan`).
Base path: `http://192.168.187.36/eSCL/`. Everything below was taken from the live device;
the full log — `docs/escl-l2710dw.md`.

Observed flow:

1. `GET /eSCL/ScannerCapabilities` → XML (`pwg:Version 2.63`, `eSCL-1_92.xsd`):
   resolutions **100/200/300/600** (150 is absent — 300 is returned silently), ColorMode
   **`BlackAndWhite1` / `Grayscale8` / `RGB24`**, formats **`application/pdf`
   and `image/jpeg`** (the PDF is assembled by the firmware), Platen regions `2550×3507` (A4) and
   ADF `2550×4200` (Legal), units 1/300″; ADF — simplex only (20 sheets). On A4 stacks
   request `Height 3507`, not the advertised 4200: with the full Legal strip the PDF's
   non-final pages carry ~6 cm blank tails (`docs/escl-l2710dw.md` §11.4).
2. `POST /eSCL/ScanJobs` (body `ScanSettings` — **on one line**) → `201` +
   `Location: http://…/eSCL/ScanJobs/<uuid-v1>`.
3. There is no separate `GET {job}/ScannerJobStatus` (404). The job state
   (`Pending|Processing|Completed|Canceled`, reasons `JobScanning|
   JobCompletedSuccessfully|JobCanceledAtDevice|JobTimedOut`) is read in
   `GET /eSCL/ScannerStatus` → `scan:Jobs/scan:JobInfo/pwg:JobState`.
4. `GET {job}/NextDocument` — blocking (A4 @300 dpi ≈ 7.8–8.6 s), returns
   JPEG/PDF; end of document — `404` with an empty body; document name — in
   `Content-Location: …/doc-N` (there is no `DocumentUri` element). Without reading it out
   ≈ 61 s → `Canceled/JobTimedOut`. On the ADF the delivery is split by format
   (M2.5 spike, `docs/escl-l2710dw.md` §11): **PDF = the whole stack in ONE
   response** (the firmware assembles the multi-page document; the next request
   gets the terminal `404`), **JPEG = one document per page** (3 sheets → three
   `200`s, then `404`). The whole-stack PDF response blocks ≈ 8 s × sheets —
   the fetch timeout scales with the feeder capacity (≥ 180 s at 20 sheets).
5. `DELETE {job}` — `200` on an active job, `404` after completion. `DELETE`
   during a blocking `NextDocument` → `200`, and the hanging fetch is aborted with
   **`410 Gone`** (0 B) — a normal cancel, not an error (checked 27.10.2026;
   `docs/escl-l2710dw.md` §10).

Mandatory caveats (verified by A/B tests):
- `ScanSettings` body — **no line breaks**: any `\n` ⇒ the settings are silently
  ignored (the scan is "successful", but 200 dpi/RGB24) — the main M1 risk.
- `InputSource` in ScanSettings — `pwg:` (`Platen`/`Feeder`); in capabilities
  the sources are described as `scan:Platen/scan:PlatenInputCaps`,
  `scan:Adf/scan:AdfSimplexInputCaps`.
- `ScanRegions` is mandatory; `BlackAndWhite1` + `image/jpeg` → `409`;
  an empty Feeder → `500`; busy → `503`.
- ADF fetches need **no pause** on this unit: back-to-back requests (measured
  1 ms gaps) cross sheet transitions cleanly; the sane-airscan "≥ 1 s between
  `NextDocument`" Brother quirk does not manifest (`docs/escl-l2710dw.md` §11.2).
- `scan:AdfState` correlates with `POST` in both directions:
  `ScannerAdfLoaded` ⇔ `201`, `ScannerAdfEmpty` ⇔ `500` (§11).
- The terminal `404` finalises the job: after the last document the job stays
  `Processing`/`JobScanning` and flips to `Completed` only after the `404` —
  the terminating request is always required (§11.6).
- Cancel mid-stack on the ADF is **not yet observed live** (the spike ran out
  of paper); the platen cancel matrix (`docs/escl-l2710dw.md` §10.1) remains
  the reference — the M2.5 live acceptance (§14) covers it.

Working `ScanSettings` (fixture: `fixtures/escl/scansettings-platen-300-gray-pdf.xml`):

```xml
<scan:ScanSettings xmlns:pwg="http://www.pwg.org/schemas/2010/12/sm" xmlns:scan="http://schemas.hp.com/imaging/escl/2011/05/03"><pwg:Version>2.0</pwg:Version><scan:Intent>Document</scan:Intent><pwg:ScanRegions><pwg:ScanRegion><pwg:ContentRegionUnits>escl:ThreeHundredthsOfInches</pwg:ContentRegionUnits><pwg:XOffset>0</pwg:XOffset><pwg:YOffset>0</pwg:YOffset><pwg:Width>2550</pwg:Width><pwg:Height>3507</pwg:Height></pwg:ScanRegion></pwg:ScanRegions><pwg:InputSource>Platen</pwg:InputSource><scan:ColorMode>Grayscale8</scan:ColorMode><pwg:DocumentFormat>application/pdf</pwg:DocumentFormat><scan:DocumentFormatExt>application/pdf</scan:DocumentFormatExt><scan:XResolution>300</scan:XResolution><scan:YResolution>300</scan:YResolution></scan:ScanSettings>
```

Results: JPEG 2512×3460 @300 dpi (Grayscale8 — 1 component, RGB24 — 3);
PDF — 1 page 602.64×830.16 pt; `BlackAndWhite1` — true 1-bit CCITT.
Multi-page ADF verified at M2.5: 2 sheets → one 2-page PDF from a single
`NextDocument` (§13.3); JPEG stacks arrive per page.

### 7.2 Status/toner — SNMP (snmp2)

Two sources, merged in status.rs:

**Standard Printer-MIB (RFC 3805)** — primary:

| Data | OID |
|---|---|
| Consumable level (toner %) | `1.3.6.1.2.1.43.11.1.1.9.1.<n>` |
| Consumable name | `1.3.6.1.2.1.43.11.1.1.6.1.<n>` |
| Max capacity | `1.3.6.1.2.1.43.11.1.1.8.1.<n>` |
| Page counter (life count) | `1.3.6.1.2.1.43.10.2.1.4.1.1` |
| Device status | `1.3.6.1.2.1.25.3.2.1.5.1` (hrDeviceStatus) |

M0 facts: `prtMarkerSuppliesLevel.1.1 = -3` ("some remaining"),
`MaxCapacity.1.1 = -2` → **toner in % via Printer-MIB is unavailable**; the drum —
`4247` pages out of `12000`; the page counter — `Counter32`, but the firmware encodes it
**with two bytes** (`1E49`), formally violating BER (a risk for `snmp2`, §13.8).
`hrDeviceStatus = 2` was also observed on a sleeping device — the M2 mapping needs clarification.

**Brother enterprise OIDs** (**mandatory** source — Printer-MIB does not give %;
the MFC-L27xx series has been reverse-engineered):

| Node | OID | Contents |
|---|---|---|
| brInfoCounter | `1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.10` | counters: pages, drum |
| brInfoMaintenance | `1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.8` | remaining resource in % (in units of 0.01) |
| brInfoNextCare / JamCount / ReplaceCount | `…5.5.x` | maintenance, jams, replacements |

Format — OctetString blobs; the grammar is **heterogeneous across nodes** (see
`docs/oids-l2710dw.md`). Fields verified at M0: `5.5.8.0` → `#81` toner 40 %
(matched the web), `#41` drum 36.00 %; `5.5.10.0` → `#00` pages 7753,
`#06` duplex 3951; `5.5.11.0` → `#82` drum 4247 pages; `5.5.21.0` → `#a1`
jams 8. The nodes respond **only with `.0`**. Parsing rule: only verified
fields, unknown ids — skip with a log (PRD §5.6). Independent verification with
`snmpwalk` — `docs/oids-l2710dw.md` §6. At runtime — `snmp2`, not the CLI.

### 7.3 Printing — via CUPS

- Sending: `lp -d Brother_MFC_L2710DW_series -n <copies> -o sides=<mode> <file>`
  (`one-sided` / `two-sided-long-edge` / `two-sided-short-edge`).
- Queue: `lpstat -W not-completed -o`, cancel: `cancel <job-id>` (only ids of
  the configured queue — a bare `cancel all` must never be reachable).
- All CUPS subprocesses run with `LC_ALL=C` and a timeout: on a ru_RU machine
  `lpstat` otherwise localizes its output and breaks parsing.
- Quality/resolution/toner-saving options from the queue's PPD
  (`lpoptions -p … -l`) — deferred (§12).
- Direct IPP client — not doing it in v1: CUPS already handles discovery, retries, PPD;
  return to it only if a need outside CUPS appears.

### 7.4 Discovery

v1: device IP in the config, on startup — an availability check.
v2: mDNS discovery (`_uscan._tcp`/`_ipp._tcp`) — the mdns-sd crate, a pleasant but not
mandatory layer.

## 8. UI/UX

Navigation model (M2): a navigation shell — a left sidebar (app name, device
reachability dot, destination list) and a content area. The dot merges the eSCL
capabilities state and the SNMP status poll; banners inside the content area
explain startup/capabilities/connection problems instead of dead ends.
Destinations (SPA, section switching, no deep links):

1. **Скан** — the start screen. Settings panel on the left (source, dpi, color,
   format, folder), a large "Scan" button, progress phase, result, "Open folder".
2. **Обзор** — device overview: connection card with manual refresh, toner/drum
   bars with %, page counter and duplex pages over SNMP, eSCL availability.
3. **Печать** — print (M2.5): dropzone/file picker, copies stepper and duplex
   select, the active CUPS queue with a per-row cancel.

Planned destinations: **Настройки** (IP/eSCL path, polling intervals, scan
folder, thresholds).

Layout behaviour: below a 820 px viewport width the sidebar collapses into a
64 px icon rail (labels hidden, only icons); the scan panel reflows by container
queries. Window: default 900×680, minimum 640×480. Dark and light themes follow
`prefers-color-scheme`.

Principles: everything in one or two clicks; the state is always visible (what the printer is doing right
now); device errors are worded with text from SNMP/web panel, not "error".

Localization: the UI language follows the system locale. If the system language is
not supported, the UI falls back to English. Russian is one of the supported
languages. The M1 build ships Russian strings only; the locale-based selection is
part of the i18n work (the strings will move to translation catalogs, English as
the fallback).

## 9. IPC contract (Tauri invoke / events)

Commands (`#[tauri::command]`):

| invoke | Input | Output | Purpose |
|---|---|---|---|
| `get_status` | — | `DeviceStatus` | cached snapshot from the status poller (SNMP only: toner, drum, counters, `polledAt`) |
| `refresh_status` | — | `DeviceStatus` | forced SNMP poll now; error string when the device is silent |
| `get_capabilities` | — | `ScanCaps` | normalized scanner capabilities |
| `start_scan` | `ScanParams` | `jobId` | POST ScanJobs, background task |
| `cancel_scan` | `jobId` | `ok` | DELETE job on the device |
| `open_scan_folder` | — | `ok` | open the scan folder (`scanDir`) via opener |
| `get_startup_error` | — | `Option<String>` | startup/config resolution error, if any |
| `print_file` | `path, copies, sides` | `cupsJobId` | sending via `lp` (M2.5; sides = the CUPS option string) |
| `get_queue` | — | `[CupsJob]` | `lpstat -W not-completed` for the configured queue (M2.5) |
| `cancel_print` | `cupsJobId` | `ok` | `cancel`, guarded to the configured queue (M2.5) |
| `get_history` | `days` | `[Snapshot]` | from SQLite (M4; the recording itself is internal since M2.5 — no IPC yet) |
| `open_printer_panel` | — | `ok` | web panel via opener |

Shipped in M2.5: `print_file`, `get_queue`, `cancel_print`; `scan-progress`
gained `pages` and `scan-done` now carries `{jobId, files, partial, warning}`.
M2 shipped `get_status`, `refresh_status`, `open_printer_panel` and the
`status`, `scan-progress`, `scan-done`, `scan-error` events; M1 —
`get_capabilities`, `start_scan`, `cancel_scan`, `get_startup_error`,
`open_scan_folder`. History is recorded silently inside the status poller
(SQLite at `~/.local/share/prindeck/history.db`) — there is no history IPC
yet; `get_history` is M4.

Events (`emit` → `listen` in the UI):

| Event | Payload | Source |
|---|---|---|
| `status` | `DeviceStatus` | status poller (`status/poll.rs`, SNMP), emitted after every poll attempt |
| `scan-progress` | `{jobId, phase, pages}` | client stages `created → scanning → done`; `pages` counts received documents (ADF, M2.5); polling `ScannerStatus` is not needed — `Completed` lags (§7.1) |
| `scan-done` | `{jobId, files, partial, warning}` | escl scan task; `partial: true` (+ `warning` on failure) when a canceled/failed job kept its received pages (M2.5); a cancel with zero pages terminates as `scan-progress {phase: "canceled"}` without `scan-error` |
| `scan-error` | `{jobId, kind, message}` | scan task failure (user-worded, no raw errors; classified `kind` since M2) |
| `alert` | `{kind, message}` | F4 thresholds (M4) |

## 10. Stack — APPROVED: Tauri 2

The decision was fixed on 26.10.2026 (A: Node zero-dep, C: Electron were considered —
rejected; the record of the comparison is preserved in the PRD's git history).

**Core: Tauri 2 (Rust) + Vite/TypeScript UI.**

- `tauri` 2.12.x + official plugins: `notification`, `autostart`,
  `single-instance`, `opener`, `dialog`; tray — from the core (feature `tray-icon`,
  on Linux the menu is mandatory, tray events are not emitted).
  Wired up: `opener` and `single-instance` (M1), `dialog` (M2.5 — the Print
  screen file picker); `notification` and `autostart` — M4.
- Rust crates (fixed list):
  - `reqwest` 0.13 — eSCL HTTP client (no TLS: the device is over HTTP);
  - `quick-xml` + `serde` — parsing capabilities/status/ScanSettings;
  - `snmp2` 0.5.x — SNMPv2c GET (dependency-free, maintained; `rasn-snmp` —
    deliberately not taken: an ASN.1 framework is overkill for GET polling);
  - `rusqlite` (bundled) — snapshot history;
  - `tokio` — comes with Tauri; `tracing` — logs; `dirs` — XDG paths;
  - `chrono` — timestamps/file names.
- Frontend: TypeScript strict, vanilla DOM; no framework introduced (see §13 question 7).
- Build: tauri bundler → `.deb`/`.rpm`/AppImage; CI (GitHub Actions) — v1.x.

M2.5 phase note: `lopdf` was added and removed within the same phase. The
plan assumed per-page PDF delivery (client-side merge); the live spike proved
the firmware assembles the whole stack into one document (§13.3), so the merge
module and the crate were dropped before any release — `lopdf` is not part of
the standing list.

Crate policy: a new crate — only with a justification in the commit description.

## 11. References

- eSCL overview (history, flow, namespace, DNS-SD): printerarchive.net/tools/escl
- Specification: "Mopria Alliance eSCL Scan Technical Specification" (behind click-through)
- Client reference implementation: github.com/alexpevzner/sane-airscan
  (eSCL+WSD; 0.99.38 in fresh repositories as of 26.10.2026)
- Rust eSCL flow reference: github.com/ElXreno/escl-scan
- Brother SNMP OctetString reverse (incl. MFC-L2720DW — the same series):
  github.com/saper-2/BRN-Printer-sCounters-Info
- Brother monitoring over SNMP (Printer-MIB, codes -3/-2/-1):
  claudiokuenzler.com/blog/1422/
- Standard OIDs: BROTHER-MIB / Printer-MIB (mibs.observium.org)
- SNMP crate: github.com/roboplc/snmp2 (dependency-free, v1/v2c/v3)
- Tauri 2 tray on Linux (a menu is needed for the icon to be visible): docs.rs/tauri 2.9.x
- Tauri plugins: v2.tauri.app/plugin/ (notification, autostart, single-instance, opener)

## 12. Roadmap

| Stage | Contents | Readiness criterion |
|---|---|---|
| M0 Spike ✅ 26.10.2026 | eSCL end-to-end (curl): JPEG and PDF scan, errors; SNMP OIDs + verification against the web, independent `snmpwalk` check; discrepancies with the PRD found | `docs/escl-l2710dw.md`, `docs/oids-l2710dw.md`, fixtures in `fixtures/` |
| M1 Skeleton ✅ 27.10.2026 | Tauri 2 scaffold (window+tray+plugins), `config.rs`, `escl.rs`, "Scan" screen | scan → PDF into the folder from the app window (600 dpi variant deferred to a future device session) |
| M2 Shell and overview ✅ 27.10.2026 | navigation shell (sidebar + icon rail), `snmp/` (batched GET, blob parsing), `status/` (cache, 5-min polling, `status` event), «Обзор» screen; the print screen stays in M3 | toner/status matches the web panel (live check 27.10.2026, `docs/oids-l2710dw.md` §6) |
| M2.5 History, print, ADF ✅ 27.10.2026 | silent SQLite history on every successful poll (`history/`, `~/.local/share/prindeck/history.db`); `print/` + the «Печать» screen (queue-first: copies 1–99, duplex, active CUPS queue with cancel, `LC_ALL=C`); ADF multi-sheet per the spike verdict — one whole-stack PDF / per-page JPEG (`-pN`), partial keep on cancel, truncated-document guard (`%%EOF`/`FF D9`), stack-scaled fetch timeout, A4 region 3507 | the live acceptance checklist (§14): 3-sheet PDF → one file (`Pages: 3`) ✅, 3-sheet JPEG → three `-pN.jpg` ✅, JPEG cancel mid-stack → partial keep ✅, PDF cancel in-flight → «отменено», no file ✅ (log 27.10.2026 20:18), history DB accumulates and survives a restart ✅; remaining §14 items: power-cycle probe, formal visual check, duplex print confirmation |
| M3 Printing — delivered in M2.5 | moved into M2.5 (`print/`, «Печать» screen, duplex via `sides=`); the leftover F3 extras — quality/resolution/toner saving from the queue's PPD — are cut from v1 and deferred to v2 | a duplex print from the UI flips like a book (pending live check, §14) |
| M4 Monitoring | `get_history` IPC + history chart, notifications, tray menu (the recording itself shipped in M2.5) | notification when toner < 15 % |
| M5 Release | icon, bundler (deb/rpm/AppImage), README+license, GitHub | repository published, install from the artifact |
| v2 | mDNS discovery, OCR add-on, «Scan to PC» from the device body, other distributions | — |

«Scan to PC» (v2), clarified 27.10.2026 from the reference: the Windows client
receives the button press via WIA/STI push events plus two dispatch services —
not portable. The Linux path: register prindeck as a scan destination in the
device web panel (EWS scan-destination/Scan-to-PC pages, see §13.10) and catch
the press by polling eSCL `ScannerStatus` — spike first (§13.11).

## 13. Spikes and open questions

1. ~~Exact XML names of eSCL fields~~ — **resolved at M0**: see §7.1 (ColorMode
   `BlackAndWhite1`/`Grayscale8`/`RGB24`; no 150 dpi; PDF from the firmware available).
2. ~~Toner % via Printer-MIB~~ — **resolved at M0**: the MIB returns `-3`/`-2`;
   the percentage — only from the blob (`#81`), verified against the web (40 %).
3. ~~ADF multi-page~~ — **resolved at M2.5 (27.10.2026, spike
   `docs/escl-l2710dw.md` §11)**: delivery is split by format — the firmware
   assembles the whole stack into ONE multi-page PDF (2 sheets → one 2-page
   document); JPEG arrives one document per page. The sane-airscan
   "≥ 1 s between `NextDocument`" quirk does not manifest on this unit —
   back-to-back fetches are clean.
4. ~~Stack: A vs B~~ — **resolved 26.10.2026: B (Tauri 2)**.
5. ~~PDF merging on the client~~ — **not needed at all**: the firmware merges
   whole stacks (№3). The M2.5 `lopdf` merge module was added per the original
   plan and removed once the spike proved this (§10) — no merge crate remains.
6. License for GitHub: **MIT or Apache-2.0** — decide by M5 (dual is possible).
7. ~~Vanilla DOM vs Preact for the UI~~ — **resolved at M2 (27.10.2026)**: vanilla
   TypeScript modules chosen (two screens ≈ 700 lines of DOM code). Revisit
   when `ui/src/screens/print.ts` exceeds ~400 lines.
8. **`snmp2` vs 2-byte Counter32**: the crate may fail to parse `1E49`;
   in M2 check `prtMarkerLifeCount` live, otherwise take pages from
   `brInfoCounter #00` (correct 4 bytes).
9. The semantics of `#63`/`#31`/`#6f` of the maintenance blob is **unknown** (`#6f` = 34 % ≠
   40 %) — do not interpret, skip with a log.
10. ~~**EWS deep-link paths**~~ — **resolved 27.10.2026** (live check; GET-only
    plus one user-authorized login; `docs/ews-l2710dw.md`): public zone verified
    (status, maintenance info, sleep, auto power-off, replace toner…); behind
    the login the reference admin/network paths exist (`/admin/date.html`,
    `/admin/firmwareupdate.html`, `/net/wired|wireless/link_tcpip_detail.html`,
    `/net/net/sntp.html`), the scan paths do not (`/scan/scandestination.html`,
    `/scan/scantopc.html` → 404 — the real page is `/scan/scanfrompc.html`,
    only a Pull Scan toggle). R1 ships the full verified set; credentials stay
    out of the repo.
11. **Scan-to-PC mechanism** (v2 gate, §12): the panel path is **dead** —
    verified 27.10.2026 that the EWS offers no destination registration
    (`/scan/scanfrompc.html` is only a Pull Scan Enabled/Disabled toggle; the
    iPrint&Scan destination pages are 404; `ScannerCapabilities` declares
    `xmlns:dest` with no elements). Remaining question for the spike: what the
    device does when the hardware scan button is pressed — does anything
    appear in eSCL `ScannerStatus`/`ScanJobs`, or is the button software-bound
    (Windows-only) on this model? If the latter, §12 «Scan to PC» is dropped
    or re-scoped.
12. **ADF cancel mid-stack** — both paths observed live 27.10.2026: between
    pages (JPEG: remaining sheets stay in the feeder, received pages kept) and
    in-flight (PDF: the device closes the chunked stream cleanly — the
    truncated-document guard rejects the partial body, «отменено», no file).
    Known residual: a cancel landing between pages is reported by the client
    as a normal completion (the terminal `404` of a canceled job is
    indistinguishable from the end of the stack) — the file content is
    correct, only the «неполный» label is lost; revisit if it bothers.

## 14. Acceptance

- `cargo test` — offline unit tests: XML parsing on fixtures taken from the device
  (capabilities/status), Supply/Counters mappings, parsing of `lpstat` output on the
  captured fixture, the ADF fetch-loop terminal table and file naming (`-pN`,
  `-partial`), history round-trip; `tsc --noEmit` for the UI. All green at M2.5.
- Live checklist `LIVE=1` (manually against the real device): GET capabilities/
  status, POST scan-job at 300 dpi Platen (150 dpi is not supported), `lp` test
  page, tray menu, notification at low toner.
- M2.5 live acceptance (**pending**): 3-sheet ADF PDF → one file, `pdfinfo`
  reports `Pages: 3` and page size ≈ 602.64 × 830.16 pt (the M0 reference),
  opens in GNOME Papers and Firefox;
  3-sheet JPEG → three `-pN.jpg` with previews; cancel on sheet 2 →
  `…-partial.pdf`, device `Idle`, no temp files; the history DB accumulates rows
  over one poll interval and survives an app restart; a duplex print from the UI
  flips like a book.
- Acceptance v1: from the app window in ≤ 3 clicks get a PDF from a scan; toner in the UI
  matches the web panel ±1 %; after the printer is disconnected the UI stays alive
  (offline badge), after it is switched on — recovers by itself; closed window → tray,
  a second launch does not create a duplicate (single-instance).
