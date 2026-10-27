# SNMP on Brother MFC-L2710DW: OID spike (M0)

Captured 26.10.2026 from a live device. Script: `scripts/probe-snmp.mjs`
(SNMPv2c GET, own BER codec, `node:dgram`, no dependencies).
Raw output: `fixtures/snmp/probe-2026-10-26.json`.

## 0. Poll parameters

| Parameter | Value |
|---|---|
| Protocol | SNMPv2c (`version = 1` in BER) |
| Community | `public` |
| Transport | UDP, port `161`, `udp4` |
| Timeout | `2000` ms per attempt |
| Retries | `3` (new request-id and new socket on every attempt) |
| Request | one `GET` per OID (not bulk, not walk) |
| Address | `192.168.187.36` |
| Run time | `2026-10-26T20:41:22Z` – `2026-10-26T20:41:25Z` |
| Web check | `2026-10-26T21:42+01:00` (≈ a minute later, values did not change) |

All OIDs answered on the first attempt; RTT from the fixture — 95–209 ms. The default
community `public` works; there is no authentication.

To read values from the web panel for the check, use the page
`http://192.168.187.36/general/information.html?kind=item` (no login):
it has `Page Counter`, `Average Coverage`, `Drum Unit %`, `Toner %`,
`Replace Count`, `Total Paper Jams`.

## 1. Standard Printer-MIB (RFC 3805)

| OID | Time (UTC) | Raw | Type | Decoding | Web check |
|---|---|---|---|---|---|
| `1.3.6.1.2.1.43.11.1.1.6.1.1` | 20:41:22.295 | `426c61636b20546f6e657220436172747269646765` | OctetString | `"Black Toner Cartridge"` | — |
| `1.3.6.1.2.1.43.11.1.1.6.1.2` | 20:41:22.437 | `4472756d20556e6974` | OctetString | `"Drum Unit"` | — |
| `1.3.6.1.2.1.43.11.1.1.6.1.3` | 20:41:22.636 | — | noSuchInstance | there is no consumable #3 | — |
| `1.3.6.1.2.1.43.11.1.1.8.1.1` | 20:41:22.841 | `-2` | Integer | toner max capacity: **unknown** | — |
| `1.3.6.1.2.1.43.11.1.1.8.1.2` | 20:41:23.045 | `12000` | Integer | drum max capacity: 12 000 pages | — |
| `1.3.6.1.2.1.43.11.1.1.9.1.1` | 20:41:23.255 | `-3` | Integer | toner level: **some remaining** (no number) | web: `Toner 40%` — **not derivable from MIB** |
| `1.3.6.1.2.1.43.11.1.1.9.1.2` | 20:41:23.460 | `4247` | Integer | drum remaining: 4 247 pages | web `Drum 36%` — matches the blob `#41`=3600, not the ratio 4247/12000 = 35.39 % (see §4) |
| `1.3.6.1.2.1.43.11.1.1.9.1.3` | 20:41:23.665 | — | noSuchInstance | there is no third consumable | — |
| `1.3.6.1.2.1.43.10.2.1.4.1.1` | 20:41:23.865 | `1e49` (2 bytes!) | Counter32 | **7753** — printed page counter | web: `Page Counter 7753` ✓ |
| `1.3.6.1.2.1.25.3.2.1.5.1` | 20:41:24.074 | `2` | Integer | hrDeviceStatus = `running` | the device was asleep at that time (`Energiesparen`) — see §5 |

Key findings on the standard MIB:

- **Toner as a percentage is not available via Printer-MIB.** `prtMarkerSuppliesLevel.1.1 = -3`
  («some remaining»), `prtMarkerSuppliesMaxCapacity.1.1 = -2` («unknown»). Therefore the
  Brother blob is a mandatory source, not an "enrichment" (accounted for in PRD §7.2).
- **The drum is available in pages**, not in percentages: `4247` of `12000`.
- The page counter is `Counter32`, but the firmware encodes it in **two bytes**
  (`1e 49`) instead of four, which formally violates BER. The decoder must read
  big-endian by the actual value length, otherwise it crashes (the first version of the script
  crashed with `Attempt to access memory outside buffer bounds`).
- There are exactly two consumables: toner (`1`) and drum (`2`); `.3`/`.4` — `noSuchInstance`.

## 2. Brother enterprise OIDs

All nodes are scalar-shaped and answer **only with instance `.0`**:
without `.0` you get `noSuchInstance` (verified for `...5.5.8`, `...5.5.10`,
`...5.5.11`, `...5.5.20`, `...5.5.21`). The value is an OctetString blob.

| OID | Time (UTC) | Raw hex | Length | Decoding | Web check |
|---|---|---|---|---|---|
| `1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.8.0` (brInfoMaintenance) | 20:41:24.254 | `6301040000000141010400000e1011010400001e49310104000000076f010400000d4881010400000028ff` | 43 B | 6 records, see §3.1 | `#41`=3600 → `Drum 36%` ✓, `#81`=40 → `Toner 40%`, `#11`=7753 → `Page Counter` |
| `1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.10.0` (brInfoCounter) | 20:41:24.384 | `00010400001e4906010400000f6fff` | 15 B | `#00`=7753, `#06`=3951 | `Page Counter 7753` ✓, `2-sided Print Total 3951` ✓ |
| `1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.11.0` (brInfoNextCare) | 20:41:24.479 | `82010400001097ff` | 8 B | `#82`=4247 — drum remaining in pages | `Drum 36%` (12000−7753=4247) ✓ |
| `1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.18.0` (brInfoCoverage) | 20:41:24.684 | `00020627ff` | 5 B | `6` / `39` → 6.39 % | `Average Coverage 6.39%` ✓ |
| `1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.20.0` (brInfoReplaceCount) | 20:41:24.893 | `820100a00104ff` | 7 B | `#82`=0 (drum), `#a0`=4 (toner) | `Replace Count Toner 4 / Drum 0` ✓ |
| `1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.21.0` (brInfoJamCount) | 20:41:25.099 | `000200000102000221020002220200022302000225020006a10400000008ff` | 31 B | see §3.4 | `Jams 8; Tray1 2; Inside 2; Rear 2; 2-sided 2; ADF 6` ✓ |

M2 note: the application fetches the three runtime nodes — `…5.5.8.0`
(brInfoMaintenance), `…5.5.10.0` (brInfoCounter), `…5.5.11.0` (brInfoNextCare) —
in **one multi-varbind GET** (three varbinds, one PDU), not one request per OID
as in the M0 survey above; the live verification of that shape is in §6.

## 3. Blob decoding

Record format as observed: `<id:1 byte> <width:1 byte> <value:width bytes, big-endian>`,
records follow one after another, terminator is `0xFF`. **The grammar is not uniform between
nodes** (see below), so below is the layout for each node separately.

### 3.1 brInfoMaintenance (`.5.5.8.0`)

```
63 01 04 00 00 00 01   → #63 = 1
41 01 04 00 00 0e 10   → #41 = 3600  → drum remaining 36.00 %   [web: Drum 36%]
11 01 04 00 00 1e 49   → #11 = 7753  → page counter             [web: Page Counter 7753]
31 01 04 00 00 00 07   → #31 = 7
6f 01 04 00 00 0d 48   → #6f = 3400
81 01 04 00 00 00 28   → #81 = 40    → toner remaining 40 %     [web: Toner 40%]
ff                     → end
```

For records of this node the second byte is `01`, the third `04` (value width 4 bytes) —
i.e. effectively `<id> 01 04 <uint32 BE>`.

### 3.2 brInfoCounter (`.5.5.10.0`)

```
00 01 04 00 00 1e 49   → #00 = 7753  → page counter             [web: Page Counter 7753]
06 01 04 00 00 0f 6f   → #06 = 3951  → duplex pages             [web: 2-sided Print Total 3951]
ff
```

### 3.3 brInfoNextCare (`.5.5.11.0`) and brInfoCoverage (`.5.5.18.0`)

```
82 01 04 00 00 10 97   → #82 = 4247  → drum remaining in pages (= 12000 − 7753)
ff
```

```
00 02 06 27            → id 00, width 02, value `06 27`
ff
```

Coverage is encoded not as a number but as two bytes: `0x06` = integer part (`6`),
`0x27` = `39` in decimal form → **6.39 %**. Matches the web.

### 3.4 brInfoReplaceCount (`.5.5.20.0`) and brInfoJamCount (`.5.5.21.0`)

```
82 01 00               → #82 = 0  → drum replacements
a0 01 04               → #a0 = 4  → toner replacements          [web: Toner 4 / Drum 0]
ff
```

```
00 02 0000             → #00 = 0
01 02 0002             → #01 = 2  → Jam Tray 1                  [web: 2]
21 02 0002             → #21 = 2  → Jam Inside                  [web: 2]
22 02 0002             → #22 = 2  → Jam Rear                    [web: 2]
23 02 0002             → #23 = 2  → Jam 2-sided                 [web: 2]
25 02 0006             → #25 = 6  → Jam ADF                     [web: 6]
a1 04 00000008         → #a1 = 8  → total jams                  [web: Total Paper Jams 8]
ff
```

Here most records have width `02` (2 bytes), the last one has `04`.

### 3.5 Grammar summary

- The nodes `brInfoMaintenance`, `brInfoCounter`, `brInfoNextCare` — records
  of fixed form `<id> 01 04 <uint32 BE>`, terminator `FF`.
- `brInfoCoverage` — `<id> 02 <2 bytes>`, where the first byte is the integer part, the second —
  the fractional part in decimal form (not BCD, not /100).
- `brInfoReplaceCount` — `<id> 01 <1 byte>`.
- `brInfoJamCount` — a mix of `<id> 02 <2 bytes>` and `<id> 04 <4 bytes>`.

A single "universal" grammar has not been derived and is **not verified**; for M2
it is recommended to parse only those nodes whose fields have been checked against the web (`#81`, `#41`,
`#00`, `#06`, `#82`, `#a1`), and to silently skip unfamiliar records — this
matches the requirement of PRD §5.6 (unknown → ignore + log).

## 4. Check against the web panel (summary)

| Metric | SNMP | Web (`information.html?kind=item`) | Match |
|---|---|---|---|
| Page counter | `43.10.2.1.4.1.1` = 7753; Brother `#00` = 7753 | `Page Counter 7753` | ✓ |
| Toner, % | `brInfoMaintenance #81` = 40 | `Toner 40%` | ✓ |
| Drum, % | `brInfoMaintenance #41` = 3600 → 36.00 % | `Drum Unit 36%` | ✓ |
| Drum, pages | `43.11.1.1.9.1.2` = 4247; `#82` = 4247; max 12000 | — (percentages only) | ✓ among themselves |
| Coverage | `brInfoCoverage` = 6.39 % | `Average Coverage 6.39%` | ✓ |
| Duplex | `brInfoCounter #06` = 3951 | `2-sided Print Total 3951` | ✓ |
| Toner/drum replacements | `#a0`=4 / `#82`=0 | `Toner 4 / Drum Unit 0` | ✓ |
| Jams | `#a1`=8, `#01`/`#21`/`#22`/`#23`=2, `#25`=6 | `8 / 2 / 2 / 2 / 2 / 6` | ✓ |
| Toner level via MIB | `-3` («some remaining») | `40%` | ✗ (MIB gives no number) |

Drum discrepancy: `4247 / 12000 = 35.39 %`, while the web shows `36%` and the blob
returns `3600` (36.00 %). That is, the blob computes the percentage on its own scale, not
as the level/capacity ratio. For the UI take `#41`, do not divide.

## 5. What remains unverified

- The semantics of `#63` (=1), `#31` (=7), `#6f` (=3400) — **unknown**. According to
  `github.com/saper-2/BRN-Printer-sCounters-Info` (MFC-L2720DW, the same series)
  `#63` — drum status, `#31` — ?, `#6f` — «remaining life: toner in 0.01%».
  In our case `#6f` = 34.00 %, which **does not match** the 40 % from `#81` and the web, so
  `#6f` cannot be treated as the toner percentage. Marked as unknown.
- `hrDeviceStatus = 2` (`running`) while the device is asleep (`Energiesparen` in the web).
  Values during printing/scanning/error were not captured — the mapping to `DeviceStatus`
  for M2 is not confirmed.
- Stability of values over time was not checked (one run; the web check a minute later
  matched).
- SNMPv1 and communities other than `public` were not checked.
- Behaviour with an unreachable device (timeout) was checked only in the script
  code (3 × 2000 ms → error, exit code 1), not on a real powered-off
  printer.

## REPORT: what did not work and what it means for M1

1. **Toner in % is not provided via the standard Printer-MIB** (`-3`/`-2`).
   → In `snmp.rs` the Brother blob `brInfoMaintenance` is a mandatory source, not an
   "enrichment" as stated in PRD §7.2. Without it there is nothing to show in the UI except
   "low toner".
2. **Brother OIDs without `.0` return `noSuchInstance`.** → In the OID
   configuration write `.5.5.8.0`, `.5.5.10.0` etc. right away.
3. **Counter32 arrives as 2 bytes** (BER violation). The `snmp2` crate may
   crash on this or return garbage — in M2 be sure to check
   `prtMarkerLifeCount` on a live device; if `snmp2` cannot handle it, take
   the page counter from `brInfoCounter #00` (4 bytes there, all correct).
4. **The blob format is not uniform.** → Do not write a "universal" parser; parse
   per node, skip unknown `id`s with a log. Cover the reference
   hex strings from `fixtures/snmp/probe-2026-10-26.json` with tests.
5. **Drum units:** take the percentage from `#41`, pages — from
   `prtMarkerSuppliesLevel.1.2` (4247) when needed.
6. **Verification in M2 (PRD §14 acceptance: "toner ±1 %")** is possible: the web panel gives
   exact numbers on `information.html?kind=item`, no login needed. The page
   `status.html` is not suitable for the check — there the toner is drawn as a picture without a number.

## 6. Independent check with `snmpwalk` (26.10.2026)

The findings above were confirmed by a second, independent tool — net-snmp
`snmpwalk` 5.9.4 (taken from the package and run locally, not installed into the system).
Raw dumps: `fixtures/snmp/walk-printer-mib.txt`,
`fixtures/snmp/get-brother-scalars.txt`.

| Check | snmpwalk |
|---|---|
| `43.10.2.1.4.1.1` | `Counter32: 7753` ✓ |
| `43.11.1.1.6.1.1/.2` | `"Black Toner Cartridge"`, `"Drum Unit"` ✓ |
| `43.11.1.1.8.1.1/.2` | `-2` / `12000` ✓ |
| `43.11.1.1.9.1.1/.2` | `-3` / `4247` ✓ — toner in % is not provided via MIB |
| `…5.5.8.0` | 43 B; `#41`=3600, `#11`=7753, `#81`=40 ✓ |
| `…5.5.10.0` | `#00`=7753, `#06`=3951 ✓ |
| `…5.5.11.0` | `#82`=4247 ✓ |
| `…5.5.18.0` | `06 27` ✓ |
| `…5.5.20.0` / `…5.5.21.0` | `#82`=0/`#a0`=4; `#a1`=8 ✓ |

All values matched the data from `scripts/probe-snmp.mjs` and the web panel —
the M0 results are confirmed by a second tool.

Operational observations:

- A full walk of the `1.3.6.1.4.1.2435` subtree does not finish quickly: inside there are large
  tables (`…5.5.53.*` — string data), the walk breaks off on timeout.
  The working tactic is **a targeted GET/walk only over the scalar nodes `…5.5.x.0`**
  (see `get-brother-scalars.txt`); this is what is being built into `snmp.rs`.
- `snmpwalk` is a verification/reconnaissance tool. The application keeps `snmp2`
  (no CLI dependency needed at runtime).

### Live verification of the M2 multi-varbind GET (27.10.2026)

The runtime fetch — one GET PDU with three varbinds (`…5.5.8.0`, `…5.5.10.0`,
`…5.5.11.0`), the same shape as `SnmpClient::fetch` in
`src-tauri/src/snmp/client.rs` — was replayed on 27.10.2026 with the batched
mode of `scripts/probe-snmp.mjs` (community `public`, first attempt, 205 ms).
The device answered in one response with all three varbinds; every blob is
byte-for-byte identical to the M0 capture of 26.10.2026:

| Varbind | Raw hex | Decoded |
|---|---|---|
| `…5.5.8.0` (brInfoMaintenance) | `6301040000000141010400000e1011010400001e49310104000000076f010400000d4881010400000028ff` (43 B) | `#81`=40 → toner 40 %, `#41`=3600 → drum 36.00 %, `#11`=7753 pages |
| `…5.5.10.0` (brInfoCounter) | `00010400001e4906010400000f6fff` (15 B) | `#00`=7753 pages, `#06`=3951 duplex pages |
| `…5.5.11.0` (brInfoNextCare) | `82010400001097ff` (8 B) | `#82`=4247 drum pages left |

Decoded snapshot (the `DeviceStatus` fields of M2): **toner 40 %, drum 36 %,
pages total 7753, duplex 3951, drum pages left 4247**.

Drift against 26.10.2026: none — the page counter, toner and drum are unchanged
(no printouts in between), so the counters were expected to move but did not.
The multi-varbind GET returns the same data as the per-OID GETs of the M0
survey; the three-varbind shape the Rust client uses is confirmed live.
