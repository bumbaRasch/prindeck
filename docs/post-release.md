# Post-release task list (v0.1.x)

Owner: the maintainer. One task per entry; each lands as its own milestone
commit series with the full eight-command ritual.

## 1. Offline-state UX flaws (from the 2026-10-29 pre-release visual pass)

One pass: code fixes, artifact rebuild, repeat visual check — including the
formal 640×480 + maximized, both themes, check deferred since M4.

1. **Scan screen offline**: the four option selects render empty and two red
   banners stack when the device is unreachable — show cached capabilities
   where available and one explaining banner instead.
2. **Overview offline presentation**: consumables/counters collapse to dashes
   with no hint that data will return — keep the last-known values with a
   timestamp, matching the shell's cached-first-paint behaviour.
3. **Print screen status chip**: shows "the print queue does not exist" on a
   machine where the queue exists — verify the queue detection wording and
   state.

Verification: rebuild artifacts, reinstall the deb, visual-check pass at
640×480 and maximized in both themes, then attach the rebuilt packages to a
`v0.1.1` release.

## 2. rpm artifact

`sudo apt install rpm` (one-time), add `rpm` to `bundle.targets`, rebuild,
inspect the package, update `docs/m5-release.md` and the README install
section (already mentions rpm).

## 3. Screenshots refresh

Done: `docs/screenshots/{scan,print,settings}.png` are in the README, shot
with the device online, `PRINDECK_LOCALE=en`, dark theme, maximized window and
the `192.0.2.10` placeholder address. The Overview screen is described in the
README bullets only — no screenshot for it, by decision on 2026-10-30.

## 4. Live acceptance leftovers (device session)

Tray menu clicks, tray working/error states on a real scan, duplex print
flip, power-cycle probe, autostart toggle on/off (`~/.config/autostart`
entry appears/removes).

## 5. Translation review

Native review of the ar/ja/zh community drafts; maintainer pass over uk
("Колірний" spelling) and the ru `scan.filesReady` 21-file plural form.

## 6. Revisit the dismissed glib alert

Dependabot alert 1 (`GHSA-wrw7-89jp-8q8g`, `glib` < 0.20.0) was dismissed as
`not_used` on 2026-10-30. `glib 0.18.5` arrives only through Tauri's GTK3
stack (`gtk = "0.18"`), the patched 0.20 needs `gtk 0.19`, and no crate in the
tree constructs or calls `glib::VariantStrIter`. Once Tauri adopts gtk 0.19 /
glib ≥ 0.20, bump the lockfile, rebuild the artifacts and re-check the alert
is gone instead of dismissed.
