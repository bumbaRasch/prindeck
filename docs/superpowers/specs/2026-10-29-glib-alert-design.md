# glib alert (RUSTSEC-2024-0429) — Design (short)

- Status: draft for approval (29.10.2026).
- Sources: crates.io checked live 29.10.2026 (tauri 2.12.5, wry 0.57.1 — latest;
  webkit2gtk crate 2.0.2, Dec 2025 — still `glib ^0.18`); Dependabot alert #1
  (glib, medium, GHSA-wrw7-89jp-8q8g / RUSTSEC-2024-0429, patched 0.20.0);
  upstream tauri#16264 (maintainer-confirmed gtk 0.19 migration, 01.10.2026);
  sibling Tauri projects tracking the same block.

## Intent

Close the "what do we do about the glib alert" question honestly: the desired
tauri/wry upgrade does not exist yet, the advisory's vulnerable code path is
unused by prindeck, and the milestone cost of a fork is not justified.

## Decisions

1. **Upstream-blocked, parked (approach A).** The alert stays open in
   Dependabot — it is the free tracker for the future bump (controller call;
   dismissing costs one `gh api` later if the noise bothers).
2. **Docs only, no code.** PRD §0 currency table gains a glib/tauri row
   (alert open, blocked by tauri#16264, re-check at each milestone per the
   existing policy); §10 gains one stack note recording the same.
3. When a Tauri release lands on gtk 0.19+/glib ≥ 0.20: plain dependency bump
   inside the existing `2` requirement, full ritual, PRD table updated —
   expected uneventful (MSRV 1.90 < our 1.99.1).

## Approaches

- **A — park upstream-blocked + document (chosen)**: zero code, honest,
  self-driving via the milestone re-check policy.
- **B — `[patch]`-fork gtk/webkit2gtk now**: own gtk-rs builds to close a
  formal alert whose code path we never execute; against crate discipline and
  KISS. Rejected.
- **C — wait, alert open, no documentation**: same outcome as A but the
  reasoning lives only in chat history. Rejected.

## Non-goals

- No dependency changes now; no `cargo audit` tooling (same signal as the
  open alert); no dismissal of the alert.
