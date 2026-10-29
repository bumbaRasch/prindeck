# Application and tray icons — Design (short)

- Status: approved in conversation 28.10.2026 (pipeline + state colors).
- Phase: M5 release prep (the "icon" item of the M5 roadmap row); independent of M4.7 i18n.
- Sources: tauri.app/develop/icons via context7 (`tauri icon [INPUT]` — squared PNG/SVG,
  transparency recommended, outputs to the `icons/` dir next to `tauri.conf.json`);
  `TrayIconBuilder::icon` Linux notes; image inspection by the `visual-check` subagent
  28.10.2026; UI palette from `ui/src/styles/base.css`.

## Intent

Replace the default Tauri scaffold icons and the current tray icons with the project's
own brand images (`images/`, added by the author 28.10.2026), before publication.

## Decisions (made with the author)

1. **App icon = `images/logo.png`** as a full-bleed dark-navy tile (two-color glyph:
   white printer + teal #00D9C8 sheet/dot on #16283A). The whole set is regenerated
   with `npm run tauri icon images/logo.png`; `tauri.conf.json` keeps its current
   icon list (generated names are unchanged).
2. **Tray = glyph-only, background removed, recolored per state**, 32×32 RGBA,
   replacing `tray-{idle,working,error,offline}.png`. Base: `images/dark background.png`
   (teal glyph on #1E2126) — background becomes alpha, the knocked-out dot stays a
   hole. State colors: idle teal `#00D9C8`, working blue `#4b8ef7`, error red
   `#e5534b`, offline grey `#9aa2ad` (author-confirmed).
3. **Tooling**: one-off maintainer script `scripts/make-icons.py` using Pillow
   (already present on the machine; maintainer call 28.10.2026 — ImageMagick
   is not installed and sudo needs a password, the algorithm is identical);
   sources stay in `images/`; `monocrome.png` / `one-colour.png` remain
   unused sources (one-colour is the README/social tile candidate).
4. `include_image!` in `src-tauri/src/tray/icons.rs` is untouched — only the PNGs change.

## Non-goals

- No per-theme app icon variants (the bundle ships one icon on Linux).
- No UI palette change (brand teal stays icon-only for now).
- No animated/progress tray icon.

## Verification

`cargo` ritual green (icons compile in); app run shows the new window/taskbar icon and
the idle tray icon; offline state via an unroutable host; `working`/`error` tray icons
are a live-session item (need a scan), spot-checked from the PNGs by `visual-check`.
