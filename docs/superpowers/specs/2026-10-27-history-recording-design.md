# Silent History Recording — Short Design

- Status: draft for approval
- Phase: post-M2. Ships **standalone, first in the phase** — it is the only
  deliverable whose value decays with every day of delay, and it blocks on
  nothing (one hook in the M2 poller, one module). *(my call, builder-check q5)*

## Intent

F4 promises a 90-day toner/drum/pages chart. A chart is only as good as its
data: every day without recording is a hole the chart can never fill back. The
M2 poller already produces a good `DeviceStatus` every 5 minutes — start
persisting it now, build the chart and notifications (M4) later on top of real
data.

## Goals

- One SQLite table, one insert per **successful** poll (a failed poll writes
  nothing — gaps stay visible later; the M2 cache already keeps last-good for
  the UI).
- `rusqlite` (bundled) — already in the approved crate list PRD §10, no new
  dependency.

## Non-goals

- Any UI, any chart, any retention policy (rows are ~50 B, 288/day at the
  default 5-minute poll — years of headroom).
- Migrations framework — schema v1 with a `user_version` pragma check.

## How

- `history.rs`: open/create `dirs::data_dir()/prindeck/history.db`
  (`~/.local/share/prindeck/` — data, not config, per XDG).
- Table: `snapshots(ts TEXT PRIMARY KEY rfc3339, toner_pct INT NULL,
  drum_pct INT NULL, pages_total INT NULL, duplex_pages INT NULL,
  drum_pages_left INT NULL)`.
- The M2 poll task calls `history::record(&status)` after a successful poll;
  a recording failure logs a `warn` and never fails the poll.
- Unit test: insert → read round-trip on the real bundled engine.

## Acceptance

- After one polling interval the DB exists and rows accumulate; killing the app
  leaves a valid DB (`PRAGMA integrity_check` clean).
