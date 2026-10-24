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

- **Last session:** started 2026-10-24T08:34:15Z, attended, in a hosted
  container, on the branch `claude/eloquent-faraday-nbo94y`, fast-forwarded
  into `main`.
- **Tree:** clean after the record commit, pushed to `main`.
- **Deployed:** [v0.1.1](https://github.com/Azathothas/errand-box/releases/tag/v0.1.1),
  the latest release, on `2ead3fd`. The prerelease
  [`artefacts`](https://github.com/Azathothas/errand-box/releases/tag/artefacts)
  holds the test assets, rebuilt from `main` by `artefacts.yml`. errand-box
  is installed nowhere: every test host of the session was torn down.
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

The build host: a hosted container, kernel 6.18, x86_64, 4 CPUs, 16 GiB.

**37 entries: 14 done, 6 partial, 1 blocked, 16 open.**

---

## What this session did

- **Research:** six write-ups in `docs/history/references/`.
- **The fork** (T-028, done): imported, gated, a library, shipped in v0.1.1.
- **Fixed in the fork:** T-029 (the provider key sent to a host a session
  named), T-030 (a push that could publish another session's repository),
  T-031 (a silent drop in the relay).
- **The web interface removed** (T-034, partial: podman and `issues`
  remain). **discord-mock** (T-035, partial).
- **`artefacts`:** `.github/workflows/artefacts.yml` builds errand-box,
  discord-mock and the guest image on both architectures, with an e2e kit;
  the first run passed. `tests/e2e/` holds the kit's inputs and
  `mock-session.sh`.
- **T-037 filed.** T-027 rewritten for the standalone agent.

Leads for T-027's run, seen during the session and not filed:

- With `tests/e2e/mock.env`, kage refuses the first turn: "no provider
  credentials found". For a provider kage does not know, errand writes it
  into kage's config only with a base URL and a models list
  (`errand/src/sandbox/agent_config.rs:291`).
- The supervised errand cannot be pointed at discord-mock: `errand-box
  __as` clears the environment (`src/procs.rs`).
- `errand-box doctor` before setup says "run `errand-box up`" for Incus that
  is not installed; `setup --skip-image` says errand waits for `--env` and
  `--creds` when both were given.
- errand's log under errand-box says sessions run "as confined host
  processes using the host's own tools" (T-016).

## What is in progress

Nothing.

---

## Start here next session

1. The issues and entries T-027's run files, in the order its entries rank.
2. T-037.
3. T-035's open items, then T-032 and T-033.
4. T-020, then T-019's backend in the fork, which retires the `bailey`
   stand-in.
5. T-024, then T-021, then T-022, which waits on T-020's link.
6. T-034's podman and `issues`, T-036, T-014, T-026, T-023, T-025.

---

## Open questions for the operator

1. **Revoke the Daytona token** that was given to the session of
   2026-10-23: it is in a session transcript.
2. **T-007's setting and secret**: the commands are in
   [`upkeep.md`](upkeep.md), T-007; they need your `gh` CLI and a token you
   create.
3. **The credentials T-015 lists**, if T-027's run is to test a real model
   turn or a real Discord application. Put each in a file, never in the
   chat.

---

## Settled, and not to be raised again

See [`RULES.md`](RULES.md) section 4: Incus system containers; errand as
errand-box's hard fork; root at setup; s6 from the distribution.
