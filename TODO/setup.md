# Setup and the host

Entries about `errand-box setup`, `up`, the supervisor and the hosts they
run on. [INDEX.md](INDEX.md) lists every entry.

---

# T-001: One command sets up a host

**Source:** the operator (next-session.md section 2 of the agent-box research)
**Category:** setup
**Priority:** P0
**Effort:** L
**Status:** done

## Problem

A host needs Incus, a storage pool, a guest image, kage, errand, errand's
config and a supervisor before errand can start one session. By hand that
is an afternoon, and each host differs.

## Premise

Measured: `errand-box setup --config F --env F --creds F` does all of it on
a host with no init and cgroup v1, and a second run changes nothing.

## Approach

`src/setup.rs` runs each step and reports `ok`, `changed`, `DEGRADED`,
`FAILED` or `skipped`. Each step checks before it acts.

## Prove

```sh
errand-box setup --config examples/errand-box.toml --env E --creds C; echo "exit $?"
errand-box setup --config examples/errand-box.toml --env E --creds C | tail -n 1
```

Exit 0 both times, and the second run says `0 changed`.

## Closing

**Closed 2026-10-23T08:40Z.** Built and run on the build host (Ubuntu
24.04, no init, cgroup v1). Twelve live runs while the code changed; the
last one rebuilt the image because its inputs had changed (E-31):

```text
guest image              changed   errand-box/debian-13 was built from other inputs; rebuilding (E-31)
guest image              changed   built errand-box/debian-13 in 220 s
golden instance          changed   eb-golden from errand-box/debian-13 (never started)
6 changed, 30 unchanged, 5 degraded, 0 failed, 0 skipped
exit 0
```

then twice with the static release binary
(`target/x86_64-unknown-linux-musl/release/errand-box`, Rust 1.99.1): the
first run installs it, the second changes nothing.

```text
1 changed, 32 unchanged, 5 degraded, 0 failed, 0 skipped
exit 0
0 changed, 33 unchanged, 5 degraded, 0 failed, 0 skipped
exit 0
```

---

# T-003: The daemons run, and come back, on a host with no init

**Source:** the operator; E-01 and E-02 in docs/edge-cases.md
**Category:** setup
**Priority:** P0
**Effort:** M
**Status:** done

## Problem

On a host whose PID 1 is not an init, nothing starts incusd or restarts it,
and a restart loses every daemon, device mode and kernel setting.

## Premise

Measured: s6 from the distribution supervises the daemons, and `errand-box
up` restores all of it. Never signal a daemon by name (E-12).

## Approach

`src/up.rs`: device modes, sysctls, the proxy setting, one s6 service per
daemon with a bounded logger, a check for a daemon that restarts in a loop,
and a second pass when s6-svscan exits under the first (E-33).

## Prove

```sh
sh tests/live/live.sh up
```

Every check PASS, exit 0.

## Closing

**Closed 2026-10-23T08:40Z.** The first live run found that `up` trusted a
zombie s6-svscan and started nothing (E-33); the fix treats a zombie as
dead and runs a second pass. After it:

```text
PASS up: daemons stopped through s6-svscanctl, as a restart loses them
PASS up: errand-box up exit 0 in 5617 ms
PASS up: incusd answers
PASS up: /dev/fuse mode 666 (E-03)
PASS up: service incusd: up (pid 5155) 5 seconds
PASS up: a second up changes nothing: 0 changed, 12 unchanged, 1 degraded, 0 failed, 0 skipped
```

---

# T-006: The config setup writes is one the pinned errand accepts

**Source:** found by the live `errand` check, 2026-10-23
**Category:** setup
**Priority:** P1
**Effort:** S
**Status:** done

## Problem

errand v0.3.1 refused the config and its supervisor restarted it every
second: "output.postImages is not a setting". The generator was written
against a later errand.

## Premise

Measured: errand checks its config before anything else and logs
"egress is brokered" once the config is accepted.

## Approach

`src/errandcfg.rs` no longer writes the key. `errand-box errand-config`
writes the config alone, and `tests/errand-config-test.sh` runs the pinned
errand against it. The gate runs it as the step `errand-config`, so a pin
bump that changes errand's schema fails before it reaches a host.

## Prove

```sh
sh scripts/ci-local.sh --step errand-config
```

## Closing

**Closed 2026-10-23T08:40Z.** With the key put back, the check fails; without
it, it passes:

```text
FAIL errand v0.3.1 refuses the config errand-box writes:
... ConfigError: configuration rejected: - output.postImages is not a setting; check the spelling
PASS errand v0.3.1 accepts the config errand-box writes
```

---

# T-008: Bounded logs, and a daemon that restarts in a loop is reported

**Source:** found while closing T-006
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** done

## Problem

All daemons wrote to one unbounded file, and `up` reported a daemon that
restarted every second as `up (pid N) 0 seconds`, which reads as healthy.

## Approach

`src/supervise.rs` `s6_log_run`: each daemon logs through `s6-log` to
`/var/log/errand-box/<daemon>/`, ten files of 1 MB. `src/up.rs` looks at a
daemon twice, 2.5 s apart, and reports a changed pid as FAILED with the last
log line.

## Prove

```sh
cargo test --locked supervise
sh tests/live/live.sh up
```

## Closing

**Closed 2026-10-23T08:40Z.** `supervise::tests::s6_log_is_bounded` and
`supervise::tests::svstat_lines` pass; `ps` on the host shows one `s6-log -b
n10 s1000000 T /var/log/errand-box/<daemon>` per daemon, and the live `up`
test passes.

---

# T-010: Setup without root at run time

**Source:** docs/research/rootless.md section 3, definition A, rank 1
**Category:** setup
**Priority:** P2
**Effort:** L
**Status:** open

## Problem

errand-box needs a root incusd. Some operators want no root process at run
time.

## Premise

Measured in the research: incusd as root of a user namespace runs guests at
runc speed with every feature but the disk quota; new instances take 1 to
3.5 s; the broker needs a unix-socket relay, which the shim already has
(`broker_link = "unix"`, E-23). On cgroup v1 with no init, a root step at
each boot makes the delegated cgroups.

## Approach

`engine.mode = "rootless"` in `src/config.rs`; a setup path that writes the
private subuid file, the delegated cgroups and a small init as PID 1 of the
namespace (research/experiments/rootless/), and records `incus_dir` and
`broker_link = "unix"` in runtime.toml.

## Prove

```sh
sh tests/live/live.sh   # with engine.mode = "rootless"; quota reports DEGRADED (E-24)
```

---

# T-011: Live test on a host with systemd

**Source:** this session; the build host has no init
**Category:** setup
**Priority:** P1
**Effort:** M
**Status:** done

## Problem

`systemd_up` in `src/up.rs` writes units and enables the packaged ones. No
live run had covered it.

## Done

2026-10-23, Ubuntu 24.04 as a nested host with systemd as PID 1
([tests/hosts/host.sh](../tests/hosts/host.sh)): bootstrap, setup twice
(idempotent but for errand's dummy credentials, T-015), the live tests (39
PASS, the quota test skipped on a nested dir pool), a container restart with
systemd bringing the sandbox back on its own, and a probe and a session
after the restart (17 PASS). The run drove out real defects on systemd,
each with an edge-case row and a fix: E-34 (start order), E-37 (the
packaged lxcfs unit skipped in a container, and stopping it stopping
incusd), E-13 (incusd's proxy through `core.proxy_*`), and the unit settle
check.

2026-10-23T13:59Z, Debian 13 as a nested host with systemd, importing a
distrobuilder build (`EB_HOSTS_IMAGE`): setup twice, the second changing
nothing; the live tests 37 PASS, 2 FAIL, 1 SKIP; a restart with systemd
bringing the sandbox back; a probe and a session after it, 17 of 17. The two
FAIL lines were podman pulls refused by Docker Hub's rate limit, which drove
out the podman half of E-16; with the fix, the build host passes the live
tests, 52 of 52. The first setup drove out E-68: incusd's first start waited
2 minutes for network-online.target, past the deadline of `incus version`.
[tests/live/2026-10-23-debian-13.txt](../tests/live/2026-10-23-debian-13.txt)
is the output.

## Prove

```sh
sh tests/hosts/host.sh u2404 ubuntu/24.04
```

Every check PASS (errand itself needs real credentials, T-015).

---

# T-012: Live test on musl and on WSL2

**Source:** the operator: "any Linux host (glibc, musl, WSL2)"
**Category:** setup
**Priority:** P1
**Effort:** M
**Status:** partial

## Problem

`src/setup.rs` has an apk branch for Alpine, and `src/host.rs` detects
WSL2. WSL2 has not run on such a host.

## Done, and open

2026-10-23, Alpine 3.22 as a nested host (musl, OpenRC, busybox): setup
installs Incus and s6, creates the `incus-admin` and `incus` groups the
package does not (E-35), writes the OpenRC boot hook (E-36), and `errand-box
up` brings every daemon back after a restart. The run drove out E-35 and
E-62 (newer `s6-svstat` prints `pid N pgid N`, which the crash-loop check
misread). A nested host cannot build the guest image, because that launches
a third level of container nesting that the test host's kernel refuses
(`forkstart` exit 1).

2026-10-23T13:35Z, musl done: Alpine 3.22.6 (musl, OpenRC, busybox) as a
nested host that imports a distrobuilder build (`EB_HOSTS_IMAGE`, T-014)
instead of building one. Setup exit 0, and the second run changed nothing;
`tests/live/live.sh` 50 PASS, 0 FAIL, 1 SKIP (the quota test on a nested
`dir` pool, E-05); after a container restart `up` brought every daemon back,
and a probe and a session passed 17 of 17.
[tests/live/2026-10-23-alpine-3.22.txt](../tests/live/2026-10-23-alpine-3.22.txt)
is the output. The first run past the image drove out E-66 (Alpine packages
`incus-user` apart, so errand reached no Incus socket) and E-67 (busybox
put the errand account in the shared group `nogroup`), and showed that
`host.sh` allowed every FAILED row whose name starts with "errand".

Open: WSL2 is untouched. T-027's run covers it.

## Measured on WSL2, 2026-10-24

T-027's run: WSL2 Ubuntu 24.04 with systemd as PID 1 (cgroup v2, Incus
7.5.1): live.sh 41 PASS and 1 FAIL, the one being rootless podman as the
agent (E-70, T-024). WSL2 Arch with no init: no guest starts, because PID
1's cgroup lies outside the distribution's namespace (T-039). WSL2 also
stops an idle distribution (T-038). Open: WSL2 without systemd, and the
E-70 failure.

## Prove

```sh
sh tests/hosts/host.sh alpine alpine/3.22
sh tests/live/live.sh   # on a real Alpine host, and on WSL2 with Ubuntu
```

All PASS.

---

# T-013: Live test on aarch64

**Source:** this session; pins.toml carries aarch64 assets that no run used
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** partial

## Done, and open

The `cross` step of [scripts/ci-local.sh](../scripts/ci-local.sh) builds the
static binary for aarch64 with `rust-lld` and runs every unit and
integration test there under `qemu-aarch64-static` (110 + 6 pass, with a
`binfmt_misc` handler for the integration tests).

2026-10-23T14:24Z, a real aarch64 host:
[.github/workflows/live.yml](../.github/workflows/live.yml) on GitHub's
`ubuntu-24.04-arm` runner, beside `ubuntu-24.04` for x86_64 (run
39968392861). Both are systemd hosts with cgroup v2, so setup took Incus
7.5.1 from the stable line, the first live run of that line. Setup passed
but for errand's daemon, which the test credentials stop (T-015), and the
second run changed nothing. The live tests: x86_64 40 of 41, aarch64 39 of
41.
[tests/live/2026-10-23-github-runners.txt](../tests/live/2026-10-23-github-runners.txt)
is the output.

Open, on both architectures: rootless podman as the agent fails with
"cannot re-exec process". Measured in run 39970746432, two causes, one
behind the other. AppArmor denies the agent's user namespace
(`operation="userns_create"`, with `kernel.apparmor_restrict_unprivileged_userns
= 1` on the runners; E-69). With the restriction off on the runner,
podman gets past it and stops at "dbus: couldn't determine address of
session bus": on cgroup v2 it asks systemd for a cgroup, and the agent has
no user session (E-70). On cgroup v1 the guest runs podman without cgroups
(E-04), which is why every cgroup v1 host passes. Also open, on aarch64:
`errand: service errand` read `activating`, because systemd restarts the
daemon the test credentials stop, so that check depends on timing until
T-015 gives real credentials.

## Prove

```sh
sh scripts/ci-local.sh --step cross   # done
sh tests/live/live.sh                 # open: on an aarch64 host
```

All PASS.

---

# T-018: Choose a pool size that fits the host disk

**Source:** E-32, found 2026-10-23: a 20 GiB pool on a disk with 14 GiB free
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** done

## Problem

`pool_size` in `src/setup.rs` made a pool of at least 20 GiB, even when the
host disk had less free space. When the disk fills, every guest gets I/O
errors.

## Done

setup reports the condition as DEGRADED with both numbers (E-32), the auto
size is half the free space with a floor of three times `box.disk` (so one
session cannot fill the pool), and `errand-box status` reports the pool and
host-disk use, degraded at 85 % and failed at 95 %.

## Prove

```sh
cargo test --locked setup
errand-box status
```

---

# T-023: Observability: metrics over time, and session traces

**Source:** the operator, 2026-10-23: "best-in-class observability/tracing
logs ... beyond errand/kage, something like system-info-collector"
**Category:** setup
**Priority:** P2
**Effort:** L
**Status:** open

## Problem

errand-box has operational logging and a point-in-time snapshot, nothing
over time. It ships: bounded per-daemon s6 logs
(`/var/log/errand-box/<daemon>/current`, `src/supervise.rs`), `errand-box
logs` (`src/ops.rs`), journald for the units on systemd, and `errand-box
status` (daemons, pool %, host-disk %, image, golden, session counts,
`src/status.rs`). A grep of `src/` for `metric|telemetry|tracing|otel|
prometheus|statsd|/proc/stat|cgroup.*stat` returns nothing: no time series,
no per-session resource accounting, no tracing, no export. An operator
cannot answer "what did this session use, and when" or "what was the pool
doing an hour ago."

## Premise

READ, not yet built: each session is a cgroup-delegated Incus instance, so
per-session CPU, memory, pids and io are readable from `/sys/fs/cgroup` and
the Incus API without errand's help, and `maint` already runs on a loop
(`src/maint.rs`) that could sample. The host daemons' state is already
computed by `status`.

## Approach

A sampler (in `maint`, or its own supervised service) writes a bounded
time series of host and per-session resource use - the shape
[system-info-collector](https://github.com/qarmin/system-info-collector)
takes, CSV or a small on-disk store, not a daemon to operate - and a
read command renders or exports it (`errand-box status --watch`, or an
export for a TSDB the operator already runs). Session lifecycle events
(start, stop, reap, gc, image rebuild) become structured records a trace can
be built from. Decide the retention and the on-disk budget the way the logs
are bounded (E-08-style: never fill the disk).

Safety: the sampler records resource numbers and lifecycle events, never the
contents of a session, its environment or its files; a trace must not become
a place secrets land (T-021).

## Prove

```sh
# after some session activity, the series shows per-session CPU and memory
# over time and the lifecycle events, bounded on disk, with no secret in it
errand-box status --watch   # or the chosen read/export path
```

---

# T-027: Use errand-box as a user does, on three hosts, and file what it finds

**Source:** the operator, 2026-10-23: "it must use our errand-box like a user
would on 3 separate environments (I will provide it access), `daytona`,
`modelscope studio` & `wsl` - it will then note down all the issues, QOL
misses, things that are broken or can be improved, it will file them as
additional task entries and then it will present a summary to me"
**Category:** setup
**Priority:** P1
**Effort:** M
**Status:** done

## Problem

Every run so far drove errand-box from its source tree, as its developer
does: `tests/hosts/host.sh` builds the binary and copies it in, and
`tests/live/live.sh` runs from the checkout. No run has fetched
`bootstrap.sh` from a release tag, followed the README and
[docs/operations.md](../docs/operations.md), written the three inputs, and
run setup on a host somebody was given. The three hosts the operator uses
have never run errand-box: Daytona, a hosted sandbox reached by ssh;
ModelScope Studio, a hosted notebook environment; and WSL2 on Windows.

## Premise

Nothing about the three hosts is measured. Whether each gives root, user
namespaces, `/dev/fuse`, a writable `/proc/sys`, nested containers and the
hosts setup fetches from is what the run finds out. `errand-box doctor`
changes nothing, so it runs first on each.

## Approach

The operator, 2026-10-24, replaced the plan of 2026-10-23: a standalone
agent on the operator's Windows machine runs a runbook the operator gives
it, outside the tree. It reads the code and does not change it.

1. On WSL2, ModelScope Studio (ssh) and Daytona (ssh), it installs
   errand-box as a user does, from v0.1.1 and from the prerelease
   `artefacts` (the guest image, discord-mock and the e2e kit), and tests
   every command and setting a user can reach, errand's chat path against
   discord-mock with `tests/e2e/mock-session.sh` included.
2. It files each finding as an issue on this repository, with the
   commands, the output, the code it traced the cause to, and a fix.
3. It files an entry per issue, per
   [authoring.md](../docs/methodology/authoring.md), that links it, runs
   the record's checks, and pushes. It builds nothing.
4. A later session works the issues.

## Pitfalls

- An ssh token is a credential. It stays in the operator's files, never in
  the tree, an issue, a log or the chat.
- `wsl.exe` writes UTF-16 ([conventions/shell.md](../docs/conventions/shell.md)
  section 7).
- A hosted container may refuse nested containers outright. That is a
  finding with its measurement, not a reason to stop the other hosts.

## Done

2026-10-24, run by a standalone agent on the operator's Windows machine,
from v0.1.1 and the prerelease `artefacts` (kit `COMMIT` 9ba63c5), on five
hosts: WSL2 Ubuntu 24.04 with systemd, WSL2 Arch with no init, a second
WSL2 Ubuntu 24.04 for the upgrade, ModelScope Studio, and Daytona.

| host | result |
| --- | --- |
| WSL2 Ubuntu 24.04, systemd | setup 0 failed (image built in 1894 s); live.sh 41 PASS, 1 FAIL (rootless podman, E-70); mock-session PASS; a real model turn through a custom gateway with a workaround (T-041); uninstall needs a reinstall to purge (T-046) |
| WSL2 Ubuntu 24.04, the upgrade | v0.1.0 then v0.1.1: the downloaded errand removed, `web` dropped, the unit runs `errand-box serve` |
| WSL2 Arch, no init | needs Incus from pacman and s6 from source (T-057); no guest starts (T-039) |
| ModelScope Studio | an unprivileged container: Incus cannot run, and nothing says so (T-044) |
| Daytona | setup completes with a `dir` pool and the imported image; no session instance starts (T-042); lxcfs dies for good on a restart (T-045) |

Every finding is an issue on this repository and an entry here, T-038 to
T-078, issues #2 to #42. Evidence for existing entries went to T-012,
T-024, T-036 and T-037. The run's prove, run 2026-10-24:
`gh issue list --repo Azathothas/errand-box --state all --limit 200`
lists the 41 issues, and `sh scripts/check-todo.sh` exits 0.

Not tested: a real chat service (T-015 still waits on credentials), the
idle timeout of a session, and Pass B on WSL2 Ubuntu with no systemd (the
Arch host showed the no-init outcome).

## Prove

```sh
gh issue list --repo Azathothas/errand-box --state all --limit 200
sh scripts/check-todo.sh
```

Every finding has an issue and an entry that links it.

---

# T-036: `uninstall` stops incusd under s6 and leaves its dnsmasq running

**Source:** the teardown of 2026-10-24, on the session's hosted container
(no init, s6, Incus 6.0.7)
**Category:** setup
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

After `errand-box uninstall --purge --yes` reported every daemon stopped,
three `dnsmasq` processes still ran as the account `incus`, one per Incus
bridge (`incusbr-0`, `incusbr-994`, `incusbr0`), and the bridges stayed up.
The account `incus` could not be deleted while they ran. `uninstall` keeps
Incus by design, but these processes outlive the incusd that started them.

## Premise

Measured once, by `ps -u incus` after the uninstall. Read, not measured:
under systemd, stopping `incus.service` ends every process in its cgroup;
under s6, a signal to incusd does not reach the dnsmasq it forked, so they
survive. Whether `up` after a host restart meets a stale dnsmasq was not
measured; the live tests, which stop and start the daemons through s6,
passed 52 of 52 on the same host.

## Approach

Measure first: on a host with no init, stop incusd through s6 and list the
processes of the account `incus`. Then either stop incusd with `incus admin
shutdown` before the supervisor's stop, which takes the networks down, or
have `uninstall` stop the remaining processes of the account `incus` by pid.
Never by name (E-12).

## Measured since, 2026-10-24

T-027's run, `uninstall --purge --yes` under s6: on WSL2 Arch (Incus
7.5.1) three dnsmasq processes of the account incus were left after every
daemon stopped, as the Problem says; on Daytona and ModelScope Studio none
were (no bridge, or incusd took them with it). Under systemd incusd is kept
running by design, with its dnsmasq.

## Prove

```sh
errand-box uninstall --purge --yes
ps -u incus -o pid= | wc -l   # 0
```

---

# T-037: errand's refusals restart every two seconds, and a transient one looks the same

**Source:** found on 2026-10-24 while documenting errand's service under
errand-box; the operator approved the entry the same day
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

errand exits 2, 3 or 4 when it refuses to run: the configuration, the
sandbox backend or the token refused (2), a guarantee it cannot enforce (3),
another daemon on the same state directory (4)
(`errand/src/serve.rs:515`). errand-box's supervisor restarts it after
every exit. Under systemd the unit says `Restart=always` and `RestartSec=2`
(`src/supervise.rs:162`); under s6 there is no `finish` script, so
s6-supervise starts it again at once, at most once a second. A refusal
whose cause has not changed is repeated forever, and the log fills with the
same lines: errand v0.3.1 refused its config and restarted every second
(T-006), and the hosts of `tests/hosts/host.sh` show errand crash-looping
on test credentials (docs/tests.md section 4).

Not restarting is not the fix. A sandbox that does not answer yet is also
exit 2 (`StartError::Unavailable`, `errand/src/serve.rs:818`), which is the
race at boot that E-34 describes, and it passes on its own.

`status` reports a daemon up for under a minute as degraded "may be
restarting" (`src/status.rs:129`), but does not say that it is refusing, or
why.

## Premise

Read, not measured: the systemd unit and the missing s6 `finish` script
(`src/supervise.rs`), and the exit codes. The restart rate under s6 is
s6-supervise's documented minimum of one second between starts; the
"every second" of v0.3.1 matches it.

## Decision

Approved by the operator on 2026-10-24: keep restarting, back off on 2, 3
and 4, and say so in `status`.

- **One policy for both inits, in errand-box.** Each exit is recorded: the
  s6 `finish` script gets the code as `$1`, systemd's `ExecStopPost=` gets
  `$EXIT_STATUS`; each writes it with a time to `/run/errand-box/errand.exit`
  through `errand-box __exited errand CODE`. The service's existing pre-step
  (`prepare`, `src/up.rs:195`) waits before the next start: 2 s, doubled for
  each refusal in a row, at most 60 s. A run that stays up for `STEADY`
  (`src/up.rs:740`) resets the count. Exit 0 and 1 are not refusals and
  keep today's restart.
- Lost: `RestartPreventExitStatus=2 3 4`, because exit 2 is also the
  transient race of E-34; systemd's `RestartSteps=` and
  `RestartMaxDelaySec=`, because they exist from systemd 254 only (Debian
  12 has 252) and do nothing under s6.
- **`status` says it.** A service whose last exit was a refusal reports
  "refusing (exit N), next start in S s" and the last line of its log.

## Approach

1. `src/supervise.rs`: the s6 service directory gets a `finish` script; the
   systemd unit gets `ExecStopPost=`. Both call `errand-box __exited`.
2. `src/up.rs`: the errand service's `prepare` gets the wait; the policy is
   a pure function of the recorded exits, unit-tested.
3. `src/status.rs`: the refusing line.
4. `tests/live/live.sh`: a `refusal` test that writes a config errand
   refuses, runs `up`, and counts starts over 60 s.

## Measured since, 2026-10-24

T-027's run, errand refusing its inputs: under systemd on WSL2 Ubuntu
24.04, 26 restarts in 2 minutes and 88 by the end ("Sent invalid
authentication", exit 2); under s6 on Daytona, one restart every 31 s
(errand waits 30 s for the chat before it exits). The same loop without a
backoff hit two other services under s6 on ModelScope Studio: lxcfs started
556 times in 9 minutes and incus-user once a second. What setup and status
report about it is T-050.

## Prove

```sh
cargo test --locked supervise:: up:: status::
sh tests/live/live.sh refusal   # at most 6 starts in 60 s; status says refusing (exit 2)
```

---

# T-038: WSL2 stops an idle distribution, and errand with it

**Source:** [issue #2](https://github.com/Azathothas/errand-box/issues/2), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P1
**Effort:** M
**Status:** open

## Problem

On WSL2 the distribution stops about 15 to 20 seconds after the last WSL
client exits, systemd or not. incusd, errand and maint stop with it, so
errand leaves the chat whenever the user closes the terminal. Nothing in
setup, doctor or the pages says so; the README lists WSL2 as a host.

## Premise

Measured on WSL2 (Ubuntu 24.04, systemd PID 1, errand-box set up): Stopped
17 s after the last client; on the next access every unit came back.
Read: `src/host.rs:74` detects WSL and nothing else uses it.

## Approach

1. A DEGRADED row on WSL in setup and doctor, with the keep-alive to use.
2. docs/operations.md "After a restart" gains a WSL paragraph: a logon task
   that holds `wsl -d DISTRO --exec sleep infinity`, or WSL's own idle
   setting where the installed WSL has one.
3. An E-row.

## Prove

```sh
errand-box doctor | grep -i wsl   # the row names the idle stop and the keep-alive
```

On a WSL2 host with the keep-alive documented: close every terminal, wait
60 s, and `errand-box status` shows errand's unit with its old start time.

---

# T-039: WSL2 without systemd cannot start a guest, and setup says all is well

**Source:** [issue #3](https://github.com/Azathothas/errand-box/issues/3), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P1
**Effort:** S
**Status:** open

## Problem

In a WSL2 distribution with no systemd, LXC cannot set up cgroups, so no
Incus container starts (`cgfsng ... Invalid cross-device link`). setup ends
`0 failed`, `bailey doctor` passes and errand's probe says the backend
enforces every guarantee; every session then fails.

## Premise

Measured on WSL2 (Arch, no init, errand-box's s6): PID 1 is WSL's init with
cgroup `0::/../..`, outside the distribution's cgroup namespace; a plain
`incus launch` fails the same way; moving incusd into its own cgroup did not
help. live.sh: every test that starts an instance failed.

## Approach

1. Detect it: a `..` in `/proc/1/cgroup` (or WSL with no systemd on cgroup v2).
2. doctor and setup refuse with the fix: `[boot] systemd=true` in
   /etc/wsl.conf, restart the distribution, run setup again.
3. README's host list and an E-row say the same.

## Prove

```sh
errand-box doctor   # on a WSL2 distribution with no systemd: FAILED, names systemd=true
```

---

# T-040: setup stops after `up` when a daemon failed, and box keys go unapplied

**Source:** [issue #4](https://github.com/Azathothas/errand-box/issues/4), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P1
**Effort:** M
**Status:** open

## Problem

setup returns right after `up` when any row failed, and says nothing about
the steps it skipped: the pool, the image, the project, the profile, the
golden instance and runtime.toml. With errand restarting in a loop, a
`box.cpus` or `box.disk` change is applied on some runs and not on others.

## Premise

Measured on WSL2 (Ubuntu 24.04): `box.cpus = "2"` left `limits.cpu: "16"`,
`box.disk = "8GiB"` left `size: 20GiB`, while `box.memory` was applied one
run earlier. On ModelScope Studio and Daytona the first setup ended at the
lxcfs row with no pool and no image. Read: `src/setup.rs:1469-1476`.

## Approach

1. errand's row from the first `up` does not stop setup; the final `up`
   (`src/setup.rs:1595-1604`) reports errand.
2. A daemon whose loss is degraded (lxcfs, E-06) does not stop setup.
3. When setup stops, one row names what it did not run, and why.
4. A unit test with a fake runner where errand's unit fails.

## Prove

```sh
cargo test --locked setup::
```

On a host where errand refuses its inputs, change `box.cpus` and run setup
three times: `incus profile get default limits.cpu` follows each time.

---

# T-042: A host that is a 65536-id user namespace cannot start isolated guests, and setup promises 15257

**Source:** [issue #6](https://github.com/Azathothas/errand-box/issues/6), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P1
**Effort:** M
**Status:** open

## Problem

On a host that is itself a user namespace with 65536 ids, Incus picks idmap
bases above that range for isolated instances, and no session instance can
start (`Failed to handle idmapped storage`). setup reports room for 15257
instances and ends `0 failed`.

## Premise

Measured on Daytona: `/proc/self/uid_map` holds one 65536 range; setup wrote
`root:1000000:1000000000`; `volatile.idmap.base` 1065536 and 1131072; every
first start failed. Read: `src/setup.rs:458` counts /etc/subuid only.

## Approach

1. Count isolated capacity against `/proc/self/uid_map` and `gid_map`.
2. Below one block beyond the base map: refuse with the reason, or fall back
   to non-isolated ids with a DEGRADED row that says what is lost.
3. doctor reports the same; an E-row for a user-namespaced host.

## Prove

```sh
cargo test --locked setup::isolated_capacity
```

On Daytona: setup reports the real capacity, and `sh tests/live/live.sh session`
passes or setup refused with the reason.

---

# T-043: setup never starts a guest

**Source:** [issue #7](https://github.com/Azathothas/errand-box/issues/7), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

No step of setup starts a guest. The golden instance is never started, and
the per-session check `run -- true` only prepares an instance. So setup and
errand's probe report success on hosts where no guest can start (T-039,
T-042), and the user learns it from the first chat message.

## Premise

Read: `src/setup.rs:1339-1395` (golden, never started), `src/shim.rs:279`
(the `-- true` path). Measured: Daytona and WSL2 Arch both ended setup with
`0 failed` and failed every session.

## Approach

After the golden instance, copy it to a throwaway name, start it, run
`true` as the agent, delete it, bounded at 60 s; FAILED with the first error
line of the instance log.

## Prove

```sh
cargo test --locked setup::
```

On WSL2 Arch without systemd, setup's new row is FAILED with the cgroup
error; on WSL2 Ubuntu it is ok in about 1 s.

---

# T-044: doctor and setup do not recognise a host where Incus cannot run

**Source:** [issue #8](https://github.com/Azathothas/errand-box/issues/8), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P2
**Effort:** M
**Status:** open

## Problem

On an unprivileged container doctor says "run errand-box setup"; setup then
installs Incus and leaves lxcfs and incus-user restarting every second.
doctor does not name the package manager (pacman: setup refuses), the
blocked egress (Daytona) or E-18, which edge-cases.md says it reports.

## Premise

Measured on ModelScope Studio (no CAP_SYS_ADMIN or CAP_NET_ADMIN, read-only
/proc/sys, no /dev/fuse): incus-user logs "Failed to create network:
operation not permitted"; lxcfs restarted 556 times in 9 minutes. Read:
`src/main.rs:489-593`, `src/setup.rs:1442-1458`, `src/up.rs:47`.

## Approach

1. doctor rows: a container, the two capabilities, /proc/sys, loop devices,
   the package manager, the hosts setup fetches from, E-18.
2. setup runs them before it installs anything and refuses on a FAILED one.
3. `choose_lxcfs` reports no /dev/fuse as DEGRADED and starts no lxcfs.

## Prove

```sh
cargo test --locked host:: up::
docker run --rm ubuntu:24.04 sh -c 'errand-box doctor'   # FAILED rows that name the capability
```

---

# T-045: `up` restarts the services it has just created, and on Daytona lxcfs never returns

**Source:** [issue #9](https://github.com/Azathothas/errand-box/issues/9), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

On a fresh s6 host every run script is new, so `up` sends `s6-svc -t` to
each service 300 ms after svscan starts it. Where a FUSE mount cannot be
unmounted (Daytona), the killed lxcfs leaves a dead mount and restarts every
second for ever; the failure row quotes "Running destructor lxcfs_exit".

## Premise

Measured on Daytona: lxcfs started 14:27:40.45 and died 0.5 s later;
`fusermount3 -u`, `-uz`, `umount -l` and `umount -l --no-canonicalize` all
failed on the dead mount; after a sandbox restart `up` left lxcfs running.
Read: `src/up.rs:694-697`, `:167-170`, `:594-605`.

## Approach

1. No `-t` for a service whose supervise directory is new in this pass.
2. lxcfs's run script stays down with one clear line on a dead mount.
3. Failure rows quote the first error line, not the last line.
4. An E-row for a runtime that cannot unmount FUSE.

## Prove

```sh
cargo test --locked up::
```

On Daytona, a fresh setup: `s6-svstat /var/lib/errand-box/s6/lxcfs` shows
one start, and `errand-box status` lists lxcfs ok.

---

# T-046: `uninstall --yes`, then `--purge --yes`, leaves every instance and image

**Source:** [issue #10](https://github.com/Azathothas/errand-box/issues/10), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P1
**Effort:** S
**Status:** open

## Problem

The three uninstall commands of docs/operations.md, in order, do not
work: `--yes` removes the program and runtime.toml, so `--purge --yes` is
"not found"; after a new bootstrap it removes files only and leaves every
`eb-` instance, the image and the build bridge, with no row about them.

## Premise

Measured on WSL2 (Ubuntu 24.04): 10 instances and the image stayed; run
first, `--purge --yes` removed them (20 changed). A purge also leaves the
lxcfs drop-in, errand-box's subuid lines, the build's base image and
incus-user's projects. Read: `src/ops.rs:294`, `:331`, `:250-270`.

## Approach

1. Keep runtime.toml, or a copy, until a purge; else find `eb-` instances
   by prefix in every project.
2. A DEGRADED row when purge cannot find them.
3. docs/operations.md: `--purge --yes` instead of, not after, `--yes`.
4. Remove, or list as kept, the drop-in and the subuid lines.

## Prove

```sh
errand-box uninstall --yes && sh bootstrap.sh --version "$v" && errand-box uninstall --purge --yes
incus list --all-projects -c n -f csv | grep -c '^eb-'   # 0
```

---

# T-047: network.proxy is not validated, and a bad proxy outlives `proxy = "auto"`

**Source:** [issue #11](https://github.com/Azathothas/errand-box/issues/11), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

Any string is accepted as `network.proxy` and written to incusd's
`core.proxy_*` and to proxy.env, which errand reads. Back on `auto` with no
HTTPS_PROXY, setup keeps it ("the previous setting is kept"); only `none`
clears it, and no page says so. A missing `proxy_ca` file is accepted too.

## Premise

Measured on WSL2 (Ubuntu 24.04): `ftp://proxy.invalid:21` stayed in
`core.proxy_https` and proxy.env after setup with `auto`. Read:
`src/config.rs:245-309`, `src/up.rs:1285`.

## Approach

1. Validate: auto, none, or an http or https URL with a host and a port;
   `proxy_ca` a readable file.
2. Record where a kept proxy came from, and drop one the config set.
3. Document `none` as the reset.

## Prove

```sh
cargo test --locked config:: up::proxy
printf '[network]\nproxy = "ftp://x:21"\n' > bad.toml && errand-box setup --config bad.toml; echo "exit $?"   # 1, names network.proxy
```

---

# T-048: setup reads `--env` and `--creds` after it changes the host

**Source:** [issue #12](https://github.com/Azathothas/errand-box/issues/12), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

A missing or incomplete env file is found only after packages, the account,
s6, the binaries and the boot hook. `--env` without `--creds` is dropped
with exit 0, and a creds file readable by everyone is used without a word.

## Premise

Measured on WSL2 (Ubuntu 24.04): `--env nope.env` failed after 11 steps;
`--env good.env` alone printed "kept"; a 0644 creds file passed. Read:
`src/setup.rs:1455-1468`, `:912-966`.

## Approach

1. Parse both files and build errand's config before the first change.
2. One without the other is a usage error (exit 2) that names it.
3. A creds file that group or others can read, or root does not own, is
   DEGRADED or FAILED.

## Prove

```sh
cargo test --locked setup::
errand-box setup --config c.toml --env e.env; echo "exit $?"   # 2, names --creds
```

---

# T-050: setup and status misreport a refusing errand

**Source:** [issue #14](https://github.com/Azathothas/errand-box/issues/14), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

setup reports a crash-looping errand as ok when it fails after the 3 s
settle (Daytona, `tests/data`), against what docs/tests.md section 5 says.
The failure row quotes the last log line, which is a warning, an info line
or systemd's "Started" line, not the error. status follows the unit's
state at the instant it reads it. Related to T-037, which backs off.

## Premise

Measured on Daytona and WSL2 (Ubuntu 24.04): `errand ok up (pid 4690) 11
seconds`, pids 4690, 5337, 5953 in 90 s; on WSL2 the error was "the chat
connection ended: Sent invalid authentication", and status said FAILED and
then ok a second apart. Read: `src/up.rs:740`, `:594-605`, `src/status.rs:129`.

## Approach

1. Quote the last error-level line, or systemd's exit status.
2. status counts restarts over a window.
3. setup waits for errand's ready line or its exit, up to startupMs.

## Prove

```sh
cargo test --locked up:: status::
```

With `tests/data` inputs, setup reports errand FAILED with the chat error on
both kinds of host, and status agrees with itself across 10 calls.

---

# T-052: On WSL2 the pool is sized from the virtual disk

**Source:** [issue #16](https://github.com/Azathothas/errand-box/issues/16), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

`engine.pool_size = "auto"` takes half of the WSL virtual disk's free space
(476 GiB of a 1 TB vhdx) while the Windows drive under it had 232 GB free;
the E-32 check reads the same virtual numbers and sees nothing wrong.

## Premise

Measured on WSL2 (Ubuntu 24.04): the pool row and `df /` against the
Windows drive. Read: `src/setup.rs:968`, `:994`, `:1001`.

## Approach

On WSL, cap `auto` (64 GiB, or a quarter of the vhdx) and say in the row
that the Windows drive's free space is not visible; an E-row.

## Prove

```sh
cargo test --locked setup::
```

On WSL2, a fresh setup's pool row names the cap and `engine.pool_size`.

---

# T-053: WSL2 distributions share one network namespace, and the build bridge name collides

**Source:** [issue #17](https://github.com/Azathothas/errand-box/issues/17), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

Every WSL2 distribution shares one network namespace, so a second Incus
sees the first one's bridges. errand-box's build bridge has a fixed name,
and a foreign or leftover `ebbuild0` makes the image build fail with "the
build bridge has no IPv4 address".

## Premise

Measured on two WSL2 distributions (Ubuntu and Arch): one `net:` namespace;
Arch's Incus listed `ebbuild0,bridge,NO`, and its image build failed at
once. Read: `src/image.rs:21`, `:105-118`, `:74-81`.

## Approach

1. Check `managed`; name a foreign interface and how to remove it.
2. A bridge name unique to the host.
3. A doctor row on WSL, and an E-row.

## Prove

```sh
cargo test --locked image::
ip link add ebbuild0 type bridge && errand-box image build   # FAILED, names the foreign ebbuild0
```

---

# T-055: setup reports the configured pool driver, and E-05 only once

**Source:** [issue #19](https://github.com/Azathothas/errand-box/issues/19), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

For an existing pool setup prints the driver from the config, not the
pool's, and ignores a changed `pool_driver` silently. The E-05 warning for a
`dir` pool appears only on the run that makes it. A `dir` pool copies the
whole rootfs per session, and nothing says how many fit.

## Premise

Measured on Daytona: `storage pool ok errand-box (btrfs)` while Incus said
`driver: dir`; sessions took 19 to 54 s to copy; the 10 GiB pool held about
three. Read: `src/setup.rs:1053`, `:1001-1007`.

## Approach

Read the pool's driver; DEGRADED for `dir` on every run and in status; say
when the config's driver differs; for `dir`, the per-session size.

## Prove

```sh
cargo test --locked setup::
```

On a host with a `dir` pool, setup and status both say DEGRADED (E-05).

---

# T-056: Pool and image failures on a sandbox name no way out

**Source:** [issue #20](https://github.com/Azathothas/errand-box/issues/20), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

With no loop devices, the pool row shows only losetup's "Permission
denied"; with images.linuxcontainers.org blocked, the image row shows only
the reset connection. The ways out are in the docs (`pool_driver = "dir"`,
image route 2), not in the rows, and `ebbuild0` stays after the failure.

## Premise

Measured on Daytona. Read: `src/setup.rs:1067-1071`, `src/image.rs:180`.

## Approach

Add the way out to each row, and remove the build bridge on that path.

## Prove

```sh
cargo test --locked setup:: image::
```

On Daytona with the defaults, the pool row names `pool_driver = "dir"`.

---

# T-057: On Arch, s6 cannot be installed with pacman

**Source:** [issue #21](https://github.com/Azathothas/errand-box/issues/21), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P2
**Effort:** M
**Status:** open

## Problem

On a pacman host with no init, the first refusal lists incus, lxcfs and
btrfs-progs; the second says "install the s6 package with pacman", and
Arch has none (skalibs, execline and s6 are in the AUR).

## Premise

Measured on WSL2 (Arch, no init): `error: target not found: s6`; s6 2.15.1.1
built from the author's releases worked. Read: `src/setup.rs:177-186`,
`:747-793`, `src/up.rs:443`.

## Approach

One refusal that lists everything for that host, with per-distribution
advice: on Arch, the AUR or the author's releases. RULES.md section 4 keeps
s6 from the distribution, so the fix is the advice, not a bundled s6.

## Prove

```sh
cargo test --locked setup::
```

On an Arch distribution without systemd, the first setup either installs
s6 or names the AUR and the source route in one row.

---

# T-062: Guest memory follows /proc/meminfo, not the container's limit

**Source:** [issue #26](https://github.com/Azathothas/errand-box/issues/26), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

In a container with a cgroup memory limit, errand-box reads the machine's
memory, and `box.memory = "80%"` becomes `limits.memory: 80%`, which Incus
resolves against the machine. Guests are then not limited inside the
sandbox's real memory.

## Premise

Measured on Daytona: `memory.max` 4 GiB, errand-box `193206 MiB`; CPUs were
right (4, from cpu.max). Read: `src/host.rs:86-92`.

## Approach

Take the smaller of MemTotal and the cgroup limit; write `limits.memory`
as a size when a limit applies; a doctor row.

## Prove

```sh
cargo test --locked host::
```

In a container with `-m 4g`, doctor's host line says 4096 MiB.

---

# T-063: Unknown options are ignored, and `gc --dryrun` deletes

**Source:** [issue #27](https://github.com/Azathothas/errand-box/issues/27), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** done

## Problem

status, doctor, up, maint, gc and reap ignore an unknown option and run, so
a misspelt `--dry-run` runs a real gc. setup and uninstall refuse one.

## Premise

Measured: `maint --frobnicate` ran a pass; `gc --frobnicate` ran a gc.
Read: `src/main.rs:130-134`, `:231`, `:256`, `:263`, `:355-413`.

## Approach

Refuse what is left after each command's options, exit 2.

## Prove

```sh
cargo test --locked --test shim
errand-box gc --dryrun; echo "exit $?"   # 2
```

## Closing

**Closed 2026-10-25.** `src/main.rs` gained `deny_leftovers`, used by `doctor`,
`status`, `up` and `reap`; `maint` and `gc` refuse anything that is not their
one flag (`--loop`, `--dry-run`) before a pass runs, so a misspelt
`gc --dryrun` exits 2 rather than deleting. `setup` and `uninstall` already
refused, so the behaviour is now uniform. The check runs after the config
loads and before `runtime.toml`, so a rejection needs no host state. A new
`tests/shim.rs` test, `unknown_options_are_refused_not_run`, drives
`gc --dryrun`, `maint --frobnicate`, `reap --x`, `status --nope` and
`doctor --nope` (each exit 2, "unknown option") and confirms `gc --dry-run`
is not rejected; it passed. fmt and clippy are clean.

---

# T-064: `logs` with no name is refused under systemd

**Source:** [issue #28](https://github.com/Azathothas/errand-box/issues/28), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

Under systemd `errand-box logs` exits 2 ("name a daemon"), though the help
and docs/operations.md promise the last line of each. maint logs only
while it waits for runtime.toml, so its last line stays a stale error.

## Premise

Measured on WSL2 (Ubuntu 24.04) and on s6 hosts. Read: `src/ops.rs:42-62`,
`src/main.rs:64-66`, `:369-382`.

## Approach

The same table under systemd from `journalctl -n 1 -u UNIT`; maint logs one
line when it starts working.

## Prove

```sh
errand-box logs; echo "exit $?"   # 0, one line per daemon, under systemd too
```

---

# T-065: The README's quick start cannot be followed

**Source:** [issue #29](https://github.com/Azathothas/errand-box/issues/29), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

The README's third line names three input files it never says how to get;
it also fetches bootstrap.sh from main against docs/operations.md, and does
not say that setup needs root or how long the first one takes.

## Premise

Measured on a fresh ModelScope Studio: the third line failed with "cannot
read ./errand-box.toml". Read: README.md, bootstrap.sh:134.

## Approach

The tag, the three inputs from examples/, root, the time, the image route;
bootstrap's last line names examples/.

## Prove

```sh
sh scripts/common/check-docs.sh
```

The README's commands, run in an empty directory on a fresh host, reach
setup's report.

---

# T-067: bootstrap.sh prints the exit code in its errors

**Source:** [issue #31](https://github.com/Azathothas/errand-box/issues/31), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

`die` prints its second argument, so errors end in "2"; `--help` prints
`set -u`; `--dry-run` needs root; an unknown version says only "download
failed".

## Premise

Measured on every host. Read: bootstrap.sh:31, :39, :49-55.

## Approach

`die` prints `$1`; help stops at the first non-comment line; dry-run without
root; a 404 names the missing release.

## Prove

```sh
sh bootstrap.sh --frobnicate; echo "exit $?"   # 2, and the message has no trailing 2
sh tests/pins-test.sh
```

---

# T-068: doctor's next step is wrong when Incus is missing or down

**Source:** [issue #32](https://github.com/Azathothas/errand-box/issues/32), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

Before setup, doctor and status say "run `errand-box up`" about an Incus
that is not installed. With Incus down, doctor says the golden instance is
missing and to run setup. `image import` before setup says only "could not
start incus".

## Premise

Measured on all four hosts. Read: `src/main.rs:556-576`, `src/status.rs:168`.

## Approach

Skip or reword the answer row without Incus; split "missing" from "cannot
check"; name `setup --skip-image` before an import.

## Prove

```sh
errand-box doctor   # fresh host: one next step, setup
```

---

# T-069: Reports print every row twice

**Source:** [issue #33](https://github.com/Azathothas/errand-box/issues/33), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

setup, up, image and uninstall print each row live on stderr and again in
the table on stdout. The uninstall plan marks every planned action
"skipped" and names a unit that may not exist.

## Premise

Measured on every host. Read: `src/report.rs:48-53`, `src/ops.rs:288-330`.

## Approach

Live rows only when stdout is not a terminal, or no table after them; a
plan status of its own; a plan built from what exists.

## Prove

```sh
cargo test --locked report:: ops::
```

---

# T-070: doctor as a user who is not root reports E-08

**Source:** [issue #34](https://github.com/Azathothas/errand-box/issues/34), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

/proc/1/personality is readable by root only, so doctor run by anyone else
reports the lxcfs 6 condition on every host. Arch's host line has a double
space.

## Premise

Measured on all four hosts. Read: `src/host.rs:80`, `:185`,
`src/main.rs:521`.

## Approach

Count it only as root, or say "run as root to check"; drop an empty codename.

## Prove

```sh
runuser -u nobody -- errand-box doctor | grep -c 'E-08'   # 0
```

---

# T-071: "No Zabbly build" when Zabbly cannot be reached

**Source:** [issue #35](https://github.com/Azathothas/errand-box/issues/35), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

For an unknown codename, any curl failure of the Zabbly probe is reported
as "no Zabbly build", including a host that cannot reach Zabbly at all.

## Premise

Measured on Daytona (Ubuntu 26.04, egress reset). Read:
`src/setup.rs:311-326`, `:386-392`.

## Approach

Tell a 404 from a transport error, and report the latter as unreachable.

## Prove

```sh
cargo test --locked setup::
```

---

# T-072: status counts ext4's root reserve as used

**Source:** [issue #36](https://github.com/Azathothas/errand-box/issues/36), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

status computes used as total minus available, so ext4's reserve reads as
used: 59 of 1006 GiB where df says 8.2 GiB.

## Premise

Measured on WSL2 (Ubuntu 24.04). Read: `src/status.rs:36-45`, `:187`.

## Approach

Used from the free blocks; the threshold from the available ones.

## Prove

```sh
cargo test --locked status::
```

---

# T-073: Input errors lose the key, the line or the file

**Source:** [issue #37](https://github.com/Azathothas/errand-box/issues/37), T-027's end-to-end run, 2026-10-24
**Category:** setup
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

A TOML type error names neither the key nor the line; an env error names a
fixed file name or no file; `--config` with no value is an unknown option;
unknown ERRAND_ keys pass; `box.cpus = "0"` is used as 1.

## Premise

Measured on WSL2 (Ubuntu 24.04), one bad input at a time. Read:
`src/config.rs:275-322`, `src/errandcfg.rs:15-41`, `:80-91`, `src/main.rs:110`.

## Approach

Keep the toml span and key path; pass real paths; "--config needs a value";
warn about unknown ERRAND_ keys; refuse cpus outside the host's range.

## Prove

```sh
cargo test --locked config:: errandcfg::
```

---

# T-079: root follows a symlink under an errand-owned directory, a second route to root

**Source:** [issue #43](https://github.com/Azathothas/errand-box/issues/43) and [issue #44](https://github.com/Azathothas/errand-box/issues/44), read on `57231da`
**Category:** setup
**Priority:** P1
**Effort:** M
**Status:** partial

## Problem

Root writes and chowns files under errand's home, which the errand account
owns, on two paths that follow a symlink the account plants there. The
account then redirects root's write or chown to any file on the host. This
is a second route from the errand account to root, and T-020 does not close
it, because it needs no proxy device.

1. `procs::lock` opens the lock file with `create`, `write` and no
   `O_NOFOLLOW`, then, when euid is 0 and the lock directory is not root's,
   `fchown`s the open file to the directory's owner (`src/procs.rs:62-76`).
   Root calls it from `reap` (`src/session.rs:538`, every
   `maintenance.reap_seconds`) and from `gc` (`src/maint.rs`), on
   `<errand home>/.local/state/errand-box/locks/<name>.lock`
   (`src/session.rs:223-230`), a directory setup makes under errand's home,
   owned by the errand account (`src/setup.rs:1556-1565`). A symlink at
   `<name>.lock` makes root open the target (creating it when the link
   dangles) and give it to the errand account.
2. `up::write_if_changed` writes `path.with_extension("errand-box-tmp")`
   with `fs::write` and `fs::set_permissions`, then renames it over the
   final path (`src/up.rs:322-325`). `fs::write` and `fs::set_permissions`
   follow a symlink at the temporary name. `errand_config` (`src/setup.rs:954`)
   and `drop_web` (`src/setup.rs:907`) call it under errand's home, so a
   symlink at `config.errand-box-tmp` makes root write JSON the account chose
   over any file, and chmod it 0600.

## Premise

Read on `57231da`, and the open step of case 1 reproduced by the reporter in
a scratch directory (the `fchown` was not run: the review sandbox has no
CAP_CHOWN). `safe_dir` already makes each directory under errand's home
symlink safe before root acts (E-54, `src/setup.rs:423-438`), and
`errand_config` removes a symlink at the final `config.json`
(`src/setup.rs:951-953`). The two paths above were missed: the lock file,
and the temporary file beside the config.

## Approach

The root cause is one rule: root never follows a symlink when it opens,
writes or chowns a file under a directory the errand account can write.

1. `procs::lock`: open with `O_NOFOLLOW` and `O_CLOEXEC`, and `fchown` only a
   file this call created, not one it opened. Or keep root's locks in a
   root-owned directory and give the errand account a separate one.
2. `up::write_if_changed`: create the temporary file with
   `OpenOptions::new().create_new(true).custom_flags(O_NOFOLLOW)`, and
   `fchmod` the open descriptor rather than `set_permissions` on the path.
3. An edge-case row for the class: root writing under an errand-owned
   directory, and what keeps it safe.

## Prove

```sh
cargo test --locked procs:: up::
```

A test plants a symlink at the lock path and asserts `lock` returns an error
and the target is untouched; another plants one at the temporary name and
asserts `write_if_changed` fails or leaves the target untouched. Each fails
on the current code.

## Done, and open

**The fix and its unit proof are done, 2026-10-25. Live re-run pending a
host.** `procs::lock` (`src/procs.rs`) opens with `O_NOFOLLOW` and `O_CLOEXEC`,
creates the lock with `create_new` and `fchown`s only a file it just created,
so a symlink or hardlink planted at the lock name is refused rather than
followed. `up::write_if_changed` (`src/up.rs`) removes any stale or planted
temporary file, creates the temporary with `create_new` and `O_NOFOLLOW`, and
`fchmod`s the descriptor rather than the path, so a link at the temporary name
cannot redirect root's write or chmod. A `forbidden-patterns.md` row names the
class, beside E-54.

Prove, run 2026-10-25 (adapted to the binary crate, which has no lib target):

```sh
cargo test --locked -p errand-box --bins -- procs:: write_if_changed
```

```text
test procs::tests::lock_refuses_a_symlink_at_the_lock_path_and_leaves_the_target ... ok
test up::tests::write_if_changed_does_not_follow_a_symlink_at_the_temp_name ... ok
test result: ok. (procs 4, up 2)
```

fmt and clippy are clean. **Open:** `write_if_changed` is a helper `up` and
`setup` use, so RULES.md asks for a live run before the entry closes; this
host has no Incus. The only step left is `sh tests/live/live.sh up` on a host,
which re-exercises the helper through `up`. No code work remains.

# T-084: doctor, setup and the shim report capability by literal, by euid, or not at all

**Source:** [issue #50](https://github.com/Azathothas/errand-box/issues/50) and [issue #51](https://github.com/Azathothas/errand-box/issues/51), read on `57231da`
**Category:** setup
**Priority:** P2
**Effort:** M
**Status:** open

## Problem

errand-box decides and reports what a host can do without probing it, so a
host that cannot run a session passes every report. T-044 is about a host
where Incus cannot run; this is the capability layer under it.

1. The shim's `bailey doctor` prints `user namespaces: yes` as a literal
   (`src/shim.rs:142`) and a fixed `cgroup delegation: memory, pids, cpu`
   line (`src/shim.rs:143-146`). errand reads only those two lines plus
   `landlock:` (docs/bailey-contract.md section 3.1), so errand's whole
   capability check is two constants. T-051 is the limits half of the same
   output; this is the user-namespace and cgroup-delegation lines.
2. `errand-box setup` gates on euid alone (`src/setup.rs:1402`). A uid 0
   with no capabilities passes the gate and fails later with a message that
   names no cause.
3. `errand-box doctor` probes the init, the cgroup version, KVM and WSL and
   reports AppArmor (`src/main.rs:489-593`, `src/host.rs:50-91`). It does not
   probe user namespaces, Landlock, seccomp, `ptrace` or a loopback bind. A
   grep of `src/host.rs` for these finds only the cgroup and AppArmor lines.
4. errand refuses to start on `user namespaces: ... no`
   (`errand/src/sandbox/bailey.rs:148-155`), though bailey's own README says
   enforcement falls back to Landlock and seccomp without the namespace. So a
   host with Landlock and seccomp and no user namespaces can run bailey and
   cannot run errand, and a second engine that reports the truth (T-086) hits
   the same refusal.

## Premise

Read on `57231da`. The reporter ran probes in a review sandbox (uid 0,
`CapEff` 0, no user namespaces): `unshare -U` returned `Operation not
permitted`, `landlock_create_ruleset` returned ABI 10, the seccomp user
notification was available, and a loopback bind returned `Permission
denied`. So each capability a probe would read differs from what the
literals assert.

## Approach

1. `errand-box doctor --plan`: probe each capability by attempting it, not by
   reading a file, and print one row per capability with what it means.
   Name the best tier for the host: Incus where root and Incus work, `lite`
   (T-086) where Landlock and seccomp work, else none, with the missing rows.
2. The shim's `doctor` prints what the probe found, not a literal.
3. The setup gate names the capability it needs (`CAP_CHOWN`, the uid map,
   writable `/proc/sys`), not just euid.
4. errand's `parse_doctor` treats `user namespaces: no` as a gap, which
   `requireFullEnforcement` governs, when the doctor output says the engine
   locks egress by other means; it stays a refusal otherwise. This is
   errand's change, in `errand/src/sandbox/bailey.rs`.

## Decision

The capability probe and the tier chooser are the floor T-086's `lite`
engine needs, so they land first whether or not `lite` is built. The shim
literal that T-051 also touches is corrected here for the user-namespace and
cgroup lines and there for the limits line; neither entry writes the other's
line.

## Prove

```sh
cargo test --locked host:: shim:: -p errand-box
cargo test --locked -p errand --lib -- bailey
```

A test runs the probe with the user-namespace attempt failing and expects
`user namespaces: no`; another feeds a `CapEff` of zero with euid 0 to the
setup gate and expects a row naming the missing capability; an errand test
gives `parse_doctor` `user namespaces: no` with an egress-locked line and
`requireFullEnforcement = false` and expects the daemon to start.

---

# T-085: errand-box is sized for a roomy host, and does not tell errand the real numbers

**Source:** [issue #53](https://github.com/Azathothas/errand-box/issues/53), [issue #54](https://github.com/Azathothas/errand-box/issues/54) and [issue #55](https://github.com/Azathothas/errand-box/issues/55), read on `57231da`
**Category:** setup
**Priority:** P2
**Effort:** M
**Status:** open

## Problem

errand-box assumes a host with room to spare, and the numbers it enforces,
reports and hands to errand drift apart on a small one.

1. The `auto` pool floor is 60 GiB. `pool_size` is
   `max(free/2, 3 * box.disk, 20 GiB)` (`src/setup.rs:968-977`), and
   `box.disk` defaults to `20GiB`, so the floor is 60 GiB on any host with
   under 120 GiB free. The E-32 check only degrades; a sparse loop file
   larger than the disk still gives guests I/O errors when the disk fills.
   T-018 fixed the report, not the size; T-052 is the WSL variant.
2. errand's limits are hardcoded: `maxConcurrentTurns: 2`,
   `maxLiveSessions: 4` (`src/errandcfg.rs:152`), with no key in
   `errand-box.toml`. Each session gets 80% of memory and CPUs
   (`src/config.rs:78-79`), and errand's admission counts sessions only
   (`errand/src/admission/scheduler.rs`). Four live sessions then promise
   320% of memory; Incus limits are caps, not reservations, so the kernel
   decides what to kill, with no priority for errand or the host daemons.
3. errand-box writes none of errand's own `sandbox.memory`, `cpus`, `pids`
   or `disk` (`src/errandcfg.rs:143-149`). errand keeps its defaults (`4g`
   memory, `5g` disk, `errand/src/config/schema.rs`) for its messages, its
   exit mapping and its disk budget, while Incus enforces other numbers. So
   errand reports and may act on limits the guest does not have.

## Premise

Read on `57231da`. Not run on a constrained host: `errand-box status` shows
no capacity line, and the idle memory of errand, incusd, lxcfs and kage is
not recorded anywhere in the tree, so the admission floor and the per-session
divisor are proposals until measured.

## Approach

1. Cap the `auto` pool at what fits: `min(max(free/2, floor), free - reserve)`,
   and refuse with the free space and the minimum when `free - reserve` is
   below the floor. Keep an explicit `engine.pool_size` as the override.
2. `[limits] max_live_sessions` and `max_concurrent_turns` in
   `errand-box.toml`, written into errand's config; a `preset = "micro"`
   that sets one live session and sizes `box.memory` from the host's memory
   divided by the limit.
3. Write errand's `sandbox.memory`, `cpus`, `pids` and `disk` from the values
   errand-box sets, so errand's numbers match Incus's.
4. Memory admission that refuses a new session when `MemAvailable` is below a
   measured floor, and a `status` capacity line (free pool, `MemAvailable`,
   sessions that fit).

## Decision

The three are one entry because they share a cause: errand-box does not read
the host's real room, and does not pass errand the numbers Incus enforces.
Steps 1 and 3 are correctness on any host; steps 2 and 4 are what a small
host needs. The floors and the divisor wait on a measurement of idle memory,
named in the Prove.

## Prove

```sh
cargo test --locked setup:: errandcfg:: -p errand-box
```

Unit tests with `free` at 27 GiB and at 200 GiB expect a pool of at most the
free space minus the reserve, then 100 GiB; a config test with
`box.memory = "512MiB"` expects `sandbox.memory` of `512m` and
`maxLiveSessions` from the key, not the constant.
