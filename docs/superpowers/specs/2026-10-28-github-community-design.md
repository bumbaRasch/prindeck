# GitHub community and CI (M5) — Design

- Status: draft for approval
- Phase: M5 (release prep). The repo is private «пока»; everything here lands
  now, the gated parts activate at publication.
- Sources: GitHub Docs via context7 (`/websites/github_en`), checked 28.10.2026 —
  issue forms syntax (`type:` key, `labels` silently skipped when the label is
  missing, `validations.required` works **only in public repos**), `config.yml`
  (`blank_issues_enabled`, `contact_links`), rulesets (required status checks
  block direct ref updates; bypass roles incl. Repository admin; bypass covers
  the whole ruleset), Discussions available on private repos, PR template
  locations, Discussions categories. Repo state via `gh api` 28.10.2026:
  private, owner type User, `has_discussions: false`, default branch `main`;
  12 labels exist including strays (`accessibility`, `javascript`).

## Intent

After publication people must be able to report problems in a form that is
triageable by one maintainer: structured bug data (version, OS, printer,
probe outputs, screenshots), no secrets, questions routed to Discussions,
vulnerabilities to SECURITY.md. CI must gate what lands on `main` before the
repo goes public, with no third-party supply chain.

## Decisions (made with the author)

1. **Personal account** (`bumbaRasch`), no organization. `type:` labels replace
   org-only issue types.
2. **Two forms** (bug report, feature request), `blank_issues_enabled: false`,
   questions → Discussions, vulnerabilities → SECURITY.md via `contact_links`.
3. **Manual triage**: no bots, no stale, no labeler.
4. **Forms and all new docs in English.**

## Approaches

- **A — static files + one workflow (chosen)**: everything as files in the
  repo; labels via one committed idempotent script; single CI job mirroring
  the local pre-commit ritual; ruleset applied once via `gh api`. Zero
  third-party actions.
- **B — split CI jobs + `actions/labeler` (rejected)**: faster feedback and PR
  auto-labeling, but +2 files, a third-party action for Dependabot to track,
  and auto-triage contradicts decision 3. Revisit only if PR volume grows.
- No CI/ruleset at all — rejected: publication was explicitly gated on CI.

## The parts

### 1. Labels (`scripts/setup-labels.mjs`)

Idempotent maintainer script (Node, shells out to `gh api`): upserts the
taxonomy below (name, color, description), deletes the leftovers
(`bug`, `accessibility`, `documentation`, `duplicate`, `enhancement`,
`good first issue`, `help wanted`, `invalid`, `javascript`, `question`,
`wontfix`). Keeps `dependencies` (Dependabot adds it to its PRs).
Run **before** the forms merge: forms reference labels by exact name and
silently skip missing ones.

| Family | Labels |
|---|---|
| `type:` | `type: bug`, `type: feature` |
| `area:` | `area: scan`, `area: overview`, `area: print`, `area: settings`, `area: tray` |
| `os:` | `os: wayland`, `os: x11` |
| `status:` | `status: needs-triage`, `status: confirmed`, `status: duplicate`, `status: wontfix` |
| `priority:` | `priority: high`, `priority: low` |
| kept | `dependencies` |

Deleting `good first issue` / `help wanted` is my call — re-add if the
community profile needs them later. *(my call)*

### 2. Bug report form (`.github/ISSUE_TEMPLATE/bug_report.yml`)

`labels: ["type: bug", "status: needs-triage"]`. Body:

1. `markdown` — what to include + **redaction warning**: no SNMP community
   strings, no real IPs/MACs/hostnames; use `192.0.2.10`-style placeholders;
   link SECURITY.md. Note that both probe commands are read-only (GET
   ScannerCapabilities, SNMP GET) and never start a scan.
2. `textarea` «What happened? / expected» — required.
3. `input` prindeck version or commit — required.
4. `input` OS/distribution + desktop session — required; placeholder
   «Ubuntu 26.04, GNOME Wayland (`cat /etc/os-release`, `echo $XDG_SESSION_TYPE`)».
5. `input` printer model + firmware — required.
6. `input` device address as configured — required; placeholder is the
   documentation example `192.0.2.10`.
7. `textarea` probe output — required, `render: shell`: paste both
   `node scripts/escl-probe.mjs caps` and `node scripts/probe-snmp.mjs --json`.
8. `upload` screenshots — optional.
9. `checkboxes` — required: «I redacted secrets», «I searched existing issues
   and discussions».

### 3. Feature request form (`.github/ISSUE_TEMPLATE/feature_request.yml`)

`labels: ["type: feature", "status: needs-triage"]`. Body: markdown intro
pointing at PRD §2 non-goals (OCR, cloud, Windows/macOS, multi-printer are
deliberately out — a request there is a discussion, not an issue);
`textarea` problem/need — required; `textarea` proposed solution — required;
`textarea` alternatives — optional; `checkboxes` searched existing — required.

### 4. Template chooser (`.github/ISSUE_TEMPLATE/config.yml`)

`blank_issues_enabled: false`; `contact_links`: «Questions — Discussions»
(`…/discussions/new/choose`) and «Security vulnerabilities»
(`…/security/policy`, about: private channel, never a public issue).
Maintainers keep the blank-issue escape hatch (docs: write access and above
still see it, marked «Maintainers only»).

### 5. PR template (`.github/pull_request_template.md`)

Sections: Summary; Changes; Related issues (`Fixes #`); **How tested** — the
exact seven-command checklist from AGENTS.md (`cargo test --workspace`,
`cargo check --workspace --all-targets`, `cargo clippy --workspace
--all-targets -- -D warnings`, `npm run typecheck`, `npm run build`,
`npm run check:code-language`, `npm run check:line-count`); a note that
commit messages are English with the milestone prefix (`M5: …`).

### 6. CONTRIBUTING.md (repo root, English)

Sections: dev environment (link `docs/m1-prerequisites.md`); the local
pre-commit ritual (same seven commands); commit style (English, milestone
prefix); PR flow (CI is required for PRs into `main`); code rules — English
code, Russian UI strings only in `ui/src/labels/` and `*_LABEL` constants
(`check:code-language` enforces), no comments in code, ≤200 lines per file
(`check:line-count` enforces), domain-named modules; how to report (forms /
Discussions / SECURITY.md); the label taxonomy table from §1.

### 7. CI (`.github/workflows/ci.yml`)

One job `checks` on `ubuntu-24.04` (pinned, not `latest`). Triggers: push to
`main`, all PRs; `concurrency` cancels superseded PR runs. Steps:

1. `actions/checkout` (latest major; verify current major at implementation).
2. `sudo apt-get install` the `docs/m1-prerequisites.md` §2 set
   (`libwebkit2gtk-4.1-dev build-essential pkg-config libxdo-dev libssl-dev
   libayatana-appindicator3-dev librsvg2-dev patchelf`) — the Rust side links
   WebKitGTK, so `cargo check/clippy` need the headers on the runner too.
3. `rustup update stable` — the runner image ships rustup; no third-party
   toolchain action.
4. The three cargo commands (working dir `src-tauri/`).
5. `actions/setup-node` with Node 26 + npm cache; `npm ci`; the four npm
   commands.

Official actions only (`checkout`, `setup-node`, optionally `cache` for
`~/.cargo` — registry only, not `target`; *(my call)*). The check scripts are
plain shell/GNU grep and run headless. Job name `checks` = the ruleset's
required check context.

### 8. Ruleset on `main` (one-time `gh api` call, after the first green run)

Branch ruleset `protect-main`, target `~DEFAULT_BRANCH`, enforcement active:

- `required_status_checks`: context `checks`, strict (PRs tested against the
  latest code). Docs confirm this blocks direct pushes of commits without a
  passing check («commits must first be pushed to another ref»).
- `non_fast_forward` (block force push), `deletion` (block branch deletion).

**Bypass: Repository admin** (the owner on a personal repo) — keeps the
current direct-`M5:`-commit workflow; bypass covers the whole ruleset, so the
owner also retains force-push ability. Accepted: the owner is the only admin
and is trusted. External PRs and Dependabot PRs must pass CI to merge.
Ruleset actor IDs for the bypass entry are verified from the API at
implementation time, not from memory. Availability of rulesets on a private
repo under the Free plan is verified by the successful `POST /repos/…/rulesets`
itself.

### 9. Discussions

Enabled now (works on private repos; currently off). Categories: keep
**Announcements** (maintainer-only posting — release notes later in M5),
**General**, **Ideas**, **Q&A**; delete Polls and Show and tell. *(my call)*

### 10. README

Add a short Community section: contributing → CONTRIBUTING.md, security →
SECURITY.md, questions → Discussions.

## Publish checklist (gated parts)

- `validations.required` starts enforcing (public-only while private —
  until then required fields are advisory).
- `contact_links` URLs to Discussions/security policy go live.
- Post a throwaway test issue via each form, check labels applied, close it.
- Ruleset behaviour: one direct push as owner (bypass) and one PR without CI
  (blocked) as a smoke test.
- Community profile: standards checklist turns green.

## Non-goals

No organization, issue types, Projects board, Code of Conduct file, bots
(stale/labeler/auto-reply), third-party CI actions, release/deploy workflow —
the bundler/release pipeline is a separate M5 item.

## What's left to the builder

Exact label colors; form wording; workflow YAML details (checkout/setup-node
current majors via context7; apt set confirmed against the runner image);
ruleset JSON incl. bypass actor resolution; CONTRIBUTING.md length (≤200
lines); commit/PR splitting of the work.
