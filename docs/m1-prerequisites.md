# M1: Tauri 2 build environment (Ubuntu 26.04)

Checked 27.10.2026. **Status: ready** — M1 is no longer blocked
(the first revision of the document from 26.10.2026 said "M1 blocked": there was neither
Rust nor WebKitGTK development headers).

## 0. Machine

| Source | Value |
|---|---|
| `/etc/os-release` | `Ubuntu 26.04.1 LTS`, `ID=ubuntu`, `ID_LIKE=debian` |
| Package managers | `apt`/`dpkg`; `dnf`/`rpm` absent |
| `sudo` | requires an interactive password (not passwordless) |

Historical correction: in the first version of PRD §3 the distribution was listed as Fedora.
That was a mistake — corrected on 26.10.2026, the spike and development machine is Ubuntu.
The Fedora package names in §4 remain unverified (see below).

## 1. What is installed (checked 27.10.2026)

| Tool | Version / state |
|---|---|
| Node.js / npm | `26.5.0` / `11.17.0` |
| `gcc` / `g++` / `make` | ✓ (`build-essential` 12.12ubuntu2.26.04.2) |
| `pkg-config` | `2.5.1-4` |
| Rust (`rustup`, stable) | `cargo 1.99.1`, `rustc 1.99.1` (2026-10-13) |
| `libwebkit2gtk-4.1-dev` | `2.52.6-0ubuntu0.26.04.1` — **matches the runtime** `libwebkit2gtk-4.1-0` 2.52.6 |
| `libayatana-appindicator3-dev` | `0.5.94-1build1` (needed for the tray) |
| `librsvg2-dev` | `2.61.3+dfsg-3ubuntu0.1` |
| `libxdo-dev` | `1:3.20160805.1-5.1build1` |
| `libssl-dev` | `3.5.5-1ubuntu3.7` |
| `patchelf` | `0.18.0-1.4build1` |
| `pdfinfo` (poppler-utils) | ✓ — used in the M1 checklist |

Rust is installed via `rustup` (stable toolchain): the binaries are in
`~/.cargo/bin`, PATH is picked up from `~/.cargo/env` in new shells. In the current
session you need `export PATH="$HOME/.cargo/bin:$PATH"`.

## 2. Installation commands (to repeat on a clean machine)

```sh
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev build-essential pkg-config \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev patchelf

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

The Tauri template is generated separately (`create-tauri-app`, see M1 plan §2).

## 3. Readiness check

```sh
pkg-config --modversion webkit2gtk-4.1   # 2.52.6 (must match the runtime)
cargo --version && rustc --version      # 1.99.1 / 1.99.1
```

If the version of the dev package does not match the runtime library — align the packages,
do not "cure" it with build flags.

Additionally: edition 2024 in the Tauri 2 template requires `rustc >= 1.85`;
the current 1.99.1 fits. The first Rust build is long (5–15 min) — this is normal.

## 4. Fedora: not verified

This machine has no `dnf`, so the package names below are taken from
`tauri-apps/tauri-docs` (`src/content/docs/start/prerequisites.mdx`) and are **not
confirmed** by a run:

```
webkit2gtk4.1-devel openssl-devel curl wget file
libappindicator-gtk3-devel librsvg2-devel libxdo-devel
+ dnf group install "c-development"
```

Fedora is not a v1 target machine: verifying these names makes sense only
if a real Fedora machine (or container) appears.

## 5. What this means for M1/M5

1. M1 is unblocked: `cargo check`/`cargo test`/`tauri dev` build on this
   machine.
2. On Linux both `libayatana-appindicator3-dev` (tray) and a menu on the icon are mandatory:
   tray events (click) are **not emitted** on Linux — interaction is only via the
   menu.
3. M5: the bundler here will build `.deb` and AppImage; `.rpm` — only with
   `rpmbuild` present (not installed). CI/release artifacts are a separate M5 task.
