# prindeck

Control panel for the Brother MFC-L2710DW on Linux — a replacement for Brother
iPrint&Scan, which does not officially exist for Linux: device status, toner and
drum levels, eSCL scanning, CUPS printing, tray icon.

Project documents: [`PRD.md`](PRD.md) is the source of truth (requirements,
stack, roadmap). Protocol facts and spike results live in [`docs/`](docs/), and
step-by-step plans in [`docs/superpowers/plans/`](docs/superpowers/plans/).

## Status

**M0 — protocol spike: done.** **M1 — Tauri shell and eSCL scanning: in
progress.**

- `ui/` + `src-tauri/` — Tauri 2 app (Rust + vanilla TypeScript): scan screen,
  background scan task, tray icon, single instance.
- `scripts/escl-probe.mjs` — eSCL scan-job lifecycle (`fetch`-only, no deps).
- `scripts/probe-snmp.mjs` — SNMPv2c GET with a hand-written BER codec
  (`node:dgram` only).
- `scripts/screenshot.sh` — screenshots for the `visual-check` agent
  (xdg-desktop-portal; `grim` does not work on this Mutter session).
- `fixtures/` — raw device responses (XML, JSON), captured on 2026-10-26.

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
(`device.host`, `scanDir`); scans are saved to `~/Documents/scans`.

## Behaviour

- Closing the window hides it to the tray; **Quit** in the tray menu ends the
  process.
- A second launch focuses the already running instance.
- M1 scans the platen only, one page per job, PDF or JPEG. ADF and multi-page
  scanning are out of scope for M1.

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
