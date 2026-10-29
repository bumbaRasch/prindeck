# ADF Multi-Sheet Scanning and Client-Side PDF Merge — Design

- Status: draft for approval (brainstorming session 27.10.2026, 5 rounds)
- Phase: post-M2 (runs after the `m1-scan` branch lands M2)
- Depends on: live spike (checklist in the appendix) — the device is occupied
  for the duration; run only with the user's explicit go-ahead and after the
  M2 agent finishes its live checks.

## Intent

PRD F2 promises «all sheets from the feeder → one PDF» with one button. M0/M1
verified the platen only; there was no paper in the ADF. The reference client
sane-airscan (`airscan-escl.c`, quirk `ESCL_NEXT_LOAD_DELAY`) says Brother
devices serve the ADF **page by page**: each `NextDocument` returns one page,
the next request must wait ≥ 1 s (capped at half the previous request's
duration), otherwise the device answers `404` + `ScannerAdfEmpty`.

So the promise «one PDF» is almost certainly **client-side merge**, and this
document designs that. If the spike shows the firmware actually returns a
multi-page PDF in one `NextDocument`, the merge layer is dropped entirely and
no new crate is added (outcome B below) — the design covers both branches.

## Goals

1. N sheets in the feeder, one button → `scan-YYYYMMDD-HHMMSS.pdf` with N pages
   (PDF format).
2. JPEG format → N files `scan-YYYYMMDD-HHMMSS-p1.jpg … -pN.jpg`. Industry
   convention: Brother's own FAQ (multi-page only for PDF/TIFF, JPEG = one file
   per page), Epson, Ricoh, Xerox, GNOME Document Scanner.
3. Cancel mid-stack keeps what was already scanned: merged
   `scan-…-partial.pdf`, UI says «неполный: N страниц». Sheets already fed
   through the machine must never be thrown away. *(user decision, round 2)*
   Under spike **outcome B** (firmware serves one multi-page PDF per job) this
   degrades unavoidably: a canceled PDF job is all-or-nothing, because pages
   the client never received cannot be kept; the UI must say that in plain
   words instead of failing silently. JPEG stays per-page in both outcomes.
4. Progress is a counter «получено страниц: N» + elapsed time, no percentage:
   the device reports no total (`DetectPaperLoaded` is binary only). *(user
   decision, round 2)*
5. Page-by-page fetch loop respects the Brother delay rule; verified live in
   the spike.

## Non-goals

- Duplex ADF — the device is simplex-only (`AdfDuplexInputCaps` absent, M0).
- JPEG→PDF conversion — the format choice is explicit; JPEG stays per-page.
- Page reordering/rotation/thumbnail editing in the UI.
- Live per-page image channel during the job: mid-job the UI shows the counter
  and a text page list (name, size); inline `<img>` previews exist only in the
  completed result block for JPEG pages, served through the asset protocol
  scoped to `scanDir` (F2 already promises a result preview, so the protocol
  scope is needed regardless). PDF pages are never previewed. *(my call,
  builder-check q2)*

## Anti-goals

- A merged PDF that fails to open in the user's viewers (GNOME Papers — the
  system default — and Firefox) or reports a wrong page count in `pdfinfo`.
- Losing scanned pages on cancel, timeout or network error.
- Occupying the device longer than the feed loop needs (no extra status polling
  during the loop; phases stay client-side per the M1 spike).

## Constraints

- One new crate at most: `lopdf` (crates.io, checked 27.10.2026: 0.46.0
  released 21.10.2026, MIT, 26.6M downloads). `default-features = false`
  (drops `rayon`, `chrono-clock`); justification goes in the commit message,
  per PRD §10 crate discipline. Added **only under spike outcome A**.
- Modules under 200 lines, named after the domain (`pdf_merge.rs`, scan loop in
  the existing scan domain); no comments in code (AGENTS.md).
- IPC changes are additive: `scan-progress` gains `pages`, `scan-done` becomes
  `{jobId, files: string[], partial: bool, warning: string | null}` (the M1
  contract said a single `file`; PRD §9 updated at implementation time).
  Terminal semantics, fixed by this design: a job that errored mid-stack **with
  pages received** ends with `scan-done {partial: true, warning: <error>}` —
  the UI shows the file and the error together; `scan-error` fires only for
  failures that produced no pages. *(my call, builder-check q4)*
- The spike captures real per-page documents into `fixtures/escl/adf/` —
  offline merge tests run on those, byte-real, not synthetic.

## Approaches

| # | Approach | Verdict |
|---|---|---|
| 1 | **Client-side merge with `lopdf`** — per-page loop, glue at the end | **Chosen (outcome A).** Pure Rust, no runtime deps; merge of trivial firmware PDFs (single page, no metadata, M0 facts) is the documented README use case |
| 2 | System `pdfunite` (poppler) | Rejected: external runtime dependency for the published deb/AppImage (M5), version drift across distros |
| 3 | Rely on a firmware multi-page PDF | Rejected as primary — contradicts sane-airscan evidence; kept as **outcome B**: if true, no merge layer, no new crate |
| 4 | Always ship N separate PDFs | Rejected: breaks the F2 promise, worst UX |

## Design (outcome A — per-page delivery)

**Fetch loop** (in the existing scan domain, `commands/scan_jobs.rs`):

- After `POST` (unchanged), loop `GET {job}/NextDocument`:
  `200` → one page document; `404` → normal end of stack; `410` → canceled
  (our DELETE, per the M1 cancel spike); timeout/transport error → treated like
  cancel: keep pages received so far, best-effort `DELETE`, report the error.
- Pause between requests: `min(1 s, previous_request_duration / 2)` per the
  sane-airscan Brother rule; the spike verifies whether the pause is required
  at all on this unit.
- `NextDocument` per-request timeout stays 120 s (M1 decision).
- The 61 s device-side job timeout is not a factor: the loop requests the next
  page immediately after the pause.

**Merge** (`pdf_merge.rs`, pure function, fully unit-tested offline):

- `merge_pages(pages: &[Vec<u8>]) -> Result<Vec<u8>, MergeError>` following the
  lopdf README pattern: renumber objects per document, collect `Page` objects,
  rebuild one `Pages` root (`Kids`, `Count`), drop `Outlines`, `compress()`.
- Inputs: the per-page PDFs as received; output written atomically
  (temp file + rename, like the existing scan path).

**Files and naming** (extends `commands/scan_files.rs`):

- PDF format, complete: `scan-YYYYMMDD-HHMMSS.pdf` (merged, one file).
- PDF format, canceled/errored mid-stack with ≥ 1 page received:
  `scan-YYYYMMDD-HHMMSS-partial.pdf`.
- JPEG format: `scan-YYYYMMDD-HHMMSS-p1.jpg…`; on cancel the received files
  stay, UI marks «неполный: N страниц».
- Collision suffix `-2` as today.

**UI (Scan screen)**:

- During the job: counter «получено страниц: N», elapsed seconds, the existing
  cancel button, a page list (JPEG pages get inline previews, PDF pages by
  name/size).
- Result block: final file path(es) + «Открыть папку»; partial results labeled.

## Acceptance

- 3 sheets, PDF, 300 dpi → one PDF, `pdfinfo` reports `Pages: 3`, page size
  ≈ 602.64 × 830.16 pt (M0 reference), opens in GNOME Papers and Firefox.
- Same with JPEG → 3 `-pN.jpg` files, sizes plausible, previews render.
- Cancel on sheet 2 of 3 → `…-partial.pdf` with the pages received before the
  cancel, device returns to `Idle`, no temp files left (M1 rules hold).
- Pause rule verified both ways in the spike (with and without the 1 s gap).
- `cargo test` gains merge tests on `fixtures/escl/adf/*`; byte-real inputs.

## Left to the builder

- Exact lopdf 0.46 API surface (verify against docs.rs at implementation time).
- Module split staying under 200 lines; where the pause lives (loop vs client).
- Error mapping (`MergeError` → user-facing Russian strings in the `*_LABEL`
  constants).
- PRD updates (§7.1 ADF facts, §9 event payloads, §12 roadmap row, §13.3/13.5
  closed or re-scoped) — after the spike, when the tree is free of the M2 agent.

## Appendix — spike checklist (throwaway, one session)

All against the live device, paper in the feeder (it is already loaded):

1. 2 sheets, PDF: `NextDocument` × N — is each response a single-page PDF or
   one multi-page PDF? Record timings per page. → decides outcome A/B.
2. 2 sheets, JPEG: same, confirm per-page JPEG and `Content-Location: doc-N`.
3. No pause between requests: reproduce `404` + `ScannerAdfEmpty`? (the
   sane-airscan quirk on our unit)
4. 3 sheets, cancel on page 2: `DELETE` → `200`, hanging fetch → `410`,
   `ScannerStatus` → `Idle`; what does the feeder do with sheet 3?
5. 1 sheet, PDF (edge: single-sheet stack through the ADF).
6. Power-cycle probe (not ADF, same session): with the app window open and
   polling, switch the printer off → chip «Нет связи» within one poll; switch
   on → recovers to «в сети» by itself (PRD §14, never tested on real power).
   Record first-request latency after boot and after deep idle.
7. Raw logs → `/tmp/opencode/`, decoded results → `docs/escl-l2710dw.md` §11;
   per-page fixtures → `fixtures/escl/adf/`.

## Decision provenance

| Decision | Source |
|---|---|
| Cancel keeps partial pages | user (round 2, q1) |
| No-percentage progress counter | user (round 2, q4) |
| JPEG = per-page files (industry convention) | my call, delegated (Brother/Epson/Ricoh/Xerox/GNOME evidence) |
| `lopdf` over `pdfunite`/`pdf`/`pdf-writer` | my call, delegated (crates.io + context7, 27.10.2026) |
| Merge only under outcome A | my call (evidence-first) |
| Viewers for acceptance: Papers + Firefox, `pdfinfo` | my call (system check: default handler `org.gnome.Papers`, poppler-utils 26.01) |
