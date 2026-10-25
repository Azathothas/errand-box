# Summary

The last session's summary table. Overwritten every session. The record is
[`PROGRESS.md`](PROGRESS.md).

Session of 2026-10-25, attended, in a hosted container, on `main`. It triaged
the open issues (see PROGRESS), implemented the security cluster, filed and
began T-090, provisioned this host, ran the live suite, then **built and
live-validated T-020** (the unix-socket broker link). Worker restarts stopped
incusd twice; `errand-box up` recovered it each time. It closed with a
strictly-technical (ASD-STE100) wording pass over the docs and record, then a
clean handoff.

| row | before | after |
| --- | --- | --- |
| Commits | triage on `main` | the security cluster (T-088, T-080, T-063, T-079, T-089, T-081, T-032, T-033, T-083), T-090 tasks 1 and 2, the live-run fixes, and **T-020 in full**: measure, prototype, three slices (configurable `brokerPort`, errand-box sets it, the disk-share switch), a marker-gate fix, and a boot-race fix. Then the wording pass: `f19d1a0` (security pages and record), `fca6d33` (a shellcheck SC2317 false positive on `mock-session.sh`), and `7752b89` (the rest of the live tree). Each the operator's identity, no trailer |
| Work | the security cluster, triaged but unbuilt | security cluster built; T-090 authored with tasks 1 (gate) and 2 (smoltcp stack + `LinkDevice`) landed; **T-020 done** (the broker link is a shared socket with a guest `systemd-socket-proxyd` relay; `restricted.devices.proxy=block`; E-61 closed) |
| Host | not provisioned | **provisioned, Incus 6.0.7**, now running the **socket-mode image** (`proxy=block`, `broker_link=socket`); `sh tests/live/live.sh` runs here |
| Entries | 89 (15 done, 6 partial, 1 blocked, 67 open) | 90 (24 done, 8 partial, 1 blocked, 57 open) |
| Tests | | regression tests for each fix; `cargo test` green on both crates; the netstack loopback test passes; **T-020 live-validated**: broker over the socket with proxy blocked, both guest relay ports carry an HTTPS CONNECT (401), rootless podman pulled a full image; **CI green** |
| Prevention | | 8 `forbidden-patterns.md` rows; a new edge case E-71 (a guest disk-device mount under `/run` is shadowed by the boot tmpfs; mount on the rootfs) |
| Checks | `gate` 9 of 9 | `gate` 9 of 9; fmt, clippy, shellcheck (SC2317 FP fixed in `fca6d33`), test, errand-config clean; live suite's sandbox checks pass. CI for `7752b89`: both `musl` arches and gate-through-clippy green; final `test` unread behind a stale API cache (check-in set to confirm) |
| Left | | Confirm `gates` run `40176641403` (`7752b89`) went `success`; then T-090 task 3 (the TUN over T-020's socket) is next, then tasks 4 to 8 and the rest of the host cluster. Open for the operator: whether to neutralise `docs/history/` too (Open question 4). The live `errand:` checks need Discord creds (T-015); the live image-pull checks fail on registry auth through this host's proxy stand-in, not the link. All in PROGRESS "Start here next session" |
| Cost | | one tracker fetch (triage); one Explore subagent (T-020 docs); one image rebuild; throwaway test instances all removed |
