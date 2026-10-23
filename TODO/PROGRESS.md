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

## The next session's task, mandated by the operator

The operator set it on 2026-10-23. Do it before anything in the work order.

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

- **Release v0.1.1** (the operator's ruling; this session ran out of time
  before it). Bump `Cargo.toml` to 0.1.1, run `cargo update -p errand-box`
  so `Cargo.lock` follows, move the `Unreleased` part of `CHANGELOG.md`
  under `## 0.1.1`, run the gate, push, wait for `gates`, then dispatch
  `release.yml` with `version` = `v0.1.1`. Section 5 of docs/tests.md
  installs `v0.1.1`, so release it first; then move `v=v0.1.0` in
  docs/operations.md to `v0.1.1`.
- **T-007's setting and secret**, by the commands in
  [`upkeep.md`](upkeep.md), T-007, "Decision". Check that no
  `FRESHNESS_TOKEN` exists before setting it.

---

## State

- **Last session:** started 2026-10-23T12:17Z, attended, ended
  2026-10-23T15:30Z. CI made green, v0.1.0 released, the host matrix; then
  the operator's rulings on every open question, four new entries, and the
  docs made current for a session on Windows.
- **Tree:** clean after the session's last commit on `main`, pushed.
- **Deployed:** released as
  [v0.1.0](https://github.com/Azathothas/errand-box/releases/tag/v0.1.0),
  the tag on `7363d89`. `main` is ahead of it, and v0.1.1 is not released.
  errand-box is installed nowhere: the session ran `errand-box uninstall
  --purge --yes` on the build host, a hosted container that is taken back,
  and Incus, its pools and the errand account stay there, as `uninstall`
  keeps them.
- **CI:** on. `gates` passes on every push since `9348da9`. `release`,
  `watchdog`, `freshness` and `live` have each run on GitHub, started by
  hand.

## Baseline, as measured this session

Measure again with the same commands before you change anything.

| check | result |
| --- | --- |
| `sh scripts/ci-local.sh` | every step passed at `ab65163`: gate, kill-by-name, shellcheck, fmt, clippy, test, pins, errand-config, musl, cross. On the record commit, every step passed but `cross`, which exits 2: the session removed its `binfmt_misc` handler at teardown, so only the unit tests ran under qemu, and they passed |
| `cargo test --locked` | 110 unit tests and 6 integration tests pass, on Rust 1.99.1 |
| shellcheck 0.11.0 and 0.9.0 | clean over the 17 shipped scripts |
| `actionlint` 1.7.7 | clean over the six workflows |
| `sh tests/live/live.sh` on the build host | 52 of 52, with v0.1.0 and with the tree at `bd619b7` |
| `tests/hosts/host.sh alpine alpine/3.22` with `EB_HOSTS_IMAGE` | live 50 PASS, 0 FAIL, 1 SKIP; after a restart 17 of 17 |
| `tests/hosts/host.sh deb13 debian/13` with `EB_HOSTS_IMAGE` | live 37 PASS, 2 FAIL (podman and Docker Hub's rate limit, since fixed, E-16), 1 SKIP; after a restart 17 of 17 |
| `.github/workflows/live.yml` | x86_64 40 of 41, aarch64 39 of 41: rootless podman (E-69, E-70, T-024); errand's service read while systemd restarts it (T-026) |

The build host: a hosted container, kernel 6.18, x86_64, 4 CPUs, 16 GiB,
cgroup v1, no init, no KVM, no AppArmor, egress through an HTTP proxy;
Incus 6.0.6 from Zabbly, errand v0.3.1, kage v0.5.0. The runners: Ubuntu
24.04, systemd, cgroup v2, AppArmor; Incus 7.5.1 from Zabbly's stable line.

**27 entries: 10 done, 4 partial, 2 blocked, 11 open.**

---

## What this session did

- **CI, the first release, the host matrix.** As in the git log and the
  entries: `gates` green, v0.1.0 by dispatch (T-009 done), the workflows run
  on GitHub (T-007), Alpine (T-012's musl half), Debian 13, and GitHub's
  runners on both architectures (T-013). Eleven defects fixed, E-66 to E-68
  among them.
- **errand's maintainer ruled on #41** at 15:02Z: errand takes an `incus`
  backend, with no proxy devices, a unix-socket broker link, one instance
  per launch, and a root incusd (`incus-admin`). T-019 and section 10 of
  [`docs/incus-backend.md`](../docs/incus-backend.md) record each ruling.
- **The operator's rulings**, each in its entry: T-019, keep the stand-in
  as the default and offer errand's backend as an opt-in reported as
  DEGRADED; T-020, a unix socket in a read-only directory; T-021, a secrets
  directory, read-only; T-022, a raw-TCP allowlist relayed by errand-box;
  T-015, on every host that passes; T-007, the next agent sets the setting
  and the secret; E-69, an opt-in key (T-024) and the research as T-025.
- **New entries:** T-024 (rootless podman, the opt-in key and the E-70
  fix), T-025 (an AppArmor route), T-026 (two live checks that depend on
  timing), T-027 (the mandated task above).
- **Docs for a Windows-hosted session:** docs/tests.md section 5, "As a
  user, on a host you are given", and its Windows notes; the README's
  install; SECURITY.md's GitHub token; two AGENTS.md rows.

Premises that a measurement disproved:

- Believed: CI is off. Measured: Actions was on, and `gates` failed at
  shellcheck on every push.
- Believed: a distrobuilder build records the digest errand-box computes.
  Measured: the build wrote the digest of the proxy URL.
- Believed: a nested Alpine host stops only at the image. Measured: past the
  image it stopped at errand's project (E-66), and `host.sh` reported PASS.
- Believed: `cross` runs every test under qemu. Measured: without a
  `binfmt_misc` handler the integration tests cannot run.

## What is in progress

Nothing in the tree. Not reached, and handed to the next session above: the
v0.1.1 release, and driving docs/tests.md section 5 from that tag.

---

## Start here next session

1. The mandated task above: T-027 on WSL2, Daytona and ModelScope Studio;
   T-015 on each host that passes; v0.1.1 first; T-007's setting and secret.
2. T-020, `TODO/sandbox.md`: the broker link without proxy devices. The
   ruling is made; measure the three points in its Decision first. The
   image changes (E-31).
3. T-024, `TODO/sandbox.md`: the opt-in key and the E-70 guest fix.
4. T-021, then T-022, which waits on T-020's link.
5. T-014: build the image in a workflow and publish it.
6. T-026, T-023, T-025, T-019's Part 1 step 1.

---

## Open questions for the operator

None open. Every question this session raised has a ruling, recorded in its
entry. Two things only the operator can do:

1. **Revoke the Daytona token** that was given to this session: it is in a
   session transcript. Give the next session a new one through the ssh
   config on the Windows machine
   ([`docs/tests.md`](../docs/tests.md) section 5).
2. **Create the credentials** T-015 and T-007 list: the Discord
   application, a capped model key, and the fine-grained `FRESHNESS_TOKEN`.
   [`SECURITY.md`](../SECURITY.md) lists each permission. Put each in a
   file, never in the chat.

---

## Settled, and not to be raised again

See [`RULES.md`](RULES.md) section 4: Incus system containers; errand
unchanged behind `bailey`; root at setup; s6 from the distribution.
