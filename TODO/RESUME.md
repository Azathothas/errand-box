# Resume

Overwritten every session. Not the record: [`PROGRESS.md`](PROGRESS.md) is.

| | |
| --- | --- |
| the task | Make CI green on GitHub and publish the first release (T-009); then the other unblocked open entries, with a second host the operator provided for the entries that need one. |
| the resume point | `errand-box setup` on the build host from the v0.1.0 release binary, then `tests/live/live.sh`, then a nested Alpine host that imports the image (T-012). |
| in flight | nothing uncommitted. Done and pushed: CI green, release v0.1.0 by dispatch, T-009 closed (commit not yet pushed: the T-009 closing). |
| the state of the tree | clean but for the unpushed T-009 commit; the gate passes. Host objects of this session: a binfmt_misc handler `qemu-aarch64` and the `binfmt_misc` mount, errand-box from bootstrap.sh in `/usr/local/lib/errand-box`, and whatever setup installs. |
| the paste | Read AGENTS.md and TODO/PROGRESS.md, then TODO/RESUME.md. Continue from its resume point; all work lands on `main`. |
