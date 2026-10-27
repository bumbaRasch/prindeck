# Print Queue (M3, queue-first) — Short Design

- Status: draft for approval
- Phase: post-M2. Execution order in the phase: **history recording ships
  standalone first** (its whole value is starting the data clock; every day of
  delay is an unrecoverable hole), print queue second (offline-codeable, no
  device), then the spike (user-gated) and the ADF work on top of its results.
  The ADF-before-print rationale from the brainstorm («heart of the product»)
  survives as landing priority, not as idle waiting. *(my call, builder-check
  q1/q5)*
- Sources: OpenPrinting CUPS docs via context7 (`doc/help/options.html`,
  `man-lp`, `man-lpstat`, `man-cancel`), checked 27.10.2026.

## Intent

On Linux every app can *send* a print job; nothing shows the queue next to the
device state. The MFC's own counters say jams happen (8 total, 6 in the ADF).
The Print screen's job is **jam recovery**: see what is stuck, cancel it in one
click — sending a file is the secondary convenience.

## Goals

- Active queue: list of not-completed jobs (id, name, owner, size, state) +
  per-job «Отменить».
- Send a file (drag-drop or file picker) with the standard option pair:
  copies (`lp -n N`) and duplex (`-o sides=one-sided | two-sided-long-edge |
  two-sided-short-edge` — exactly the three values from the CUPS docs).
- Job id from `lp` output is the handle surfaced in the UI.

## Non-goals

- PPD quality/draft options — cut from v1, driver-specific, return on a real
  need *(my call, delegated)*.
- Pause/resume/hold, reordering, moving jobs between queues.
- Background queue polling — the queue refreshes every 5 s **only while the
  Print screen is visible** (PRD §5.4).

## Hard rules from the CUPS docs (implementation landmines)

1. `cancel` **without** an argument cancels the job currently printing on the
   default destination — always pass an explicit job id.
2. `lpstat` output embeds locale-formatted dates (e.g. `mié 09 mar 2016` on
   es_ES) — spawn `lp`/`lpstat`/`cancel` with `LC_ALL=C` or parsing breaks on
   any non-English system (this one included).
3. `lpstat -W not-completed -o` — the `-W` must precede `-o` (man page).

## How

- `print.rs` (new domain module): spawn the three commands with `LC_ALL=C`,
  5 s timeout, parse minimal stable fields; queue name from config
  (`cups_queue`, default `Brother_MFC_L2710DW_series` — PRD §3).
- **Queue scope**: `lpstat -W not-completed -o <cups_queue>` — the list and the
  cancel actions cover only the configured queue; jobs of other queues never
  appear in the jam-recovery list. `CupsJob` fields are `id`, `owner`, `size`,
  `state` — plain `lpstat -o` output carries no document name; a real output
  capture on this machine (cheap, does not occupy the device) becomes the
  parsing fixture. *(my call, builder-check q3)*
- **Queue validation is soft**: a missing/absent CUPS queue is not a startup
  `ConfigError` (CUPS may be down transiently); the first `print_file`/
  `get_queue` call surfaces the error in the UI («очередь CUPS не найдена»)
  and logs a `warn`. *(my call, builder-check q3)*
- IPC: `print_file(path, copies, sides) -> jobId`, `get_queue() -> [CupsJob]`,
  `cancel_print(jobId)` (PRD §9 rows, unchanged).
- `tauri-plugin-dialog` added in this phase for the file picker (deferred from
  M1 by the M1 plan; drag-drop paths come from the core event).
- UI: dropzone + copies stepper + duplex select + «Печать» primary; below —
  the jobs table with cancel buttons; empty queue is an explicit «Очередь
  пуста», not a blank area.

## Acceptance

- Print a PDF from the UI with `two-sided-long-edge` — output flips like a
  book (user checks the paper).
- Queue a 3-copy job, cancel it from the UI, `lpstat` confirms removal.
- `LC_ALL=ru_RU.UTF-8` environment (this machine's reality) — parsing still
  correct.
