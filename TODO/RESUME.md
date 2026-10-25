# Resume

Overwritten every session. Not the record: [`PROGRESS.md`](PROGRESS.md) is.

| | |
| --- | --- |
| the task | The operator, 2026-10-25, hosted session: triage every open GitHub issue into entries by root cause, then work all the security entries and the adjacent ones so they and issues like them do not recur. Default identity, on `main`, push when asked |
| the resume point | [`PROGRESS.md`](PROGRESS.md), "Start here next session". The host-free security cluster is done; the egress work is now T-090 (a design filed this session, not started: needs an Incus host and a sealed sandbox), and the host cluster needs an Incus host; T-079's one live `up` run also remains |
| in flight | nothing mid-edit. Nine security entries worked (T-088, T-080, T-063, T-089, T-081, T-032, T-033 done; T-079, T-083 partial), each committed on `main`, pushed, with a test and a forbidden-patterns row. Then T-090 was authored (a virtual network layer superseding T-022 and T-082) and its task 1 first slice implemented: a backend-agnostic `gate` module (admission verdict + provider routing, verdict parity). T-090 is partial; its remaining stages need a host or the sealed sandbox |
| the state of the tree | `main`, pushed to origin. `gate` 9 of 9, check-todo 90 consistent; `cargo test -p errand --lib` 1,107 pass and `-p errand-box --bins` 115 pass; fmt and clippy clean on both crates. Full `ci-local` not re-run this session (`musl`/`cross` unaffected; the pre-existing `shellcheck` 0.9.0 SC2317 artifact stands). No live tests: no Incus here |
| the paste | Read AGENTS.md and TODO/PROGRESS.md, then start at "Start here next session": close T-079 with one live `sh tests/live/live.sh up`, then continue T-090 (task 1's gate module is in; finish the predicate gathering and the flow API, then task 2's smoltcp stack, both host-free; tasks 3 to 7 need an Incus host, task 8 the sealed sandbox), then the host cluster. |
