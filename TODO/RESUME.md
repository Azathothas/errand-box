# Resume

Overwritten every session. Not the record: [`PROGRESS.md`](PROGRESS.md) is.

| | |
| --- | --- |
| the task | The operator, 2026-10-24, hosted session: research errand and eight projects (done); hard-fork errand into `errand/` (T-028); a Discord stand-in (T-035, merged); then the rulings: errand as a library (`errand-box serve`, `errand-box threads`), the web interface removed (T-034, partial), notices, zig in `cross`, and a merge into `main` with the release v0.1.1 |
| the resume point | the live test of the library shape: `setup` with the static binary of `9b27e60` and the inputs of `tests/data/`, then `sh tests/live/live.sh`. Then the teardown, the merge into `main` (fast-forward), the version 0.1.1, `gates` on `main`, and `release.yml` with `v0.1.1` |
| in flight | host objects of the live run, if it stopped half way: Incus and its packages (compare `dpkg-query -W` with the 841 packages before it; purge only the difference, simulated first), the accounts `errand` and `incus`, the bridges and Incus's dnsmasq (stop it by PID), root's lines in `/etc/subuid` and `/etc/subgid`, the Zabbly repository. Also qemu-user-static and a `binfmt_misc` mount with a `qemu-aarch64` handler, made for `cross` |
| the state of the tree | `claude/eloquent-faraday-nbo94y`, pushed at `9b27e60`; `main` is at `a9abde4` and can fast-forward to it |
| the paste | Read AGENTS.md, TODO/RESUME.md and TODO/PROGRESS.md. Check the host for the objects named in RESUME.md's "in flight" row and remove what is left. Then run sh scripts/ci-local.sh, merge the branch into main, and release v0.1.1 as PROGRESS.md says. |
