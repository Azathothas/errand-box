# Summary

The last session's summary table. Overwritten every session. The record is
[`PROGRESS.md`](PROGRESS.md).

Session of 2026-10-25, attended, in a hosted container, on `main`, from the
triage commit through `47be7f1`. It triaged the open issues (see PROGRESS) and
then implemented the security cluster.

| row | before | after |
| --- | --- | --- |
| Commits | triage on `main` | 9 more, each the operator's identity, no trailer: T-088, T-080, T-063, T-079, T-089, T-081, T-032, T-033, T-083 |
| Work | the security cluster, triaged but unbuilt | 7 entries fixed (T-088, T-080, T-063, T-089, T-081, T-032, T-033) and 2 carried partway (T-079 live-pending, T-083) |
| Entries | 89 (15 done, 6 partial, 1 blocked, 67 open) | 89 (22 done, 8 partial, 1 blocked, 58 open) |
| Tests | | each fix has a regression test that fails without it; `cargo test -p errand --lib` 1,106 pass (1 ignored), `-p errand-box --bins` 115 pass |
| Prevention | | 7 `forbidden-patterns.md` rows: TOCTOU check-then-use, `.expect` under `panic = abort`, redirect-with-credential, body rewrite keeping `Content-Length`, chunked scrub with no boundary carry, unbounded/metadata read, raw-bytes field skipping the redactor |
| Checks | `gate` 9 of 9 | `gate` 9 of 9 after each commit; fmt and clippy clean on both crates; `errand-config` green. Full `ci-local` not re-run (`musl`/`cross` unaffected; pre-existing `shellcheck` SC2317 artifact stands). Live tests not run: no Incus |
| Left | | T-082 (egress through a proxy; needs a real proxy, changes the SSRF posture) and the host cluster (need Incus) not started; T-079's one live `up` run remains. All in PROGRESS "Start here next session" |
| Cost | | one read-only tracker fetch during triage; no subagents spawned; no host objects created |
