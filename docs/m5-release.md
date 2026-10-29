# M5 release packaging

Linux release artifacts for prindeck 0.1.0, built and verified on 29.10.2026
(Tauri CLI 2.12.4, `tauri` crate 2.12.5).

## Build

```sh
export PATH="$HOME/.cargo/bin:$PATH"
npm run tauri build -- --bundles deb,appimage
```

Artifacts:

| File | Size |
|---|---|
| `target/release/bundle/deb/prindeck_0.1.0_amd64.deb` | 5.0 MiB |
| `target/release/bundle/appimage/prindeck_0.1.0_amd64.AppImage` | 86.7 MiB |

Install the deb:

```sh
sudo apt install ./target/release/bundle/deb/prindeck_0.1.0_amd64.deb
```

Run the AppImage directly; it needs FUSE 2 (`libfuse2t64` on Ubuntu 24.04+).

## Why the target list is `deb` and `appimage`

An `.rpm` is produced by `rpmbuild`, which is not installed here and cannot be
installed without an interactive `sudo` password. `targets: "all"` on Linux
therefore advertised a format a plain `tauri build` cannot deliver on this
machine, so the committed list names only the formats that are built and
verified. Adding `rpm` to the list is a one-line change once `rpm-build`
(Fedora) or `rpm` (Debian/Ubuntu) is present.

## Bundle metadata

Field names verified against the Tauri 2 configuration reference
(`https://tauri.app/reference/config`, BundleConfig and DebConfig) and the
schema shipped with the pinned CLI (`node_modules/@tauri-apps/cli/config.schema.json`):
`category`, `publisher`, `homepage`, `copyright`, `license`, `licenseFile`,
`shortDescription`, `longDescription`, `targets`, plus
`linux.deb.section` and `linux.deb.files`.

The Tauri deb bundler reads `shortDescription`/`longDescription` for the
control file and the desktop `Comment`, but it does not copy `licenseFile` into
a deb — that field is consumed by the rpm bundler. The MIT text is therefore
shipped at `/usr/share/doc/prindeck/copyright` through
`bundle.linux.deb.files`, which keeps the package policy-clean.

## Verified deb contents

`dpkg-deb -I prindeck_0.1.0_amd64.deb`:

```
Package: prindeck
Version: 0.1.0
Architecture: amd64
Installed-Size: 11932
Maintainer: Dimitri Pisarev
Section: utils
Priority: optional
Homepage: https://github.com/bumbaRasch/prindeck
Depends: libayatana-appindicator3-1, libwebkit2gtk-4.1-0, libgtk-3-0
Description: Unofficial Linux control panel for the Brother MFC-L2710DW
 prindeck is an unofficial desktop control panel for the Brother MFC-L2710DW on Linux. ...
```

`dpkg-deb -c` lists `usr/bin/prindeck` (executable, 12 MB),
`usr/share/applications/prindeck.desktop`, the hicolor icons
`32x32`, `128x128`, `256x256@2` and `usr/share/doc/prindeck/copyright`.

The desktop entry inside the package:

```
[Desktop Entry]
Categories=Utility;
Comment=Unofficial Linux control panel for the Brother MFC-L2710DW
Exec=prindeck
StartupWMClass=prindeck
Icon=prindeck
Name=prindeck
Terminal=false
Type=Application
```

`Icon=prindeck` resolves to `usr/share/icons/hicolor/*/apps/prindeck.png`, all
present in the package. `Exec=prindeck` resolves through `PATH`, where
`/usr/bin` is always present after installation.

Known upstream quirk: the bundler names the 256 px directory
`hicolor/256x256@2` (from the `128x128@2x.png` file name) instead of
`hicolor/256x256`; the 32 and 128 px icons cover the lookup, so the launcher
icon renders.

## AppImage

The AppImage is executable (`-rwxr-xr-x`) and contains the same binary at
`usr/bin/prindeck`, the same desktop entry and the same hicolor icons.

## Not covered here

Nothing in this milestone touches the printer: no scan, print or SNMP request
was made. A publishable release still needs a git tag and the artifacts
uploaded to a GitHub release, and an `.rpm` needs `rpmbuild`.
