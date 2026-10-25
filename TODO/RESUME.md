# Resume

Overwritten every session. Not the record: [`PROGRESS.md`](PROGRESS.md) is.

| | |
| --- | --- |
| the task | The operator, 2026-10-25, hosted session: triage every open GitHub issue into entries by root cause, then work all the security entries and the adjacent ones so they and issues like them do not recur. Default identity, on `main`, push when asked |
| the resume point | [`PROGRESS.md`](PROGRESS.md), "Start here next session". The host-free security cluster is done; the egress work is now T-090 (a design filed this session, not started: needs an Incus host and a sealed sandbox), and the host cluster needs an Incus host; T-079's one live `up` run also remains |
| in flight | nothing mid-edit. Nine security entries worked (T-088, T-080, T-063, T-089, T-081, T-032, T-033 done; T-079, T-083 partial), each committed on `main`, pushed, with a test and a forbidden-patterns row. Then T-090 was authored: a virtual network layer design that supersedes T-022 and T-082 (one gated egress path, smoltcp + a shared gate, a TUN link for Incus and the operator's `cfrs` LD_PRELOAD link for the lite backend). Not implemented, per authoring.md |
| the state of the tree | `main`, pushed to origin. `gate` 9 of 9, check-todo 90 consistent; `cargo test -p errand --lib` 1,106 pass and `-p errand-box --bins` 115 pass; fmt and clippy clean on both crates. Full `ci-local` not re-run this session (`musl`/`cross` unaffected; the pre-existing `shellcheck` 0.9.0 SC2317 artifact stands). No live tests: no Incus here |
| the paste | Read AGENTS.md and TODO/PROGRESS.md, then start at "Start here next session": close T-079 with one live `sh tests/live/live.sh up`, then T-090 (start with task 1, the host-free gate-as-a-library refactor; tasks 3 to 7 need an Incus host, task 8 the sealed sandbox), then the host cluster. |
