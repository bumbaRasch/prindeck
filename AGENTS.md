# prindeck: agent instructions

Tauri 2 (Rust) + vanilla TypeScript control panel for the Brother MFC-L2710DW.
Requirements live in [`PRD.md`](PRD.md), protocol facts in [`docs/`](docs/), the
current plan in `docs/superpowers/plans/`.

## Environment

- Rust: `export PATH="$HOME/.cargo/bin:$PATH"` (cargo/rustc 1.99.1).
- Before committing: `cargo test --workspace`,
  `cargo check --workspace --all-targets`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `npm run typecheck`,
  `npm run build`, `npm run check:code-language` — all green, no warnings.
- Device: the printer address from the local config (the docs use
  `192.0.2.10` as an example), eSCL at `http://<host>/eSCL/`, scans in
  `~/Documents/scans`, config at `~/.config/prindeck/config.json`.
- The printer is physical: a scan takes ~8 s and occupies the device. Start a
  scan (from code, curl, or the UI) only when the user explicitly asks for it.

## Visual checks

Visual checks in this project are run by the **`visual-check`** subagent pinned
to the **Muse Spark 1.3 Contributor** model
(`opencode-go/muse-spark-1.3-contributor`, OpenCode Go provider):
`.opencode/agents/visual-check.md`.

- Launch it through the `subagent` tool with `agent: "visual-check"`.
- It is read-only: it starts the app, takes screenshots with
  `scripts/screenshot.sh`, inspects them, and reports back with PNG paths.
- The main agent applies any fixes; `visual-check` never edits code.
- This GNOME/Wayland session has no working `grim`, no `xdotool`/`ydotool`, and
  Mutter RemoteDesktop does not expose a session: screenshots go through
  `scripts/screenshot.sh` (xdg-desktop-portal), and in-window clicks cannot be
  automated. The app window must be brought to the front by the user before a
  check — a restart will not raise it above a maximized window.
- A visual check does not replace live scan/cancel/offline checks; those are
  separate items in the plan checklist.

## Documents

| File | Contents |
|---|---|
| `PRD.md` | requirements, stack, roadmap, open questions |
| `docs/escl-l2710dw.md` | eSCL device facts, including the cancel spike (§10) |
| `docs/oids-l2710dw.md` | SNMP, OIDs, response shapes (for M2) |
| `docs/ews-l2710dw.md` | device web panel: URL scheme, auth model, verified public links, gated-zone notes |
| `docs/m1-prerequisites.md` | Tauri build environment |
| `docs/superpowers/plans/` | step-by-step work plans |
| `docs/ref/` | local-only, **gitignored**: research notes and raw evidence (installer anatomy, feature inventory, gap analysis); not tracked and not distributed |

## Rules

- Do not invent device facts: verify with a live request or in `docs/`.
- Ignore unknown fields in device responses and log them (PRD §5.6).
- A `job_id` coming from IPC is only a `/eSCL/ScanJobs/<id>` path, never an
  arbitrary URL.
- Write no comments in code. Keep code self-explanatory; explanations belong in
  `docs/` or `AGENTS.md`.
- All documentation (Markdown, plans, agent files) is English. UI strings stay
  Russian until the i18n work — don't translate them. When i18n lands, the UI
  language follows the system locale and falls back to English for unsupported
  locales.
- Nothing is done from memory when a lookup is cheap. For library, framework,
  SDK, CLI or API questions use the `context7` MCP (`resolve-library-id`, then
  `query-docs`) and follow the version actually pinned in the project. For
  "how do people do this today", comparisons and current practice use the
  `searxng` MCP. Say which source the answer came from when it matters.
- The same applies to subagents: a fresh subagent inherits no memory of this
  conversation, so pass the lookup requirement into the task you hand over.
- Build UI full responsive: every screen stays usable at every window size the
  app allows, in both themes. No fixed pixel layout that clips, overlaps or
  hides controls; prefer fluid grids, container queries and wrapping, and
  check the smallest and the largest window before calling the work done.
- Design for the user first: one obvious primary action per screen, the state
  always visible, no dead ends, no placeholder controls. When something is not
  possible, say it in the interface instead of failing silently or greying
  out a control without a reason.
- No source file over 200 lines. When one grows past that, split it along
  domain responsibilities before adding more code, and name the modules after
  the domain (`status`, `scan`, `tray`) — never `utils`, `common` or `helpers`.
  Files already above the limit are split when they are next touched.
- KISS first, then DRY, then SOLID: the simplest structure that names the
  domain wins; extract shared code only at the second real use; one module, one
  reason to change; depend on the IPC/domain types, not on screen internals.
  Rust: `rustfmt` plus the `[lints]` tables in `src-tauri/Cargo.toml` and
  clippy `-D warnings` carry the rule (Pragmatic Rust Guidelines
  M-BALANCED-MODULES and M-STATIC-VERIFICATION, read through `context7`).
  TypeScript: the strict flags in `tsconfig.json` play the same role.
- Commit step by step, with the milestone as the message prefix (`M1: ...`).
