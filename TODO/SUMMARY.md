# Summary

The last session's summary table. Overwritten every session. The record is
[`PROGRESS.md`](PROGRESS.md).

Session of 2026-10-25, attended, in a hosted container, on `main`. It triaged
the open issues (see PROGRESS), implemented the security cluster, filed and
began T-090, provisioned this host and ran the live suite, then drove T-090
task 3's gate (T-020) to a live-proven design. A worker restart mid-session
stopped incusd; `errand-box up` recovered it.

| row | before | after |
| --- | --- | --- |
| Commits | triage on `main` | the security cluster (T-088, T-080, T-063, T-079, T-089, T-081, T-032, T-033, T-083), then T-090 tasks 1 and 2, the live-run fixes (T-063 regression, T-079 closed), CI-green doc fix, and three T-020 record commits (measurement, prototype, this handoff). Each the operator's identity, no trailer |
| Work | the security cluster, triaged but unbuilt | security cluster built (7 fixed, T-079 + T-083 partial); T-090 authored (supersedes T-022/T-082) with tasks 1 (gate module) and 2 (smoltcp stack + `LinkDevice` seam) landed; **T-020 proven live** (measurement + full-path prototype) and the build mapped, one design point open |
| Host | not provisioned ("no Incus") | **provisioned: Incus 6.0.7**, golden image built, `sh tests/live/live.sh` runs here and passes every sandbox check (the `errand:` checks need Discord creds, T-015) |
| Entries | 89 (15 done, 6 partial, 1 blocked, 67 open) | 90 (23 done, 8 partial, 1 blocked, 58 open) |
| Tests | | each security fix has a regression test; `cargo test -p errand --lib` 1,107 pass, `-p errand-box --bins` 115 pass; the netstack loopback round-trip test passes; **CI green on the last push** |
| Prevention | | 8 `forbidden-patterns.md` rows (the 7 security classes plus a rejecting-guard-tested-only-on-bad-input row from the live `maint --once` regression) |
| Checks | `gate` 9 of 9 | `gate` 9 of 9; fmt and clippy clean; live suite green here. Full `ci-local` `musl`/`cross` not re-run (unaffected); pre-existing `shellcheck` SC2317 artifact stands |
| Left | | **T-020's one open design point** (the dynamic broker port; recommend a fixed port in errand) gates its build (disk-share link + guest relay, proxy=block, image rebuild, live). Then T-090 task 3 (the TUN over the same socket) and the rest of the host cluster. All in PROGRESS "Start here next session" |
| Cost | | one read-only tracker fetch (triage); one Explore subagent (T-020 doc constraints); throwaway test instances all removed |
