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
   a refresh button. Chip states: `Проверяем…` (no poll has finished yet),
   `На связи`, `Нет связи`, `Сканируем`. Grey + old timestamp for stale data.
2. **Consumables**: toner % and drum % as two bars with numeric labels. A value
   the device did not report renders as `—`, never as 0.
3. **Counters**: pages printed, duplex pages, drum pages left.
4. **Actions**: `Скан` (switch to the scan screen), `Веб-панель` (a new
   `open_printer_panel` command from PRD §9, which builds the URL from the
   configured host).

The device model leaves the scan header: with two screens it belongs to the
device card, not to every screen title.

Stale data is never silently fresh: when the last poll failed, the chip turns
grey, the timestamp and the last good values stay visible.

**Two independent failures, two signals.** The chip reports the status poll
(SNMP). The scan screen keeps its own banner and retry for eSCL capabilities.
They normally fail together (same host), but a firewall can block SNMP while
eSCL works, so the app never claims "device unavailable" on the scan screen
because of a status poll, and never hides a broken scanner behind a green chip.
The status poll disables nothing on the scan screen.

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
- Poll interval: `pollSeconds`, default 300. Values below 60 clamp to 60 with a
  log line, garbage falls back to the default; validation never throws the rest
  of the config file away. One in-flight request, 5 s timeout, one retry, then
  offline. The task starts in `setup` and stops with the app.
- SNMP community: `snmp.community`, default `public`; an empty value is allowed.
  The SNMP port is fixed at 161 for M2.

### 7.7 IPC and config

```rust
pub struct DeviceStatus {
    pub online: bool,
    pub host: String,
    pub model: Option<String>,
    pub toner_percent: Option<u32>,
    pub drum_percent: Option<u32>,
    pub pages_total: Option<u32>,
    pub duplex_pages: Option<u32>,
    pub drum_pages_left: Option<u32>,
    pub polled_at: Option<String>,
    pub error: Option<String>,
}
```

- `get_status()` returns the **cached** snapshot and never touches the network,
  so the first paint is instant. Before the first poll finishes `polled_at` is
  `None` and the UI shows `Проверяем…`; a cache hit from a failed poll keeps the
  last good numbers with the old `polled_at` and a non-empty `error`.
- `refresh_status()` performs one poll now and returns the snapshot; it backs
  the Overview refresh button and the retry after a failure.
- `open_printer_panel()` opens `http://<host>/` through the opener plugin
  (PRD §9).
- Event `status` carries the same payload after every poll.
- Config shape, all new keys optional with the stated defaults:

```json
{
  "device": { "host": "192.168.187.36" },
  "scanDir": "/home/user/Documents/scans",
  "snmp": { "community": "public" },
  "pollSeconds": 300
}
```

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
