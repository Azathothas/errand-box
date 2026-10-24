# Resume

Overwritten every session. Not the record: [`PROGRESS.md`](PROGRESS.md) is.

| | |
| --- | --- |
| the task | The operator, 2026-10-24, hosted session: research errand and eight projects (done); then hard-fork errand into `errand/` and patch it (T-028 to T-031 done or shown live); then a Discord stand-in for tests (T-035), researched and handed to an implementing agent |
| the resume point | T-035: the implementing agent works in a git worktree under `.claude/worktrees/` on a branch of its own, and commits there without pushing. Review its commits, run the gate, merge them into `claude/eloquent-faraday-nbo94y`, then rewrite PROGRESS.md and SUMMARY.md |
| in flight | the commit `WIP, not built: errand as a library` holds the operator's rulings on errand (serve, threads, no errand link) and the zig cross step, unbuilt: build, test and fix it before anything else. Then the web removal, the live test, the merge and v0.1.1. Also the T-035 worktree; musl-tools stays installed for its gate. The live host objects are torn down (uninstall, account, 87 packages, bridges); 38 base packages a wrong purge removed were reinstalled at their versions |
| the state of the tree | `claude/eloquent-faraday-nbo94y` ends in the WIP commit above, pushed; the gate's file checks pass, cargo was not run on it |
| the paste | Read AGENTS.md, TODO/RESUME.md and TODO/PROGRESS.md. Finish T-035: if `git worktree list` shows the implementing agent's worktree, review its commits against docs/history/references/2026-10-24-discord-mock.md, run sh scripts/ci-local.sh, merge, and update the record. Otherwise start T-035 from that plan. |
