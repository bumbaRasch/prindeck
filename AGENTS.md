# prindeck: agent instructions

Tauri 2 (Rust) + vanilla TypeScript control panel for the Brother MFC-L2710DW.
Requirements live in [`PRD.md`](PRD.md), protocol facts in [`docs/`](docs/), the
current plan in `docs/superpowers/plans/`.

## Environment

- Rust: `export PATH="$HOME/.cargo/bin:$PATH"` (cargo/rustc 1.99.1).
- Before committing: `cargo test --workspace`,
  `cargo check --workspace --all-targets`, `npm run typecheck`, `npm run build`
  — all green, no warnings.
- Device: `192.168.187.36`, eSCL `http://192.168.187.36/eSCL/`, scans in
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
| `docs/m1-prerequisites.md` | Tauri build environment |
| `docs/superpowers/plans/` | step-by-step work plans |

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
- Commit step by step, with the milestone as the message prefix (`M1: ...`).
