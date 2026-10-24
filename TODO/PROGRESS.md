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

## Decisions that wait for the operator

The session of 2026-10-24 researched one question the operator set: errand-box
stops depending on errand as an upstream, but still needs its chat layer. Is
it better to vendor errand and patch it, or to rewrite the chat layer? It also
read eight projects the operator named, as ideas or replacements. Five
write-ups hold the evidence, each listed in
[`docs/history/README.md`](../docs/history/README.md). The recommendation,
for the operator to rule on:

1. **Vendor errand at `715e9b0b777a` and prune it. Do not rewrite it, and do
   not take errand's releases.** Both errand write-ups reached this verdict
   independently. errand-box needs about 76% of errand's non-test lines. A
   rewrite on the same Discord library keeps about three quarters of the
   crates and a C compiler. The fixes for at least seven closed security
   reports come with the code. A native Incus backend in the vendored copy then retires
   the `bailey` stand-in.
2. **Fix errand#48 first, in the vendored copy.** A guest that holds its
   nonce can send the operator's provider key to a host it names, when the
   provider's base URL has no path. The same code is in the errand v0.3.1
   that `pins.toml` installs today. Read at both commits; no request was
   sent.
3. **None of the eight projects replaces errand, errand-box or the broker.**
   OpenShell and raxis need KVM or cgroup v2, and their sandboxes run no
   systemd or Docker. None has a chat layer. Adopt mechanisms only, each
   cited in its write-up, into T-020 to T-023.
4. **New entries to approve**, per
   [`authoring.md`](../docs/methodology/authoring.md): vendor errand; fix
   errand#48 and #54 item 1, then errand#52; the silent drop in
   `src/relay.rs:62` and `:79` when a relay cannot connect. T-016 and T-019
   change scope if 1 is approved.
5. **The corpus.** `references/` holds the ten clones and their trackers, 271
   MB with git data. It is too large for `main`. Push it to its own branch,
   without raxis's tree, which is SSPL-1.0? Until then it exists only in the
   hosted container, excluded by `.git/info/exclude`, and each
   `PROVENANCE.md` gives the command that fetches it again.

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

**27 entries: 10 done, 4 partial, 2 blocked, 11 open.**

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

Nothing in the tree. The recommendation above waits for the operator's
ruling, and no entry is filed until the operator approves it.

---

## Start here next session

1. If the operator ruled on the decisions above: file the approved entries,
   then start with errand#48.
2. On the Windows machine: the mandated task above.
3. T-020, `TODO/sandbox.md`: the broker link without proxy devices. Read the
   adopted mechanisms in the 2026-10-24 write-ups first.
4. T-024, then T-021, then T-022, which waits on T-020's link.
5. T-014, T-026, T-023, T-025.

---

## Open questions for the operator

The five decisions at the top of this file. Two things only the operator can
do are unchanged:

1. **Revoke the Daytona token** that was given to the session of
   2026-10-23: it is in a session transcript.
2. **Create the credentials** T-015 and T-007 list. Put each in a file,
   never in the chat.

---

## Settled, and not to be raised again

See [`RULES.md`](RULES.md) section 4: Incus system containers; errand
unchanged behind `bailey`; root at setup; s6 from the distribution.
Decision 1 above, if the operator approves it, replaces the second.
