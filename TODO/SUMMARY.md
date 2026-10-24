# Summary

The last session's summary table. Overwritten every session. The record is
[`PROGRESS.md`](PROGRESS.md).

Session of 2026-10-24T08:34:15Z, attended, in a hosted container, on the
branch `claude/eloquent-faraday-nbo94y`, merged into `main` at its end. The
counts of lines and files are from git, through the record commit, this
file included.

| row | before | after |
| --- | --- | --- |
| Elapsed | 2026-10-24T08:34:15Z | 2026-10-24T12:51:25Z: about 4 h 17 min |
| Commits | `a9abde4` | 26 on the session branch, `main` fast-forwarded to them; each by the operator's identity, with no trailer |
| Work | 6 asks: errand vendor or rewrite with its tracker; eight projects; delete the references and hard-fork errand; a Discord stand-in researched and handed to an implementor; settle the open questions; then the rulings (library, web removal, CI-only stand-in, notices, zig, merge and v0.1.1) | 6 completed. T-035 is merged and partial; T-034 is partial (podman and `issues` not ruled). Deferred by the operator: the Windows mandate (T-027, T-007). Failed: none |
| Changes | | 273 files changed, 80112 insertions(+), 368 deletions(-) (`git diff --shortstat a9abde4`); most of it the errand import |
| Size | 151 files, 30,034 lines at `a9abde4` | 383 files, 109,777 lines |
| Entries | 10 done, 4 partial, 2 blocked, 11 open (27) | 14 done, 6 partial, 1 blocked, 15 open (36): T-028 to T-036 new, T-028 to T-031 done |
| Checks | `ci-local` every step but `cross` (exit 2, no qemu); `check-gate` 9 of 9; live tests not run | `ci-local` every step exits 0, `cross` with zig and qemu (112 unit and 6 integration tests on aarch64); 1,243 tests pass, 1 ignored; `check-gate` 9 of 9; `gates` passed on `2ead3fd`; live 54 of 54 |
| Release | v0.1.0 | [v0.1.1](https://github.com/Azathothas/errand-box/releases/tag/v0.1.1) on `2ead3fd`, release run 40095205542; the assets downloaded again match `SHA256SUMS`, and the binary reports 0.1.1 |
| Cost | | five research agents, about 1.97 million tokens, as their reports state; the Discord research agent and the implementing agent: not measured here. GitHub Actions: one `gates` run (about 4.5 min) and one release run (about 4 min) from this session's pushes to `main`. Downloads (Incus packages, the guest image, container images in the live tests): not measured |
| Health | errand#48 open in the errand errand-box installed | T-029 to T-031 fixed in the fork and released. Found and proposed, not filed: errand's refusals restart every 2 s. Repaired: a purge in the first teardown removed 38 base packages, each reinstalled at its version. Tree clean; deployed: v0.1.1 released, installed nowhere |
