# Contributing to prindeck

prindeck is an unofficial Linux control panel for the Brother MFC-L2710DW: a
Tauri 2 app — Rust core, vanilla TypeScript frontend — that talks to one
printer over eSCL, SNMP and CUPS, without Brother binary drivers. The
requirements, stack and roadmap live in [`PRD.md`](PRD.md); read it before
changing behaviour, and check the
[non-goals](PRD.md#2-goals-and-non-goals) before proposing a feature.

## Development environment

- Node.js >= 26 and npm.
- Rust stable (currently built with 1.99.1).
- WebKitGTK 4.1 development headers and the other Tauri system packages —
  the full list is in [`docs/m1-prerequisites.md`](docs/m1-prerequisites.md).

```sh
npm install
npm run tauri dev
```

## Before every commit

The pre-commit ritual — all seven commands green, no warnings:

```sh
cargo test --workspace
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
npm run typecheck
npm run build
npm run check:code-language
npm run check:line-count
```

## Commit style

Commit messages are English, one step per commit, with the current milestone
as the prefix: `M5: add ci workflow running the pre-commit ritual`.

## Pull requests

- PRs target `main` and fill in the pull request template.
- The CI check `checks` must pass before merge.
- A change that touches the device pipeline states how it was verified
  against the physical printer — the device is physical, so never start a
  scan from code or a script casually.

## Code rules

- Code is English. Russian UI copy lives only in `ui/index.html`,
  `ui/src/labels/` and `*_LABEL` constants — `npm run check:code-language`
  enforces this.
- No comments in code. Keep it self-explanatory; explanations belong in
  `docs/` or the commit message.
- No source file over 200 lines. When a file grows past the limit, split it
  along domain responsibilities — `npm run check:line-count` enforces the
  limit.
- Name modules after the domain (`status`, `scan`, `tray`), never `utils`,
  `common` or `helpers`.
- Do not invent device facts: verify with a live request or in `docs/`, and
  ignore-and-log unknown fields in device responses.

## Reporting

- Bugs and feature requests use the GitHub issue forms. A feature request
  must clear the [PRD non-goals](PRD.md#2-goals-and-non-goals) — OCR, cloud
  destinations, Windows/macOS builds and multi-printer support are
  deliberately out.
- Questions and ideas belong in
  [Discussions](https://github.com/bumbaRasch/prindeck/discussions), not
  issues.
- Vulnerabilities are reported privately as described in
  [`SECURITY.md`](SECURITY.md) — never in a public issue, discussion or
  pull request.

## Labels

| Label | Meaning |
|---|---|
| `type: bug` | Something does not work as documented or expected |
| `type: feature` | New capability or behaviour request |
| `area: scan` | Scan screen and the eSCL pipeline |
| `area: overview` | Overview screen and SNMP status data |
| `area: print` | Print screen and the CUPS pipeline |
| `area: settings` | Settings screen and config handling |
| `area: tray` | Tray icon, menu, autostart, window behaviour |
| `os: wayland` | Reproduces on a Wayland session |
| `os: x11` | Reproduces on an X11 session |
| `status: needs-triage` | Fresh report, not yet reviewed by the maintainer |
| `status: confirmed` | Reproduced and accepted for fixing |
| `status: duplicate` | Same root cause as an existing issue |
| `status: wontfix` | Deliberately not addressed (see PRD non-goals) |
| `priority: high` | Breaks a core flow: scanning or printing |
| `priority: low` | Cosmetic or minor inconvenience |
