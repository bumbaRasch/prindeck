# M4: Monitoring — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the monitoring layer — live-applied settings, edge-triggered notifications, status tray, history chart, human status texts (R5–R7) — and the «Настройки» destination.

**Architecture:** Four sequential-merge waves of independent tasks with disjoint file sets (M2.5 precedent): Wave 1 = config live-apply foundation (everything consumes it); Wave 2 = notify module ∥ SNMP fourth varbind + status fields ∥ tray states; Wave 3 = history chart + overview texts ∥ settings screen with EWS links; Task 7 = live acceptance + docs (user-gated). Backend-only notification path so alerts fire with the window closed.

**Tech Stack:** Tauri 2.12 (tray + notification plugin), `tauri-plugin-notification` 2 (new crate, PRD §10-approved), rusqlite (existing), vanilla strict TS + hand-rolled SVG, snmp2 (existing, 4th varbind).

**Spec:** `docs/superpowers/specs/2026-10-27-m4-monitoring-design.md` — the plan argues from it; executors read both.

## Global Constraints

- Precondition: starts on `main` **after** the M2.5 phase merges back (stop-and-ask point, already user-approved).
- Crates: only `tauri-plugin-notification = "2"` joins (PRD §10); justification in its commit.
- Files < 200 lines, split by domain, never `utils`/`common`/`helpers`; Rust 2024, **no code comments**.
- Gate, all green, no warnings: `cargo test --workspace`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`, `npm run typecheck`, `npm run build`, `npm run check:code-language`, `npm run check:line-count`.
- Code strings English; Russian UI copy only in `ui/index.html`, `ui/src/labels.ts`, `*_LABEL` constants.
- Config schema is **current-only — no backward-compatibility shims** (user
  decision 27.10.2026): new keys are required fields with validation bounds;
  an outdated or invalid config fails startup validation loudly through the
  existing `ConfigError` path, and the error names the offending key
  (PRD §5.5). Clean schema over migration defaults.
- Notifications fire backend-side only (poller path), never from the UI; UI gets the `alert` event (PRD §9) for in-app banners.
- Idle cost: one batched SNMP GET per pollSeconds, now with 4 varbinds; verified OIDs only (`docs/oids-l2710dw.md`).
- Responsive 640×480 and up, both themes; one primary action per screen (AGENTS.md).
- Commit prefix `M4:`; branch `main`; parallel waves in per-task worktrees (M2.5 Ruling 5 pattern), sequential merge-back with the full gate after each merge.
- Context7-verified API facts to rely on: `TrayIcon::set_icon(Option<Image>)` / `set_menu` work at runtime (tauri 2.9 docs; on Linux a set menu cannot be removed — we never remove it); `tauri-plugin-notification` Rust API: `app.notification().builder().id(i32).title(..).body(..).show()`; asset scope runtime extension: `app.asset_protocol_scope().allow_directory(path, false)` — **extends only**, an old scan dir stays allowed (harmless, accepted).

## Review Focus

- A stuck low-toner state re-notifies on every poll (spam) → pinned by the transition state-machine test in Task 2 (`notify::evaluate` twice with unchanged values → empty).
- Config missing the new keys silently loads with defaults → forbidden:
  missing key = `ConfigError` naming it → Task 1 test.
- Host change leaves the stale eSCL client / caps cache alive (settings silently lie) → rebuild unit test in Task 1.
- Poll-interval change ignored until restart → per-tick `next_delay` re-read test in Task 1.
- Chart fed 0/1 snapshots or >90 days → empty-state + downsample tests in Task 5.
- Device answer missing the `#a1` blob → `jam_count: None`, UI hides the jam line → parse test in Task 3.

---

### Task 1: Config live-apply foundation + get/set IPC

**Files:**
- Modify: `src-tauri/src/config/mod.rs`, `src-tauri/src/config/store.rs`
- Create: `src-tauri/src/config/notify.rs` (NotifyConfig + TonerConfig structs), `src-tauri/src/commands/settings.rs`
- Modify: `src-tauri/src/commands/mod.rs` (AppState.config → shared handle), `src-tauri/src/status/poll.rs` (per-tick interval), `src-tauri/src/commands/caps.rs` + `lib.rs` (host rebuild path)
- Test: `src-tauri/src/config/tests/` (extend), `src-tauri/src/commands/tests/settings_tests.rs`

**Interfaces:**
- Consumes: existing `Config`, `Config::validate`, `ConfigError` path; M2.5 serde-default pattern.
- Produces (everything downstream relies on these):
  - `ConfigHandle` in `config/mod.rs`: `ConfigHandle::new(Config)`, `fn get(&self) -> Config` (cheap clone), `fn set(&self, Config)`; `AppState.config: ConfigHandle`.
  - `Config` gains required keys (no serde defaults — current-only schema):
    `notify: NotifyConfig { enabled: bool, toner_percent: u8, drum_percent: u8 }`,
    `toner: TonerConfig { model: Option<String>, yield_pages: Option<u32> }`;
    bounds: percents ≤ 100, `yield_pages` ≤ 100_000, else `ConfigError`
    naming the offending key; a config file missing `notify`/`toner` is a
    startup `ConfigError` (dialog), not a silent default.
  - IPC: `get_config() -> Config` (serde-visible DTO of the same shape), `set_config(config: Config) -> Result<Config, String>` — validates, stores via the existing store write path, applies: poller interval (next tick), host → rebuild `EsclClient` + reset caps cache, scan dir → `asset_protocol_scope().allow_directory(dir, false)`.
  - Poller: interval read per tick from `ConfigHandle` (test with a tiny fake interval source).

- [ ] **Step 1: Write failing tests** — missing-key failure (a valid pre-M4 config JSON without `notify`/`toner` → `ConfigError` whose message names the absent key); bounds violation (`toner_percent: 200` → `ConfigError`); per-tick interval re-read (fake handle, interval 5 s → poller's `next_delay` returns 5 s, after `set` with 300 s → 300 s); host rebuild (call the rebuild fn with another host → produced `EsclClient` base URL carries it, caps cache empty).
- [ ] **Step 2: Run** `cargo test --workspace` — expected: FAIL (missing `notify`/`toner` keys, `ConfigHandle`, rebuild fn).
- [ ] **Step 3: Implement** — `ConfigHandle` (RwLock), config structs + defaults, `set_config` applying in this order: validate → store write → swap in handle → live-apply (interval is read per tick, so nothing to do; host rebuild; scan-dir scope extend). Keep `caps.rs::resolve_state` as the single place that constructs `EsclClient` so the rebuild reuses it.
- [ ] **Step 4: Run the full Gate** — expected: PASS.
- [ ] **Step 5: Commit** — `M4: config live-apply foundation, get/set_config IPC`

### Task 2: notify module — edge-triggered rules + notifications (Wave 2 ∥)

**Files:**
- Create: `src-tauri/src/notify/mod.rs` (wiring), `src-tauri/src/notify/rules.rs` (pure state machine), `src-tauri/src/notify/tests.rs`
- Modify: `src-tauri/src/status/poll.rs` (hook after each poll), `src-tauri/src/lib.rs` (plugin init), `src-tauri/Cargo.toml` (+ `tauri-plugin-notification = "2"`, justification in commit body), `src-tauri/capabilities/*.json` (+ `notification:default`)
- Test: `src-tauri/src/notify/tests.rs`

**Interfaces:**
- Consumes: `ConfigHandle` (Task 1: `notify.*`), `DeviceStatus` (status/mod.rs as of main).
- Produces: in `notify/rules.rs` — `pub struct Measure { online: bool, toner: Option<u8>, drum: Option<u8> }` (glue maps `DeviceStatus`/poll failure to it), `pub struct NotifyConfig { enabled: bool, toner_percent: u8, drum_percent: u8 }` (mirrors Task 1 config); `notify::evaluate(prev: Option<&Measure>, next: &Measure, silent_streak: u32, cfg: &NotifyConfig) -> Vec<NotifyKind>` — pure, `NotifyKind { TonerLow(u8), DrumLow(u8), Silent, BackOnline }`; state kept in `notify::State` (re-arm on recovery, no hysteresis); `fn show(app: &AppHandle, kind: NotifyKind)` — `app.notification().builder().id(kind_id).title(..).body(..).show()`, stable id per kind so a re-notify replaces, permission-denied → `warn!` log, no panic; emits `alert {kind, message}` (PRD §9) to the UI. Poll hook fires only when `notify.enabled`.

- [ ] **Step 1: Failing tests (pure rules)** — `Measure` snapshots: threshold crossing fires once (poll1: toner 20→14, cfg 15 → `[TonerLow]`; poll2: unchanged → `[]`); re-arm on recovery (14→16 → `[]`, then 16→14 → `[TonerLow]` again); offline→online transition fires `BackOnline`; 3 consecutive failed polls (silent streak, `next = None`-measured) → one `Silent`; `enabled=false` → always `[]`; drum 8% with cfg 10 → `[DrumLow]`.
- [ ] **Step 2: Run** `cargo test --workspace` — expected: FAIL (module absent).
- [ ] **Step 3: Implement** rules (pure) + poller hook + plugin registration + `show()` per the Context7-verified API; notification titles/bodies composed through `labels`-style constants (Russian copy lives in `labels.ts` for UI, backend notification strings get their own `NOTIFY_*` consts — English codes in events, Russian in the notification text).
- [ ] **Step 4: Full Gate** — PASS.
- [ ] **Step 5: Commit** — `M4: edge-triggered notifications with backend poller hook` (crate justification in body)

### Task 3: SNMP fourth varbind + jam count + status fields (Wave 2 ∥)

**Files:**
- Modify: `src-tauri/src/snmp/client.rs` (4th varbind `1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.21.0`), `src-tauri/src/snmp/blob.rs` (if the `#a1` record needs a mapping entry), `src-tauri/src/status/mod.rs` (`DeviceStatus` fields)
- Test: existing snmp/status test modules (extend)

**Interfaces:**
- Consumes: existing batched `get_many` + blob record parser; `docs/oids-l2710dw.md` §`5.5.21` (`#a1` = jam count).
- Produces: `DeviceStatus` gains `jam_count: Option<u32>`; the batched GET carries 4 varbinds in one PDU; missing/absent blob → `None` (never an error).

- [ ] **Step 1: Failing tests** — fixture with a `…5.5.21` OctetString `#a1 <u32 BE>` record parses to `jam_count = Some(n)`; response without that node → `jam_count = None`; unknown `#` ids in that blob are skipped with a log (existing convention).
- [ ] **Step 2: Run** `cargo test --workspace` — expected: FAIL.
- [ ] **Step 3: Implement** — extend the varbind list and the mapping; arithmetic for «drum left» stays in the UI task (Task 5) per `docs/oids-l2710dw.md` semantics — backend reports raw verified numbers only.
- [ ] **Step 4: Full Gate** — PASS.
- [ ] **Step 5: Commit** — `M4: jam count via fourth varbind in the batched GET`

### Task 4: Tray states + menu (Wave 2 ∥)

**Files:**
- Modify: `src-tauri/src/tray.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/commands/mod.rs` (activity flag on `AppState`)
- Modify: `src-tauri/src/commands/scan_jobs.rs`, `src-tauri/src/print/mod.rs` (set/clear the activity flag around runs)
- Test: `src-tauri/src/tray.rs` unit parts (state → icon asset mapping, pure)

**Interfaces:**
- Consumes: existing tray setup (M1 menu), `status` event payload (offline/error already known there).
- Produces: `AppState.activity: Arc<AtomicU8>` (0 idle / 1 scanning / 2 printing); `tray::update(app, TrayInput { state, activity })` mapping to icon states idle (neutral) / working (accent, when `activity != 0`) / error (red) / offline (dimmed); menu «Показать окно», «Скан» (show+focus window, navigate to Scan screen via existing event), «Открыть панель» (opener → `http://<host>/general/status.html`), «Выход». Icons: 4 PNG assets generated from the existing icon source (script step or committed assets).

- [ ] **Step 1: Failing test** — mapping table: (offline, any) → OfflineIcon; (error, idle) → ErrorIcon; (ok, scanning) → WorkingIcon; (ok, idle) → IdleIcon.
- [ ] **Step 2: Run** — FAIL.
- [ ] **Step 3: Implement** — `update()` calls `TrayIcon::set_icon` (runtime, Context7-verified) and keeps the menu set once (Linux caveat); wire flag set/clear in scan job runner and print paths; menu handlers reuse the opener command's logic for the panel link.
- [ ] **Step 4: Full Gate** — PASS.
- [ ] **Step 5: Commit** — `M4: tray status icon and full menu`

### Task 5: History chart + R5/R6 on «Обзор» (Wave 3 ∥)

**Files:**
- Create: `src-tauri/src/commands/history.rs` (`get_history`), `ui/src/screens/overview_chart.ts` (SVG render + downsample)
- Modify: `src-tauri/src/history/mod.rs` (read-side `read_days(days: u32) -> Vec<Snapshot>` over the existing post-M2.5 table), `src-tauri/src/commands/mod.rs` (register), `ui/src/screens/overview.ts`, `ui/src/api.ts`, `ui/src/types.ts`, `ui/src/labels.ts`, `ui/src/styles/overview.css`
- Test: Rust read-side test on an in-memory DB; TS downsample test (`ui/src` tests run via the existing TS test setup — mirror how existing UI modules are tested; if none, pin the math in a Rust-side mirror or a vitest-style test per repo convention)

**Interfaces:**
- Consumes: M2.5 history schema (post-merge `history/mod.rs` — read the actual columns); Task 3 `DeviceStatus.jam_count`; Task 1 `Config.toner.yield_pages`.
- Produces: IPC `get_history(days: u32) -> [Snapshot]` (matches PRD §9); `overview_chart.ts: renderChart(el, points: ChartPoint[], max: number)` pure-ish SVG builder; downsample `toPoints(rows, cap = 90)`.

- [ ] **Step 1: Failing tests** — Rust: `read_days` returns rows newest-last for a seeded in-memory DB; TS: `toPoints` caps 200 rows → 90, keeps first/last; 0/1 rows → empty state flag.
- [ ] **Step 2: Run** — FAIL.
- [ ] **Step 3: Implement** — `get_history` command (default 90 days); «Обзор» renders an SVG polyline chart (toner-% line, page-counter line, axis-less, current-value dots) + empty state «данных пока нет — копим» at <2 rows; R5 lines from `DeviceStatus` (jam text only when `jam_count` present; drum text per `docs/oids-l2710dw.md` semantics); R6 estimate next to toner % only when `yield_pages` set, otherwise the hint «укажите ресурс картриджа в настройках» linking to «Настройки»; R7 firmware link row → `/admin/firmwareupdate.html`.
- [ ] **Step 4: Full Gate** — PASS.
- [ ] **Step 5: Commit** — `M4: history chart, human status texts and supplies estimate on overview`

### Task 6: «Настройки» screen + EWS deep links (Wave 3 ∥)

**Files:**
- Create: `ui/src/screens/settings.ts`, `ui/src/screens/settings_form.ts`, `ui/src/styles/settings.css`
- Modify: `ui/index.html` (nav destination), `ui/src/router.ts`, `ui/src/shell.ts`, `ui/src/labels.ts`, `ui/src/api.ts`, `ui/src/types.ts`
- Test: form validation mirror test per repo convention (bounds from Task 1 config)

**Interfaces:**
- Consumes: Task 1 `get_config`/`set_config` (exact DTO shape), Task 5 empty-estimate hint target.
- Produces: nav destination «Настройки»; settings form (host, pollSeconds, notify enabled + toner/drum thresholds, scan folder, cartridge model + yield, notification test button); R1 link group (verified URLs from `docs/ews-l2710dw.md`): status, maintenance info, sleep, auto power-off, replace toner, date&time, SNTP, firmware update; each opens `http://<host><path>` via the opener plugin.

- [ ] **Step 1: Failing test** — form→DTO mapping: valid input produces `Config` DTO within bounds; out-of-bounds percent is blocked before `set_config` and shows the inline error (assert on the mapping/validation function).
- [ ] **Step 2: Run** — FAIL.
- [ ] **Step 3: Implement** — load on open via `get_config`, save via `set_config` (success → status line «сохранено, применено»), inline field errors from the `Err` string; deep-link group as a link list with a one-line hint that authenticated pages open the panel's login first (verified behavior, `docs/ews-l2710dw.md`); notification test button calls a tiny `test_notification` command (backend `show()` with a fixed id) so the user can verify desktop notifications from settings.
- [ ] **Step 4: Full Gate** — PASS.
- [ ] **Step 5: Commit** — `M4: settings destination with live-applied config and EWS links`

### Task 7: Live acceptance + docs (user-gated, after Waves 1–3 merge)

**Files:**
- Modify: `PRD.md` (§12 M4 row → ✅ with date; §9 add `get_config`/`set_config`/`get_history`/`test_notification`; §14 live checklist additions), `README.md`, `docs/superpowers/plans/` checkboxes
- No device writes: SNMP reads, notifications, UI checks only.

**Interfaces:** consumes everything above, on merged `main`.

- [ ] **Step 1 (user session):** live checklist — settings change applies without restart (interval visibly changes poll cadence); notification fires at a temporarily high threshold (real toner ≈40 % → set 45 %, then restore 15 %); one notification per stuck state; tray icon reflects offline (unplug LAN cable / power off → dimmed, back → idle); tray menu «Скан»/«Открыть панель»/«Выход»; chart fills as history accrues (≥2 polls); jam/drum/estimate lines render from live values; EWS links open the right pages (public ones directly, authenticated ones via the panel login); notification test button works.
- [ ] **Step 2:** visual check pass (visual-check subagent), both themes, 640×480 and maximized.
- [ ] **Step 3:** docs updates + commit `M4: live acceptance and docs`.
