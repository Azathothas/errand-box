# Progress

Read this first. It has the baseline, what the last session did, and the
work order. It carries no history: every session rewrites it. The history
is the git log and the entries.

How this repository is worked on: [`RULES.md`](RULES.md). Every entry, one
line each: [`INDEX.md`](INDEX.md). Routing for an agent:
[`AGENTS.md`](../AGENTS.md). What the last session had in flight when it
stopped: [`RESUME.md`](RESUME.md). Its summary table:
[`SUMMARY.md`](SUMMARY.md).

---

## The operator's rulings of 2026-10-24

1. **errand is errand-box's hard fork, in `errand/`** (T-028), and a
   library errand-box runs: `errand-box serve`, `errand-box threads`, no
   `errand` program. A host runs errand-box, kage and Incus.
2. **The web interface leaves the fork** (T-034).
3. **discord-mock** (T-035) runs in CI and in-process tests, and on test
   hosts as an asset of the prerelease `artefacts`; it is no asset of a
   version.
4. **A release carries `THIRD-PARTY-NOTICES`**; `cross` takes zig.
5. **T-037 is approved**: back off errand's refusals rather than stop them.
6. **T-027 runs as a standalone agent's end-to-end test** on WSL2,
   ModelScope Studio and Daytona, from a runbook the operator holds outside
   the tree. It files GitHub issues and an entry per issue, and builds
   nothing. The plan of 2026-10-23 for a session on the Windows machine is
   withdrawn.

---

## State

- **Last session:** T-027's end-to-end run, 2026-10-24, unattended, a
  standalone agent on the operator's Windows machine; it changed only
  `TODO/`, on `main`. Record commit stamped 2026-10-24T16:44:26Z.
- **Tree:** clean after the record commit, pushed to `main`.
- **Deployed:** [v0.1.1](https://github.com/Azathothas/errand-box/releases/tag/v0.1.1),
  the latest release, on `2ead3fd`. The prerelease
  [`artefacts`](https://github.com/Azathothas/errand-box/releases/tag/artefacts)
  holds the test assets, rebuilt from `main` by `artefacts.yml`. errand-box
  is installed nowhere: the run purged it from every test host and removed
  its WSL2 distributions; the ModelScope Studio and Daytona sandboxes are the
  operator's to delete, with the Incus packages setup installed.
- **CI:** on. `gates` runs on every push to `main`.

## Baseline, as measured this session

Measure again with the same commands before you change anything.

| check | result |
| --- | --- |
| `sh scripts/ci-local.sh` | every step passes on `2ead3fd`. `cross` needs zig 0.16 (`pip install ziglang==0.16.0`), `qemu-aarch64-static` and a `binfmt_misc` handler; with them it ran 112 unit and 6 integration tests on aarch64. A virtualenv's `bin` first on PATH hides the system python3, and `discord-contract` then exits 2. Since then only scripts, workflows and pages changed: the `gate`, `kill-by-name` and `shellcheck` steps pass locally, and `gates` on GitHub runs the rest |
| `cargo test --locked --workspace` | 1,243 pass and 1 is ignored: errand 1,094, errand-box 112 unit and 6 integration, 31 in the others; Rust 1.99.1 |
| `sh scripts/common/check-gate.sh --strict` | 9 of 9 |
| live tests | 54 of 54 with the static binary of `9b27e60`, on a hosted container: Ubuntu 24.04, kernel 6.18, x86_64, cgroup v1, no init, Incus 6.0.7 ([output](../tests/live/2026-10-24-errand-library.txt)) |
| the `artefacts` assets | on the same kind of host: v0.1.1 installed by the kit's `bootstrap.sh`, the image imported from `artefacts` and taken as current, setup 0 failed and then 0 changed, `tests/e2e/mock-session.sh` PASS |
| live tests on WSL2 | T-027's run: WSL2 Ubuntu 24.04 with systemd, cgroup v2, Incus 7.5.1, v0.1.1 from its release: 41 of 42, rootless podman failed (E-70, T-024); WSL2 Arch with no init: no guest starts (T-039) |

The build host: a hosted container, kernel 6.18, x86_64, 4 CPUs, 16 GiB.

**78 entries: 15 done, 6 partial, 1 blocked, 56 open.**

---

## What this session did

- **T-027, done:** errand-box used as a user uses it, from v0.1.1 and
  `artefacts`, on WSL2 (Ubuntu 24.04 with systemd, a second one for the
  upgrade from v0.1.0, and Arch with no init), ModelScope Studio and Daytona.
  T-027's entry has the table per host.
- **41 issues** on this repository, #2 to #42, and **41 entries**, T-038 to
  T-078, one per issue. Evidence added to T-012, T-024, T-036 and T-037.
- **Leads of the last session, settled:** the provider kage refuses is
  T-041; the supervised errand and the stand-in, T-049; doctor's advice
  before setup, T-068.
- Built nothing and changed no code.

## What is in progress

Nothing.

---

## Start here next session

1. The P1 entries of T-027's run: T-040 (setup stops after `up` and skips
   the sandbox steps), T-041 (a custom provider never reaches kage), T-046
   (uninstall's documented order), T-038 and T-039 (WSL2), T-042 (a
   user-namespaced host).
2. T-043 and T-044 (a setup that proves a guest starts, and refuses where
   none can), then T-037 with T-050.
3. The other P2 entries of the run, smallest first: T-051, T-063, T-047,
   T-048, T-058, T-059, T-060, T-061, T-062, T-064, T-065, T-045, T-052,
   T-053, T-055, T-057, T-049, T-054, T-066.
4. T-035's open items, then T-032 and T-033.
5. T-020, then T-019's backend in the fork, which retires the `bailey`
   stand-in.
6. T-024, then T-021, then T-022, which waits on T-020's link.
7. T-034's podman and `issues`, T-036, T-014, T-026, T-023, T-025, and the
   P3 entries of the run.

---

## Open questions for the operator

1. **Revoke the Daytona token** that was given to the session of
   2026-10-23: it is in a session transcript.
2. **T-007's setting and secret**: the commands are in
   [`upkeep.md`](upkeep.md), T-007; they need your `gh` CLI and a token you
   create.
3. **The credentials T-015 lists**, for a real Discord application. T-027's
   run tested a real model turn through a gateway the operator named, and no
   real chat service.

---

## Settled, and not to be raised again

See [`RULES.md`](RULES.md) section 4: Incus system containers; errand as
errand-box's hard fork; root at setup; s6 from the distribution.
