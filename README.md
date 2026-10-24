# errand-box

errand-box puts errand and its agent in a sandbox that is a whole Linux
system. Each errand session gets its own Incus system container: systemd is
PID 1, and Docker, podman and FUSE work inside it. errand is errand-box's
hard fork of [QaidVoid/errand](https://github.com/QaidVoid/errand), in
[errand/](errand/README.md), built into the errand-box binary. It calls
errand-box through its `bailey` backend.

`bootstrap.sh` installs the release binary for this machine, after it checks
the digest, and one command sets up the host:

```sh
curl -fsSLO https://raw.githubusercontent.com/Azathothas/errand-box/main/bootstrap.sh
sh bootstrap.sh
errand-box setup --config ./errand-box.toml --env ./errand-box.env --creds ./creds.env
```

It installs Incus, builds the guest image, links errand to errand-box,
installs kage at a pinned digest, writes errand's configuration and starts
the daemons. Run it
again at any time: a second run changes nothing. After a host restart, run
`errand-box up`.

## What a session gets

| | |
| --- | --- |
| a system | Debian 13, systemd as PID 1, the agent as uid 1000 with sudo |
| inside | Docker (and build), podman as root, rootless podman on a cgroup v1 host (not yet where AppArmor restricts user namespaces or on cgroup v2: E-69, E-70), FUSE, a compiler set, Python, Node |
| disk | 20 GiB, enforced (btrfs quota) |
| memory and CPUs | 80% of the host's, enforced; lxcfs makes `free` and `nproc` show the limits |
| network | none, except errand's broker; direct egress fails |
| files on the host | only the workspace and the session's state directory, owned by errand's account |
| start | a stopped instance starts in about 0.3 s; a new one is a copy of a golden instance in about 0.5 s |

## Hosts

errand-box is made for Linux on x86_64 or aarch64, with glibc or musl,
with or without an init (systemd, or s6 that errand-box installs), and for
WSL2. It needs root for setup. The live tests have run on Ubuntu 24.04 with
no init and cgroup v1; Ubuntu 24.04 and Debian 13 with systemd, and Alpine
3.22 with OpenRC and musl, each nested in an Incus container; and GitHub's
Ubuntu 24.04 runners with systemd and cgroup v2, on x86_64 and aarch64.
[docs/tests.md](docs/tests.md) has each result. WSL2 has not run yet, and
the other open hosts are entries in [TODO/INDEX.md](TODO/INDEX.md).
[docs/research/engines.md](docs/research/engines.md) has the numbers above
with their conditions.

Without root at run time, Incus in a user namespace works at the same speed
but loses the disk quota. Without root at all, only emulation works, at 18
to 53 times the cost. [docs/research/rootless.md](docs/research/rootless.md)
ranks every route.

## Read next

| to | read |
| --- | --- |
| install, update and repair | [docs/operations.md](docs/operations.md) |
| see how it works | [docs/design.md](docs/design.md) |
| see what errand expects of `bailey` | [docs/bailey-contract.md](docs/bailey-contract.md) |
| see each host problem and its fallback | [docs/edge-cases.md](docs/edge-cases.md) |
| run the tests | [docs/tests.md](docs/tests.md) |
| report a vulnerability | [SECURITY.md](SECURITY.md) |
| work on this repository | [AGENTS.md](AGENTS.md) |

## Licence

0BSD. See [LICENSE](LICENSE).
