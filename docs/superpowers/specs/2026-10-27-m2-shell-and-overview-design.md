# M2 design: navigation shell and device overview

Date: 27.10.2026. Status: draft for approval.
Predecessor: M1 (single "Scan" screen, eSCL scanning, tray, single-instance).

## 1. Intent

M1 ships one screen. Every later milestone adds another: overview (M2),
print (M3), settings, history. Building the overview inside the current
single-screen layout means rebuilding it at M3, so the navigation shell lands
with the second screen, not after it.

Why a sidebar and not tabs: the app must show the device state on every screen
(PRD §8 "the state is always visible"), the scarce dimension of a 900×680 window
is height, and the roadmap already reaches five destinations, where a tab strip
is the documented wrong answer.

## 2. Goals

1. A shell with a left sidebar and a content area; both themes (light/dark,
   following the system) work for every new component.
2. Two destinations: **Scan** (start screen, one click to a PDF) and **Overview**
   (device state, toner, drum, counters).
3. `snmp.rs` + `status.rs`: SNMP polling every 300 s, mapped to one
   `DeviceStatus`, exposed as `get_status` and a `status` event.
4. The UI split into modules (no framework) with maximal TypeScript strictness.
5. `npm run check:code-language` still passes: new Russian copy only in
   `ui/index.html`, `ui/src/labels.ts` and `*_LABEL` constants.

## 3. Non-goals (M2)

- Print screen and `print.rs` (M3), Settings screen and config IPC (M2c),
  history chart and notifications (M4), i18n, ADF multi-page.
- Real-time device state (`Idle`/`Processing`) — see §7.6.
- Manual theme switch: both themes follow `prefers-color-scheme`; a switch is
  a Settings item later.

## 4. Anti-goals

- Placeholder or disabled navigation items ("Print — soon").
- Auto-navigation on background events (a finished scan must not steal the
  current screen).
- Icon-only rail, animated collapse, or a hamburger at desktop widths.
- Two sources of truth for device state: the shell footer and the screen must
  render the same `DeviceStatus`.
- Introducing a UI framework while two screens fit in vanilla DOM.
- Russian text outside the three allowed places.

## 5. Constraints

- Window 900×680 default; the layout must stay usable down to **640×480**
  (`minWidth: 640`, `minHeight: 480` in `src-tauri/tauri.conf.json`, currently
  unset).
- Screen widths drive the layout: ≥ 820 px → sidebar 224 px with labels;
  640–819 px → icon rail 64 px with `title` + `aria-label`; the content column
  reflows by a container query (two field columns ≥ 560 px, one below).
- Vanilla TypeScript, Vite, no runtime dependencies. Icons: inline SVG sprite.
- No code comments (AGENTS.md). Async background work in `tokio::task`.
- Device facts only from `docs/oids-l2710dw.md` and `docs/escl-l2710dw.md`;
  unknown fields are logged and ignored (PRD §5.6).
- The printer is not a test fixture: live checks only on explicit request.

## 6. Approaches

**Navigation shell: sidebar (chosen) / top tab strip / icon rail.**
Tabs eat 44–48 px of the 680 px height, have no home for the device status, and
the GNOME and Windows guidance both cap a view switcher at 3–5 items. An icon
rail needs an invented icon vocabulary and tooltips; Windows only recommends it
when categories are unambiguous as icons. A 224 px sidebar is 25 % of 900 and
inside the libadwaita 180–280 band.

**Screen switching: section toggling (chosen) / hash router / one HTML per
screen.** Toggling matches the existing architecture (one state, listeners
registered once); an in-flight scan survives navigation because only visibility
changes. Deep links and back/forward do not exist in a tray window. Separate
documents would re-run `init()` and drop the scan state.

**Status block: top of the sidebar (chosen) / bottom footer / window-wide
strip.** Apple HIG warns that the bottom edge of a window is often off-screen.
The top block doubles as a link to Overview. A VS Code style bottom strip stays
the fallback if the sidebar gets crowded after History lands.

**UI stack: vanilla modules (chosen) / Preact.** Two screens are ~700 lines of
DOM code; Preact pays off at four screens with a polled jobs table (M3).
Revisit when `ui/src/screens/print.ts` exceeds ~400 lines. The threshold in
PRD §13.7 is updated to this wording.

## 7. Design

### 7.1 Layout

```
┌────────────────────┬──────────────────────────────────────────┐
│ prindeck           │  Скан                          [status]  │
│ ● На связи         ├──────────────────────────────────────────┤
│   Тонер 40 %       │                                          │
│                    │  Источник        Разрешение              │
│ ▌Скан              │  [Планшет  ▾]    [300 dpi   ▾]           │
│  Обзор             │  Цвет            Формат                  │
│                    │  [Оттенки… ▾]    [PDF       ▾]           │
│                    │  [ Сканировать ]  [ Отмена ]             │
└────────────────────┴──────────────────────────────────────────┘
```

- Shell: `#shell { display: grid; grid-template-columns: var(--sidebar-w) minmax(0, 1fr); height: 100vh; }`.
  Sidebar scrolls on its own, the content column scrolls on its own.
- Sidebar: app name, status block, navigation list. Nothing pinned to the
  bottom.
- Responsive rules: `--sidebar-w: 224px` at ≥ 820 px window width; at
  640–819 px the sidebar becomes a 64 px rail (icons only, label hidden but
  kept in `title` and `aria-label`, status block reduced to the state dot).
  The content column reflows with a container query: the field grid is two
  columns while the column is ≥ 560 px wide, one column below, and the action
  row stacks when the primary button would drop under ~200 px.
- Content: a header row (screen title, then the global status line) and the
  screen body. Banners (config error, device unreachable) sit at the top of the
  content column, above the screen body, so they are visible on every screen.
- New tokens, defined for both themes: `--sidebar-bg`, `--nav-active-bg`,
  `--nav-active-text`, `--nav-accent`. Sidebar separation uses `--border`,
  never a shadow (the dark theme sets `--shadow: none`).
- The existing 560 px media query keys on window width and becomes wrong: with
  a sidebar the form's columns live in a narrower box. It becomes a container
  query on the content column.

### 7.2 Navigation

- `type Screen = "scan" | "overview"`; sections carry `data-screen`; inactive
  ones use `[hidden]`.
- Items, in order: **Скан** (default on every launch), **Обзор**. `Печать`
  appears with M3, `Настройки` with M2c.
- Active item: tinted row + 3 px accent bar on the inline-start edge +
  `aria-current="page"`. Hover is the same tint at lower opacity.
- No collapse toggle, no hash, no history. The rail is only a width-driven
  breakpoint, not a user preference.
- Keyboard: the list is one Tab stop; Up/Down move, Enter/Space activate; after
  a switch the focus goes to the screen's `<h1 tabindex="-1">`.
- The **Скан** row carries live scan state as a dot only: accent while a job
  runs, cleared when it ends. No page count — the firmware returns one page per
  job and ADF is unsupported, so the number would always be 1 (it comes back
  with ADF support). The row never steals focus and never navigates.
- In the rail mode (640–819 px) navigation keeps keyboard behaviour and every
  item gets `title` and `aria-label`; the active item stays visible by its tint
  and accent bar.

### 7.3 Shell state and rendering

- One `AppState` object (scan phase, job, pages, last file, caps, status,
  errors) lives in `state.ts` and outlives screen switches.
- Two renderers: `renderShell()` (status block, banners, global status line,
  nav badges) and the per-screen `render()` of `screens/scan.ts` /
  `screens/overview.ts`. `scan-progress` ticks call both, but only the shell
  renderer touches shell nodes.
- The global status line (`role="status" aria-live="polite"`) moves out of the
  scan form into the content header: hiding the form must not silence
  announcements.

### 7.4 Scan screen after the move

Behaviour is unchanged: same fields, same names, same events, same result card.
Only the location of banners and the status line changes. The screen module owns
its DOM and its renderer; it does not own the job lifecycle.

### 7.5 Overview screen

Cards, top to bottom:

1. **Connection**: state chip, the device model and host, `обновлено N с назад`,
   a refresh button.
2. **Consumables**: toner % and drum % as two bars with numeric labels. A value
   the device did not report renders as `—`, never as 0.
3. **Counters**: pages printed, duplex pages, drum pages left.
4. **Actions**: `Скан` (switch to the scan screen), `Веб-панель` (the
   `open_printer_panel` command, which builds the URL from the configured host).

The device model leaves the scan header: with two screens the model and the host
belong to the device card, not to every screen title. The model still comes from
`get_capabilities` (already fetched); the status payload does not repeat it.

**What the chip knows and what it does not.** The chip reflects the only channel
we can watch continuously — the SNMP status poll — with its age (`обновлено N с
назад`). eSCL is never polled: a request may wake the printer (PRD §5.4). So the
chip never claims eSCL freshness; instead, the last eSCL outcome is recorded from
signals we already get for free (`get_capabilities` at startup, the `status()`
probe inside `start_scan`, a transport failure during `NextDocument`, and the
manual retry). A known eSCL failure adds one line to the Overview and the banner
on the Scan screen; when no failure is known, no line is shown — absence of news,
not a promise.

| SNMP poll | last eSCL outcome | Chip | Overview | Scan screen |
|---|---|---|---|---|
| ok / pending | ok | `На связи` / `Проверяем…` | numbers + timestamp | usable |
| failed | ok | `Нет связи` | grey, last values, old timestamp | usable; a failed scan then shows its own banner |
| ok | failed | `На связи` | numbers + `сканирование недоступно` + retry | banner + retry |
| failed | failed | `Нет связи` | grey + `сканирование недоступно` | banner + retry |

- `Сканируем` replaces the chip text while our job runs, and the numbers stay.
- A single head banner appears only when the poll fails **and** eSCL is known to
  fail — a blocked SNMP port never claims the printer is down while scanning
  still works.
- Stale numbers are never presented as live: the card says so in words and keeps
  the old `обновлено N с назад`.
- The banner is announced once per state change (`polite` live region), not on
  every failed poll — otherwise a flapping access point turns into a screen
  reader loop.
- Recovery: the SNMP half recovers on its own at the next successful poll. The
  eSCL half has no timer by design; it recovers on the next scan attempt or via
  the retry button (PRD §14 "recovers by itself" applies to the status poll in
  M2; a background eSCL probe is a later decision with a wake-up budget).

### 7.6 Data sources and polling

- Consumables and counters come from SNMP only: `brInfoMaintenance` `#81`
  (toner %) and `#41` (drum %), `#11` (pages), `brInfoCounter` `#00`/`#06`,
  `brInfoNextCare` `#82`. Grammar and decoded values are in
  `docs/oids-l2710dw.md` §3.
- `hrDeviceStatus` answers `running` even while the device sleeps, so it is not
  used for state.
- eSCL is **not** polled: a status request may wake the device, and PRD §5.4
  requires a light idle footprint. The chip therefore reports connectivity and
  our own scan job, not the device's internal state.
- Poll interval: `pollSeconds`, default 300. Values below 60 clamp to 60 and
  values above 3600 clamp to 3600, with a log line; a value that is not a number
  falls back to the default. Only the **new** keys are repaired this way: a bad
  `device.host`, an empty `scanDir` or invalid JSON stay hard errors with the
  existing banner and tests. The clamped value is logged, not written back to
  the user's file.
- Request shape: one `SyncSession::get` with three varbinds
  (`brInfoMaintenance`, `brInfoCounter`, `brInfoNextCare`) — a single PDU, one
  round trip. `new_v2c(addr, community, Some(5 s), 0)` sets the socket timeout;
  the retry lives at the poll level (one retry after 2 s), not inside `snmp2`,
  so a dead host costs at most ~12 s in the background and never blocks the UI.
- The `snmp2` client is blocking: every request runs inside
  `tauri::async_runtime::spawn_blocking`, never on the async runtime thread.
- On failure the last good snapshot stays in memory. The task starts in `setup`
  and stops with the app.
- SNMP community: `snmp.community`, default `public`, trimmed, an empty value is
  allowed. The SNMP port is fixed at 161 for M2 — no device in this class
  changes it and a second port field invites mistakes.
- The multi-varbind request shape is unverified on this firmware (M0 used one GET
  per OID). Step 2 ships with a live probe through the existing
  `scripts/probe-snmp.mjs`; if the firmware rejects varbind batching, the
  fallback is three sequential GETs inside the same 5 s budget per poll.

### 7.7 IPC and config

```rust
pub struct DeviceStatus {
    pub ok: bool,
    pub host: String,
    pub toner_percent: Option<u32>,
    pub drum_percent: Option<u32>,
    pub pages_total: Option<u32>,
    pub duplex_pages: Option<u32>,
    pub drum_pages_left: Option<u32>,
    pub polled_at: Option<String>,
    pub error: Option<String>,
}
```

`ok` describes the **status poll**, not the printer: `true` only when the last
SNMP batch succeeded. The shell combines it with the eSCL capabilities state to
produce the single chip in §7.5.

- `get_status()` returns the **cached** snapshot and never touches the network,
  so the first paint is instant and no invoke can block for 10 s. No poll yet →
  `polled_at: None`, `ok: false`, `error: None`, which the UI renders as
  `Проверяем…`. After a failed poll the previous numbers stay with their old
  `polled_at` and a non-empty `error`.
- `refresh_status()` performs one poll now and returns the snapshot; it backs the
  Overview refresh button, the consumables retry, and the both-failed banner
  retry.
- `open_printer_panel()` opens `http://<host>/` through the opener plugin
  (PRD §9) — the UI never builds the URL itself.
- Event `status` carries the same payload after every poll. Events are not
  replayed to late listeners, and the poller starts in `setup`, before the
  webview can listen — hence the cached `get_status` on startup, which is the
  documented way to combine "current state" with "updates".
- Config shape, new keys optional with the stated defaults:

```json
{
  "device": { "host": "192.168.187.36" },
  "scanDir": "/home/user/Documents/scans",
  "snmp": { "community": "public" },
  "pollSeconds": 300
}
```

`pollSeconds` is top level: it will later also drive the tray tooltip refresh,
not only SNMP. `snmp` is nested because more SNMP keys are plausible (version,
timeout).

### 7.8 Module map and TypeScript

```
ui/src/
  main.ts              bootstrap: create state, register listeners, first render
  state.ts             AppState type, createState(), setters
  shell.ts             shell DOM, renderShell()
  router.ts            Screen type, setScreen(), keyboard handling
  screens/scan.ts      scan screen DOM and renderer
  screens/overview.ts  overview screen DOM and renderer
  api.ts, types.ts, labels.ts
ui/index.html          shell markup, screens, inline SVG sprite
```

`tsconfig.json` gains `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`,
`noImplicitOverride`, `noImplicitReturns`, `noPropertyAccessFromIndexSignature`,
`allowUnreachableCode: false`, `verbatimModuleSyntax`. No `any`, no `as` casts
except at the IPC boundary, where payloads are narrowed explicitly.

### 7.9 Icons

Lucide (MIT, https://github.com/lucide-icons/lucide), copied as `<symbol>`
paths into an inline sprite in `ui/index.html`: `scan-line`, `gauge`, later
`printer`, `settings`. 18 px, stroke 1.75, `currentColor`. No icon package, no
build step, no extra request; attribution goes to the README credits at M5.

## 8. Order of work

1. **Shell** — `index.html` shell markup, `styles.css` grid + sidebar + rail
   breakpoint + container query + tokens, module split, strict TS flags, icon
   sprite, `minWidth`/`minHeight: 640×480`. One destination (Скан) + status
   block driven by the scan phase. The app works as before, plus navigation
   chrome, at every window size down to the minimum.
2. **Status backend** — `snmp.rs`, `status.rs`, `get_status`, `status` event,
   config keys, unit tests on the documented hex blobs.
3. **Overview** — the second destination, rendering `DeviceStatus`, the stale
   state, and the two actions.

Each step ends in a working app and its own commit.

## 9. Risks

| Risk | Mitigation |
|---|---|
| Sidebar shrinks the scan card; fields and buttons reflow | Content is 676 px at the default size and 576 px in rail mode: two ~250 px columns, single column below 560 px via the container query |
| The visual baseline is stale (all screenshots are one screen) | Re-run `visual-check` after step 1 and step 3, at 900×680 and at 640×480, both themes |
| `mustQuery` throws on a typo and blanks the window | Screen section ids equal `Screen` values; shell ids enumerated once |
| Russian labels are wider than the mock | 224 px fits the longest current label with 40 px headroom; `nowrap` + ellipsis on labels |
| The status event fires while another screen is open | Shell renderer only; no auto-navigation; the scan row shows progress |
| A flapping poll turns the banner into a screen-reader loop | The banner text changes only when the aggregated chip state changes |
| `snmp2` fails on the 2-byte `Counter32` | Pages come from the Brother blob `#11`/`#00`, not from `prtMarkerLifeCount` |
| A wrong SNMP value reaches the UI | Unit tests on the exact hex from the M0 docs; a value the device omits renders as `—` |

## 10. Open items for the builder

- Which SNMP records the parser may skip without losing a displayed value, and
  how a truncated blob is reported (the documented grammar is per node, and a
  mismatch must not take the whole poll down).
- Wording of the English error strings and the Russian UI copy (Russian only in
  `labels.ts`).
- Whether the status block in the sidebar shows the toner percentage while the
  Overview card shows it again; the design says yes (a summary is a summary).

## 11. Documents to update with this milestone

| Document | Change |
|---|---|
| `PRD.md` §6 | `status.rs` merges SNMP only; the shell combines reachability from eSCL capabilities and the status poll |
| `PRD.md` §8 | navigation model: left sidebar, two destinations now, Scan as the start screen, rail below 820 px |
| `PRD.md` §9 | `get_status`, `refresh_status`, `open_printer_panel`, `status` event |
| `PRD.md` §12 | M2 split into shell, status backend and overview |
| `PRD.md` §13.7 | vanilla modules chosen; the revisit trigger is `screens/print.ts` over ~400 lines |
| `AGENTS.md` | already updated: responsive UI, user-first design, MCP lookups instead of memory |
