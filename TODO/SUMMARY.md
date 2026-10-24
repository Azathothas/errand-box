# Summary

The last session's summary table. Overwritten every session. The record is
[`PROGRESS.md`](PROGRESS.md).

Session of 2026-10-24T08:34:15Z, attended, in a hosted container, on the
branch `claude/eloquent-faraday-nbo94y`, fast-forwarded into `main`. The
counts of lines and files are from git, through the record commit, this
file included.

| row | before | after |
| --- | --- | --- |
| Elapsed | 2026-10-24T08:34:15Z | 2026-10-24T13:44:39Z: about 5 h 10 min |
| Commits | `a9abde4` | 29, each by the operator's identity, with no trailer |
| Work | 6 asks, then the rulings; then 4 more: file T-037, publish `artefacts`, a runbook for T-027's agent, withdraw the plan of 2026-10-23 | all completed. The runbook is in chat, not in the tree. T-034 and T-035 stay partial |
| Changes | | 278 files changed, 80653 insertions(+), 511 deletions(-) (`git diff --shortstat a9abde4`); most of it the errand import |
| Size | 151 files, 30,034 lines at `a9abde4` | 387 files, 110,175 lines |
| Entries | 10 done, 4 partial, 2 blocked, 11 open (27) | 14 done, 6 partial, 1 blocked, 16 open (37): T-028 to T-037 new, T-028 to T-031 done |
| Checks | `ci-local` every step but `cross` (exit 2, no qemu); `check-gate` 9 of 9; live tests not run | `ci-local` every step on `2ead3fd`, `cross` with zig and qemu; 1,243 tests pass, 1 ignored; `gates` green on every push to `main`; live 54 of 54; the `artefacts` assets passed setup and `mock-session.sh` on a host |
| Release | v0.1.0 | [v0.1.1](https://github.com/Azathothas/errand-box/releases/tag/v0.1.1) on `2ead3fd`; the prerelease [`artefacts`](https://github.com/Azathothas/errand-box/releases/tag/artefacts), never the latest |
| Cost | | five research agents, about 1.97 million tokens, as their reports state; the Discord research agent and the implementing agent: not measured here. GitHub Actions: the `gates` runs of the pushes to `main`, one release run (about 4 min) and the `artefacts` runs (about 11 min each). Downloads: not measured |
| Health | errand#48 open in the errand errand-box installed | T-029 to T-031 fixed and released. Filed: T-037. Repaired: a purge in the first teardown removed 38 base packages, each reinstalled at its version from apt's history; no list from the session's start exists to compare with. The later teardowns match the lists taken before them. Tree clean; v0.1.1 installed nowhere |
