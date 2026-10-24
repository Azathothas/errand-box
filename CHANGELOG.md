# Changelog

What changed, when, and where the evidence is. The version is the version
of the `errand-box` program (`Cargo.toml`); a version is a statement about
its commands, its configuration keys and the `bailey` interface it gives
errand.

---

## Unreleased

### 2026-10-24T13:18Z: assets for an end-to-end test, and T-037

**Record:** T-035 and T-037 in [TODO/](TODO/INDEX.md). Not deployed and not
released: `artefacts` is a prerelease of test assets, not a version.

- `.github/workflows/artefacts.yml`, dispatched by hand on main, publishes
  the prerelease `artefacts`: errand-box and discord-mock as static binaries
  for x86_64 and aarch64, the guest image built with distrobuilder for
  `errand-box image import`, and a kit with `bootstrap.sh`, the inputs of
  setup and the live tests. It is never the latest release.
- `tests/e2e/mock-session.sh` runs one errand session against discord-mock
  on a host set up with `tests/e2e/mock.env` and `mock-creds.env`.
- `scripts/notices.sh --workspace` adds discord-mock's crates.
- T-037 filed: errand's refusals restart every two seconds.

## 0.1.1

### 2026-10-24T12:21Z: the release v0.1.1

**Record:** [TODO/PROGRESS.md](TODO/PROGRESS.md), and T-028 in
[TODO/errand.md](TODO/errand.md). Released by dispatching
`.github/workflows/release.yml` on `main`; deployed to no host by this
entry. Version 0.1.1 is every entry below, down to 0.1.0.

- **Each release carries errand.** The binaries now include errand's
  daemon; `LICENSE-errand` and `THIRD-PARTY-NOTICES`, generated from
  `Cargo.lock` by a pinned cargo-about, are published beside them.

### 2026-10-24: errand is errand-box's hard fork, built in

**Record:** T-028 to T-035 in [TODO/errand.md](TODO/errand.md); the
research in [docs/history/references/](docs/history/README.md). Released in
0.1.1 (the entry above); deployed to no host by this entry. The live tests
passed 54 of 54 on a hosted container with the static binary of `9b27e60`.

- errand, imported at QaidVoid/errand `715e9b0b777a`, is the workspace
  member `errand/`, a library: `errand-box serve` is the daemon and
  `errand-box threads` its thread command. setup removes a downloaded
  errand instead of fetching one. `[errand]` left `pins.toml`, and the
  `errand.version` key has no effect.
- errand's local web interface is gone, and setup takes `web` out of a
  kept errand config (T-034).
- A session could make errand's broker send the operator's provider key to
  a host it named (T-029, errand#48). Fixed in the fork. Also fixed: the
  host-side push could publish another session's repository through a git
  directory that borrows from it (T-030, errand#52), and the relay now logs
  a connection it drops when its target does not answer (T-031).
- A Discord stand-in, `discord-mock/`, tests errand's chat path in-process
  and in CI; the gate checks its answers against Discord's OpenAPI spec
  (T-035).
- The static build compiles C now: install musl-tools to run the `musl`
  step. The `cross` step takes zig 0.16 as the other architecture's C
  compiler.
- The gate's placeholder and public email rules are narrower, each with a
  planted positive that still fails.

### 2026-10-23T15:30Z: errand's ruling on #41, the operator's rulings, and docs for a Windows-hosted session

**Record:** T-019 to T-027 in [TODO/](TODO/INDEX.md); section 10 of
[docs/incus-backend.md](docs/incus-backend.md). Not deployed and not
released. No code changed.

- errand's maintainer accepted an `incus` backend on QaidVoid/errand#41,
  with no proxy devices and a root incusd. errand-box keeps its `bailey`
  stand-in as the default (T-019).
- The operator's rulings on T-007, T-015, T-019 to T-022 and E-69 are in
  their entries. New: T-024 to T-027.
- docs/tests.md section 5 says how to test a release as a user on a host
  you are given, and from a Windows machine.

### 2026-10-23T14:42Z: live tests on GitHub's runners, x86_64 and aarch64

**Record:** T-013 in [TODO/setup.md](TODO/setup.md); E-69 and E-70 in
[docs/edge-cases.md](docs/edge-cases.md). Not deployed and not released.

- `.github/workflows/live.yml`, started by hand, runs setup twice and the
  live tests on an `ubuntu-24.04` and an `ubuntu-24.04-arm` runner: hosts
  with systemd and cgroup v2, where setup takes Incus 7.5.1 from the stable
  line. Setup passes but for errand's daemon (test credentials), and the
  live tests pass 40 of 41 on x86_64 and 39 of 41 on aarch64.
- Found, not fixed: rootless podman fails on those hosts. AppArmor denies
  the agent's user namespace (E-69), and with that off, podman on cgroup v2
  needs a user session the agent does not have (E-70).

### 2026-10-23T14:07Z: Debian 13, a late incusd, and podman's mirror

**Record:** T-011 in [TODO/setup.md](TODO/setup.md); E-16 and E-68 in
[docs/edge-cases.md](docs/edge-cases.md). Not deployed and not released.
The guest image changes, so every host rebuilds it (E-31).

- **E-68.** On a systemd host whose network nothing reports online, incusd's
  first start waits 2 minutes for network-online.target. The first setup's
  `incus version` hit its 120 s deadline and reported Incus as not
  installed. It now has 300 s, and a wait of a minute or more is reported
  as degraded, because each boot waits as long.
- **E-16, podman.** podman in a guest, root and rootless, now uses
  `image.registry_mirror` as Docker does. On Debian 13 it got Docker Hub's
  `toomanyrequests` while Docker, on the mirror, passed.
- **The host tests** select a report's summary line by its shape, not as
  the last line: a progress line on stderr can come after it.

### 2026-10-23T13:36Z: Alpine passes the live tests

**Record:** T-012 in [TODO/setup.md](TODO/setup.md); E-66 and E-67 in
[docs/edge-cases.md](docs/edge-cases.md). Not deployed and not released.

- **E-66.** Alpine packages `incus-user` apart from `incus`. setup did not
  install it, `up` ran no incus-user, and errand's project never appeared.
  setup now installs it on apk hosts, and `up` reports a packaging with no
  incus-user program as FAILED where it left the service out.
- **E-67.** busybox `adduser -S` put the errand account in `nogroup`, which
  other system accounts share. setup now gives the account a group of its
  own. An account that exists keeps its group.
- **The host tests.** `tests/hosts/host.sh` imports a distrobuilder build
  when `EB_HOSTS_IMAGE` names one, so a nested host gets past the image it
  cannot build. Its setup check allowed every FAILED row whose name starts
  with "errand", `errand project` included; it now allows errand's own row
  only. `tests/live/live.sh` gives the session files the account's own group
  by number, not by the name `errand`.

### 2026-10-23T13:23Z: a distrobuilder build imports as current

**Record:** T-014 in [TODO/sandbox.md](TODO/sandbox.md). Not deployed and
not released.

- `scripts/build-image.sh` wrote an inputs digest of the proxy URL, not of
  the kage version: a check for a single quote reused the variable. Setup
  took every imported image as stale and rebuilt it. The script now takes
  the digest once, and `--print-inputs`, which the unit test runs, prints
  the value the build writes.

### 2026-10-23T13:07Z: the run that makes the pool reports E-32

**Record:** E-32 in [docs/edge-cases.md](docs/edge-cases.md). Not deployed
and not released.

- setup made a 60 GiB pool on a host with 27 GiB free and reported
  "changed"; only the next run reported the pool degraded. The run that
  makes a loop-file pool now applies the same check, so the first setup on
  a small disk says so. Seen with v0.1.0 on the build host.

## 0.1.0

### 2026-10-23T12:44Z: the release v0.1.0

**Record:** T-009 in [TODO/upkeep.md](TODO/upkeep.md).
`.github/workflows/release.yml` builds the static binary on x86_64 and
aarch64 and publishes it with `SHA256SUMS`; `bootstrap.sh` reads that
release. Not deployed to a host by this entry. Version 0.1.0 is every entry
below.

- **A release by dispatch.** `release.yml` also runs on a manual dispatch on
  `main` with the input `version`. It checks that the version is
  Cargo.toml's and that the tag does not exist, and the release creates the
  tag on the dispatched commit. Amended 2026-10-23T12:50Z: this entry first
  said a pushed tag starts the release; the session's git route refused the
  tag push, so the dispatch was added.

### 2026-10-23T12:36Z: CI passes on GitHub

**Record:** [TODO/PROGRESS.md](TODO/PROGRESS.md) and
[docs/tests.md](docs/tests.md) section 2. Not deployed and not released. The
program does not change; the scripts and the gate do.

- **shellcheck 0.11.** The `gates` job runs on `ubuntu-latest`, which is now
  Ubuntu 26.04 with shellcheck 0.11.0, and it refused 42 SC2329 and 2 SC2319
  findings. The live tests and the host steps were called as `"t_$name"`,
  which shellcheck cannot follow, so it had checked none of their bodies.
  Each is now called by its own name. Checked, the bodies held 17 more
  SC2319 findings, and one was a defect: the `bootstrap` step of
  `tests/hosts/host.sh` passed "errand-box is on PATH" on any output,
  a "not found" included. It now reads the exit code.
- **The `errand-config` step** runs errand with the environment errand-box
  gives it: cleared, then `HOME` and `PATH`. errand reads `XDG_CONFIG_HOME`
  before `HOME`, and a GitHub runner sets it, so on GitHub errand looked for
  its config in the runner's home and refused to start.
- **The `cross` step** exits 2, not 0, when it runs less than every test.
  Without a `binfmt_misc` handler it runs the unit tests and names the
  integration tests it left out, which start the shim as a child.

### 2026-10-23T10:28Z: hardening, QOL commands, and a host matrix

**Record:** [TODO/PROGRESS.md](TODO/PROGRESS.md); entries T-011 and T-018
closed, T-012, T-013 and T-014 to partial, T-020 added; E-34 to E-65 in
[docs/edge-cases.md](docs/edge-cases.md). Not deployed and not released: no tag, no release asset. CI is off;
every step ran through `scripts/ci-local.sh`.

- **Start-up order (E-34).** Each daemon waits for the ones it needs before
  it starts (`errand-box __wait`), and `up` watches them all until each
  keeps one pid. The crash-loop check reads s6 2.13's `pid N pgid N` format
  (E-62).
- **systemd (T-011).** `up` verifies each unit stays up, sets incusd's
  egress proxy through Incus `core.proxy_*` (E-13), and manages the packaged
  lxcfs unit with a drop-in that never stops it  -  stopping it stopped
  incusd, which `Requires=` it (E-37). Ubuntu 24.04 passes setup, the live
  tests, a reboot and a session after it.
- **OpenRC and musl.** setup creates the `incus-admin` and `incus` groups
  Alpine omits (E-35) and writes an OpenRC boot hook (E-36).
- **New commands.** `status`, `shell`, `logs`, `uninstall [--purge]`,
  `image import`, `gc --dry-run`, and `--json` for setup, up, status, doctor
  and image.
- **Lifecycle.** gc and reap do not abort on one instance (E-57) and share
  the launcher's lock (E-40); a run does not reuse a running orphan (E-56);
  maint keeps its timers across restarts (E-41); `up` falls back to the last
  applied config when the live one will not parse (E-42).
- **Security.** symlink-safe chown under errand's home (E-54); proxy.env at
  0600, quoted (E-46); the Zabbly keyring must hold exactly the pinned key
  (E-55); a per-instance process limit, and `bailey doctor` reporting the
  real limits (E-63). Found and not closed: the broker needs proxy devices,
  which let the errand account reach the admin socket (E-61, T-020).
- **The image.** `scripts/build-image.sh` builds it with distrobuilder and
  records an inputs digest; `errand-box image import` checks SHA256SUMS and
  records the digest so setup takes it as current.
- **aarch64.** The `cross` CI step builds for the other architecture and
  runs every test under qemu-user.

### 2026-10-23T08:30Z: errand-box 0.1.0, built and tested on one host

**Record:** [TODO/PROGRESS.md](TODO/PROGRESS.md), entries T-001 to T-008 in
[TODO/INDEX.md](TODO/INDEX.md), and [docs/tests.md](docs/tests.md).
Not deployed and not released: no tag, no release asset. CI is off on this
repository; every step ran through `scripts/ci-local.sh`.

- `errand-box setup` installs Incus (Zabbly, the 6.0 LTS line on cgroup v1),
  s6 on a host with no init, errand and kage at pinned digests; builds the
  guest image; and configures errand's restricted Incus project, a golden
  instance and errand itself. A second run changes nothing.
- The `bailey` shim gives each errand session its own system container:
  systemd, Docker, podman (root and rootless) and FUSE inside; a 20 GiB
  quota; 80% of the host's memory and CPUs; no network device except the
  link to errand's broker. errand v0.3.1 runs unchanged, and its own probe
  reports that the backend enforces every guarantee.
- `errand-box up` restores everything a restart loses. The supervisor's
  daemons log to bounded files; a daemon that restarts in a loop is
  reported with its last log line.
- 33 host conditions, each with a fallback and a test, in
  [docs/edge-cases.md](docs/edge-cases.md). Four of them (E-29, E-30, E-33,
  and errand's refused config key) were found by the live tests of this
  session.
- The research: the engine claims checked on a second host
  ([docs/research/engines.md](docs/research/engines.md)), and every route
  without root measured and ranked
  ([docs/research/rootless.md](docs/research/rootless.md)).
- Scheduled workflows: `freshness` opens a pull request for each newer
  errand, kage or Rust; `watchdog` opens an issue when an upstream that
  setup reads is gone.
