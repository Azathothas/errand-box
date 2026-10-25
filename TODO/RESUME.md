# Resume

Overwritten every session. Not the record: [`PROGRESS.md`](PROGRESS.md) is.

| | |
| --- | --- |
| the task | The operator, 2026-10-25, hosted session: triage every open GitHub issue into entries by root cause, then work all the security entries and the adjacent ones so they and issues like them do not recur. Default identity, on `main`, push when asked |
| the resume point | [`PROGRESS.md`](PROGRESS.md), "Start here next session". The security cluster is done; **this host is provisioned (Incus 6.0.7) and `sh tests/live/live.sh` runs here**, so host work is no longer blocked. The open egress work is T-090 (task 1's gate module landed; the rest is drivable here, task 8 wants the sealed sandbox) |
| in flight | nothing mid-edit. Nine security entries worked (T-088, T-080, T-063, T-089, T-081, T-032, T-033, T-079 done; T-083 partial). Then T-090 was authored (a virtual network layer superseding T-022 and T-082) and its task 1 first slice implemented (a backend-agnostic `gate` module). Then the host was provisioned and the live suite run: it closed T-079 and caught a T-063 regression (`maint --once` wrongly refused), now fixed and re-verified live. All committed on `main`, pushed |
| the state of the tree | `main`, pushed to origin. `gate` 9 of 9, check-todo 90 consistent; `cargo test -p errand --lib` 1,107 pass, `-p errand-box` 7 + 115 pass; fmt and clippy clean. **This host now runs Incus 6.0.7**; `errand-box setup` 0-failed and `sh tests/live/live.sh` passes every sandbox check (the `errand:` checks need Discord creds, T-015). Full `ci-local` `musl`/`cross` not re-run; pre-existing `shellcheck` SC2317 artifact stands |
| the paste | Read AGENTS.md and TODO/PROGRESS.md, then start at "Start here next session". The host is set up, so you can run `sh tests/live/live.sh` directly. Continue T-090: task 1 (the `gate` module) and task 2 (the smoltcp stack + `LinkDevice` seam, loopback test passing) are in. Next is task 3, the Incus TUN link over T-020's unix socket, live-testable here. Then tasks 4 to 7 here, task 8 in the sealed sandbox. |
