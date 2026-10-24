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

The session of 2026-10-24 researched whether to vendor errand or rewrite its
chat layer, and eight projects the operator named. Five write-ups hold the
evidence, each listed in [`docs/history/README.md`](../docs/history/README.md).
The operator ruled on the recommendation the same day:

1. **errand is errand-box's hard fork, in `errand/`** (T-028). Imported byte
   for byte at `715e9b0b777a`; no upstream release is taken after it. It
   builds and passes its tests in the workspace and the gate, and it is
   built into errand-box, which setup links as `errand`.
2. **The corpus is deleted.** The write-ups stay; each names the commit it
   read, and `git clone` with
   [`scripts/fetch-tracker.sh`](../scripts/fetch-tracker.sh) fetches it again.
3. **The eight projects are ideas only.** The mechanisms worth taking are in
   the write-ups and in T-029 to T-033.

New entries: T-028 to T-034, in [`errand.md`](errand.md). T-019 is open, not
blocked: its backend is errand-box's to build in the fork. T-029, a P0, is
fixed in the fork. A host installed from v0.1.0 still runs errand v0.3.1,
which has it, until it takes a release with the fork; until then an operator
can keep it from reaching the key by giving `ERRAND_PROVIDER_BASE_URL` a
path, where the provider allows one: read, not tested.

---

## The next session's task on the Windows machine, mandated by the operator

The operator set it on 2026-10-23. On 2026-10-24 the operator ran a session
in a hosted container instead and deferred this task. It stands.

**Where it runs.** On the operator's own Windows machine, not in a hosted
container. Run the gate and the shell scripts in WSL2
([`docs/tests.md`](../docs/tests.md), "On a Windows machine"). The
operator gives access to three hosts at the start, and the `gh` CLI.

**The task is T-027** ([`setup.md`](setup.md)). Use errand-box as a user
would, on three separate hosts: `daytona`, `modelscope studio` and `wsl`.

1. On each host, follow [`docs/tests.md`](../docs/tests.md) section 5 as
   written: download the release binary with `bootstrap.sh` from the tag,
   run setup, then the live tests. Read only what a user reads.
2. Note every finding as it happens: what is broken, what fails, a QOL
   miss, a wrong or missing line in a page, a message that does not say
   what to do, anything that can be improved.
3. On each host that passes, run T-015, a real chat session, with the
   credentials the operator prepared (the operator's ruling: every host).
4. File every finding as its own entry, per
   [`authoring.md`](../docs/methodology/authoring.md), and each new host
   condition as an E-row. File; do not fix.
5. Present a summary to the operator: per host, what worked, what failed,
   and the entries filed.

Also in that session, with the operator's `gh` CLI:

- **Release v0.1.1** (the operator's ruling). Bump `Cargo.toml` to 0.1.1,
  run `cargo update -p errand-box` so `Cargo.lock` follows, move the
  `Unreleased` part of `CHANGELOG.md` under `## 0.1.1`, run the gate, push,
  wait for `gates`, then dispatch `release.yml` with `version` = `v0.1.1`.
  Section 5 of docs/tests.md installs `v0.1.1`, so release it first; then
  move `v=v0.1.0` in docs/operations.md to `v0.1.1`.
- **T-007's setting and secret**, by the commands in
  [`upkeep.md`](upkeep.md), T-007, "Decision". Check that no
  `FRESHNESS_TOKEN` exists before setting it.

---

## State

- **Last session:** started 2026-10-24T08:34:15Z, attended, in a hosted
  container, on the branch `claude/eloquent-faraday-nbo94y`. Research only:
  five sweeps and the recommendation above.
- **Tree:** clean after the record commit on that branch, pushed to that
  branch. `main` did not move; merging the branch is the operator's call.
- **Deployed:** released as
  [v0.1.0](https://github.com/Azathothas/errand-box/releases/tag/v0.1.0),
  the tag on `7363d89`. `main` is ahead of it, and v0.1.1 is not released.
  errand-box is installed nowhere.
- **CI:** on. `gates` passes on every push to `main` since `9348da9`. This
  session did not push to `main`.

## Baseline, as measured this session

Measure again with the same commands before you change anything.

| check | result |
| --- | --- |
| `sh scripts/ci-local.sh` | every step passed but `cross`, which exits 2: it built for aarch64, and this host has no `qemu-aarch64-static`, so no test ran there |
| `cargo test --locked` | 110 unit tests and 6 integration tests pass, on Rust 1.99.1 |
| `sh scripts/common/check-gate.sh --strict` | 9 of 9 checks pass with `references/` excluded; with the corpus visible to it, 6 fail on third-party text |
| shellcheck 0.11.0 | clean over the 18 shipped scripts, `scripts/fetch-tracker.sh` added |
| live tests | not run: this session changed no code that reaches a host |

The build host: a hosted container, kernel 6.18, x86_64, 4 CPUs, 16 GiB, no
Incus. The earlier hosts and their results are in git history.

**34 entries: 10 done, 6 partial, 1 blocked, 17 open.**

---

## What this session did

- **The corpus.** Ten repositories cloned at captured commits under
  `references/`, every agent instruction file deleted from them, and each
  tracker fetched by the new
  [`scripts/fetch-tracker.sh`](../scripts/fetch-tracker.sh), read-only.
- **Five sweeps**, run by research agents at the operator's instruction:
  errand's code, errand's tracker, two groups of credential brokers, and two
  agent runtimes. [`research.md`](../docs/methodology/research.md) section
  3.7 says not to hand a reference to a sub-agent; the operator's
  instruction in the session ruled. 15 of their citations were opened again
  at their lines, and all 15 hold.
- **Findings in this tree:** `src/relay.rs:62` and `:79` drop a connection
  with no log line when the relay cannot reach its target. T-019 cited a
  v0.3.1 line for errand's `main`; the correction is in the entry.

Candidates that a measurement refuted:

- A rewrite sheds errand's dependencies. Measured: serenity 0.12.5 with
  errand's features resolves to 150 crates and builds C code.
- errand-box can fix errand#48 and T-016 from outside errand. Read: the
  route match and the fixed text are in errand's code.

## What is in progress

T-028, the fork: imported, gated, and built into errand-box, which setup
links as `errand`. Open: the web bundle (T-034's ruling), the other crates'
notices, and a live run of setup on a host with Incus. T-029 is fixed in the
fork and reaches a host with that live run.

---

## Start here next session

1. T-029 in the fork, then T-028's shipping steps, so the fix reaches a host.
2. T-030, then T-031 to T-033.
3. On the Windows machine: the mandated task above.
4. T-020, then T-019's backend in the fork, which retires the `bailey`
   stand-in. Read the adopted mechanisms in the 2026-10-24 write-ups first.
5. T-024, then T-021, then T-022, which waits on T-020's link.
6. T-014, T-026, T-023, T-025, T-034.

---

## Open questions for the operator

1. **T-034**: keep or remove errand's web interface and its `errand threads`
   command. The recommendation is to keep both.
2. **Revoke the Daytona token** that was given to the session of
   2026-10-23: it is in a session transcript.
3. **Create the credentials** T-015 and T-007 list. Put each in a file,
   never in the chat.

---

## Settled, and not to be raised again

See [`RULES.md`](RULES.md) section 4: Incus system containers; errand as
errand-box's hard fork; root at setup; s6 from the distribution.
