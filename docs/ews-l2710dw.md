# Web panel (EWS) — MFC-L2710DW facts

Device: `http://192.0.2.10/` (RFC 5737 example address; use your printer's).
Checked 27.10.2026 with GET-only probes
(two parallel sessions, ~35 requests, 5 s timeouts, no login POSTs, no scan
jobs). Marking: **observed** / **inferred** / **unknown**. Feeds PRD F5/R1, R7
and §13.10, §13.11.

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

## Login (observed, 27.10.2026, user-authorized)

The login form POSTs to `/general/status.html` with a password field and the
hidden `loginurl` return path; a successful login sets an `AuthCookie`
session cookie. Credentials are **not recorded in any repository file**.

## Verified authenticated pages (200 with session)

| URL | Page | Notes |
|---|---|---|
| `/admin/date.html` | Date&Time | the reference path exists |
| `/admin/firmwareupdate.html` | Firmware Update | the reference path exists (F5/R7) |
| `/admin/password.html` | password change | in nav |
| `/admin/securitysettings.html` | Security Settings | in nav |
| `/admin/settinglock.html` | Setting Lock | in nav |
| `/admin/default.html` | factory defaults | in nav |
| `/net/net/net.html` | Network Status | in nav |
| `/net/net/sntp.html?pageid=347` | SNTP (NTP time sync) | relevant to date/time |
| `/net/wired/link_tcpip_detail.html` | wired TCP/IP details | the reference path exists |
| `/net/wireless/link_tcpip_detail.html` | wireless TCP/IP details | the reference path exists |
| `/scan/scanfrompc.html` | **Scan from PC** | only a **Pull Scan Enabled/Disabled** toggle — no destination management |
| `/address/onetouchdial.html` | One Touch Dial | fax address book |
| `/copy/copy.html`, `/fax/fax/fax.html`, `/print/print.html` | section roots | Copy/Fax/Print |

**404 (do not exist on this firmware):** `/scan/scandestination.html`,
`/scan/scantopc.html` — the iPrint&Scan resource paths are stale for this
model. The EWS does **not** offer scan-destination/network-target
registration; the only scan-related page is the Pull Scan toggle (currently
Enabled — consistent with eSCL scanning working). Relevant to PRD §13.11:
Scan-to-PC target registration happens in software (Windows driver/iPrint&Scan),
not in the panel.

## Not done here

No settings were changed: the only POST was the login itself; every page visit
was a GET. No forms submitted, no `/eSCL/` write endpoints touched.

## eSCL interplay (observed 27.10.2026)

`GET /eSCL/ScannerCapabilities` → 200 while the panel was being mapped; the
XML declares `xmlns:dest` but contains **no `dest:` elements** — no eSCL scan
destinations are currently registered on the device. Combined with the panel
finding (the EWS exposes only the Pull Scan toggle, no destination
management), the Linux-facing picture: pull scanning needs no registration —
eSCL just works while Pull Scan stays Enabled (PRD §13.11).