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
| `cross` | the static binary builds for the other architecture (x86_64 or aarch64) with `rust-lld`, and the unit and integration tests pass there under qemu-user. It needs a musl C compiler for that architecture: `<arch>-linux-musl-gcc`, `CC_<target>`, or zig 0.16, on PATH or from `pip install ziglang==0.16.0`, which compiles C for either. It also needs `qemu-<arch>-static`, and a `binfmt_misc` handler for that architecture, because `tests/shim.rs` starts the shim as a child. Without the C compiler it exits 2 before it builds; without the handler it runs the unit tests and exits 2; without qemu it only builds and exits 2 |

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
| `errand` | errand runs under the supervisor, accepts its config, and its own probe of the shim says the backend enforces every guarantee | |

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

## 5. As a user, on a host you are given

This is how an operator installs errand-box, run on a host that has no
checkout: a remote machine over ssh, a web terminal, or a WSL2 distribution.
It tests the release, not the tree. Every file comes from the release tag,
so the scripts match the binary.

### Before you start

- The host needs Linux on x86_64 or aarch64, and root or `sudo` without a
  password. `bootstrap.sh` uses `sudo -n` and never prompts.
- The host needs to reach `github.com`, `raw.githubusercontent.com`,
  `release-assets.githubusercontent.com` (where a release download redirects,
  measured 2026-10-23), its package mirrors, `pkgs.zabbly.com` on a Debian or
  Ubuntu host, and `images.linuxcontainers.org`. With an egress proxy, export
  `HTTPS_PROXY` before every command.
- Write down what the host is before you change it: the distribution, the
  init, the cgroup version, whether it is a container, the free disk. The
  probe of this repository answers that from a checkout; on the host,
  `cat /etc/os-release`, `cat /proc/1/comm`, `stat -fc %T /sys/fs/cgroup`
  and `df -h /` do.

On a Windows machine that drives the hosts, the ssh client is Windows'
OpenSSH. A host whose credential is a token in the ssh user name, as
Daytona's is, gets an alias in `%USERPROFILE%\.ssh\config`, a file only your
account can read, so the token is never on a command line:

```text
Host eb-daytona
  HostName ssh.app.daytona.io
  User THE_TOKEN
```

Then `ssh eb-daytona` is the only command that names it. For WSL2, use a
distribution made for this test, not the one that builds this repository,
because setup installs Incus and changes the host. A distribution with
systemd as PID 1 has `systemd=true` under `[boot]` in `/etc/wsl.conf`. Read
`wsl.exe`'s own output as UTF-16
([conventions/shell.md](conventions/shell.md) section 7).

### Install and set up

```sh
v=v0.1.1                     # v0.1.0 until v0.1.1 is released
base="https://raw.githubusercontent.com/Azathothas/errand-box/$v"
mkdir -p ~/eb && cd ~/eb
curl -fsSLO "$base/bootstrap.sh"
sh bootstrap.sh --version "$v" --dry-run
sh bootstrap.sh --version "$v"
errand-box doctor
```

`doctor` changes nothing and says what the host lacks. Then the three
inputs. The values in `tests/data` are not real, so errand's daemon stops at
the chat connection, and setup reports that one row as FAILED (T-015). With
real credentials, write the two env files from
[`../SECURITY.md`](../SECURITY.md) and T-015 instead.

```sh
curl -fsSLO "$base/examples/errand-box.toml"
curl -fsSL -o errand-box.env "$base/tests/data/errand.env"
curl -fsSL -o creds.env "$base/tests/data/creds.env"
chmod 600 creds.env
sudo errand-box setup --config ./errand-box.toml --env ./errand-box.env --creds ./creds.env
sudo errand-box setup --config ./errand-box.toml --env ./errand-box.env --creds ./creds.env
sudo errand-box status
```

The second setup ends `0 changed`. Keep both outputs.

### The live tests on that host

They need `python3`, `socat` and `curl`. The `inside` test pulls container
images through a relay that stands in for errand's broker, so it needs an
HTTP CONNECT proxy on the host. Use the host's own egress proxy as
`EB_LIVE_PROXY`. On a host with direct egress, run `tinyproxy` on loopback,
as `.github/workflows/live.yml` does:

```sh
sudo apt-get install -y python3 socat curl tinyproxy
printf 'Port 8888\nListen 127.0.0.1\nAllow 127.0.0.1\nTimeout 600\nConnectPort 443\n' | sudo tee /etc/tinyproxy/tinyproxy.conf >/dev/null
sudo systemctl restart tinyproxy
mkdir -p live
for f in live.sh acp.py; do curl -fsSL -o "live/$f" "$base/tests/live/$f"; done
sudo env EB_LIVE_PROXY=127.0.0.1:8888 sh live/live.sh
```

On Alpine, `apk add python3 socat curl tinyproxy` and start it with
`rc-service tinyproxy start`. Where nothing supervises it, run `tinyproxy`
in the foreground of a second terminal, with the same configuration file.

### Record and remove

Save the outputs of `doctor`, both setups, `status` and the live tests in
`tests/live/DATE-HOST.txt`. Cut every 64-character digest to 12 characters,
because the public sweep of the gate refuses a long hex string. A failure is
a finding: give it a row in [edge-cases.md](edge-cases.md) or an entry in
`TODO/`, with the command and the output that show it. Then remove what the
test made:

```sh
sudo errand-box uninstall --purge --yes
```

`uninstall` keeps Incus, its pools and the errand account. On a host made
for the test, delete the host itself.
