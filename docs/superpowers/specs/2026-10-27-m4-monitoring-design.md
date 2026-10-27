# M4: Monitoring — design (2026-10-27)

**Status:** draft for builder check (check done 27.10.2026 — 6 findings, all
resolved in this revision, none user-blocking). Predecessor: PRD §4 F4 +
§4 F5 (R5–R7), §8, §9, §13.10/13.11; user decision 27.10.2026: **release
(M5) happens after M4**; scan polish (R1–R4) follows in v1.x.
**Sequencing precondition:** the M4 plan builds on `main` **after** the
M2.5 phase merges back (user-approved stop-and-ask point).

## Intent and why

Close the v1 promise «one window: status + toner + scan + print + history»
(PRD §1). After M2.5 the app scans (incl. ADF), prints, records history
silently — but does not yet *inform* (no notifications, no tray affordances,
no visible history) and does not speak human (raw counters instead of «drum
end soon»). M4 adds the monitoring layer and the «Настройки» destination the
PRD already promises (§8). The reference findings R5–R7 land here by design.

## Goals

1. **Notifications** (tauri-plugin-notification, already in the approved
   §10 list): toner < threshold (default 15 %), drum < threshold (default
   10 %), offline → online transition, device-silent (SNMP poll failure
   streak — the only verified «device error» signal; `hrDeviceStatus` error
   semantics are §13-unknown until a real error is observed on the wire).
   Edge-triggered: notify once per transition, re-arm when the value recovers
   above its threshold (no hysteresis band in v1 — recovery itself is the
   re-arm). At most one notification per event (PRD F4). The PRD §9 `alert`
   event stays: in-app banners while the window is open; system
   notifications cover the tray/closed-window case.
2. **Tray**: status-colored icon + menu «Показать окно», «Скан», «Открыть
   панель», «Выход» (PRD F4; on Linux the menu is mandatory for the icon —
   already wired in M1, extend it). «Скан» raises the window on the Scan
   screen; «Открыть панель» opens the verified public status page.
3. **History chart** on «Обзор»: 90-day toner-% and page-counter lines from
   the existing SQLite history (recording has shipped in M2.5 Task 1).
   Hand-rolled SVG, no chart crate.
4. **R5 — human status texts** in «Обзор» from the already-parsed blobs:
   jam counter (`#a1`, fetched by adding the fourth varbind to the existing
   batched GET) → «замятия: N — проверьте тракт бумаги»; drum pages
   (`#82`, remaining-vs-used arithmetic per `docs/oids-l2710dw.md` — the
   doc is the authority) → «барабан: осталось ≈N стр.»; wording kept in
   `labels.ts` (Russian UI strings).
5. **R6 — toner pages-left estimate** next to the %: `yieldPages − pages
   printed since cartridge change` is NOT computable from the life counter;
   honest v1 formula: show `yieldPages` from settings + current % →
   approximate remaining pages = `yieldPages × %/100`. Source of yield: new
   config key `toner.yieldPages` (+ optional `model` string), empty default
   → the estimate is hidden with a hint «укажите ресурс картриджа в
   настройках» (no dead ends, PRD UI principles). No hardcoding of cartridge
   facts.
6. **R7 — firmware notice/link**: menu item in «Обзор» + settings link →
   `http://<host>/admin/firmwareupdate.html` (verified live, behind the
   panel login; `docs/ews-l2710dw.md`). No version-compare logic in v1.
7. **Live-apply settings** (builder-check finding: no reload path exists
   today — `AppState.config` is immutable, the poller reads the interval
   once, the eSCL client is built once): config moves behind a lock; the
   poller re-reads its interval per tick; a host change rebuilds the
   `EsclClient` and resets the caps cache; a scan-dir change updates the
   asset-protocol scope (runtime `allow_directory` pattern already shipped
   in M2.5 Task 7). A first-class task of the M4 plan, not an afterthought.
8. **«Настройки» destination** (§8): device host, pollSeconds, notification
   thresholds (toner/drum %), notifications on/off, scan folder, toner
   cartridge yield (+ model), EWS deep-link group (R1: status, maintenance
   info, sleep, power-off, replace-toner + authenticated date/time, SNTP,
   firmware — every link verified in `docs/ews-l2710dw.md`).

## Non-goals

- No firmware flashing or update checking (no trustworthy version source —
  §13 R7 note).
- No multi-device, no cloud, no separate «История» screen (chart lives in
  «Обзор»; split later if it outgrows).
- No new **unverified** SNMP OIDs. The verified `…5.5.21.0` (`#a1` jams)
  joins the existing batched GET as a fourth varbind in the same single PDU
  (idle cost unchanged); R5/R6 otherwise use what `docs/oids-l2710dw.md`
  already verifies.

## Anti-goals (failure even if it works)

- Idle CPU burn: polling stays one batched SNMP GET per pollSeconds; chart
  reads the local DB only.
- Notification spam: any duplicate notification for the same stuck state.
- A dead settings screen: every field must persist, validate, and take
  effect without app restart (the live-apply path is built in M4 — see
  Goal 7; nothing reloadable exists today).

## Constraints

- Crates: only `tauri-plugin-notification` joins (PRD §10 approved). Chart,
  badges — vanilla DOM/SVG per §13.7 ruling.
- Files < 200 lines, domain-named modules (`notify/`, not `utils`);
  Rust 2024, no code comments; TS strict; Russian UI strings in
  `labels.ts`/`*_LABEL` only; responsive 640×480, both themes (AGENTS.md).
- Config evolution follows the established serde-default pattern
  (`cups_queue` from M2.5): new keys have defaults, old configs load clean;
  validation errors surface through the existing `ConfigError` path.
- Notifications must fire with the window closed (backend-side, in the
  status poller), never UI-side.

## Approaches considered

| Fork | Options | Decision |
|---|---|---|
| Chart | hand-rolled SVG in «Обзор» / chart crate / separate screen | **SVG in «Обзор»** — zero deps, sidebar stays lean; a crate for one polyline is bloat |
| R6 source | config key / SNMP model probe / hardcode TN-2420 | **config key** — no verified OID for the cartridge model; probe is v2 spike material; hardcoding invents device facts |
| R5 texts | map existing blobs / new SNMP queries | **map existing blobs** (`#a1` jams, `#82` drum pages — already parsed in M2) |
| Notify wiring | backend poller / UI listener | **backend poller** — must work from the tray with the window closed |
| Settings storage | extend `config/` store / separate settings DB | **extend config store** — one config surface, existing validation |

## Parts decided (my calls, veto during review)

- Re-arm semantics: recovery above threshold re-arms; no hysteresis band.
- Tray icon states: idle (grey/neutral), working (accent) — «working» means
  a prindeck-initiated scan or print is in progress (the only verifiable
  signal; device-busy is not), error (red), offline (dimmed) — same palette
  as the sidebar dot.
- Chart downsampling client-side to ≤90 points; empty state text «данных
  пока нет — копим» when history has <2 snapshots.
- `get_config`/`set_config` IPC commands added; `set_config` validates and
  applies live (poller interval changes take effect on the next tick).
- The live acceptance for notifications uses a temporarily high threshold
  (real toner is ~40 %) — simulates the crossing without draining toner.

## Left to the builder

Exact module layout, tests (offline: mapping tables, config defaults,
notify-transition state machine; live checklist per §14 pattern), commit
splitting with the `M4:` prefix. The plan comes from
superpowers:writing-plans after this design is approved.

## Not yet decided by the user (non-blocking)

- The installed cartridge model/yield — entered by the user in settings
  after release; if they know it now, it can become the config example value.
