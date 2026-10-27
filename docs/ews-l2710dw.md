# Web panel (EWS) — MFC-L2710DW facts

Device: `http://192.168.187.36/`. Checked 27.10.2026 with GET-only probes
(two parallel sessions, ~35 requests, 5 s timeouts, no login POSTs, no scan
jobs). Marking: **observed** / **inferred** / **unknown** per the REA-adapted
discipline. Feeds PRD F5/R1, R7 and §13.10, §13.11.

## Panel structure (observed)

- HTTP server: `debut/1.30`; root `GET /` → **301** → `/general/status.html` (200).
- URL scheme: `/<category>/<page>.html` — the unauthenticated index links only
  `/general/…` pages. The `/EWS/…` prefix from the iPrint&Scan resources does
  not appear in the index; `/EWS/<bogus>` 404s cleanly, so the prefix exists
  but its namespace is **unknown** on this firmware.
- The index is locale-mixed (English chrome, German status text) — follows the
  device's web-language setting.

## Auth model (observed)

Everything outside `/general/` is auth-gated: `302 →
/etc/loginerror.html?url=<urlencoded path>` → 200 page "Please Login" with a
hidden `loginurl` field. The gate fires **before the existence check** —
bogus and real paths under gated prefixes get byte-identical redirects.
Therefore unauthenticated probing cannot confirm or deny the existence of
admin/scan pages: they are **unknown** until someone logs in (login is a
POST — not done in this check).

## Verified public links (200, no auth)

| URL | Page | Evidence |
|---|---|---|
| `/general/status.html` | Status — device state, toner level | title + toner table |
| `/general/information.html?kind=item` | Maintenance information | h2 "Maintenance Information" |
| `/general/sleep.html` | Sleep time setting | h2 "Sleep Time" |
| `/general/powerdown.html` | Auto power-off setting | in index nav |
| `/general/modetimer.html` | Mode timer setting | in index nav |
| `/general/volume.html` | Volume setting | in index nav |
| `/general/panel.html` | Panel lock setting | in index nav |
| `/general/replacetoner.html` | Replace toner guide | in index nav |
| `/general/lists.html` | Reports/lists | in index nav |
| `/general/find.html` | Find device | in index nav |
| `/general/contact.html` | Contact & location | in index nav |

(Entries marked "in index nav" are **observed** as links on the live index;
only the first three were additionally fetched and content-checked.)

## Auth-gated candidates (existence unknown)

From the iPrint&Scan resources — all land on the login gate, existence
unproven: `/admin/date.html`, `/admin/firmwareupdate.html`,
`/net/wired/link_tcpip_detail.html`, `/net/wireless/link_tcpip_detail.html`,
`/scan/scandestination.html`, `/scan/scantopc.html`. A dedicated gated
`/scan/` subtree exists in both styles (`/scan/…`, `/EWS/scan/…`) —
**inferred**: an authenticated Scan section exists, page names unknown.

The `loginurl` return mechanism means a deep link into a gated area still
lands the user on the target page after they log in by hand — usable UX for
prindeck, provided the path actually exists (**inferred**, not verified).

## eSCL interplay (observed 27.10.2026)

`GET /eSCL/ScannerCapabilities` → 200 while the panel was being mapped; the
XML declares `xmlns:dest` but contains **no `dest:` elements** — no eSCL scan
destinations are currently registered on the device. Relevant to PRD §13.11
(Scan-to-PC): the namespace exists, so destination registration may be
supported; unverified.

## Not done here

Login POST (needs explicit user confirmation), any form submission, any
`/eSCL/` write endpoint.
