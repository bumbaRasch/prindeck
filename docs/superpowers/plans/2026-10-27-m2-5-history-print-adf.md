# Post-M2 Phase: History, Print Queue, ADF Multi-Sheet — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Record status history silently, add the queue-first Print screen, and complete ADF multi-sheet scanning (per-page fetch + client-side PDF merge) with partial-keep on cancel.

**Architecture:** Three independent deliverables sequenced by value decay and device availability: `history.rs` hooks one insert into the existing M2 poller; `print.rs` wraps `lp`/`lpstat`/`cancel` with `LC_ALL=C` behind three IPC calls and a new screen; the ADF work reworks the scan fetch loop page-by-page, merges per-page PDFs with `lopdf`, and extends the scan IPC/UI contract. The live spike (Task 4) is the only device-dependent task and gates Tasks 5–7.

**Tech Stack:** Tauri 2, Rust 2024, `rusqlite` (bundled), `lopdf` (added only if the spike confirms per-page delivery), vanilla strict TypeScript, Vite.

**Spec:** `docs/superpowers/specs/2026-10-27-history-recording-design.md`, `docs/superpowers/specs/2026-10-27-print-queue-design.md`, `docs/superpowers/specs/2026-10-27-adf-multisheet-design.md`

## Global Constraints

- Gate, all green, no warnings: `cargo test --workspace`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `npm run typecheck`, `npm run build`, `npm run check:code-language`, `npm run check:line-count`.
- Source files under 200 lines, split by domain, never `utils`/`common`/`helpers`; no comments in code.
- Code strings (errors, logs, tests) English; Russian UI copy only in `ui/index.html`, `ui/src/labels.ts` and `*_LABEL` constants.
- New crates: `lopdf` (default-features = false, justification in the commit message) only after the spike confirms per-page delivery; `rusqlite` (bundled) and `tauri-plugin-dialog` are already approved in PRD §10 / the M1 plan.
- Device: never start a scan or occupy the printer without the user's explicit confirmation in the session (AGENTS.md). SNMP GETs and read-only CUPS queries (`lpstat`) are safe. Anything that creates a print job must either be user-gated or held (`-o job-hold-until=indefinite`). Tasks 3, 4 and 8 contain device actions and carry explicit gates.
- CUPS subprocesses always run with `LC_ALL=C` and a 5 s timeout; `cancel` always receives an explicit job id.
- The branch is `m1-scan`; commit prefix for this phase is `M2.5:`.
- Window/UI rules per AGENTS.md: responsive at 640×480 and up, both themes, one primary action per screen.

## Review Focus

- **Merged PDF unreadable in real viewers despite passing unit tests.** Task 6 pins page count and structure via lopdf on device-captured fixtures; Task 8 opens the merged output in GNOME Papers/Firefox and checks `pdfinfo` pages = sheets.
- **Locale-formatted `lpstat` output breaking the parser on this ru_RU machine.** Task 2 captures real output into a fixture and parses only under `LC_ALL=C`.
- **Temp files or a wedged device after cancel/error mid-stack.** Task 5 unit-tests the terminal paths: every exit removes temp files and issues the best-effort `DELETE`.
- **A history write failure killing the poll loop.** Task 1 proves `record` errors are swallowed with a `warn` and polling continues.
- **A scan that errored mid-stack showing a hard failure over an existing file.** Task 5 pins the terminal-event choice: pages>0 + error → `scan-done {partial: true, warning}`, never `scan-error`.

## File Structure

Rust (`src-tauri/src/`):

| File | Responsibility |
|---|---|
| `history.rs` | open/create the DB, `record(&Snapshot)`, schema `user_version` |
| `status/poll.rs` | one `history::record` call after a successful poll |
| `print/mod.rs` | `PrintOptions`, `Sides`, `CupsJob`, `JobState`, spawn helpers |
| `print/lp.rs` | `print_file`, `queue_jobs`, `cancel` — `LC_ALL=C`, parsing |
| `print/tests.rs` | parsing tests on the captured fixture |
| `commands/print.rs` | `print_file`, `get_queue`, `cancel_print` IPC bodies |
| `config/mod.rs` | `cups_queue` key (serde default `Brother_MFC_L2710DW_series`) |
| `commands/scan_jobs.rs` | job orchestration (existing), calls into `scan_pages` |
| `commands/scan_pages.rs` | per-page fetch loop, `PagesOutcome`, `next_delay` pause rule |
| `pdf_merge.rs` | `merge_pdfs(&[Vec<u8>]) -> Result<Vec<u8>, MergeError>` |
| `commands/mod.rs` | payload structs: `ScanDone {files, partial, warning}`, progress `pages` (payloads live here today) |

UI (`ui/src/`):

| File | Responsibility |
|---|---|
| `screens/print.ts` | Print screen: dropzone/picker, copies+duplex, queue table, cancel |
| `screens/scan.ts` | scan form and actions (existing) |
| `screens/scan_result.ts` | page counter, text page list, partial/warning result block |
| `shell.ts`, `router.ts`, `labels.ts`, `index.html` | third nav entry, sprite icon, Russian labels |
| `api.ts`, `types.ts` | print IPC wrappers, updated scan payloads |

Fixtures/docs: `fixtures/print/lpstat-not-completed.txt` (Task 2), `fixtures/escl/adf/` (Task 4), `docs/escl-l2710dw.md` §11 (Task 4), PRD/README (Task 8).

## Parallelism

Wave model (updated 27.10.2026 for speed, at the user's request): parallel tasks run in **separate git worktrees**, each on its own branch off `m2-5-phase`; the controller merges them back sequentially and re-runs the full gate after each merge.

| Wave | Tasks | Worktree | Why safe to parallelize |
|---|---|---|---|
| 1 | 4 (spike, user-gated) ∥ 5 (fetch loop + IPC) ∥ 6a (pure `pdf_merge` module) | device / `m2-5-t5` / `m2-5-t6a` | Disjoint files: T5 touches `commands/` + UI mirror, T6a touches only new files + `Cargo.toml`/`lib.rs` mod line, T4 touches no product code |
| 2 | 6b (wire merge into the pipeline, tiny) ∥ 7 (scan UI) | `m2-5-phase` after merges | 6b consumes `PagesOutcome` (T5) and `merge_pdfs` (T6a); T7 consumes T5's payloads |
| 3 | 8 (docs + live acceptance, user-gated live parts) | `m2-5-phase` | needs everything landed |

Task 5 does NOT block on Task 4's pause-rule verdict: implement the pause per the design (`next_delay` = min(1 s, prev/2), the sane-airscan Brother rule) behind that single function — if §11 later proves the pause unnecessary on this unit, removing it is a one-function change, not a redesign. Task 6a does NOT block on Task 4's fixtures: tests run on the device-produced platen PDF from M1 (`fixtures/escl/platen-300-gray.pdf`, copied from `~/Documents/scans/scan-20261027-095553.pdf`); real ADF per-page fixtures from Task 4 upgrade those tests in Task 6b.

---

### Task 1: Silent history recording

**Files:**
- Create: `src-tauri/src/history.rs`, `src-tauri/src/history/tests.rs`
- Modify: `src-tauri/src/status/poll.rs` (one call), `src-tauri/src/lib.rs` (`pub mod history;`, wire the DB handle), `src-tauri/Cargo.toml` (`rusqlite = { version = "0.37", features = ["bundled"] }` — verify the current version on crates.io at implementation time)

**Interfaces:**
- Produces:

```rust
pub struct Snapshot {
    pub ts: String,               // rfc3339, taken from DeviceStatus::polled_at
    pub toner_percent: Option<u32>,
    pub drum_percent: Option<u32>,
    pub pages_total: Option<u32>,
    pub duplex_pages: Option<u32>,
    pub drum_pages_left: Option<u32>,
}
impl Snapshot { pub fn from_status(status: &DeviceStatus) -> Snapshot }
pub struct History { /* rusqlite::Connection */ }
impl History {
    pub fn open(path: &std::path::Path) -> Result<History, HistoryError>;
    pub fn record(&self, snap: &Snapshot) -> Result<(), HistoryError>;
}
pub struct HistoryHandle(pub std::sync::Mutex<Option<History>>);   // Connection is !Sync
pub fn try_record(handle: &HistoryHandle, snap: &Snapshot);       // swallows Err with warn!
```

- Consumes: `DeviceStatus` from `status/mod.rs` (fields already exist from M2).

- [ ] **Step 1: Write the failing tests**

In `history/tests.rs`: `round_trip` — `History::open` on a temp path under `std::env::temp_dir()`, `record` a `Snapshot` with all fields, re-open, assert the row reads back field-for-field; `gaps_stay_gaps` — two records with distant timestamps, row count is 2; `try_record_swallows_errors` — a `HistoryHandle(Mutex::new(None))` (open failed) plus a `try_record` call returns `()` without panicking.

- [ ] **Step 2: Run to see them fail**

Run: `cargo test --workspace history` — expected: compile error, module missing.

- [ ] **Step 3: Implement `history.rs` and the poll hook**

Schema v1: `snapshots(ts TEXT PRIMARY KEY, toner_pct INTEGER NULL, drum_pct INTEGER NULL, pages_total INTEGER NULL, duplex_pages INTEGER NULL, drum_pages_left INTEGER NULL)`, `PRAGMA user_version = 1` guard. `History::open` is called once in `lib.rs` setup with `dirs::data_dir()/prindeck/history.db` (created via `create_dir_all`); the handle is stored in Tauri managed state as `HistoryHandle(Mutex<Option<History>>)` — `None` on open failure after an `error!` log, polling runs unchanged. In `status/poll.rs`, after a successful poll: `try_record(&handle, &Snapshot::from_status(&status))` — an insert failure logs `warn!` and the poll result is returned unchanged.

- [ ] **Step 4: Run the tests and the gate**

Run: `cargo test --workspace` — all green including the new ones; `cargo clippy --workspace --all-targets -- -D warnings` — clean.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/history.rs src-tauri/src/history src-tauri/src/status/poll.rs src-tauri/src/lib.rs src-tauri/Cargo.toml Cargo.lock
git commit -m "M2.5: silent history recording on every successful poll"
```

---

### Task 2: `print/` domain module and CUPS fixtures

**Files:**
- Create: `src-tauri/src/print/mod.rs`, `src-tauri/src/print/lp.rs`, `src-tauri/src/print/tests.rs`, `fixtures/print/lpstat-not-completed.txt`
- Modify: `src-tauri/src/config/mod.rs` (`cups_queue`), `src-tauri/src/lib.rs` (`pub mod print;`)

**Interfaces:**
- Produces:

```rust
pub enum Sides { OneSided, LongEdge, ShortEdge }
impl Sides { pub fn as_lp_option(&self) -> &'static str }   // sides=...
pub struct PrintOptions { pub copies: u32, pub sides: Sides }
pub struct CupsJob { pub id: String, pub owner: String, pub size_kb: u64 }
pub fn print_file(queue: &str, path: &Path, opts: &PrintOptions) -> Result<String, PrintError>;  // "queue-N"
pub fn queue_jobs(queue: &str) -> Result<Vec<CupsJob>, PrintError>;
pub fn cancel(job_id: &str) -> Result<(), PrintError>;
```

`CupsJob` carries `Serialize` with `rename_all = "camelCase"` (IPC convention). Verified live 27.10.2026: `LC_ALL=C lpstat -W not-completed -o <queue>` prints exactly `id owner size date` — **no state token** (not even for a held job); all not-completed jobs render as «в ожидании», there is no state field to parse.

- Consumes: nothing from other tasks; `std::process::Command` with env `LC_ALL=C` in `spawn_blocking` with a `try_wait` loop for the 5 s timeout — the same pattern as `snmp/client.rs`.

- [ ] **Step 1: Capture the real fixture**

Run `LC_ALL=C lpstat -W not-completed -o Brother_MFC_L2710DW_series` on this machine. If the queue is empty, create a **held** job first — `lp -o job-hold-until=indefinite -d Brother_MFC_L2710DW_series /etc/hostname` (verified: identical `lpstat` row, zero device contact) — then `cancel` it after capture. Save the exact bytes to `fixtures/print/lpstat-not-completed.txt`.

- [ ] **Step 2: Write the failing parsing tests**

`parses_fixture_rows` (id `Brother_MFC_L2710DW_series-N`, owner, size from the captured file); `non_matching_queue_is_dropped` (a line of a foreign queue never yields a `CupsJob`); `sides_option_strings` (`one-sided`, `two-sided-long-edge`, `two-sided-short-edge` — verbatim from the CUPS docs); `empty_output_yields_empty_vec`; `garbage_line_is_skipped_not_fatal`.

- [ ] **Step 3: Run to see them fail**

Run: `cargo test --workspace print` — expected: compile error.

- [ ] **Step 4: Implement `print/mod.rs` + `print/lp.rs`**

`queue_jobs` runs `lpstat -W not-completed -o <queue>` (`-W` before `-o`), splits lines, keeps only rows whose id starts with `<queue>-`; the fields after the id token are `owner` and `size` (the date tail is ignored). `print_file` runs `lp -d <queue> -n <copies> -o sides=<..> <path>` and returns the job id token from the output line (`request id is <queue>-N ...`). `cancel` runs `cancel <job_id>` — never bare. All three set `LC_ALL=C`.

- [ ] **Step 5: Add `cups_queue` to the config**

`Config` gains `cups_queue: String` with `#[serde(default = "default_cups_queue")]` returning `Brother_MFC_L2710DW_series`; test: JSON without the key parses to the default; empty string → `ConfigError::InvalidCupsQueue` alongside the existing host validation.

- [ ] **Step 6: Run tests and gate; commit**

Run: `cargo test --workspace` and clippy — green/clean; `wc -l src-tauri/src/print/*.rs` under 200.

```bash
git add src-tauri/src/print fixtures/print src-tauri/src/config src-tauri/src/lib.rs src-tauri/Cargo.toml Cargo.lock
git commit -m "M2.5: CUPS print domain — lp/lpstat/cancel under LC_ALL=C"
```

---

### Task 3: Print IPC commands and the Print screen

**Files:**
- Create: `src-tauri/src/commands/print.rs`, `ui/src/screens/print.ts`
- Modify: `src-tauri/src/commands/mod.rs` (declare + re-export + `__cmd__*` lines), `src-tauri/src/lib.rs` (register), `ui/index.html`, `ui/src/{shell.ts,router.ts,labels.ts,api.ts,types.ts,styles.css}` or domain style split, `package.json` + `src-tauri/Cargo.toml` + capabilities (`tauri-plugin-dialog`), `ui/src/main.ts`

**Interfaces:**
- Produces: IPC `print_file(path: String, copies: u32, sides: String) -> Result<String, String>`, `get_queue() -> Result<Vec<CupsJob>, String>`, `cancel_print(jobId: String) -> Result<(), String>`; UI `initPrintScreen(getState)`, `renderPrintScreen(state)`, `Screen` gains `"print"`; `api.printFile/getQueue/cancelPrint` (no queue event listener — the 5 s interval lives inside `screens/print.ts`).
- Consumes: `print::{print_file, queue_jobs, cancel}` (Task 2), shell/router patterns (M2), `config.cups_queue`.

- [ ] **Step 1: Rust IPC bodies**

`commands/print.rs` maps `Sides` from the string — the IPC vocabulary is the CUPS option strings themselves, `"one-sided" | "two-sided-long-edge" | "two-sided-short-edge"` (no second enum vocabulary to translate); unknown → `Err`. Validates `copies` 1–99, maps `PrintError` to English log + Russian `*_LABEL` message (incl. «очередь CUPS не найдена» for the soft queue-missing case). Unit tests: sides mapping, copies bounds.

- [ ] **Step 2: UI — nav, screen, labels**

Third nav entry «Печать» (icon: Lucide `printer` in the sprite); screen: dropzone + «Выбрать файл…» (dialog plugin), copies stepper, duplex select («односторонняя / по длинному краю / по короткому краю»), primary «Печать», queue table (id, владелец, размер, «Отменить»; every row renders «в ожидании» — there is no state token in `lpstat -o` output), explicit «Очередь пуста» state.

- [ ] **Step 3: Queue refresh only while visible**

`setInterval` 5 s started when the screen becomes current, cleared on leave (router hook); manual refresh after `printFile`/`cancelPrint` resolves. Drag-drop file paths from the core `onDragDropEvent` (paths, not web `File`).

**GATE — device action:** anything that sends paper needs the user's explicit go in this session.

- [ ] **Step 4: Verify**

Gate + `npm run check:line-count`; app at 900×680 and 640×480, both themes — the M2 visual-check subagent pattern. **Then, with the user's go** (occupies the printer): send one real print with `two-sided-long-edge` (user checks the paper flips like a book), then queue a 3-copy job and cancel it from the UI, `lpstat` confirms removal.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands ui src-tauri/Cargo.toml src-tauri/capabilities package.json package-lock.json
git commit -m "M2.5: queue-first Print screen with copies and duplex"
```

---

### Task 4: LIVE SPIKE — ADF behavior, power cycle, wake timing

**Files:**
- Create: `fixtures/escl/adf/` (captured pages), a section in `docs/escl-l2710dw.md`
- Test: manual checklist below; no product code in this task

**GATE — read first:** every scan in this task occupies the physical device. Before starting, tell the user the checklist and get an explicit go in this session. The paper is already in the feeder. Run the scenarios with `curl`/`scripts/escl-probe.mjs`-style throwaway scripts in `/tmp/opencode/`, never through product code.

**Interfaces:**
- Produces for Tasks 5–7: `fixtures/escl/adf/page-N.pdf|.jpg` (per-page documents from a 2-sheet stack), and a verdict block in `docs/escl-l2710dw.md` §11: outcome **A** (per-page) or **B** (one multi-page PDF), the pause-rule verdict (is the ≥1 s gap required on this unit), cancel-mid-stack behavior, power-cycle and wake timings.

- [ ] **Step 1: 2 sheets, PDF — the A/B question**

`POST` + repeated `NextDocument` (with a 1 s pause): one response per sheet or one multi-page PDF? Save each response body to `fixtures/escl/adf/`; `pdfinfo` each file (pages, size). Record per-page timings. Scrub device identifiers (serial/UUID) from anything committed — the M0 publication-hygiene rule (`docs/escl-l2710dw.md`, end of §REPORT).

- [ ] **Step 2: 2 sheets, JPEG** — confirm per-page JPEGs and `Content-Location: doc-N`; save fixtures.

- [ ] **Step 2b: 1 sheet, PDF** — the single-sheet edge: does the loop still need the terminating `NextDocument` → `404`?

- [ ] **Step 3: No pause** — repeat Step 1 back-to-back without the gap; does `404`/`ScannerAdfEmpty` appear? Verdict for the pause rule.

- [ ] **Step 4: Cancel on sheet 2 of 3** — `DELETE` mid-fetch: `200`, hanging fetch `410`, `ScannerStatus` → `Idle`; what does the feeder do with sheet 3; is a partial body possible.

- [ ] **Step 5: Power-cycle probe** — with the app open and polling: power off → chip «Нет связи» within one poll; power on → recovers to «в сети» unattended; record boot-to-first-response latency and first-request latency after deep idle.

- [ ] **Step 6: Write the verdict into `docs/escl-l2710dw.md` §11** — A/B, pause rule, cancel, timings; fixtures listed.

- [ ] **Step 7: Commit**

```bash
git add fixtures/escl/adf docs/escl-l2710dw.md
git commit -m "M2.5: ADF spike — per-page delivery verdict, cancel, power-cycle"
```

---

### Task 5: Per-page fetch loop and the extended scan IPC

**Files:**
- Create: `src-tauri/src/commands/scan_pages.rs`
- Modify: `src-tauri/src/commands/scan_jobs.rs` (orchestration calls the loop), `src-tauri/src/commands/mod.rs` (payload structs), `src-tauri/src/commands/scan_files.rs` (`-pN`/`-partial` naming), tests under `src-tauri/src/commands/tests/`, `ui/src/types.ts` + `ui/src/main.ts` (TS mirror of the new payload shape — done here, not in Task 7, so every green gate reflects a working app; the rich UI comes in Task 7)

**Interfaces:**
- Produces:

```rust
pub enum PagesOutcome {
    Complete { pages: Vec<Vec<u8>> },
    Canceled { pages: Vec<Vec<u8>> },
    Failed { pages: Vec<Vec<u8>>, error: EsclError },
}
fn next_delay(prev_request: Duration) -> Duration;   // min(1s, prev/2) per sane-airscan
pub struct ScanDone { pub job_id: String, pub files: Vec<String>, pub partial: bool, pub warning: Option<String> }
```

Terminal-event table (complete, both outcomes — `pages` is received PDF/JPEG pages):

| Outcome | Terminal event |
|---|---|
| `Complete` | `scan-done {partial: false, warning: None}` |
| `Canceled`, ≥ 1 page | `scan-done {partial: true, warning: None}` |
| `Canceled`, 0 pages | terminal `scan-progress {phase: "canceled"}` (existing M1 UX «отменено»); **no** `scan-error` — a user cancel is not an error |
| `Failed`, ≥ 1 page | `scan-done {partial: true, warning: Some(msg)}` |
| `Failed`, 0 pages | `scan-error {message}` |

`scan-progress` gains `pages: u32`, emitted on every received page. The loop is outcome-agnostic: under outcome B the second `NextDocument` → `404` ends it with one page.

- Consumes: `EsclClient::next_document` (200/404/410 semantics, M1). The pause rule is NOT gated on Task 4: implement `next_delay` per the design (min(1 s, prev/2), the sane-airscan Brother quirk) — if the spike's §11 later proves the pause unnecessary on this unit, removing it is a one-function change.

- [ ] **Step 1: Failing tests** — `next_delay_clamps` (8 s prev → 1 s; 1.2 s prev → 600 ms); `terminal_event_failed_with_pages_is_done_partial`; `terminal_event_failed_empty_is_error`; `terminal_event_canceled_with_pages_is_done_partial`; `terminal_event_canceled_empty_is_progress_canceled_only`; `canceled_keeps_pages`; naming tests: JPEG stack → `-p1..-pN`, canceled PDF → `-partial.pdf`, collision suffix `-2`.

- [ ] **Step 2: Run to fail** — `cargo test --workspace commands` compile error.

- [ ] **Step 3: Implement** — the loop collects pages until `404`/`410`/error, applies `next_delay` between requests, on `Canceled`/`Failed` issues the best-effort `DELETE` on a separate connection and removes temp files for unreceived pages; files are written per the naming tests; `pages` counter emitted on every received page.

- [ ] **Step 4: Tests + gate green; commit**

```bash
git add src-tauri/src/commands
git commit -m "M2.5: per-page ADF fetch loop with partial-keep terminals"
```

---

### Task 6: `pdf_merge` module (outcome A)

Split for parallelism: **6a** (pure module + tests, runs in wave 1 in its own worktree, does not touch the scan pipeline) and **6b** (wiring, wave 2, after Task 5 merges).

**Files (6a):**
- Create: `src-tauri/src/pdf_merge.rs` + tests; Modify: `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs` (`pub mod pdf_merge;`)

**Files (6b):**
- Modify: `src-tauri/src/commands/scan_jobs.rs` (call site), `fixtures/escl/adf/` real fixtures from Task 4 upgrading the 6a tests

**Interfaces:**
- Produces: `pub fn merge_pdfs(pages: &[Vec<u8>]) -> Result<Vec<u8>, MergeError>`
- Consumes (6b): `PagesOutcome::Complete/Canceled{pages}` with >1 PDF page (invoke only then; a single page saves as-is — outcome B needs no merge).

- [ ] **Step 0 (6b): Branch on the spike verdict** — §11 says B and `fixtures/escl/adf` holds a single multi-page PDF? Then do not wire the merge at all, record the decision, and 6a's module stays dormant behind its tests. Otherwise wire it.

- [ ] **Step 1: Failing tests (6a) on the device PDF** — copy `~/Documents/scans/scan-20261027-095553.pdf` to `fixtures/escl/platen-300-gray.pdf` (a real firmware-produced single-page PDF); `merge_pdfs(&[page, page, page])` → lopdf `get_pages().len() == 3`; `merge_single_page` → pages == 1; `merge_rejects_garbage` → `Err(MergeError::Parse(_))`; merged bytes start with `%PDF-`. 6b upgrades these to the real ADF per-page fixtures.

- [ ] **Step 2: Run to fail** — compile error.

- [ ] **Step 3: Implement** — lopdf README merge pattern (renumber objects per document, rebuild one `Pages` root with `Kids`/`Count`, drop `Outlines`, `compress()`); `default-features = false`; write atomically (temp + rename) in `scan_jobs.rs` before `scan-done`.

- [ ] **Step 4: Tests + gate; `wc -l` under 200; commit**

```bash
git add src-tauri/src/pdf_merge.rs src-tauri/Cargo.toml Cargo.lock src-tauri/src/lib.rs src-tauri/src/commands/scan_jobs.rs
git commit -m "M2.5: lopdf client-side merge of per-page ADF PDFs"
```

---

### Task 7: Scan screen — counter, page list, partial results, previews

**Files:**
- Create: `ui/src/screens/scan_result.ts` (the new DOM block — `screens/scan.ts` is already at 185 lines)
- Modify: `ui/src/screens/scan.ts`, `ui/src/{api.ts,labels.ts,state.ts}`, `ui/index.html`, `src-tauri/tauri.conf.json` + capabilities (asset protocol, scope: `scanDir`)

**Interfaces:**
- Consumes: `scan-progress {pages}`, `scan-done {files, partial, warning}` (Task 5; the TS types already mirror it since Task 5), `convertFileSrc` from `@tauri-apps/api/core` (needs only the file path — no `caps.rs` change).

- [ ] **Step 1: Types + labels** — `ScanDone`/progress types updated; Russian copy: «Получено страниц: N», «неполный: N страниц», warning line, «Открыть папку» (exists).

- [ ] **Step 2: Mid-job UI** — counter + elapsed (10 s tick, M2 pattern), text page list (имя, размер) appended per `scan-progress`; JPEG preview only in the result block via `convertFileSrc` (asset protocol scope `scanDir`); PDF pages listed without preview.

- [ ] **Step 3: Result block** — files list (previews for `.jpg`), partial badge + warning text, no fake percentage anywhere.

- [ ] **Step 4: Verify** — gate + `check:line-count`; offline states exercised by hand (progress events simulate via a `listen` smoke check in dev); visual check at both sizes/themes via the `visual-check` subagent.

- [ ] **Step 5: Commit**

```bash
git add ui src-tauri/tauri.conf.json src-tauri/capabilities src-tauri/src/commands/caps.rs
git commit -m "M2.5: scan screen page counter, partial results and JPEG previews"
```

---

### Task 8: Documentation and live verification

**Files:**
- Modify: `PRD.md` (§7.1 ADF facts from §11, §9 payloads, §12 roadmap row «M2.5», §13.3/13.5 closed or re-scoped), `README.md`, this plan (tick boxes)

- [ ] **Step 1: PRD/README updates** — roadmap row, event table, the lopdf exception in §10, ADF facts, README «ADF multi-sheet» section.

- [ ] **Step 2: GATE — live acceptance (device).** With the user's explicit go: 3 sheets PDF → one file, `pdfinfo` reports `Pages: 3` and page size ≈ 602.64 × 830.16 pt (M0 reference), opens in GNOME Papers and Firefox; 3 sheets JPEG → 3 `-pN.jpg` with previews; cancel on sheet 2 → `…-partial.pdf`, device `Idle`, no temp files.

- [ ] **Step 2b: History live check** — the DB exists at `~/.local/share/prindeck/history.db`, rows accumulate over one poll interval, and `PRAGMA integrity_check` is clean after killing and restarting the app.

- [ ] **Step 3: Whole gate + line-count + `check:code-language` green; visual check both themes/sizes.**

- [ ] **Step 4: Commit**

```bash
git add PRD.md README.md docs/superpowers/plans/2026-10-27-m2-5-history-print-adf.md
git commit -m "M2.5: docs and live ADF acceptance"
```
