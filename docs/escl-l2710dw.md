# eSCL on the Brother MFC-L2710DW: protocol spike (M0)

Captured 26.10.2026 from the live device. All values are observed, not from documentation.
Script: `scripts/escl-probe.mjs`. Raw responses: `fixtures/escl/`.

## 0. Bench

| Parameter | Value |
|---|---|
| Device | Brother MFC-L2710DW series |
| Address | `192.168.187.36`, HTTP, port 80 |
| HTTP server | `Server: debut/1.30` (Brother built-in httpd) |
| eSCL base | `http://192.168.187.36/eSCL/` |
| Protocol version | `<pwg:Version>2.63</pwg:Version>` in capabilities and status |
| Schema | `xsi:schemaLocation="http://schemas.hp.com/imaging/escl/2011/05/03 ../../schemas/eSCL-1_92.xsd"` |
| Main Firmware | `ZJ` (web panel, Maintenance Information) |
| Control | `sane-airscan` 0.99.36 (`scanimage -d 'airscan:e0:Brother MFC-L2710DW series'`) — works, used for cross-checking |

Namespaces (both visible in every response):

```
xmlns:scan="http://schemas.hp.com/imaging/escl/2011/05/03"
xmlns:pwg ="http://www.pwg.org/schemas/2010/12/sm"
```

All requests are unauthenticated, without `Authorization`, without cookies.

## 1. Endpoints (verified)

| Method | Path | Response |
|---|---|---|
| GET | `/eSCL/ScannerCapabilities` | `200`, `text/xml`, 6845 B, 0.25–0.80 s |
| GET | `/eSCL/ScannerStatus` | `200`, `text/xml`, 441 B (idle) — 7.5 kB with active scan jobs |
| POST | `/eSCL/ScanJobs` | `201 Created`, empty body, `Location: <job-URI>` |
| GET | `<job-URI>/NextDocument` | `200` + chunked document; `404` — no more documents |
| GET | `<job-URI>/ScannerJobStatus` | **`404`** (HTML from httpd) — see §5 |
| GET | `<job-URI>` | `405` (only `DELETE`) |
| DELETE | `<job-URI>` | `200` — scan job still active; `404` — already finished/removed |

Response headers for `ScannerCapabilities`:

```
HTTP/1.1 200 OK
Cache-Control: no-cache
X-Frame-Options: DENY
Content-Length: 6845
Content-Type: text/xml
Connection: close
Server: debut/1.30
Pragma: no-cache
```

Response headers for `POST /eSCL/ScanJobs`:

```
HTTP/1.1 201 Created
Cache-Control: no-cache
Accept-Encoding: identity
X-Frame-Options: DENY
Content-Length: 0
Content-Type: text/xml
Connection: close
Server: debut/1.30
Pragma: no-cache
Location: http://192.168.187.36/eSCL/ScanJobs/<uuid-v1>
```

Scan job URI pattern: `http://192.168.187.36/eSCL/ScanJobs/<uuid-v1>`. The UUID in the
`Location` header is an absolute URL; in `ScannerStatus` the same scan job appears as the relative
`<pwg:JobUri>/eSCL/ScanJobs/<uuid></pwg:JobUri>` and as `<pwg:JobUuid>urn:uuid:<uuid>`.

## 2. ScannerCapabilities — what is actually advertised

Structure (names as in the XML, with prefixes):

```
scan:ScannerCapabilities
├─ pwg:Version 2.63
├─ pwg:MakeAndModel "Brother MFC-L2710DW series"
├─ pwg:SerialNumber, scan:Manufacturer, scan:UUID
├─ scan:AdminURI, scan:IconURI, scan:Certifications
├─ scan:Platen / scan:PlatenInputCaps
├─ scan:Adf    / scan:AdfSimplexInputCaps, scan:FeederCapacity, scan:AdfOptions
└─ scan:StoredJobRequestSupport (MaxStoredjobRequests 0, TimeoutInSeconds 0)
```

### 2.1 Resolutions (`scan:SupportedResolutions/scan:DiscreteResolutions`)

`100`, `200`, `300`, `600` (X = Y), the same for Platen and ADF.
`scan:MaxOpticalXResolution`/`MaxOpticalYResolution` = `2400`.

> **150 dpi is not supported.** A request with `150` is accepted (`201`), but the scanner
> silently substitutes `300` — see §4.3.

### 2.2 ColorMode

Exact values from the XML: **`BlackAndWhite1`**, **`Grayscale8`**, **`RGB24`**.
These are not `Gray8`/`BW1` — names from the eSCL schema. Exactly one of them goes into `ScanSettings`.

### 2.3 InputSource: how it is named in the XML

In capabilities the sources are **elements**, not strings:

| Source | XML in capabilities | Value for `ScanSettings` |
|---|---|---|
| Platen | `<scan:Platen><scan:PlatenInputCaps>` | `Platen` |
| ADF | `<scan:Adf><scan:AdfSimplexInputCaps>` | `Feeder` |

In `ScanSettings` the string is `Feeder` (not `Adf`, not `ADF`) — verified, `201 Created`.
The ADF is declared simplex only: `scan:AdfDuplexInputCaps` is absent.
`<scan:FeederCapacity>20</scan:FeederCapacity>`, `<scan:AdfOption>DetectPaperLoaded</scan:AdfOption>`.

### 2.4 Document formats

Both formats are declared twice — in PWG and in extension form, for Platen and for ADF:

```xml
<scan:DocumentFormats>
  <pwg:DocumentFormat>application/pdf</pwg:DocumentFormat>
  <pwg:DocumentFormat>image/jpeg</pwg:DocumentFormat>
  <scan:DocumentFormatExt>application/pdf</scan:DocumentFormatExt>
  <scan:DocumentFormatExt>image/jpeg</scan:DocumentFormatExt>
</scan:DocumentFormats>
```

PDF is served by the device directly (see §6), JPEG — page by page.

### 2.5 Maximum area

| Source | `MaxWidth` × `MaxHeight` (units of 1/300 inch) | Physically |
|---|---|---|
| Platen | `2550` × `3507` | 215.9 × 296.9 mm (A4) |
| ADF | `2550` × `4200` | 215.9 × 355.6 mm (Legal) |

`scan:MinWidth`/`scan:MinHeight` = `64`. `scan:MaxScanRegions` = `1`.
`scan:MaxPhysicalWidth`/`Height` coincide with `MaxWidth`/`MaxHeight`.
Risky fields (fields) — all `0`.

### 2.6 Other

`scan:Intent`: `Document`, `TextAndGraphic`, `Photo`, `Preview`.
`scan:BinaryRendering`: `Halftone`, `Threshold`. `scan:ColorSpace`: `CMYK`, `YCC`, `sRGB`.

## 3. ScannerStatus

The complete Idle response (441 B):

```xml
<scan:ScannerStatus ...>
  <pwg:Version>2.63</pwg:Version>
  <pwg:State>Idle</pwg:State>
  <scan:AdfState>ScannerAdfEmpty</scan:AdfState>
</scan:ScannerStatus>
```

With an active scan job an array is added:

```xml
<scan:Jobs>
  <scan:JobInfo>
    <pwg:JobUri>/eSCL/ScanJobs/ab59e0e1-...</pwg:JobUri>
    <pwg:JobUuid>urn:uuid:ab59e0e1-...</pwg:JobUuid>
    <scan:Age>14779</scan:Age>
    <pwg:ImagesCompleted>0</pwg:ImagesCompleted>
    <pwg:ImagesToTransfer>1</pwg:ImagesToTransfer>
    <pwg:JobState>Pending</pwg:JobState>
    <pwg:JobStateReasons>
      <pwg:JobStateReason>JobScanning</pwg:JobStateReason>
    </pwg:JobStateReasons>
  </scan:JobInfo>
  ...
</scan:Jobs>
```

Observed values:

- `pwg:State` — `Idle`, `Processing`.
- `scan:AdfState` — `ScannerAdfEmpty` (we have not seen others: there is no paper in the ADF).
  *(M2.5, §11: with sheets loaded the value is `ScannerAdfLoaded` — observed 27.10.2026.)*
- `pwg:JobState` — `Pending`, `Processing`, `Completed`, `Canceled`.
- `pwg:JobStateReason` — `JobScanning`, `JobCompletedSuccessfully`,
  `JobCanceledAtDevice`, `JobTimedOut`.
- `pwg:ImagesToTransfer` = `1` both before and after the scan — this is «how many pages
  are planned», not «how many are left to fetch».
- `scan:Age` — a value that looks like milliseconds, but is **not monotonic** between
  scan jobs and does not match the real lifetime. The semantics are **unknown**; do not
  use it in logic.

The `scan:Jobs` list is a **ring buffer of 16 entries**: the oldest is evicted by the
new one (verified 27.10.2026, see §10: after three new scan jobs the oldest
entry disappeared, the count is 16 again). `Completed`/`Canceled` entries remain
visible until evicted. The parser is obliged to read all `scan:JobInfo` and look up the
needed one by `pwg:JobUri`; the entry of a canceled scan job may disappear if new ones
are created in parallel.

## 4. ScanSettings: a working request and three traps

### 4.1 The body the device actually executes

File: `fixtures/escl/scansettings-platen-300-gray-pdf.xml` (one line, 754 B):

```xml
<scan:ScanSettings xmlns:pwg="http://www.pwg.org/schemas/2010/12/sm" xmlns:scan="http://schemas.hp.com/imaging/escl/2011/05/03"><pwg:Version>2.0</pwg:Version><scan:Intent>Document</scan:Intent><pwg:ScanRegions><pwg:ScanRegion><pwg:ContentRegionUnits>escl:ThreeHundredthsOfInches</pwg:ContentRegionUnits><pwg:XOffset>0</pwg:XOffset><pwg:YOffset>0</pwg:YOffset><pwg:Width>2550</pwg:Width><pwg:Height>3507</pwg:Height></pwg:ScanRegion></pwg:ScanRegions><pwg:InputSource>Platen</pwg:InputSource><scan:ColorMode>Grayscale8</scan:ColorMode><pwg:DocumentFormat>application/pdf</pwg:DocumentFormat><scan:DocumentFormatExt>application/pdf</scan:DocumentFormatExt><scan:XResolution>300</scan:XResolution><scan:YResolution>300</scan:YResolution></scan:ScanSettings>
```

`Content-Type: text/xml`. An XML declaration is not needed (it also works with one, if the
body stays a single line).

### 4.2 Traps (each verified with a separate experiment)

1. **`pwg:InputSource`, not `scan:InputSource`.** In capabilities the element is
   called `scan:Platen`/`scan:Adf`, but in `ScanSettings` the field lives in the PWG
   namespace. With `scan:InputSource` the scan job is created (`201`), but the settings
   are ignored.
2. **`pwg:ScanRegions` is mandatory.** Without a region block the device also answers
   `201`, but scans with its default area.
3. **The body must be a single line.** Any `\n` inside the XML (line breaks
   between elements, pretty-print) → the settings are **silently** ignored, there is no
   error. A trailing `\n` after the closing tag is harmless (verified).

If the settings are ignored, the result is always the same default:
**200 dpi, RGB24 (`components=3`), 1680 × 2193 px** (≈ 8.4 × 11.0 inches).
It is exactly this «silent fallback» that is the main danger: neither `4xx` nor a warning.

### 4.3 Unsupported resolution

There is no `150` in `scan:DiscreteResolutions`. A request with `XResolution=150` is accepted, but
`300 dpi` arrives (2512 × 3460, `dpi=(300,300)` in JFIF). I.e. the invalid
resolution is substituted with `300`, not ignored entirely (colour and region are
honoured in this case). The application must validate dpi against capabilities, not
hope for an error.

### 4.4 Incompatible pair: `BlackAndWhite1` + `image/jpeg`

`POST /eSCL/ScanJobs` → **`409 Conflict`**, the body is empty, there is no `Location`.
Reproduced twice. The same `BlackAndWhite1` with `application/pdf` — works.
Conclusion: JPEG only for `Grayscale8`/`RGB24`; for b/w — PDF (or client-side
conversion). The error arrives at scan job creation, not when fetching the document.

## 5. Scan job lifecycle

```
POST /eSCL/ScanJobs            → 201 + Location
GET  <job>/NextDocument        → 200 (blocking, returns the document)
GET  <job>/NextDocument (again) → 404 (no more pages) → scan job finished
GET  <job>/NextDocument        → 410, if the job was removed externally (our DELETE) — §10
DELETE <job>                   → 200, if the job is still active; 404, if already removed
```

### 5.1 Scan job status is taken only from `/eSCL/ScannerStatus`

`GET <job>/ScannerJobStatus` — **`404 Not Found`**, the body is an HTML page of the
built-in httpd (`text/html`, 942 B), not XML. The variants
`/ScannerJobStatus`, `/ScannerJobStatus/`, `/scannerjobstatus`,
`/ScannerJobStatus?x=1`, `/JobStatus` were checked — all `404`.

The only source of state is `<scan:Jobs>` in the shared `ScannerStatus`
(see §3). This diverges from the typical eSCL implementation (there a per-job status
resource exists) — the device does not support it. The fact is taken into account in PRD §7.1.

### 5.2 `NextDocument` is a blocking call, not polling

`GET <job>/NextDocument` does not answer until the page is scanned:
300 dpi A4 → **8.6 s** to the first byte (PDF), 300 dpi Gray JPEG → ~7.8 s.
Therefore «polling the status up to Completed, then fetching the document» is the wrong
strategy: the scan starts exactly from the `NextDocument` request.

### 5.3 Idle timeout ≈ 60 s

If `NextDocument` is not requested after `POST`, the scan job hangs in `Pending`
(`JobScanning`) and after **61 012 ms** goes to `Canceled` / `JobTimedOut`
(measured). After that `POST` is accepted again.

### 5.4 End of document and DELETE

- An empty `404` response (Content-Length 0, `Content-Type: image/jpeg` — a firmware
  artifact, does not match the requested format) means «no more documents» and at the
  same time finalises the scan job.
- **There is no element with the document URI in the XML.** Neither in `ScannerStatus` nor anywhere else
  does `<scan:DocumentUri>` / `<pwg:DocumentUri>` occur; the file name arrives
  only in the `Content-Location` header of the `NextDocument` response. The only
  way to get the document is `NextDocument` itself.
- After a complete fetch `DELETE <job>` returns `404` — the resource has already been removed, but
  the `scan:JobInfo` entry remains in `ScannerStatus`.
- `DELETE` during a scan returns `200` within 0.42–0.60 s, the device goes to
  `Idle` immediately. The hanging `NextDocument` is then aborted with **`410 Gone`**
  (0 bytes, ~0.2 s after the DELETE response) — details in §10.

### 5.5 Observed timeline (300 dpi, Gray, PDF, Platen)

```
t=0        POST → 201 + Location (382 ms)
t≈1.19 s   ScannerStatus: pwg:State=Processing, JobState=Pending
t≈1.91 s   ScannerStatus: JobState=Processing
t≈8.97 s   NextDocument: 200 application/pdf, 142 020 B (blocked 8.59 s)
t≈9.0 s    NextDocument #2: 404 → document ended
           DELETE: 404 (scan job already removed); pwg:State=Idle
```

Response headers with the document:

```
HTTP/1.1 200 OK
Content-Location: /eSCL/ScanJobs/<uuid>/doc-31
Accept-Encoding: identity
Content-Type: application/pdf
Transfer-Encoding: chunked
```

`Content-Location` — `/eSCL/ScanJobs/<uuid>/doc-<n>`; the document was not requested separately at this
path (it was not needed), but the name appears in the response.

## 6. What the device actually returns

All scans were made from the empty glass (white sheet), so the sizes are the only
meaningful metric.

| Request | Result | Bytes |
|---|---|---|
| 100 dpi, Gray JPEG | 832 × 1153, 1 component (L), dpi 100 | 12 209 |
| 200 dpi, Gray JPEG | 1680 × 2306, L, dpi 200 | 49 950 |
| 300 dpi, Gray JPEG | 2512 × 3460, L, dpi 300 | 106 997 |
| 600 dpi, Gray JPEG | 5072 × 6918, L, dpi 600 | 420 679 |
| 300 dpi, RGB24 JPEG | 2512 × 3460, 3 components, dpi 300 | 140 132 |
| 300 dpi, Gray PDF | 1 page, 602.64 × 830.16 pt, 142 020 B | 142 kB |
| 200 dpi, BW1 PDF | 1 page, image 1680 × 2304, `gray`, 1 bpc, CCITT | 13 465 |
| 150 dpi, Gray JPEG | **same as 300 dpi**: 2512 × 3460, L, dpi 300 | 106 997 |
| 300 dpi, BW1 JPEG | **409** on `POST` | — |

Details:

- JPEG: magic `FF D8 FF`, JFIF, SOF0 (`0xC0`). The sizes are read from SOF without a
  decoder (that is what `scripts/escl-probe.mjs` does).
- Physical area: we requested `2550 × 3507` (1/300"), got `2512 × 3460` —
  the device trims ~1.6 mm at the edges in X and ~2 mm in Y.
- PDF: 602.64 × 830.16 pt = 212.6 × 292.9 mm, one page, no metadata
  (`Custom Metadata: no`, `Metadata Stream: no`).
- **PDF from the device = one file per page (Platen).** A multi-page
  PDF from the ADF is **not verified** — there is no paper in the feeder.
  *(M2.5, §11: verified — the ADF returns the whole stack as ONE multi-page
  PDF; JPEG is per-page.)*

## 7. Errors: exact responses

| Situation | Response |
|---|---|
| `Feeder`, tray empty | `POST` → `HTTP/1.1 500 Internal Server Error`, `Content-Length: 0`, `Content-Type: text/xml`, no `Location` |
| `BlackAndWhite1` + `image/jpeg` | `POST` → `409 Conflict`, the body is empty |
| Device busy with someone else's scan job | `POST` → `503 Service Temporarily Unavailable` (HTML body) |
| `GET <job>/ScannerJobStatus` | `404`, `text/html` (942 B) |
| `GET <job>` without a method | `405` |
| `GET <job>/NextDocument` after the end of the document | `404`, `Content-Length: 0`, `Content-Type: image/jpeg` |
| `GET <job>/NextDocument` after our `DELETE` | **`410 Gone`**, `Content-Length: 0` — a normal cancel, not an error (§10) |

`500` on an empty ADF is a crude but unambiguous reaction: the application must
check `scan:AdfState` in `ScannerStatus` beforehand (for us it is always
`ScannerAdfEmpty`) and must not create a scan job blindly.

A useful detail from the sane-airscan sources (`airscan-escl.c`): for `Brother *`
devices the quirk `ESCL_NEXT_LOAD_DELAY` is enabled — when scanning from the ADF
the next `NextDocument` must not be requested immediately after the previous one, otherwise
the MFC-L2710DW returns `404` and `ScannerAdfEmpty`. The delay is 1000 ms, but not more than
half the time of the previous request. It is **not verified** on our device
(no paper), but it is directly about our model — take it into account in M1.

## 8. Web panel (for cross-checking)

- `GET /general/status.html` — `200`, without login. Status: `Energiesparen` (sleep).
  «Toner Level» is an `<img class="tonerremain" height="22">`, **there is no number**;
  the toner percentage is not obtainable from the status page.
- `GET /general/information.html?kind=item` — `200`, without login. Here all the
  numbers are present: `Page Counter 7753`, `Average Coverage 6.39%`,
  `Drum Unit 36%`, `Toner 40%`, `Total Pages Scanned ADF 529 / Flatbed 2082`,
  `Scan Page Count 1437`, `Replace Count Toner 4 / Drum Unit 0`,
  `Total Paper Jams 8` (Tray1 2, Inside 2, Rear 2, 2-sided 2, ADF 6).
- The login form is present on the page (`POST /general/status.html`, field
  `B8d9`), but reading the status and maintenance information is available without it.
- The toner/drum percentages and the page counter are the only reliable source
  for cross-checking SNMP (see `docs/oids-l2710dw.md`).

## 9. Timings (measurements)

| Operation | Time |
|---|---|
| `GET ScannerCapabilities` | 0.25–0.80 s (the first request after idleness — 0.80 s) |
| `GET ScannerStatus` | 0.25–0.30 s |
| `POST ScanJobs` → `201` | 196–570 ms |
| `NextDocument` (300 dpi, Gray, PDF, A4) | 8.59 s of blocking, 8.97 s from `POST` |
| `NextDocument` (300 dpi, Gray, JPEG, A4) | ~7.8 s |
| `NextDocument` #2 (end of document) | ~0.24 s, `404` |
| Scan job timeout without `NextDocument` | 61 012 ms → `Canceled`/`JobTimedOut` |
| `DELETE` of an active scan job (the job is not fetching a document) | < 0.3 s, `200` |
| `DELETE` during a blocking `NextDocument` | 0.42–0.60 s, `200`; fetch → `410` in ~0.2 s |
| `GET ScannerStatus` during a scan | 0.41–0.62 s (not blocked by the scan) |
| Web panel `status.html` | 0.30 s |

## 10. M1 spike 27.10.2026: cancel and parallel requests

Verified on the live device (`curl`, 300 dpi / Grayscale8 / PDF / Platen).
Raw logs — `/tmp/opencode/`, summary — `/tmp/opencode/escl-spike-report.md`.

### 10.1 Cancel during a blocking `NextDocument`

| Step | Result |
|---|---|
| `DELETE <job>` at ~2.6 s of the ~8.6 s scan | **`200`**, 0.42–0.60 s (two runs) |
| the hanging `NextDocument` | **`410 Gone`**, 0 B, ~0.2 s after the DELETE response |
| `pwg:State` immediately after | `Idle` |
| scan job entry in `ScannerStatus` | `Canceled` / `JobCanceledAtDevice`, `ImagesCompleted=0` |
| repeat `DELETE` | not needed |

`410` is a new code (in M0 there was only `404`). Interpretation:
`200` — a document, `404` — no more documents (normal end), `410` — the job was
removed externally, there will be no document. `410` **is not a scan error**.

Caveat: the cancel was verified before the first byte of the document. When cancelling at a moment
when the PDF is already partly in the socket, a partially received file is possible — delete the temporary
file unconditionally.

### 10.2 `ScannerStatus` during a scan

It answers immediately and fully: 0.41–0.62 s, complete XML, all 16 `JobInfo` — the httpd
does not serialise the status behind the scan.

But **the state lags behind the actual transfer**:
`JobState=Processing`/`JobScanning` persists even after the PDF has been received
in full (`ImagesCompleted=1`), and `Completed` appears only after the terminating
`NextDocument` → `404`.

Consequence: `JobState=Completed` is **unsuitable** as a «document ready» trigger.
The status is only good for phase indication and for checking that the device has
become free; the document is fetched exclusively via `NextDocument`.

### 10.3 What this changes in M1

- Cancel = `DELETE` on a separate connection, **without waiting** for the fetch; in the fetch
  task `410` is treated as a normal cancel (see plan M1 §4–§5).
- `DELETE` after a successful fetch is not needed (the device has already finalised the job,
  the response would be `404`) — but calling it is harmless, do not count `404` as an error.
- Status polling is not mandatory in M1: the phases can be tracked on the client
  (`created → scanning → done`). If polling is needed — 500 ms–1 s, only
  for indication, with a wait timeout for `Canceled` of ~2 s (ring buffer of 16 entries).

## 11. M2.5 ADF spike (27.10.2026): the delivery model is split by format

Live tests with A4 sheets in the ADF (Feeder, 300 dpi, Grayscale8, region
`2550×4200`, single-line `ScanSettings`). Three sessions: rounds 1–2 ended with
the feeder reading empty (sensor + `500`, see the spike report); round 3 ran
after the user re-seated the sheets. `scan:AdfState = ScannerAdfLoaded` was
observed for the first time in round 3 — both sensor states are now verified:
`ScannerAdfLoaded` ⇔ Feeder `POST` accepted (`201`), `ScannerAdfEmpty` ⇔ `500`
(the M0 §7 correlation now confirmed in both directions).

### 11.1 The A/B verdict: PDF = one multi-page document, JPEG = per page

| Format | Delivery | Evidence |
|---|---|---|
| `application/pdf` | **B: the whole stack in ONE `NextDocument`** | 2 sheets → one `200` (326 305 B, `Content-Location: …/doc-43`), `pdfinfo` → `Pages: 2`; the next `NextDocument` → `404` (278 ms). Web counter +2. |
| `image/jpeg` | **A: one document per page** | 3 sheets → three `200` responses (`doc-44`, `doc-45`, `doc-46`, ~107 kB each, one JPEG per request); then `404` (199 ms). Web counter +3. |

The sane-airscan per-page model matches only the JPEG path. The PDF path
assembles the stack server-side — the client receives a finished multi-page
PDF in a single response. Consequence: no client-side merge is needed for PDF
(the M2.5 lopdf plan branch for PDF is void); the JPEG path is the per-page
fetch loop.

### 11.2 Pause rule: not required

The JPEG pages above were fetched strictly back-to-back: measured gap between
responses and the next request **1 ms** (no sleep, no status poll between
fetches). Both sheet transitions and the terminal request succeeded — no
spurious `404`, no `ScannerAdfEmpty` mid-stack. The sane-airscan
`ESCL_NEXT_LOAD_DELAY` quirk (for `Brother *`, 1000 ms) does not manifest on
this unit (firmware `ZJ`). The fetch loop needs no delay.

### 11.3 Timings (300 dpi, Grayscale8, A4 sheets)

| Event | Time |
|---|---|
| `POST ScanJobs` (Feeder, accepted) | 176–846 ms |
| JPEG page 1 (feed + scan + encode, blocks) | 8.0 s |
| JPEG pages 2…n (pipeline overlaps with transfer) | 3.1–3.6 s each |
| JPEG `POST` → terminal `404`, 3 sheets | 16.3 s |
| PDF whole-stack fetch, 2 sheets (ONE blocking response) | 15.6 s |
| PDF `POST` → terminal `404`, 2 sheets | 19.9 s |
| Terminal `404` response time | 199–278 ms |
| Platen reference, same settings (M0 §9 / this spike) | 8.6 s / 12.6 s first-after-idle |

The PDF response blocks for the whole stack: a 2-sheet PDF is ready no earlier
than ~15 s after the `NextDocument`, while the JPEG path serves the first page
at ~8 s and each next one every ~3.5 s.

### 11.4 Page geometry: region `4200` on A4 paper behaves differently per format

Requested region `2550×4200` (the ADF maximum, Legal length) on A4 sheets:

- **PDF:** non-final pages are scanned to the full requested strip — page 1:
  602.64 × 1002.24 pt, image 2512 × 4177 px (≈ 6 cm of blank past the sheet's
  trailing edge). The final page is truncated at the sheet end: 602.64 × 835.92 pt,
  2512 × 3485 px.
- **JPEG:** every page is truncated at its own sheet end: 2512 × 3485 / 3485 / 3481 px.

For A4-only scanning request `Height 3507`, otherwise PDF stacks carry a
blank tail on all pages except the last.

### 11.5 Encoding: Grayscale8 + PDF carries RGB streams, JPEG is true grayscale

A `Grayscale8` request delivers **3-component (rgb) JPEG streams inside the PDF**
(`pdfimages`: `rgb 3 8 jpeg`, DCTDecode) — the platen PDF behaves the same
(round-2 sanity scan), so this is the firmware's PDF encoder, not an ADF
effect. The JPEG format delivers real 1-component grayscale (M0 §6 confirmed
on the ADF path too). Sizes: 2-page PDF 326 kB, per-page JPEG ≈ 107 kB.

### 11.6 Job state machine on the ADF path

- During the stack: `JobState=Processing` / `JobScanning`; `ImagesCompleted`
  counts transferred documents (1 after the single whole-stack PDF, 3 after
  three JPEGs). `ImagesToTransfer` reads `0` on feeder jobs (on platen jobs M0
  saw `1`) — the field is not a reliable "pages remaining" signal.
- **The terminal `404` finalises the job:** after the last document the job
  stays `Processing` (verified on the PDF stack job immediately after its only
  document) and flips to `Completed` / `JobCompletedSuccessfully` only after
  the `NextDocument` → `404`. A single-sheet job is structurally identical
  (one `200`, then the required `404`); the literal 1-sheet-at-`POST` case was
  not observed (every reload was consumed by the larger scenarios).
- `DELETE` after the terminal `404` → `404` (resource already removed) — same
  as the platen (§5.4).
- `scan:AdfState` flips to `ScannerAdfEmpty` when the last sheet is fed
  (observed right after the whole-stack PDF response; it was `ScannerAdfLoaded`
  at `POST` time).
- The 61 s idle timeout was re-verified on a feeder job created but never
  fetched (an aborted probe): `Pending`/`JobScanning` → `Canceled` /
  `JobTimedOut` at ~61 s, **no sheet fed** (web counter unchanged).
- The web counter "Total Pages Scanned ADF" counts fed sheets exactly (+2 and
  +3 in the two runs) — usable as a consumption cross-check.

### 11.7 Cancel mid-stack: not verified on the ADF

Paper ran out after the delivery runs (the user reloaded once mid-session; a
6-minute watch with periodic `POST` probes saw no further reload). The
platen cancel matrix (§10.1) remains the reference: `DELETE` during a blocking
`NextDocument` → `200`, the hanging fetch → `410`, device `Idle`,
`Canceled`/`JobCanceledAtDevice`. Open ADF-specific questions: the fate of the
remaining sheets, `ImagesCompleted` after a partial stack, whether a partially
transferred JPEG page is possible, and counter accounting for an in-flight
sheet.

### 11.8 Fixtures

| File | Contents |
|---|---|
| `fixtures/escl/adf/adf-2sheets-300-gray.pdf` | 2-sheet stack as ONE 2-page PDF (outcome B; merge-test input or single-document save path) |
| `fixtures/escl/adf/adf-page-1.jpg` … `adf-page-3.jpg` | per-page JPEGs from a 3-sheet stack (outcome A; true grayscale, 2512×3485/3481, 300 dpi) |

Hygiene: `pdfinfo`/`strings` show no device identifiers and no metadata in any
body; raw logs and status XML stay in `/tmp/opencode/` (uncommitted).

### 11.9 What this changes in M2.5

- PDF ADF scan = one blocking fetch, one file, no merge (Task 5's per-page
  branch applies to JPEG only; the lopdf module already landed in Task 6a is
  not needed for the PDF path — controller's call whether to keep it for
  JPEG-side merging or drop it).
- JPEG ADF scan = per-page loop, no pause between requests, terminate on `404`.
- `AdfState` precheck is meaningful: `ScannerAdfLoaded` before `POST`,
  `ScannerAdfEmpty` explains a `500`.
- The fetch timeout must scale with the stack: the whole-stack PDF blocks
  ~8 s × sheets + overhead (2 sheets ≈ 15.6 s; FeederCapacity is 20 → budget
  ≥ 180 s for PDF, ~8 s + 3.5 s × (n−1) for JPEG).

## REPORT: what did not work and what it means for M1

**Did not work / diverges from the PRD:**

1. **`GET <job>/ScannerJobStatus` does not exist** (404 HTML for all path variants).
   Earlier PRD §7.1 assumed it exists (fixed 26.10.2026). → In `escl.rs` read the scan job state
   only from `/eSCL/ScannerStatus` (`scan:Jobs` → `pwg:JobState`), do not implement the per-job
   resource.
2. **The scan does not start from `POST`.** `POST` only creates the job; scanning
   begins with the first `NextDocument`, which blocks the connection for seconds.
   → Progress in the UI: `start_scan` creates the job and immediately goes into a background task
   that performs `NextDocument` with a timeout of at least 120 s; status polling —
   in parallel, only for the phase (Pending/Processing), not for deciding «it is time
   to fetch the document».
3. **`scan:InputSource` does not work** — `pwg:InputSource` is required.
4. **Pretty-print XML breaks the settings silently.** → the `quick-xml` serialiser
   must produce a compact string; in tests assert that there is no
   `\n` in the body.
5. **`ScanRegions` is mandatory** — without it the default is 200 dpi / RGB24 / Letter.
6. **`150 dpi` is not in the capabilities**, the device substitutes 300 without an error. →
   Build the UI dpi list strictly from the capabilities; in PRD §F2 150 dpi has been removed
   (27.10.2026).
7. **`BlackAndWhite1` + JPEG = 409.** → In the UI forbid the pair (b/w only PDF),
   or merge the PDF on the client.
8. **Empty ADF = `500` on `POST`.** → Precheck `scan:AdfState`
   (`ScannerAdfEmpty`) before creating the job; in the UI — a comprehensible message.
9. **DELETE after a successful fetch gives 404** — this is normal, not an error. The job
   remains in `ScannerStatus` as `Completed` anyway.
10. **Multi-page ADF is not verified** (no paper). The open question PRD §13.3
    remains: it is unknown whether the firmware returns a multi-page PDF in one
    `NextDocument` or page by page. According to the sane-airscan code for this model —
    page by page, with a delay ≥ 1 s between requests; this does not carry over to the PDF path
    automatically.

**What works and carries over to Rust 1:1:** `GET` capabilities/status, `POST`
ScanJobs (`201` + `Location`), `NextDocument` (chunked, JPEG/PDF), `DELETE`.
PDF is served by the device itself — no client-side merging is needed for the Platen.

**Publication hygiene:** the device's real `pwg:SerialNumber` and `scan:UUID`
were scrubbed from the fixtures before the first commit (`REDACTED-SERIAL`,
`00000000-…`); this does not matter for the tests.
