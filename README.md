# prindeck

Control panel for the Brother MFC-L2710DW on Linux — a replacement for Brother
iPrint&Scan, which does not officially exist for Linux: device status, toner and
drum levels, eSCL scanning, CUPS printing, tray icon.

Project documents: [`PRD.md`](PRD.md) is the source of truth (requirements,
stack, roadmap). Protocol facts and spike results live in [`docs/`](docs/), and
step-by-step plans in [`docs/superpowers/plans/`](docs/superpowers/plans/).

## Status

**M0 — protocol spike: done.** **M1 — Tauri shell and eSCL scanning: done.**
**M2 — navigation shell and device overview: done.** **M2.5 — print, history
recording and ADF multi-sheet: done** (the final live device checks are
pending, tracked in the PRD §14).

- `ui/` + `src-tauri/` — Tauri 2 app (Rust + vanilla TypeScript): navigation
  shell (left sidebar with three destinations — Скан, Обзор and Печать; icon
  rail on narrow windows), scan screen, device overview over SNMP
  (toner/drum levels, page and duplex counters), background status polling
  every 5 minutes with a cached first paint, tray icon, single instance. Both
  colour themes follow the system setting; the window can shrink to 640×480.
- Print screen («Печать», M2.5) — queue-first: pick or drop a file, set copies
  (1–99) and duplex (one-sided / long-edge / short-edge), send via CUPS and
  watch the queue — unfinished jobs with a per-row cancel, refreshed every 5 s
  while the screen is open. All CUPS subprocesses run with `LC_ALL=C`, so
  parsing never depends on the machine locale (`lpstat` localizes its output
  on this ru_RU system otherwise).
- ADF multi-sheet scanning (M2.5) — a PDF job returns one file with the whole
  stack (the printer's firmware assembles the multi-page document itself); a
  JPEG job returns one `scan-…-pN.jpg` per sheet. Cancelling or failing
  mid-stack keeps the pages already received (`…-partial.pdf`), and the scan
  screen shows a partial badge with the reason.
- History recording (M2.5) — every successful status poll writes a snapshot
  (toner, drum, counters) to `~/.local/share/prindeck/history.db` (SQLite).
  Recording failures are logged and never interrupt polling; there is no UI
  for the history yet (M4).
- `scripts/escl-probe.mjs` — eSCL scan-job lifecycle (`fetch`-only, no deps).
- `scripts/probe-snmp.mjs` — SNMPv2c GET with a hand-written BER codec
  (`node:dgram` only), including the batched three-varbind GET the app uses.
- `scripts/screenshot.sh` — screenshots for the `visual-check` agent
  (xdg-desktop-portal; `grim` does not work on this Mutter session).
- `fixtures/` — raw device responses (XML, JSON, ADF PDF/JPEG documents,
  `lpstat` output), captured 26–27.10.2026.

## Building and running

Requirements: Node.js >= 26, Rust 1.99.1, WebKitGTK 4.1 development headers and
the other Tauri packages — see [`docs/m1-prerequisites.md`](docs/m1-prerequisites.md).

```sh
npm install
npm run tauri dev     # builds the Rust side and opens the window
npm run typecheck     # tsc --noEmit
npm run build         # frontend bundle
```

Backend checks:

```sh
cargo test --workspace
cargo check --workspace --all-targets
```

The config is created on first run at `~/.config/prindeck/config.json`
(`device.host`, `scanDir`; optional `cupsQueue`, `snmp.community`,
`pollSeconds`); scans are saved to `~/Documents/scans`.

## Behaviour

- Closing the window hides it to the tray; **Quit** in the tray menu ends the
  process.
- A second launch focuses the already running instance.
- Scanning: the platen (one page per job) and the ADF (multi-sheet — one
  whole-stack PDF or one JPEG per page, see above). Cancelling mid-stack keeps
  the pages already received.
- The Overview screen shows the device data from the last SNMP poll; the poller
  runs every `pollSeconds` (default 300 s, clamped to 60–3600) and the UI keeps
  showing the cached snapshot when the device goes silent.
- The sidebar dot combines the eSCL capabilities state with the SNMP poll; a
  problem is explained by a banner with a retry action, not by a greyed-out
  control.

## Probe scripts

```sh
node scripts/escl-probe.mjs caps
node scripts/escl-probe.mjs scan --source platen --dpi 300 --mode Grayscale8 \
  --format application/pdf --out ./scans
node scripts/probe-snmp.mjs --json
```

## Disclaimer

This project is not affiliated with Brother Industries, Ltd. "Brother" and
"MFC-L2710DW" are trademarks of their respective owners and are mentioned only
to indicate compatibility.
