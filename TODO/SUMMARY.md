# Summary

The last session's summary table. Overwritten every session. The record is
[`PROGRESS.md`](PROGRESS.md).

Session of 2026-10-23T12:17Z, attended. The counts of lines and files are
from git, through the record commit, this file included.

| row | before | after |
| --- | --- | --- |
| Elapsed | 2026-10-23T12:17:09Z | 2026-10-23T15:30Z: about 3 h 13 min |
| Commits | `c37acad` | 16 on `main`, each by the operator's identity, unsigned, with no trailer |
| Work | 6 asks, then 6 more in the second message | First message: 5 completed, 1 failed (Daytona: the network policy refuses TCP 22). Second: 5 completed (the entries filed; #41 read twice and the maintainer's ruling recorded; every open question ruled and recorded; the docs made current for Windows; the next session mandated in PROGRESS and RESUME); 1 not reached (the v0.1.1 release), handed to the next session |
| Entries | 9 done, 4 partial, 2 blocked, 8 open | 10 done, 4 partial, 2 blocked, 11 open. Closed T-009; filed T-024 to T-027; rulings recorded in T-007, T-015, T-019 to T-022 |
| Checks | `gates` on GitHub failed at shellcheck on every push | `ci-local` every step passed at `ab65163`; on the record commit every step but `cross` (exit 2: no `binfmt_misc` handler after teardown; the unit tests passed under qemu); `gates` passes on every push since `9348da9`; live on the build host 52 of 52; Alpine 50 of 51 with 1 SKIP; Debian 37 of 40; runners 40 of 41 and 39 of 41 |
| Cost | | GitHub Actions minutes not measured. Runs: `gates` 11, `release` 1, `watchdog` 1, `freshness` 2, `live` 4 |
| Health | CI red; no release | 12 defects fixed; E-69 and E-70 measured and filed as T-024; 2 checks that depend on timing filed as T-026; v0.1.0 released and `main` ahead of it; nothing installed on any host |
