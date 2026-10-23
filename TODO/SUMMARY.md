# Summary

The last session's summary table. Overwritten every session. The record is
[`PROGRESS.md`](PROGRESS.md).

Session of 2026-10-23T12:17Z, attended. The counts of lines and files are
from git, through the record commit, this file included.

| row | before | after |
| --- | --- | --- |
| Elapsed | 2026-10-23T12:17:09Z | 2026-10-23T14:47:50Z: 2 h 31 min |
| Commits | `c37acad` | 15 on `main`, each by the operator's identity, unsigned, with no trailer. The deep review found 6 claims or checks to correct; each is fixed in the last commit |
| Work | 6 asks of the operator | 5 completed (push access checked; work on `main`; the operator's identity and no credit; `main` fast-forwarded; the unblocked entries, CI and the release first); 1 failed (testing on the Daytona host: the network policy refuses TCP 22) |
| Entries | 9 done, 4 partial, 2 blocked, 8 open | 10 done, 4 partial, 2 blocked, 7 open. Closed T-009; moved and still partial T-007, T-012, T-013, T-014; deferred T-020 with its questions written |
| Changes | | 30 files, 1105 lines added, 226 removed |
| Size | 143 files, 28556 lines | 151 files, 29435 lines |
| Checks | `gates` on GitHub failed at shellcheck on every push; `ci-local` not run before | `ci-local` every step passed; `gates` passes on every push since `9348da9`; live on the build host 52 of 52; Alpine 50 of 51 with 1 SKIP; Debian 37 of 40; runners 40 of 41 and 39 of 41 |
| Cost | | GitHub Actions minutes and bandwidth not measured. Runs: `gates` 10, `release` 1, `watchdog` 1, `freshness` 2, `live` 4 |
| Health | CI red; no release; the record said CI is off | 12 defects fixed, 9 of them in setup, the image and the test scripts; 2 found and not fixed (E-69, E-70); 2 checks that depend on timing; v0.1.0 released and `main` ahead of it; nothing installed on any host |
