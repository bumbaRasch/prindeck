# M2 Shell and Device Overview Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a navigation shell (sidebar plus content) and a second screen that shows the printer's state, toner, drum and counters read over SNMP.

**Architecture:** Rust gains an SNMP client and a status poller behind three IPC calls and one event; the UI splits from one module into a shell, a router, a shared state and one module per screen, then renders the new Overview screen from the status payload. Both halves are independent until the Overview screen consumes the payload.

**Tech Stack:** Tauri 2, Rust 2024, `snmp2` 0.5 (v2c, blocking), vanilla TypeScript with strict flags, Vite, plain CSS.

**Spec:** `docs/superpowers/specs/2026-10-27-m2-shell-and-overview-design.md`

## Global Constraints

- Every source file stays under 200 lines; split by domain, never name a module `utils`, `common` or `helpers` (AGENTS.md).
- No comments in code. Identifiers carry meaning; explanations live in `docs/`.
- All code strings — errors, logs, test messages — are English. Russian UI copy lives only in `ui/index.html`, `ui/src/labels.ts` and `*_LABEL` constants in Rust.
- Commit gate, all green: `cargo test --workspace`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `npm run typecheck`, `npm run build`, `npm run check:code-language`.
- Device: `192.168.187.36`, community `public`, SNMP port 161, eSCL `http://192.168.187.36/eSCL/`.
- Never start a scan or occupy the printer without an explicit request from the user; SNMP GETs are read-only and safe.
- Window: default 900×680, `minWidth: 640`, `minHeight: 480`.
- UI: sidebar 224 px at ≥ 820 px window width, 64 px icon rail at 640–819 px; content reflows by container query at 560 px.
- Both themes follow `prefers-color-scheme`; every new component must be verified in light and dark.
- Status poll: `pollSeconds` default 300, clamped to 60–3600; request timeout 5 s; one retry at poll level; the blocking SNMP client runs on `spawn_blocking`.
- No new npm dependencies, no UI framework, no JS test runner in M2.

## Review Focus

- **Printer off for hours while the app runs hidden.** The chip must not stay green: the next SNMP poll fails, the chip turns to `Нет связи`, and the numbers keep their old timestamp. Covered by `failed_poll_keeps_the_last_good_values` (Task 5) and the live offline check (Task 8/9).
- **SNMP blocked by a firewall while eSCL works.** No "device unavailable" banner, scanning still works, the Overview says only that consumption data is unavailable. Covered by the wrong-host and blocked-port checks in Task 8 step 5.
- **Window dragged to 640×480 with a long Russian label.** Nothing clips, the rail keeps every destination reachable by keyboard, the field grid collapses to one column. Covered by the visual check at the minimum size in Task 7/8.
- **A scan finishes while the user is on Overview.** No auto-navigation, the scan row shows the dot, and returning to Scan shows the result and the path. Covered by the manual check in Task 8 step 5.
- **A device returns a truncated or unfamiliar SNMP record.** No panic, missing values render `—`, unknown ids are logged and skipped. Covered by `truncated_blob_is_an_error` and `unknown_record_ids_are_skipped` (Task 3).

## File Structure

Rust (`src-tauri/src/`):

| File | Responsibility |
|---|---|
| `commands/mod.rs` | `AppState`, IPC payload types, `emit_*` helpers, module re-exports |
| `commands/caps.rs` | `get_capabilities`, `get_startup_error`, `open_scan_folder` |
| `commands/scan.rs` | `start_scan`, `cancel_scan`, `run_scan`, `fetch_document` |
| `commands/scan_files.rs` | part path, unique path, temp-file removal, format sniffing |
| `commands/status.rs` | `get_status`, `refresh_status`, `open_printer_panel` |
| `commands/tests/mod.rs` (+ siblings) | the unit tests moved out of the old single file |
| `config/mod.rs` | `Config`, `DeviceConfig`, `SnmpConfig`, validation, load/create |
| `config/host.rs` | host validation and `base_url` |
| `config/tests.rs` | config unit tests |
| `snmp/mod.rs` | `SnmpCounters`, mapping record ids to fields |
| `snmp/blob.rs` | the `<id> 01 04 <u32> FF` record grammar |
| `snmp/client.rs` | `SnmpClient::fetch` — one PDU, three varbinds, 5 s timeout |
| `status/mod.rs` | `DeviceStatus`, cache, poll task |
| `status/poll.rs` | interval loop, retry, `spawn_blocking` |
| `lib.rs` | builder wiring: poll task start, command registration |

UI (`ui/src/`):

| File | Responsibility |
|---|---|
| `main.ts` | bootstrap only: create state, init shell and screens, register listeners |
| `state.ts` | `AppState`, `createState`, mutations, reachability selector |
| `shell.ts` | shell DOM, `renderShell`, banners, status line, sidebar status block |
| `router.ts` | `Screen`, `setScreen`, keyboard handling, `navigate` event |
| `dom.ts` | `mustQuery`, `fillSelect` |
| `caps.ts` | capabilities helpers: pick resolution/format/color, normalize selection |
| `screens/scan.ts` | scan screen DOM, `renderScanScreen`, scan actions |
| `screens/overview.ts` | overview screen DOM, `renderOverviewScreen` |
| `api.ts`, `types.ts`, `labels.ts` | unchanged responsibilities, extended types |

## Parallelism

Tasks are grouped so that a wave can be dispatched to independent subagents at
the same time; a task starts only after every task it consumes is merged.

| Wave | Tasks | Why they do not collide |
|---|---|---|
| 1 | 1, 3, 6 | separate files: `commands/`, `snmp/`, `ui/src/` |
| 2 | 2, 4, 7 | 2 needs 1 (same `commands/mod.rs`), 4 needs 3, 7 needs 6 |
| 3 | 5 | needs 2 and 4, and it is the only task editing `lib.rs` command wiring |
| 4 | 8 | needs 5 (payload) and 7 (shell) |
| 5 | 9 | needs the finished app for the live checks |

A wave-1 task that fails its gate must be fixed before wave 2 starts, because
wave 2 builds on its interfaces.

---

### Task 1: Split `commands.rs` into modules (no behaviour change)

**Files:**
- Create: `src-tauri/src/commands/mod.rs`, `commands/caps.rs`, `commands/scan.rs`, `commands/scan_files.rs`, `commands/tests/mod.rs`
- Delete: `src-tauri/src/commands.rs`
- Modify: nothing outside the new module tree — `pub mod commands;` resolves to `commands/mod.rs` unchanged

**Interfaces:**
- Produces: `commands::{AppState, ScanPhase, ScanProgress, ScanDone, ScanError}`, `commands::caps::{get_capabilities, get_startup_error, open_scan_folder}`, `commands::scan::{start_scan, cancel_scan}`, `commands::scan_files::{part_path, unique_scan_path, remove_part, format_matches}`.
- Consumes: nothing new; `escl`, `config` unchanged.

- [ ] **Step 1: Move the code, keep every item public as today**

Cut the existing file by responsibility into the four modules above, with `commands/mod.rs` holding `AppState`, the payload structs and `emit_*`. Re-export at the bottom of `mod.rs`:

```rust
pub use caps::{get_capabilities, get_startup_error, open_scan_folder};
pub use scan::{cancel_scan, start_scan};
```

- [ ] **Step 2: Move the tests next to the code they test**

The current `#[cfg(test)] mod tests` block moves into `commands/tests/mod.rs`, with `commands/tests/scan_tests.rs`, `commands/tests/caps_tests.rs`, `commands/tests/error_tests.rs`; each submodule is declared with `#[path = "..."]` only if the module layout needs it — otherwise plain `mod scan_tests;` inside `commands/tests/mod.rs`.

- [ ] **Step 3: Verify nothing regressed**

Run: `cargo test --workspace` — expected: `35 passed`; `cargo clippy --workspace --all-targets -- -D warnings` — expected: no output about warnings.

- [ ] **Step 4: Verify the file sizes**

Run: `wc -l src-tauri/src/commands/*.rs src-tauri/src/commands/tests/*.rs` — every file under 200 lines.

- [ ] **Step 5: Commit**

```bash
git add -A src-tauri/src/commands
git commit -m "M2: split IPC commands into a module tree"
```

---

### Task 2: Split `config.rs` and add the new keys

**Files:**
- Create: `src-tauri/src/config/mod.rs`, `config/host.rs`, `config/tests.rs`
- Delete: `src-tauri/src/config.rs`
- Modify: `src-tauri/src/commands/mod.rs` (type paths only)

**Interfaces:**
- Produces:

```rust
pub struct Config {
    pub device: DeviceConfig,
    pub scan_dir: PathBuf,
    pub snmp: SnmpConfig,
    pub poll_seconds: u64,
}
pub struct SnmpConfig { pub community: String }
pub const DEFAULT_POLL_SECONDS: u64 = 300;
pub const MIN_POLL_SECONDS: u64 = 60;
pub const MAX_POLL_SECONDS: u64 = 3600;
impl Config {
    pub fn poll_interval(&self) -> Duration;   // clamped, never panics
    pub fn snmp_community(&self) -> &str;      // trimmed, may be empty
    pub fn base_url(&self) -> Result<reqwest::Url, ConfigError>;
}
```

- Consumes: `DeviceConfig`, `ConfigError` as today.

- [ ] **Step 1: Write the failing tests**

In `config/tests.rs`, next to the moved tests:

```rust
#[test]
fn poll_seconds_default_is_300() { /* defaults_for(...).poll_interval() == Duration::from_secs(300) */ }

#[test]
fn poll_seconds_below_minimum_is_clamped() { /* 5 -> 60 */ }

#[test]
fn poll_seconds_above_maximum_is_clamped() { /* 999999 -> 3600 */ }

#[test]
fn missing_new_keys_use_defaults() { /* JSON without `snmp` and `pollSeconds` parses */ }

#[test]
fn invalid_host_is_still_a_hard_error() { /* from_json_str keeps Err(InvalidHost) */ }
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test --workspace config::` — expected: compile error or assertion failures for the new behaviour.

- [ ] **Step 3: Implement the split, defaults and clamping**

`Config` fields get `#[serde(default = "...")]`; `poll_seconds` clamps inside `poll_interval`; `snmp.community` trims, falls back to `public` when absent and logs a `warn` when the value was clamped. Existing hard errors (host, `scanDir`, JSON) stay untouched.

- [ ] **Step 4: Run the tests**

Run: `cargo test --workspace` — expected: `40 passed` (35 + 5); `wc -l src-tauri/src/config/*.rs` — every file under 200 lines.

- [ ] **Step 5: Commit**

```bash
git add -A src-tauri/src/config src-tauri/src/commands/mod.rs
git commit -m "M2: config module split, snmp community and poll interval"
```

---

### Task 3: SNMP blob parsing

**Files:**
- Create: `src-tauri/src/snmp/mod.rs`, `snmp/blob.rs`, `snmp/tests.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod snmp;`)

**Interfaces:**
- Produces:

```rust
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SnmpCounters {
    pub toner_percent: Option<u32>,
    pub drum_percent: Option<u32>,
    pub pages_total: Option<u32>,
    pub duplex_pages: Option<u32>,
    pub drum_pages_left: Option<u32>,
}
pub fn parse_records(blob: &[u8]) -> Result<Vec<(u8, u32)>, BlobError>;
pub fn counters_from(maintenance: &[u8], counter: &[u8], next_care: &[u8]) -> SnmpCounters;
```

- Consumes: the grammars in `docs/oids-l2710dw.md` §3.

- [ ] **Step 1: Write the failing tests with the documented hex**

```rust
#[test]
fn parses_maintenance_blob() {
    let blob = hex("6301040000000141010400000e1011010400001e4931010400000007\
                    6f010400000d4881010400000028ff");
    let records = parse_records(&blob).unwrap();
    assert_eq!(records, vec![(0x63, 1), (0x41, 3600), (0x11, 7753), (0x31, 7), (0x6f, 3400), (0x81, 40)]);
}

#[test]
fn maps_counters_from_three_nodes() { /* toner 40, drum 36, pages 7753, duplex 3951, drum left 4247 */ }

#[test]
fn truncated_blob_is_an_error() { /* missing FF -> Err */ }

#[test]
fn unknown_record_ids_are_skipped() { /* a record id not in the mapping list -> Ok, field None */ }
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test --workspace snmp::` — expected: `cannot find function parse_records`.

- [ ] **Step 3: Implement `parse_records` and `counters_from`**

`parse_records` walks `<id:1> <01> <04> <u32 BE>` until `0xFF`, returns `Err(BlobError::Truncated)` on a short record and `Err(BlobError::MissingTerminator)` on a missing `0xFF`. `counters_from` maps `0x81`→toner, `0x41`→drum percent (integer division of the hundredths), `0x11`→pages, `0x06`→duplex, `0x82`→drum pages left; a parse error in one node leaves that node's fields `None` and logs a `warn`.

- [ ] **Step 4: Run the tests**

Run: `cargo test --workspace` — expected: `44 passed`.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/snmp src-tauri/src/lib.rs
git commit -m "M2: SNMP blob parsing for toner, drum and counters"
```

---

### Task 4: SNMP client (one PDU, three varbinds)

**Files:**
- Create: `src-tauri/src/snmp/client.rs`, extend `snmp/tests.rs`
- Modify: `src-tauri/src/snmp/mod.rs` (re-export), `src-tauri/Cargo.toml` (`snmp2 = "0.5"`)

**Interfaces:**
- Produces:

```rust
pub struct SnmpClient { /* host, community, timeout */ }
impl SnmpClient {
    pub fn new(host: &str, community: &str) -> SnmpClient;
    pub fn fetch(&self) -> Result<SnmpCounters, SnmpError>;   // blocking, call on spawn_blocking
}
pub enum SnmpError { Transport(String), Protocol(String) }
```

- Consumes: `counters_from` from Task 3.

- [ ] **Step 1: Write the failing test for the error path**

```rust
#[test]
fn unreachable_host_is_a_transport_error() {
    let client = SnmpClient::new("127.0.0.1:9", "public");
    assert!(matches!(client.fetch(), Err(SnmpError::Transport(_))));
}
```

- [ ] **Step 2: Run it to see it fail**

Run: `cargo test --workspace snmp::client` — expected: compile error, the client does not exist.

- [ ] **Step 3: Implement the client**

`SyncSession::new_v2c("host:161", community.as_bytes(), Some(Duration::from_secs(5)), 0)`, then one `get(&[&maintenance, &counter, &next_care])`; read three `OctetString` varbinds in order; anything else is `SnmpError::Protocol`. The OIDs are the constants from `docs/oids-l2710dw.md` §2 (with `.0`).

- [ ] **Step 4: Run the tests and the gate**

Run: `cargo test --workspace` — expected: `45 passed`; `cargo clippy --workspace --all-targets -- -D warnings` — clean.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/snmp src-tauri/Cargo.toml Cargo.lock
git commit -m "M2: SNMP client with one batched GET"
```

---

### Task 5: Status cache, poller and IPC

**Files:**
- Create: `src-tauri/src/status/mod.rs`, `status/poll.rs`, `status/tests.rs`, `src-tauri/src/commands/status.rs`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/Cargo.toml` (`tokio` feature for `spawn_blocking` if not already available through Tauri)

**Interfaces:**
- Produces:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
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
pub fn start_polling(app: tauri::AppHandle);
pub fn cached(app: &tauri::AppHandle) -> DeviceStatus;
pub async fn poll_now(app: tauri::AppHandle) -> DeviceStatus;
```

Commands: `get_status() -> DeviceStatus`, `refresh_status() -> Result<DeviceStatus, String>`, `open_printer_panel(state) -> Result<(), String>`.

- Consumes: `SnmpClient` (Task 4), `Config::poll_interval`/`snmp_community` (Task 2), opener helper already used by `open_scan_folder`.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn failed_poll_keeps_the_last_good_values() { /* apply success once, then failure -> numbers stay, ok=false, error set, polled_at unchanged */ }

#[test]
fn first_snapshot_is_empty_and_not_ok() { /* fresh cache: ok false, polled_at None, error None */ }

#[test]
fn timestamp_is_rfc3339_utc() { /* parse with chrono::DateTime::parse_from_rfc3339 */ }
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test --workspace status::` — expected: compile error.

- [ ] **Step 3: Implement the cache and the poller**

`StatusCache` holds `Mutex<DeviceStatus>`; `poll_now` builds `SnmpClient` from the config, calls `fetch` inside `spawn_blocking`, maps success/failure into the cache, and emits `status` with the new snapshot. `start_polling` runs a `tokio` interval of `config.poll_interval()`, one poll per tick, one retry after 2 s, and stops when the app exits. `get_status` returns the cache without touching the network; `refresh_status` awaits `poll_now` and returns an error string only when the poll failed and there is no previous good data.

- [ ] **Step 4: Wire it**

`lib.rs`: call `status::start_polling(app.handle().clone())` in `setup` and register the three commands. `commands/status.rs` holds the command bodies; `commands/mod.rs` re-exports them.

- [ ] **Step 5: Run the gate**

Run: `cargo test --workspace` — expected: `48 passed`; clippy clean; `wc -l src-tauri/src/status/*.rs src-tauri/src/commands/status.rs` — under 200.

- [ ] **Step 6: Commit**

```bash
git add -A src-tauri/src src-tauri/Cargo.toml Cargo.lock
git commit -m "M2: status cache, SNMP poller and IPC"
```

---

### Task 6: UI module split and strict TypeScript (no behaviour change)

**Files:**
- Create: `ui/src/state.ts`, `shell.ts`, `router.ts`, `dom.ts`, `caps.ts`, `screens/scan.ts`
- Modify: `ui/src/main.ts` (bootstrap only), `ui/src/types.ts`, `tsconfig.json`

**Interfaces:**
- Produces:

```ts
// types.ts
export type Screen = "scan" | "overview";
export interface DeviceStatus {
  ok: boolean; host: string; tonerPercent: number | null; drumPercent: number | null;
  pagesTotal: number | null; duplexPages: number | null; drumPagesLeft: number | null;
  polledAt: string | null; error: string | null;
}
// state.ts
export interface AppState { /* today's fields + status: DeviceStatus | null */ }
export function createState(): AppState;
// router.ts
export function initRouter(onChange: (screen: Screen) => void): void;
export function setScreen(screen: Screen): Screen;
export function currentScreen(): Screen;
// shell.ts
export function initShell(): void;
export function renderShell(state: AppState): void;
// screens/scan.ts
export function initScanScreen(getState: () => AppState): void;
export function renderScanScreen(state: AppState): void;
```

- Consumes: today's `main.ts` behaviour, `api.ts`, `labels.ts`.

- [ ] **Step 1: Move the code without changing behaviour**

`dom.ts` gets `mustQuery`/`fillSelect`; `caps.ts` gets `isSource`/`isColorMode`/`isDocFormat`/`pickFirst`/`pickDpi`/`pickFormat`/`activeInput`/`normalizeSelections`; `state.ts` gets the state object, `createState`, `pluralPages`; `screens/scan.ts` gets the scan DOM handles, `renderScanScreen` and the scan actions; `main.ts` keeps `init()`, listener registration and the command calls.

- [ ] **Step 2: Turn on the strict flags**

`tsconfig.json` adds `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `noImplicitOverride`, `noImplicitReturns`, `noPropertyAccessFromIndexSignature`, `allowUnreachableCode: false`, `verbatimModuleSyntax`.

- [ ] **Step 3: Fix what the flags reveal**

Run: `npm run typecheck` — expected: FAIL on indexed access and optional properties; fix each with an explicit narrow (no `any`, no non-null assertion).

- [ ] **Step 4: Verify the app still behaves and the files are small**

Run: `npm run typecheck && npm run build && npm run check:code-language`; `wc -l ui/src/*.ts ui/src/screens/*.ts` — every file under 200 lines.

- [ ] **Step 5: Commit**

```bash
git add -A ui/src tsconfig.json
git commit -m "M2: UI modules and stricter TypeScript"
```

---

### Task 7: Shell markup, sidebar, rail breakpoint, icons

**Files:**
- Modify: `ui/index.html`, `ui/src/styles.css`, `ui/src/main.ts`, `ui/src/shell.ts`, `ui/src/router.ts`, `ui/src/labels.ts`, `src-tauri/tauri.conf.json`
- Create: `ui/src/screens/overview.ts` (placeholder markup only, filled in Task 8)

**Interfaces:**
- Produces: `[data-screen]` sections for `scan` and `overview`; nav buttons `[data-nav="scan"|"overview"]`; `#device-status` block in the sidebar; `#status` live region in the content header; inline sprite symbols `#i-scan`, `#i-gauge`.

- Consumes: `renderShell` (Task 6), `setScreen`/`initRouter` (Task 6).

- [ ] **Step 1: Restructure `index.html`**

`#shell` grid: `<nav class="sidebar">` (app name, status block, nav list) and `<main class="content">` (header with `h1` + `#status`, banners, then `<section data-screen="scan">` and `<section data-screen="overview">`). Move the two banners and the status line out of the scan form. Add the inline SVG sprite with Lucide `scan-line` and `gauge` paths.

- [ ] **Step 2: Add the tokens, the grid and the rail breakpoint**

`--sidebar-w: 224px`; `--sidebar-bg`, `--nav-active-bg`, `--nav-active-text`, `--nav-accent` for both themes; `#shell { display: grid; grid-template-columns: var(--sidebar-w) minmax(0, 1fr); height: 100vh }`; `@media (max-width: 819px)` → `--sidebar-w: 64px`, labels hidden but present for `aria-label`/`title`; container query on the content column replaces the 560 px media query.

- [ ] **Step 3: Implement navigation**

`router.ts` toggles `hidden` on sections, sets `aria-current="page"`, moves focus to the screen `h1`, and handles ArrowUp/ArrowDown/Enter/Space with the list as a single Tab stop.

- [ ] **Step 4: Set the window minimum**

`tauri.conf.json`: `"minWidth": 640, "minHeight": 480`.

- [ ] **Step 5: Verify**

Run: `npm run typecheck && npm run build && npm run check:code-language`; `wc -l ui/src/*.ts ui/src/screens/*.ts ui/index.html ui/src/styles.css` — under 200 each; start the app and check 900×680, 820×680 and 640×480 in both themes.

- [ ] **Step 6: Commit**

```bash
git add -A ui src-tauri/tauri.conf.json
git commit -m "M2: sidebar shell, responsive rail and window minimum"
```

---

### Task 8: Overview screen and the live scan indicator

**Files:**
- Modify: `ui/src/screens/overview.ts`, `ui/src/shell.ts`, `ui/src/state.ts`, `ui/src/api.ts`, `ui/src/labels.ts`, `ui/index.html`

Task 7 creates `ui/src/screens/overview.ts` as markup plus two empty exports; this task fills them.

**Interfaces:**
- Produces: `initOverviewScreen(getState, refresh)`, `renderOverviewScreen(state)`, `api.getStatus/refreshStatus/openPrinterPanel`, `connectivity(state)` in `state.ts` returning `"checking" | "online" | "offline" | "scanning"`.
- Consumes: `DeviceStatus` (Task 5), shell markup (Task 7).

- [ ] **Step 1: Add the IPC wrappers**

`api.ts`: `getStatus(): Promise<DeviceStatus>`, `refreshStatus(): Promise<DeviceStatus>`, `openPrinterPanel(): Promise<void>`, `onStatus(handler)`; `types.ts` already has the payload.

- [ ] **Step 2: Implement the chips and the cards**

Chip text from `connectivity(state)`; the consumables card renders `—` for `null`; the connection card shows model (from capabilities), host, `обновлено N с назад` (ticking every 10 s) and a refresh button that calls `refreshStatus` and disables while pending.

- [ ] **Step 3: Implement the eSCL knowledge line**

`state.ts` records `esclFailed: boolean` from the last capabilities fetch, scan start or scan failure; the Overview shows `сканирование недоступно` plus a retry only when it is true. A failed status poll plus `esclFailed` shows the head banner once per state change.

- [ ] **Step 4: Add the scan dot in the nav row**

`renderShell` sets `data-active` on `[data-nav="scan"]` while `state.jobId !== null`; no text, no navigation.

- [ ] **Step 5: Verify**

Run: `npm run typecheck && npm run build && npm run check:code-language`; with the app running, check: offline (wrong host), scanning from the Overview, returning to the Scan screen with the result shown, 900×680 and 640×480, both themes.

- [ ] **Step 6: Commit**

```bash
git add -A ui
git commit -m "M2: device overview screen and scan indicator"
```

---

### Task 9: Documentation and live verification

**Files:**
- Modify: `PRD.md` (§6, §8, §9, §12, §13.7), `README.md`, `docs/oids-l2710dw.md` (multi-varbind note), `docs/superpowers/plans/2026-10-27-m2-shell-and-overview.md`

- [ ] **Step 1: Update the PRD rows listed in the spec §11**

- [ ] **Step 2: Record the live SNMP check**

Extend `scripts/probe-snmp.mjs` with one batched GET; run it against the device; paste the decoded values into `docs/oids-l2710dw.md` §6 and confirm they still match the web panel.

- [ ] **Step 3: Run the whole gate and the visual check**

`cargo test`, `cargo check`, clippy, `npm run typecheck`, `npm run build`, `npm run check:code-language`, then the `visual-check` subagent at 900×680 and 640×480 in both themes.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "M2: docs and live SNMP verification"
```

---
