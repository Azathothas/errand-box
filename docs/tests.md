# Tests

Five kinds, from the cheapest. Each one prints PASS or FAIL per check and
exits 0 only when every check passed. Read the exit code unpiped.

| kind | needs | command |
| --- | --- | --- |
| 1. unit and integration tests | cargo, on Linux | `cargo test --locked` |
| 2. the local CI steps | cargo, shellcheck, python3 with python3-jsonschema, and musl-tools for the static build, on Linux | `sh scripts/ci-local.sh` |
| 3. the live tests | root on a host where `errand-box setup` finished | `sh tests/live/live.sh` |
| 4. other kinds of host | root on a host where Incus runs | `sh tests/hosts/host.sh NAME IMAGE` |
| 5. as a user, on a host you are given | sudo on that host, and a release | section 5 |

The crate builds for Linux only. On a Windows machine, run kinds 1 and 2 in
WSL2, from a checkout that has LF line endings (`.gitattributes` gives every
checkout LF): `wsl -e sh scripts/ci-local.sh` from the checkout's directory.

## 1. Unit and integration tests

`cargo test` runs the unit tests of every module and `tests/shim.rs`.

`tests/shim.rs` runs the real `bailey` shim against `tests/fake-incus`, a
POSIX sh stand-in for the Incus client that keeps instances as directories
and logs every call. It checks `doctor`, the probe, trust and a rewritten
policy, a whole session with errand's pipes, SIGTERM, SIGKILL caught by the
watchdog, and `reap`. Every call the shim makes goes through the `Runner`
trait; the unit tests answer it with `run::Fake`, so a test can state the
exact Incus calls a function makes.

A test named in [edge-cases.md](edge-cases.md) as `module::test` is one of
these.

errand's chat path is tested against `discord-mock/`, a local stand-in for
the part of Discord errand reaches (T-035). It runs in process, in the tests
and in CI only. errand reaches it only through
`errand/src/chat/endpoint.rs`, when `ERRAND_DISCORD_MOCK_URL` names a
loopback `http://` URL and the bot token begins with `mock.`. The stand-in's
own tests check serenity's handshake, Discord's thread rules, uploads, and
that every gateway event parses in serenity and in twilight-model; errand's
`chat::endpoint` and `daemon::tests::stand_in` tests drive errand's own code
against it. A route or a body field it does not implement answers 501 and
fails the test.

## 2. The local CI steps

[`scripts/ci-local.sh`](../scripts/ci-local.sh) runs the steps that
`.github/workflows/gates.yml` runs, one by one, and the workflow calls the same
script, so the two cannot differ. CI runs them on every push to `main` and on
every pull request, all but `cross`. Run the script before every push all the
same: a CI job stops at its first failed step, and the steps after it are not
checked there.

| step | what it proves |
| --- | --- |
| `gate` | the template's checks (docs, markers, one home per sentence, placeholders, control bytes, changelog, attribution, secrets) and `scripts/check-todo.sh` |
| `kill-by-name` | no code signals a daemon by name (E-12) |
| `shellcheck` | every shell script this repository ships |
| `fmt`, `clippy` | format, and clippy with warnings as errors |
| `test` | section 1 |
| `discord-contract` | `scripts/discord-spec.sh --check`: the Discord stand-in answers the fifteen REST operations errand reaches in the shapes of Discord's OpenAPI spec, `discord/discord-api-spec` at a pinned commit. It refuses a snapshot whose digest is not the pinned one. Without python3-jsonschema it exits 2 |
| `pins` | `tests/pins-test.sh`: `scripts/pins.sh` against releases on `file://` URLs. A bump writes both digests and moves `latest`; a digest that differs from upstream's `.sha256` changes nothing |
| `errand-config` | `tests/errand-config-test.sh`: the errand built into errand-box accepts the config errand-box writes. It runs `errand-box serve` against the config from `tests/data/`. This step caught `output.postImages`, a key that errand v0.3.1 refused |
| `musl` | the static release binary builds for this machine, runs, and carries errand's `serve`. errand's crates compile C (T-028), so it needs `musl-gcc` from musl-tools, or `CC_<target>` set; without one it exits 2 |
| `cross` | the static binary builds for the other architecture (x86_64 or aarch64) with `rust-lld`, and the unit and integration tests pass there under qemu-user. It needs a musl C compiler for that architecture: `<arch>-linux-musl-gcc`, `CC_<target>`, or zig 0.16, on PATH or from `pip install ziglang==0.16.0`, which compiles C for either. Installed into a virtualenv, put that `python3` on PATH for this step only: first on PATH for the whole gate, it hides the system python3, and `discord-contract` exits 2 for want of python3-jsonschema. It also needs `qemu-<arch>-static`, and a `binfmt_misc` handler for that architecture, because `tests/shim.rs` starts the shim as a child. Without the C compiler it exits 2 before it builds; without the handler it runs the unit tests and exits 2; without qemu it only builds and exits 2 |

## 3. The live tests

[`tests/live/live.sh`](../tests/live/live.sh) starts real session
instances through the installed shim, as the errand account, with the policy
shape errand writes. It changes the host: `up` stops every daemon and starts
them again. Run it after `setup` and after any change to the shim, the image
or `up`. Name tests to run some of them: `sh tests/live/live.sh session inside`.

| test | what it checks | edge cases |
| --- | --- | --- |
| `probe` | `bailey doctor` as errand; the per-session check `run -- true` | |
| `session` | a session with a stand-in broker: the agent's uid and groups, the workspace, host ownership of what the agent writes, no network device and no default route, the broker reachable, direct egress blocked, the file size limit, the environment file deleted, systemd as PID 1, the instance stopped and its lease cleared | E-19, E-29 |
| `acp` | `kage rpc` in a session answers errand's `initialize` and `session/new` | |
| `inside` | Docker run and build, `sudo podman`, rootless podman, a FUSE mount by the agent, the CPU and memory limits as the guest sees them. Images are pulled through a relay that stands in for errand's broker: set `EB_LIVE_PROXY` (or `HTTPS_PROXY`), and `EB_LIVE_CA` for a proxy that intercepts TLS | E-03, E-04, E-06, E-16, E-29, E-30 |
| `quota` | a 5 GiB copy of the golden instance accepts 1 GiB and refuses 6 GiB | E-05, E-21 |
| `signals` | SIGTERM, then SIGKILL, to the launcher's process group: the instance stops and the lease clears | E-27 |
| `reap` | with the launcher and its watchdog both killed, `errand-box reap` stops the instance | E-27 |
| `maint` | `maint --once`, with fstrim | E-25 |
| `up` | every daemon stopped through s6, device modes reset to 0600, then `errand-box up`: every service up, a second `up` changes nothing, the probe passes | E-01, E-02, E-03, E-33 |
| `errand` | errand runs under the supervisor as `errand-box serve`, with no `errand` program installed, accepts its config, and its own probe of the shim says the backend enforces every guarantee | |

### The last run

2026-10-23, on the build host (Ubuntu 24.04 in a hosted container, kernel
6.18, x86_64, 4 CPUs, 16 GiB, cgroup v1, no init, no KVM), Incus 6.0.6,
errand v0.3.1, kage v0.5.0. The first run found four defects; each has a
fix, a row in [edge-cases.md](edge-cases.md), and a test:

| found by | defect | fixed by |
| --- | --- | --- |
| `inside` | `docker`: "permission denied" on its socket, because `incus exec --user` gives no supplementary groups | E-29 |
| `inside` | rootless podman: no subordinate ids, then pasta could not copy a default route | E-30 |
| `up` | after s6-svscan stopped, `up` reported it running and started nothing: the dead process was a zombie that PID 1 never reaped | E-33 |
| `errand` (setup) | errand v0.3.1 refused the config and restarted every second: `output.postImages` is a key of a later errand | the `errand-config` step |

The run after the fixes passed every check, 52 of 52:
[`tests/live/2026-10-23.txt`](../tests/live/2026-10-23.txt) is its output.

errand as a library errand-box runs, 2026-10-24, on a host of the same kind
with Incus 6.0.7: the static binary of `9b27e60` ran setup (30 changed, 0
failed; a second run changed nothing) and passed every check, 54 of 54,
two of them new for the library:
[`tests/live/2026-10-24-errand-library.txt`](../tests/live/2026-10-24-errand-library.txt).
The tests ran in two calls, the first three tests and then the rest.

The release v0.1.0 passed every check, 52 of 52, on a second host of the
same kind with nothing installed before: `bootstrap.sh` from the tag
installed the release binary, and that binary ran `setup` and the shim.
[`tests/live/2026-10-23-v0.1.0.txt`](../tests/live/2026-10-23-v0.1.0.txt)
is its output.

### On GitHub's runners

[`.github/workflows/live.yml`](../.github/workflows/live.yml), started by
hand, runs setup twice and these tests on an `ubuntu-24.04` and an
`ubuntu-24.04-arm` runner: real hosts with systemd as PID 1 and cgroup v2,
where setup takes Incus's stable line. A CONNECT proxy on the runner's
loopback is the host's only way out, as on the build host.

### What the live tests do not cover

- WSL2. It is an open entry in [`TODO/INDEX.md`](../TODO/INDEX.md).
- Rootless podman on a host with AppArmor's restriction on user namespaces,
  or with cgroup v2: it fails there, E-69 and E-70, and passes on cgroup v1.
- errand with a real chat service and a real model gateway. That needs a
  bot token, a channel, a user id, and a gateway key and URL; the operator
  supplies them as described in [`SECURITY.md`](../SECURITY.md).

## 4. Other kinds of host

[`tests/hosts/host.sh`](../tests/hosts/host.sh) makes another kind of host
as a nested Incus system container: systemd as PID 1 on Ubuntu and Debian,
OpenRC and musl on Alpine. In it, it installs errand-box through
`bootstrap.sh` from a release served the way GitHub serves one, runs setup
twice, runs the live tests, restarts the container, and checks that
errand-box comes back. The test credentials are not real, so the errand
daemon crash-loops (T-015); that is the one service the setup checks allow
to fail.

```sh
sh tests/hosts/host.sh u2404 ubuntu/24.04
sh tests/hosts/host.sh deb13 debian/13
sh tests/hosts/host.sh alpine alpine/3.22
sh tests/hosts/host.sh u2404 ubuntu/24.04 destroy
```

A nested host cannot make a loop-file pool, so it uses a `dir` pool and the
`quota` live test is skipped (E-05). Set `EB_HOSTS_CA` to the proxy's CA
when the egress proxy intercepts TLS.

A nested host cannot build the guest image either: the build is a third
level of containers. Set `EB_HOSTS_IMAGE` to a directory that
`scripts/build-image.sh` wrote, and the `setup` step imports that image
with `errand-box image import` before it runs setup. With it, Alpine 3.22
passed the live tests:
[`tests/live/2026-10-23-alpine-3.22.txt`](../tests/live/2026-10-23-alpine-3.22.txt).

## 5. End to end, as a user, on a host you are given

This is how an operator installs and uses errand-box, on a host with no
checkout: a remote machine over ssh, a web terminal, or a WSL2
distribution made for the test (setup installs Incus and changes the host).
It tests what a user downloads, not the tree.

Two downloads serve it. A version, such as
[v0.1.1](https://github.com/Azathothas/errand-box/releases/tag/v0.1.1), is
what `bootstrap.sh` installs. The prerelease
[`artefacts`](https://github.com/Azathothas/errand-box/releases/tag/artefacts),
rebuilt from `main` by dispatching `.github/workflows/artefacts.yml`, holds
what a test needs beside it, for x86_64 and aarch64, each in `SHA256SUMS`:

| asset | what it is |
| --- | --- |
| `errand-box-<arch>-linux-musl` | errand-box built from that commit |
| `discord-mock-<arch>-linux-musl` | the Discord stand-in (T-035), as a process |
| `errand-box-image-<arch>.tar` | the guest image, for a host that cannot build it |
| `errand-box-e2e-kit.tar.gz` | `bootstrap.sh`, `examples/`, `tests/data/`, `tests/e2e/`, the live tests |

### Before you start

- Linux on x86_64 or aarch64, and root or `sudo` without a password.
  `bootstrap.sh` uses `sudo -n` and never prompts.
- The host reaches `github.com`, `raw.githubusercontent.com`,
  `release-assets.githubusercontent.com` (where a release download
  redirects), its package mirrors, `pkgs.zabbly.com` on a Debian or Ubuntu
  host, and `images.linuxcontainers.org`. With an egress proxy, export
  `HTTPS_PROXY` before every command.
- Write down what the host is before you change it: `cat /etc/os-release`,
  `cat /proc/1/comm`, `stat -fc %T /sys/fs/cgroup`, `df -h /`.

### Install and set up

```sh
a=https://github.com/Azathothas/errand-box/releases/download/artefacts
mkdir -p ~/eb && cd ~/eb
curl -fsSLO "$a/SHA256SUMS" && curl -fsSLO "$a/errand-box-e2e-kit.tar.gz"
sha256sum --ignore-missing -c SHA256SUMS && tar -xzf errand-box-e2e-kit.tar.gz
cd errand-box-e2e-kit
sudo sh bootstrap.sh --version v0.1.1      # or --base-url "$a" for errand-box from main
errand-box doctor
```

`doctor` changes nothing and says what the host lacks. The values in
`tests/data` are not real, so errand stops at the chat connection, and
setup reports that one row as FAILED (T-015). `tests/e2e/mock.env` and
`mock-creds.env` point errand at discord-mock instead.

```sh
sudo errand-box setup --config examples/errand-box.toml --env tests/e2e/mock.env --creds tests/e2e/mock-creds.env
sudo errand-box setup --config examples/errand-box.toml --env tests/e2e/mock.env --creds tests/e2e/mock-creds.env
sudo errand-box status
```

The first setup builds the guest image, which takes minutes and needs
nested containers. Where that cannot run, take the image from `artefacts`:
setup with `--skip-image` starts Incus and stops at the missing image, then

```sh
arch=$(uname -m)
curl -fsSLO "$a/errand-box-image-$arch.tar" && sha256sum --ignore-missing -c ../SHA256SUMS
mkdir image && tar -xf "errand-box-image-$arch.tar" -C image
sudo errand-box image import image && sudo errand-box setup --config ... --env ... --creds ...
```

The second setup ends `0 changed`. Keep every output.

### errand against discord-mock

On a host set up with the two `tests/e2e` files:

```sh
curl -fsSLO "$a/discord-mock-$(uname -m)-linux-musl" && chmod +x discord-mock-*-linux-musl
sudo sh tests/e2e/mock-session.sh "demo: say hello"
```

It needs `curl` and `python3`. It starts the stand-in, runs errand
against it as the errand account, posts the prompt as the stand-in's
person, and passes when errand opens a thread and posts in it. Its logs and the stand-in's journal of every
request go to `./mock-session-<time>/`. The provider in `mock.env` answers
nothing, so the first model turn fails; give the `ERRAND_PROVIDER_*` lines
a real gateway for a model turn.

### The live tests on that host

They need `python3`, `socat` and `curl`. The `inside` test pulls container
images through a relay that stands in for errand's broker, so it needs an
HTTP CONNECT proxy on the host: the host's own egress proxy as
`EB_LIVE_PROXY`, or `tinyproxy` on loopback, as
`.github/workflows/live.yml` runs it.

```sh
sudo apt-get install -y python3 socat curl tinyproxy
printf 'Port 8888\nListen 127.0.0.1\nAllow 127.0.0.1\nTimeout 600\nConnectPort 443\n' | sudo tee /etc/tinyproxy/tinyproxy.conf >/dev/null
sudo systemctl restart tinyproxy            # Alpine: rc-service tinyproxy start
sudo env EB_LIVE_PROXY=127.0.0.1:8888 sh tests/live/live.sh
```

### Record and remove

Keep the outputs of `doctor`, both setups, `status`, the live tests and
`mock-session.sh`. A failure is a finding: an issue with the command and the
output that show it, and a row in [edge-cases.md](edge-cases.md) for a new
host condition. Cut every 64-character digest to 12 characters before it
goes into the tree, because the gate's public sweep refuses a long hex
string. Then remove what the test made:

```sh
sudo errand-box uninstall --purge --yes
```

`uninstall` keeps Incus, its pools and the errand account, and under s6
Incus's dnsmasq outlives it (T-036). On a host made for the test, delete the
host itself.
