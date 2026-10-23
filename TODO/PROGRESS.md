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

## State

- **Last session:** started 2026-10-23T12:17Z, attended: CI made green on
  GitHub, the first release, and the host matrix (Alpine, Debian 13, and
  GitHub's runners for x86_64 and aarch64).
- **Tree:** clean after the session's last commit on `main`, pushed.
- **Deployed:** released as
  [v0.1.0](https://github.com/Azathothas/errand-box/releases/tag/v0.1.0),
  the tag on `7363d89`. `main` is ahead of it: the fixes below are
  unreleased. errand-box is installed nowhere: the session ran `errand-box
  uninstall --purge --yes` on the build host, a hosted container that is
  taken back, and Incus, its pools and the errand account stay there, as
  `uninstall` keeps them.
- **CI:** on. `gates` passes on every push since `9348da9`. `release`,
  `watchdog`, `freshness` and `live` have each run on GitHub, started by
  hand.

## Baseline, as measured this session

Measure again with the same commands before you change anything.

| check | result |
| --- | --- |
| `sh scripts/ci-local.sh` | every step passed: gate, kill-by-name, shellcheck, fmt, clippy, test, pins, errand-config, musl, cross |
| `cargo test --locked` | 110 unit tests and 6 integration tests pass, on Rust 1.99.1 |
| `sh scripts/ci-local.sh --step cross` | 110 + 6 under `qemu-aarch64-static` with a `binfmt_misc` handler; without one, the unit tests and exit 2 |
| shellcheck 0.11.0 and 0.9.0 | clean over the 17 shipped scripts |
| `actionlint` 1.7.7 | clean over the six workflows |
| `gates` on GitHub | 15 of 15 steps and both `musl` jobs |
| `sh tests/live/live.sh` on the build host | 52 of 52, with v0.1.0 and with the tree at `bd619b7` |
| `tests/hosts/host.sh alpine alpine/3.22` with `EB_HOSTS_IMAGE` | live 50 PASS, 0 FAIL, 1 SKIP; after a restart 17 of 17 |
| `tests/hosts/host.sh deb13 debian/13` with `EB_HOSTS_IMAGE` | live 37 PASS, 2 FAIL (podman and Docker Hub's rate limit, since fixed, E-16), 1 SKIP; after a restart 17 of 17 |
| `.github/workflows/live.yml` | x86_64 40 of 41, aarch64 39 of 41: rootless podman (E-69, E-70); errand's service read while systemd restarts it |

The build host: a hosted container, kernel 6.18, x86_64, 4 CPUs, 16 GiB,
cgroup v1, no init, no KVM, no AppArmor, egress through an HTTP proxy;
Incus 6.0.6 from Zabbly, errand v0.3.1, kage v0.5.0. The runners: Ubuntu
24.04, systemd, cgroup v2, AppArmor; Incus 7.5.1 from Zabbly's stable line.

**23 entries: 10 done, 4 partial, 2 blocked, 7 open.**

---

## What this session did

- **CI.** `gates` had failed on every push: the runner moved to Ubuntu 26.04
  and shellcheck 0.11. The live tests and the host steps were called as
  `"t_$name"`, so shellcheck had checked none of their bodies; called by
  name, they held 17 more findings, and one was a check that could not
  fail. The `errand-config` step, which CI had never reached, failed on
  `XDG_CONFIG_HOME`. `cross` now exits 2 when it runs less than every test.
- **The first release (T-009, done).** The session's git route pushes `main`
  and refuses a tag, so `release.yml` also runs on a dispatch on `main`,
  which creates the tag. v0.1.0 is published; its digests check, and
  `bootstrap.sh` from the tag installs it on a host with nothing on it.
- **The workflows (T-007).** `watchdog` and `freshness` ran on GitHub: every
  upstream answers, and errand, kage and Rust are current.
- **The host matrix.** Alpine passes the live tests (T-012, the musl half),
  Debian 13 nearly does (T-011), and GitHub's runners run setup and the live
  tests on x86_64 and on aarch64 (T-013, new `live.yml`).
- **Defects found and fixed:** E-32 on the run that makes the pool;
  `build-image.sh` wrote the wrong inputs digest (T-014); E-66, Alpine's
  `incus-user` package; E-67, errand's group on busybox; E-68, a late
  incusd under systemd; E-16, podman had no registry mirror; and in the
  test scripts, a setup check that let `errand project` fail, and summary
  lines read by position.
- **Found and not fixed:** rootless podman on the runners, E-69 and E-70,
  each measured. errand's service check on systemd depends on timing while
  the test credentials stop errand (T-015). `reap` read a stopped instance
  once on aarch64, in a run where it passed before.

Premises that a measurement disproved:

- Believed: CI is off. Measured: Actions was on, and `gates` failed at
  shellcheck on every push.
- Believed: a distrobuilder build records the digest errand-box computes; a
  unit test said so. Measured: the build wrote the digest of the proxy URL;
  the test ran a path that exited before the bug.
- Believed: a nested Alpine host stops only at the image. Measured: past the
  image it stopped at errand's project (E-66), and `host.sh` reported PASS.
- Believed: `cross` runs every test under qemu. Measured: on a host with no
  `binfmt_misc` handler the integration tests cannot run, and the step
  passed anyway.

## What is in progress

Nothing. T-007, T-012, T-013 and T-014 are partial; each entry names its
open half.

---

## Start here next session

1. T-020, `TODO/sandbox.md`: a broker link without proxy devices. The
   highest open item. Decide first where the unix socket lives, because the
   guest can write the session's state directory; then the socket's mode
   across the isolated uid map, because guest services run as the guest's
   root; and what relays inside the guest. The image changes (E-31).
2. Rootless podman on hosts with AppArmor's restriction, or with cgroup v2
   (E-69, E-70): the operator decides whether to file it as an entry, and
   which fix (the open question below).
3. T-014, `TODO/sandbox.md`: build the image in a workflow and publish it.
   The entry records how distrobuilder builds here.
4. T-015, `TODO/sandbox.md`: when the operator has given the chat and
   gateway credentials, a real session.
5. T-021 and T-022, `TODO/sandbox.md`, resolved together.
6. T-023, `TODO/setup.md`: the observability sampler and session traces.
7. T-012 and T-007: WSL2, and a pull request from `freshness`.

---

## Open questions for the operator

1. **Credentials for T-015.** A Discord bot token, a channel id and a user
   id; a model gateway key and base URL. [`SECURITY.md`](../SECURITY.md)
   lists the permissions. Put them in the environment's secrets, not in a
   chat.
2. **Pull requests from Actions.** Set "Allow GitHub Actions to create and
   approve pull requests", and add `FRESHNESS_TOKEN` so that CI runs on the
   pull requests `freshness` opens. Nothing here can read whether it is set.
3. **Rootless podman under AppArmor (E-69).** Three routes, each a trade:
   setup sets `kernel.apparmor_restrict_unprivileged_userns=0`, which turns
   a host hardening off for every process; an AppArmor profile that allows
   user namespaces for Incus guests, which a restricted project may refuse;
   or report it as degraded and leave rootless podman off on such hosts.
   Recommendation: the profile, measured first; E-70 needs a guest change
   either way.
4. **The broker's proxy devices (E-61, T-020).** Unchanged: until T-020,
   SECURITY.md states the gap.
5. **The second host.** The Daytona host was not reachable: this
   environment's network policy refuses outbound TCP 22, and port 443 of
   Daytona's ssh gateway, the one other port tried, sent no ssh banner.
   A session that may reach
   `ssh.app.daytona.io` on port 22 can use it. Revoke the token that was
   given, because it is in a session transcript.

---

## Settled, and not to be raised again

See [`RULES.md`](RULES.md) section 4: Incus system containers; errand
unchanged behind `bailey`; root at setup; s6 from the distribution.
