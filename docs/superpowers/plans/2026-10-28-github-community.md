# GitHub Community and CI (M5) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Before publication, give the repo issue forms, templates, labels, CI on `main` and a ruleset — no third-party Actions, no bots.

**Architecture:** Wave 1 = six independent file tasks in parallel worktrees (disjoint files, no interfaces). Wave 2 = sequential maintainer operations on origin (apply labels, enable Discussions, create ruleset after the first green CI run).

**Tech Stack:** GitHub issue forms YAML, GitHub Actions (official actions only), `gh api`, Node ≥ 26.

**Spec:** `docs/superpowers/specs/2026-10-28-github-community-design.md` — the plan argues from the spec; executors read both.

## Global Constraints

- All new files English. No comments in code or workflow YAML. Commit prefix `M5:`.
- No source file over 200 lines (`scripts/setup-labels.mjs` included).
- Zero third-party Actions: `actions/checkout@v6`, `actions/setup-node@v7`, `actions/cache` (pin current major via context7 in Task 3). Nothing else.
- Repo facts (verified 28.10.2026): root `Cargo.toml` is the workspace root (`members = ["src-tauri"]`) — cargo commands run at repo root, no `working-directory`. Repo `bumbaRasch/prindeck`, private, default branch `main`, `has_discussions: false`.
- The seven commands, verbatim: `cargo test --workspace`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `npm run typecheck`, `npm run build`, `npm run check:code-language`, `npm run check:line-count`.
- Label strings are exact and shared across Tasks 4, 5, 7, 8: `type: bug`, `type: feature`, `area: scan`, `area: overview`, `area: print`, `area: settings`, `area: tray`, `os: wayland`, `os: x11`, `status: needs-triage`, `status: confirmed`, `status: duplicate`, `status: wontfix`, `priority: high`, `priority: low` (+ kept `dependencies`).
- `ui/src/labels/overview.ts` is maintainer WIP — never stage or commit it.
- Do not contact the printer; the probe commands in this plan talk only to unroutable test addresses.

## Review Focus

1. `probe-snmp.mjs --json` must never echo the configured community again (a paste of its output goes straight into public issues). Pinned by Task 1 verification; re-run it whenever the envelope changes.
2. Forms apply labels by exact name and **silently skip missing ones**. Pinned by Task 8: the live label list must equal the taxonomy before forms see real use.
3. Dependabot recreates ecosystem labels (`javascript`, later `rust`), fighting the delete list. Pinned by Task 2's config + Task 7 note: if an ecosystem label reappears after pinning, apply the spec §1 fallback (leave + document), do not delete in a loop.
4. The ruleset's required check context must equal the real check-run name the first CI run produces (`checks`). Pinned by Task 10: read the name from the API, never from memory.
5. `validations.required` is not enforced while the repo is private — reviewers must not "verify" required fields by opening issues now. Pinned by the At-publication checklist.

---

## Wave 1 — parallel worktree tasks (disjoint files)

Each task: own worktree, own branch `m5/<slug>`, merge to `main` after review. No task reads another task's output.

### Task 1: probe-snmp `--json` stops echoing the SNMP community

**Files:**
- Modify: `scripts/probe-snmp.mjs:363`

**Interfaces:**
- Produces: `--json` envelope key `community` with the constant value `'<community>'` (shape otherwise unchanged).

- [ ] **Step 1: Edit the JSON envelope**

In the `if (values.json)` branch replace `community: opts.community` with `community: '<community>'`. Nothing else in the file changes.

- [ ] **Step 2: Verify no live community reaches the output**

Run: `grep -n "opts.community" scripts/probe-snmp.mjs`
Expected: matches only in CLI parsing (~line 315), none in the `console.log(JSON.stringify(...))` line.

Run: `node scripts/probe-snmp.mjs --help` (confirm flag names), then `node scripts/probe-snmp.mjs --json --host 127.0.0.1 --port 2`
Expected: JSON envelope prints `"community": "<community>"`; per-OID connection errors are expected and fine.

- [ ] **Step 3: Repo checks**

Run: `npm run check:code-language && npm run check:line-count`
Expected: both pass.

- [ ] **Step 4: Commit**

```bash
git add scripts/probe-snmp.mjs
git commit -m "M5: stop probe-snmp --json from echoing the SNMP community"
```

### Task 2: Dependabot PRs pinned to `dependencies`

**Files:**
- Modify: `.github/dependabot.yml`

**Interfaces:**
- Produces: every ecosystem update entry carries `labels: ["dependencies"]` (stops `javascript`/`rust` ecosystem labels, unblocks Task 5's delete list).

- [ ] **Step 1: Add the labels key**

To each of the three `updates:` entries (cargo, npm, github-actions) add, inside the entry:

```yaml
    labels:
      - dependencies
```

- [ ] **Step 2: Syntax check**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/dependabot.yml'))"` (if PyYAML is absent, a careful visual check + merge is acceptable; GitHub validates the config on its next scheduled run).

- [ ] **Step 3: Commit**

```bash
git add .github/dependabot.yml
git commit -m "M5: pin Dependabot PRs to the dependencies label"
```

### Task 3: CI workflow

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Produces: a check run named exactly `checks` on every push to `main` and every PR — Task 10 requires that exact context name.

- [ ] **Step 1: Confirm the cache action major**

Via context7 (`/websites/github_en`): current major of `actions/cache` (`checkout@v6`, `setup-node@v7` confirmed 28.10.2026).

- [ ] **Step 2: Write the workflow**

```yaml
name: ci

on:
  push:
    branches: [main]
  pull_request:

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}

jobs:
  checks:
    runs-on: ubuntu-24.04
    timeout-minutes: 45
    steps:
      - uses: actions/checkout@v6
      - name: Install Tauri build dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential pkg-config libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev patchelf
      - name: Rust stable
        run: rustup update stable && rustup default stable
      - name: Cargo registry cache
        uses: actions/cache@<major from step 1>
        with:
          path: ~/.cargo
          key: cargo-registry-${{ runner.os }}-${{ hashFiles('src-tauri/Cargo.lock') }}
      - name: Rust checks
        run: |
          cargo test --workspace
          cargo check --workspace --all-targets
          cargo clippy --workspace --all-targets -- -D warnings
      - uses: actions/setup-node@v7
        with:
          node-version: 26
          cache: npm
      - name: Frontend checks
        run: |
          npm ci
          npm run typecheck
          npm run build
          npm run check:code-language
          npm run check:line-count
```

- [ ] **Step 3: Syntax check**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` (fallback as in Task 2; the first real parse is the first Actions run after merge).

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "M5: add ci workflow running the pre-commit ritual"
```

### Task 4: Issue forms and template chooser

**Files:**
- Create: `.github/ISSUE_TEMPLATE/bug_report.yml`
- Create: `.github/ISSUE_TEMPLATE/feature_request.yml`
- Create: `.github/ISSUE_TEMPLATE/config.yml`

**Interfaces:**
- Consumes: label strings from Global Constraints (runtime existence comes from Task 8, not from this task).

**Field tables — every field needs `id`, exact `label`, `validations.required` as listed; prose wording is the implementer's, tone plain English.**

`bug_report.yml`: top-level `name: Bug report`, `description: Something in prindeck does not work as expected.`, `labels: ["type: bug", "status: needs-triage"]`.

| id | type | label | required | pinned content |
|---|---|---|---|---|
| — | markdown | — | — | intro: what to include; **redaction warning** (no SNMP community strings, no real IPs/MACs/hostnames, `192.0.2.10`-style placeholders, link SECURITY.md); both probe commands are read-only and never start a scan |
| what-happened | textarea | What happened? | true | description asks for actual vs expected behaviour |
| version | input | prindeck version or commit | true | placeholder `Example: 0.1.0 or the commit you built` |
| os | input | OS, distribution and desktop session | true | placeholder `Example: Ubuntu 26.04, GNOME Wayland (cat /etc/os-release, echo $XDG_SESSION_TYPE)` |
| printer | input | Printer model and firmware | true | — |
| address-kind | input | How is the device addressed? | false | description: IP literal / hostname / mDNS name / non-standard port; do not paste the real address |
| probes | textarea | Probe output | true | description: paste both `node scripts/escl-probe.mjs caps` and `node scripts/probe-snmp.mjs --json`; `render: shell` |
| screenshots | upload | Screenshots | false | — |
| checks | checkboxes | Before submitting | true | two options, each `required: true`: «I redacted secrets (community strings, addresses, MACs, hostnames)», «I searched existing issues and discussions» |

`feature_request.yml`: `name: Feature request`, `description: Suggest an idea for prindeck.`, `labels: ["type: feature", "status: needs-triage"]`. Markdown intro points at the PRD §2 non-goals (OCR, cloud destinations, Windows/macOS, multi-printer are deliberately out — such requests belong in Discussions). Fields: `problem` (textarea, The problem or need, true), `solution` (textarea, Proposed solution, true), `alternatives` (textarea, Alternatives or workarounds, false), `searched` (checkboxes, one required option: «I searched existing issues and discussions»).

`config.yml` — exact content:

```yaml
blank_issues_enabled: false
contact_links:
  - name: Questions and ideas
    url: https://github.com/bumbaRasch/prindeck/discussions/new/choose
    about: Ask questions and propose ideas in Discussions, not issues.
  - name: Security vulnerabilities
    url: https://github.com/bumbaRasch/prindeck/security/policy
    about: Report privately via the security policy — never a public issue.
```

- [ ] **Step 1: Write the three files** per the tables and schema (form schema reference: GitHub Docs, issue forms syntax).

- [ ] **Step 2: Syntax check** — `python3 -c "import yaml, glob; [yaml.safe_load(open(p)) for p in glob.glob('.github/ISSUE_TEMPLATE/*.yml')]"`.

- [ ] **Step 3: Commit**

```bash
git add .github/ISSUE_TEMPLATE/
git commit -m "M5: add bug and feature issue forms with a chooser config"
```

### Task 5: Label setup script

**Files:**
- Create: `scripts/setup-labels.mjs`

**Interfaces:**
- Produces: `node scripts/setup-labels.mjs [--dry-run]`; consumed by Task 8. Idempotent: safe to re-run.

- [ ] **Step 1: Implement the script**

Node ≥ 26, no dependencies, no comments, English output. Shells out to `gh api` (`child_process.execFileSync`). Constants:

- `TAXONOMY` — 15 entries `{name, color, description}`:

| name | color | description |
|---|---|---|
| `type: bug` | d73a4a | Something does not work as documented or expected |
| `type: feature` | a2eeef | New capability or behaviour request |
| `area: scan` | 1d76db | Scan screen and the eSCL pipeline |
| `area: overview` | 1d76db | Overview screen and SNMP status data |
| `area: print` | 1d76db | Print screen and the CUPS pipeline |
| `area: settings` | 1d76db | Settings screen and config handling |
| `area: tray` | 1d76db | Tray icon, menu, autostart, window behaviour |
| `os: wayland` | c2e0c6 | Reproduces on a Wayland session |
| `os: x11` | c2e0c6 | Reproduces on an X11 session |
| `status: needs-triage` | eade9b | Fresh report, not yet reviewed by the maintainer |
| `status: confirmed` | 0e8a16 | Reproduced and accepted for fixing |
| `status: duplicate` | d4c5f9 | Same root cause as an existing issue |
| `status: wontfix` | f9d0c4 | Deliberately not addressed (see PRD non-goals) |
| `priority: high` | d93f0b | Breaks a core flow: scanning or printing |
| `priority: low` | fef2c0 | Cosmetic or minor inconvenience |

- `DELETE` — 11 names: `accessibility`, `bug`, `documentation`, `duplicate`, `enhancement`, `good first issue`, `help wanted`, `invalid`, `javascript`, `question`, `wontfix`.

Behaviour: GET `/repos/bumbaRasch/prindeck/labels`; for each taxonomy entry POST when missing, PATCH when color/description differ; DELETE each entry of the delete list tolerating 404; `dependencies` is never touched. `--dry-run` prints the planned operations without calling mutating endpoints (GET allowed).

- [ ] **Step 2: Verify dry run**

Run: `node scripts/setup-labels.mjs --dry-run`
Expected: reports 15 create/update operations + 11 deletions, no `gh api` POST/PATCH/DELETE executed.

- [ ] **Step 3: Repo checks**

Run: `npm run check:code-language && npm run check:line-count`
Expected: both pass.

- [ ] **Step 4: Commit**

```bash
git add scripts/setup-labels.mjs
git commit -m "M5: add idempotent label taxonomy setup script"
```

### Task 6: PR template, CONTRIBUTING, README community section

**Files:**
- Create: `.github/pull_request_template.md`
- Create: `CONTRIBUTING.md` (≤ 200 lines)
- Modify: `README.md` (Community section before `## License`)

**Interfaces:**
- Consumes: the seven commands and the label table (copied verbatim from Global Constraints / Task 5).

- [ ] **Step 1: PR template** — sections: `## Summary`; `## Changes`; `## Related issues` (`Fixes #`); `## How tested` with the seven commands as literal checkboxes; closing note: commit messages are English with the milestone prefix (`M5: …`).

- [ ] **Step 2: CONTRIBUTING.md** — sections: project intro (one paragraph, link PRD); dev environment (link `docs/m1-prerequisites.md`; Node ≥ 26, Rust stable, WebKitGTK deps); the seven-command pre-commit ritual as a code block; commit style (English, milestone prefix); PR flow (PRs into `main`, CI `checks` must pass); code rules (English code; Russian UI strings only in `ui/src/labels/` and `*_LABEL` constants — `check:code-language` enforces; no comments; ≤200 lines — `check:line-count` enforces; domain-named modules); how to report (bug/feature forms, Discussions for questions, SECURITY.md for vulnerabilities); the label table from Task 5 (names + descriptions, colors optional).

- [ ] **Step 3: README** — add `## Community`: contributing → `CONTRIBUTING.md`, security → `SECURITY.md`, questions → Discussions.

- [ ] **Step 4: Repo checks**

Run: `npm run check:code-language && npm run check:line-count`
Expected: both pass.

- [ ] **Step 5: Commit**

```bash
git add .github/pull_request_template.md CONTRIBUTING.md README.md
git commit -m "M5: add PR template, contributing guide and community readme section"
```

---

## Wave 2 — sequential maintainer operations (main session, against origin)

### Task 7: Merge wave 1 and first green run

- [ ] **Step 1: Merge the six branches** into `main` (merge commits, current practice), push once.
- [ ] **Step 2: Watch the first CI run** — `gh run watch` (or `gh run list`). Expected: workflow `ci`, job `checks`, green.
- [ ] **Step 3: Record the check-run name** — `gh api repos/bumbaRasch/prindeck/commits/$(git rev-parse HEAD)/check-runs --jq '.check_runs[].name'`. Expected output: `checks`. This exact string is Task 10's required context.
- [ ] **Step 4: Full local ritual** (the seven commands) — all green; commit `M5: wave-1 merge completion — github community files` if any merge fixes were needed.
- [ ] **Step 5: Dependabot PR #1** (typescript bump, fixes the open moderate alert): if still open and CI green on main after merge, merge it; re-run the ritual if npm deps changed.

### Task 8: Apply the label taxonomy

- [ ] **Step 1:** `node scripts/setup-labels.mjs`
- [ ] **Step 2:** `gh api repos/bumbaRasch/prindeck/labels --paginate --jq '.[].name' | sort`
Expected: exactly 16 names — the 15 taxonomy labels + `dependencies`.

### Task 9: Enable Discussions

- [ ] **Step 1:** `gh api -X PATCH repos/bumbaRasch/prindeck -f has_discussions=true`
- [ ] **Step 2:** In Settings → Discussions pick the welcome post; keep categories **Announcements**, **General**, **Ideas**, **Q&A**; delete **Polls** and **Show and tell**.
- [ ] **Step 3:** Verify — `gh api graphql -f query='{ repository(owner: "bumbaRasch", name: "prindeck") { discussionCategories(first: 10) { nodes { name } } } }'`
Expected: the four kept categories.

### Task 10: Ruleset on `main`

- [ ] **Step 1:** Settings → Rules → Rulesets → New branch ruleset: name `protect-main`; enforcement **Active**; bypass: **Repository admin**; target `~DEFAULT_BRANCH`; rules: Require status checks → context `checks` (the string from Task 7), strict policy on; Block force pushes; Restrict deletions. (UI avoids guessing `RepositoryRole` actor_id; API `POST /repos/…/rulesets` with `bypass_actors` `RepositoryRole` is the fallback.)
- [ ] **Step 2:** If rulesets are unavailable on the private repo (Free-plan gate): record the blocker, defer creation to the publication moment (public repos have rulesets unconditionally). Do not silently skip.
- [ ] **Step 3:** Verify — `gh api repos/bumbaRasch/prindeck/rulesets` → `protect-main`, `active`; detail GET shows the three rules and the bypass actor.
- [ ] **Step 4:** Bypass smoke test — `git commit --allow-empty -m "M5: ruleset bypass smoke" && git push`. Expected: push accepted; the triggered run goes green. A PR without CI being blocked is verified at publication (needs an external PR).

## At publication (from the spec's publish checklist)

- [ ] `validations.required` starts enforcing (public-only — until then advisory)
- [ ] `contact_links` URLs live (Discussions, security policy)
- [ ] Throwaway test issue per form: fields render, labels applied, close it
- [ ] PR without CI is blocked by the ruleset (external PR smoke)
- [ ] Community profile standards checklist green
