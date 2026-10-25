# Resume

Overwritten every session. Not the record: [`PROGRESS.md`](PROGRESS.md) is.

| | |
| --- | --- |
| the task | The operator, 2026-10-25, hosted session: triage every open GitHub issue into entries by root cause, then work all the security entries and the adjacent ones so they and issues like them do not recur. Default identity, on `main`, push when asked |
| the resume point | [`PROGRESS.md`](PROGRESS.md), "Start here next session". The security cluster is done but for T-082 (needs a real HTTP proxy to verify, and changes the SSRF posture) and the host cluster (needs an Incus host); T-079's one live `up` run also remains |
| in flight | nothing mid-edit. Nine security entries worked: T-088, T-080, T-063, T-089, T-081, T-032, T-033 done; T-079 and T-083 partial. Each committed on `main` and pushed, with a regression test and a forbidden-patterns row or a check |
| the state of the tree | `main` through `47be7f1`, pushed to origin. `gate` 9 of 9, check-todo 89 consistent; `cargo test -p errand --lib` 1,106 pass and `-p errand-box --bins` 115 pass; fmt and clippy clean on both crates. Full `ci-local` not re-run this session (`musl`/`cross` unaffected by the change; the pre-existing `shellcheck` 0.9.0 SC2317 artifact stands). No live tests: no Incus here |
| the paste | Read AGENTS.md and TODO/PROGRESS.md, then start at "Start here next session": close T-079 with one live `sh tests/live/live.sh up`, then T-082 behind a real proxy (deciding the address-check posture), then the host cluster on an Incus host. |
