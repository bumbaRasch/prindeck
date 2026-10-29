# M5 finish — first public release — Design

- Status: revised draft for approval (builder check done 29.10.2026: six
  questions — one resolved by inspection, five answered by the author /
  settled as maintainer calls, see Decisions).
- Phase: M5 completion, on top of the neighboring session's work (commits
  `49a5bc2` autostart, `11ced02` bundle metadata + built deb/AppImage +
  `docs/m5-release.md` runbook — both local, unpushed).
- Sources: tauri.app via context7 29.10.2026 (Linux bundling targets and
  `bundle.linux.*` metadata; `tauri build --bundles <list>`); repo state
  verified 29.10.2026 evening: working tree clean except this spec,
  `origin/main` two commits behind local `main`, deb (5.0 MiB) and AppImage
  (86.7 MiB) built and verified per `docs/m5-release.md`, rpm deliberately
  absent (`rpmbuild` not installed, interactive sudo), rulesets API 403 on
  the private plan (creation deferred to publication per the
  github-community plan).

## Intent

Ship the M5 roadmap row — "repository published, install from the artifact"
— with the smallest honest machinery: push the neighboring session's work,
add rpm to the artifact set, finish the README, pass the live acceptance
gate, then flip the repo public and publish the v0.1.0 GitHub Release from
local builds.

## Decisions

1. **Local build release path** (author, 29.10.2026): maintainer builds on
   Ubuntu 26.04, `gh release create/upload`; CI stays checks-only; no
   third-party Actions (the repo's own rule — also why `tauri-action` is
   out). CI release automation is an explicit v2 item.
2. **Artifacts: deb + rpm + AppImage** (author): rpm joins the set — the
   author runs the one-time `sudo apt install rpm`; the target list, build
   and runbook update follow. If the system package turns out unavailable,
   v0.1.0 ships deb+AppImage and rpm moves to v0.1.1 — recorded, not
   silent.
3. **Verification bar** (author): deb — installed and launched (already
   done in the neighboring session); AppImage — launched once on the build
   machine (it was content-inspected only; FUSE 2 / `libfuse2t64` noted in
   the runbook); rpm — package inspection only in v1 (a Fedora container
   install test is out of proportion; the README states the distro honesty
   line).
4. **Autostart stays a v1 feature** (author): the neighboring commit is
   accepted; the acceptance gate gains the no-device check — toggle on →
   `~/.config/autostart` entry appears, toggle off → removed, launch at
   login spot-checked once.
5. **The formal visual check happens before publication** *(my call, from
   the builder check)*: both themes × 640×480 + maximized, via the
   `visual-check` subagent with the user fronting the window — deferred
   since M4, publication is the last honest moment for it.
6. **Release notes mention the open glib advisory** *(my call)*: one line —
   open, upstream-blocked (tauri#16264), vulnerable code path unused;
   Dependabot will show it publicly anyway.
7. **Publication is gated on the live acceptance session** (user + device):
   ten locales eyes-on, tray menu clicks, offline banner + grey tray, tray
   working/error on a real scan, duplex flip, power-cycle probe, autostart
   toggle (4), visual check (5).
8. **Version v0.1.0** (all three version files already agree).

## Approaches

- **A — local builds + `gh release` (chosen)**: zero new infrastructure; the
  acceptance criterion is verified on the machine that builds; matches a
  single-maintainer Linux-only project. Known cost: AppImage built on a 2026
  glibc won't run on old distros — documented honestly, engineered around
  only if a real user ever needs it (then: CI on an older base, the v2 item).
- **B — tag-triggered CI release on official actions only**: reproducible,
  but permanent YAML and runner-built artifacts nobody smoke-installs.
  Rejected for v1.
- **C — `tauri-action`**: third-party, against the repo's rule. Rejected.

## The parts

### 1. Land the neighboring session

Push `origin/main` (both commits — CI must go green on them). Update PRD
§10/§12 where they say autostart is "deferred past v1" and the M5 row to
name what actually ships.

### 2. rpm

Author installs the system package (`sudo apt install rpm`); add `rpm` to
`bundle.targets`; `tauri build --bundles deb,rpm,appimage`; inspect the
package (`.desktop`, icons, deps, descriptions); extend
`docs/m5-release.md` with the artifact row and drop the "why no rpm"
paragraph.

### 3. AppImage launch check

Run the built AppImage once on the build machine (FUSE 2 present?), close
it, record in the runbook.

### 4. README

Fix the stale Status/Language claims ("interface is Russian for now" — ten
languages now); add the install section (one command per artifact), the
supported-distros honesty line (built on Ubuntu 26.04; current
Debian/Ubuntu/Fedora; older bases — build from source via CONTRIBUTING);
verify the non-affiliation disclaimer; keep the community section.

### 5. Release notes template

In `docs/m5-release.md`: a v0.1.0 notes skeleton — what it is, what it
does, the three artifacts with install commands, supported-distros line,
the glib advisory line (decision 6), known limitations (paper-tray class
of UX gaps — from the parked list), link to CONTRIBUTING translations.

### 6. Live acceptance + visual check (user session, before publication)

The gate from decision 7; checklist lives in the plan. The visual check
runs via the `visual-check` subagent (user fronts the window); the device
items run with the printer.

### 7. Publication day (maintainer session, from the github-community plan)

`gh release create v0.1.0` with notes + artifacts (repo still private);
then flip public; create the `protect-main` ruleset (deferred by 403);
verify `validations.required` enforcing; `contact_links` live; one
throwaway test issue per form; external-PR smoke (ruleset blocks no-CI);
community profile green; Discussions welcome post.

## Constraints

- Zero third-party Actions. No new npm/Rust project dependencies (the rpm
  toolchain is a system package, documented in the runbook).
- English docs. Code untouched except `tauri.conf.json` targets.
- Eight-command ritual green for every commit; CI green before publication.

## Non-goals

- CI release automation, old-base AppImage builds (v2).
- Updater channel / release signatures (nothing consumes them).
- Flatpak/Snap; macOS/Windows artifacts; translated README.

## What is left to the builder

- Exact README wording; the release-notes text under the skeleton headings.
- Whether the deb/AppImage checks live in the runbook or the acceptance
  checklist (both defensible).
- RPM metadata beyond defaults if inspection shows gaps.

## Execution

Subagent-driven (M4.7 precedent): fresh implementer + fresh reviewer per
task, whole-branch review, merge to main. Subagents use the context7 MCP
for Tauri/CLI facts (AGENTS rule). Parts 6–7 are user/maintainer sessions,
not subagent tasks; the plan splits accordingly.
