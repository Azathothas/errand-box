# TODO

Every entry, one line each, sorted by id. The entry itself is in the file
its row links to, and it closes there with its own acceptance command, run,
and the output recorded.

What to work on next is not here: the work order is in
[`PROGRESS.md`](PROGRESS.md), and only there. This file has the list, the
definitions, the counts and the argument behind the priorities.
[`RULES.md`](RULES.md) is how this repository is worked on.

The counts below come from the rows. Do not type them:

```sh
sh scripts/check-todo.sh --set T-NNN done   # move a status; every count follows
sh scripts/check-todo.sh --fix              # rewrite every count from the rows
sh scripts/check-todo.sh                    # check; the gate runs this
```

---

## Priority

- **P0** breaks correctness, loses data, or takes the process down.
- **P1** a documented capability does not work, or was never shown to work.
- **P2** worth doing; nothing is wrong without it.
- **P3** worth recording so it is not found again.

## Effort

S is under a day. M is a few days. L is a week. XL is two entries.

## Status

`open`, `partial`, `blocked`, `done`. There is no `wontfix` and no
`deferred`: a blocked entry stays open with the blocker named.

---

## The argument behind the priorities

The P0 entries are what errand-box is: one command that sets a host up, a
session that is a whole system, and daemons that come back after a restart.
All four are done. The fourth, T-029, was a guest that could send the
operator's provider key off the host through errand's broker; it is fixed
in errand-box's fork, and v0.1.1 is the first release that carries the
fork (T-028, done).

The open P1 entries are a Discord stand-in for testing errand's chat path
(T-035, partial), a broker link
that does not need proxy devices,
which closes the path from the errand account to the admin socket, a
session with a real chat service, rootless podman on Ubuntu 24.04 hosts,
the one path of the scheduled workflows that has not run (a pull request),
and what T-027's run on WSL2, Daytona and ModelScope Studio found: a setup
that stops early and skips the sandbox steps without a word (T-040), a
provider setting that never reaches the agent (T-041), an uninstall order
that leaves every instance (T-046), WSL2 that stops an idle distribution
(T-038) or cannot start a guest without systemd (T-039), and a
user-namespaced host where no guest starts (T-042). Three P1 entries from
the third-party review of 2026-10-25 are security: root follows a symlink
under an errand-owned directory, a second route to root beyond E-61 and
T-020 (T-079); the freshness workflow runs an upstream tag through `sh -c`
with its write token (T-080); and the clone of a pull request reads the
session's git directory seconds after the check that T-030 added, so a guest
can swap it in between (T-081).
systemd and musl are shown live, the first release is out, and every
workflow has run on GitHub. A host type is P1 rather than P0 because the
code paths exist and the failure, if any, is at setup, before errand runs.

P2 and P3 make errand-box cheaper or clearer: rootless setup, a published
image, a pool size that fits the disk, errand's wording, a docs check, a
narrower AppArmor route, two live checks that depend on timing, the smaller
defects in errand's broker and file reads, a smaller fork, and a native Incus backend in the fork, which
retires the `bailey` stand-in. The 2026-10-25 review adds the broker's egress
boundary (T-082 a proxy the broker and sessions use, T-083 the allowlist and
its silent refusals, and the private-address arms folded into T-032), the
host-capability probe and tier chooser (T-084), the constrained-host sizing
and the limits errand is told (T-085), the broker that panics rather than
returning an error (T-089), the gate that never requires an entry's prove
output (T-087), the two credential files that are not ignored (T-088), and a
second `lite` engine for hosts Incus cannot serve (T-086, a researched design
the operator filed as non-blocking: it does not replace the settled Incus
decision).

---

## Counts

**89 items. 61 open, 7 partial, 1 blocked, 20 done.**

| priority | open | partial | blocked | done | total |
| --- | --- | --- | --- | --- | --- |
| P0 | 0 | 0 | 0 | 4 | 4 |
| P1 | 8 | 4 | 1 | 10 | 23 |
| P2 | 36 | 2 | 0 | 6 | 44 |
| P3 | 17 | 1 | 0 | 0 | 18 |
| **All** | 61 | 7 | 1 | 20 | 89 |

---

## Entries

| ID | Priority | Category | Status | Item |
| --- | --- | --- | --- | --- |
| [T-001](setup.md) | P0 | setup | done | One command sets up a host |
| [T-002](sandbox.md) | P0 | sandbox | done | errand runs unchanged, each session in its own system container |
| [T-003](setup.md) | P0 | setup | done | The daemons run, and come back, on a host with no init |
| [T-004](sandbox.md) | P1 | sandbox | done | A session's processes cannot outlive it |
| [T-005](sandbox.md) | P1 | sandbox | done | Docker, podman and FUSE work inside a session |
| [T-006](setup.md) | P1 | setup | done | The config setup writes is one the pinned errand accepts |
| [T-007](upkeep.md) | P1 | upkeep | partial | Scheduled workflows that open pull requests and issues |
| [T-008](setup.md) | P2 | setup | done | Bounded logs, and a daemon that restarts in a loop is reported |
| [T-009](upkeep.md) | P1 | upkeep | done | The first release |
| [T-010](setup.md) | P2 | setup | open | Setup without root at run time |
| [T-011](setup.md) | P1 | setup | done | Live test on a host with systemd |
| [T-012](setup.md) | P1 | setup | partial | Live test on musl and on WSL2 |
| [T-013](setup.md) | P2 | setup | partial | Live test on aarch64 |
| [T-014](sandbox.md) | P2 | sandbox | partial | A distrobuilder image, built once and published |
| [T-015](sandbox.md) | P1 | sandbox | blocked | A live session with a real chat service and model gateway |
| [T-016](sandbox.md) | P3 | sandbox | open | errand's description of this backend is partly wrong |
| [T-017](upkeep.md) | P3 | upkeep | open | A documentation check: every command, key and path a page names exists |
| [T-018](setup.md) | P2 | setup | done | Choose a pool size that fits the host disk |
| [T-019](sandbox.md) | P2 | sandbox | open | An incus backend in errand |
| [T-020](sandbox.md) | P1 | sandbox | open | A broker link that does not need proxy devices |
| [T-021](sandbox.md) | P2 | sandbox | open | Operator-chosen session credentials, safely |
| [T-022](sandbox.md) | P2 | sandbox | open | Egress a session can use beyond the HTTP broker |
| [T-023](setup.md) | P2 | setup | open | Observability: metrics over time, and session traces |
| [T-024](sandbox.md) | P1 | sandbox | open | Rootless podman under AppArmor's user namespace restriction, and on cgroup v2 |
| [T-025](sandbox.md) | P2 | sandbox | open | An AppArmor route for user namespaces in guests, without the host-wide sysctl |
| [T-026](upkeep.md) | P2 | upkeep | open | Live checks that read a state once, at a moment they do not control |
| [T-027](setup.md) | P1 | setup | done | Use errand-box as a user does, on three hosts, and file what it finds |
| [T-028](errand.md) | P1 | errand | done | errand is errand-box's hard fork, built, tested and shipped from this tree |
| [T-029](errand.md) | P0 | errand | done | A guest sends the operator's provider key to a host it names |
| [T-030](errand.md) | P1 | errand | done | The host-side push can publish another session's repository |
| [T-031](errand.md) | P2 | errand | done | The relay drops a connection with no log line when its target does not answer |
| [T-032](errand.md) | P2 | errand | open | The broker's provider route: the smaller defects errand's tracker names |
| [T-033](errand.md) | P2 | errand | open | Files the daemon reads from a session are read whole, and an upload is not redacted |
| [T-034](errand.md) | P3 | errand | partial | Remove from the fork what errand-box does not run |
| [T-035](errand.md) | P1 | errand | partial | A Discord stand-in, so errand's chat path is tested without Discord |
| [T-036](setup.md) | P3 | setup | open | `uninstall` stops incusd under s6 and leaves its dnsmasq running |
| [T-037](setup.md) | P2 | setup | open | errand's refusals restart every two seconds, and a transient one looks the same |
| [T-038](setup.md) | P1 | setup | open | WSL2 stops an idle distribution, and errand with it |
| [T-039](setup.md) | P1 | setup | open | WSL2 without systemd cannot start a guest, and setup says all is well |
| [T-040](setup.md) | P1 | setup | open | setup stops after `up` when a daemon failed, and box keys go unapplied |
| [T-041](errand.md) | P1 | errand | open | ERRAND_PROVIDER with a custom gateway never reaches kage |
| [T-042](setup.md) | P1 | setup | open | A host that is a 65536-id user namespace cannot start isolated guests, and setup promises 15257 |
| [T-043](setup.md) | P2 | setup | open | setup never starts a guest |
| [T-044](setup.md) | P2 | setup | open | doctor and setup do not recognise a host where Incus cannot run |
| [T-045](setup.md) | P2 | setup | open | `up` restarts the services it has just created, and on Daytona lxcfs never returns |
| [T-046](setup.md) | P1 | setup | open | `uninstall --yes`, then `--purge --yes`, leaves every instance and image |
| [T-047](setup.md) | P2 | setup | open | network.proxy is not validated, and a bad proxy outlives `proxy = "auto"` |
| [T-048](setup.md) | P2 | setup | open | setup reads `--env` and `--creds` after it changes the host |
| [T-049](errand.md) | P2 | errand | open | A `mock.` token without the stand-in's URL goes to Discord |
| [T-050](setup.md) | P2 | setup | open | setup and status misreport a refusing errand |
| [T-051](sandbox.md) | P2 | sandbox | open | bailey doctor reports every limit as unset |
| [T-052](setup.md) | P2 | setup | open | On WSL2 the pool is sized from the virtual disk |
| [T-053](setup.md) | P2 | setup | open | WSL2 distributions share one network namespace, and the build bridge name collides |
| [T-054](sandbox.md) | P2 | sandbox | open | The image build is fsync-bound and prints nothing for half an hour |
| [T-055](setup.md) | P2 | setup | open | setup reports the configured pool driver, and E-05 only once |
| [T-056](setup.md) | P3 | setup | open | Pool and image failures on a sandbox name no way out |
| [T-057](setup.md) | P2 | setup | open | On Arch, s6 cannot be installed with pacman |
| [T-058](sandbox.md) | P2 | sandbox | open | A session with no broker loses a race with the guest's tmpfiles |
| [T-059](errand.md) | P2 | errand | open | The message that starts a thread never gets its reaction |
| [T-060](errand.md) | P2 | errand | open | `!allow` confirms access for an account errand ignores |
| [T-061](sandbox.md) | P2 | sandbox | open | A session that cannot start tells the chat only Incus's hint |
| [T-062](setup.md) | P2 | setup | open | Guest memory follows /proc/meminfo, not the container's limit |
| [T-063](setup.md) | P2 | setup | done | Unknown options are ignored, and `gc --dryrun` deletes |
| [T-064](setup.md) | P2 | setup | open | `logs` with no name is refused under systemd |
| [T-065](setup.md) | P2 | setup | open | The README's quick start cannot be followed |
| [T-066](sandbox.md) | P2 | sandbox | open | `image.source = "release"` downloads the published image |
| [T-067](setup.md) | P3 | setup | open | bootstrap.sh prints the exit code in its errors |
| [T-068](setup.md) | P3 | setup | open | doctor's next step is wrong when Incus is missing or down |
| [T-069](setup.md) | P3 | setup | open | Reports print every row twice |
| [T-070](setup.md) | P3 | setup | open | doctor as a user who is not root reports E-08 |
| [T-071](setup.md) | P3 | setup | open | "No Zabbly build" when Zabbly cannot be reached |
| [T-072](setup.md) | P3 | setup | open | status counts ext4's root reserve as used |
| [T-073](setup.md) | P3 | setup | open | Input errors lose the key, the line or the file |
| [T-074](errand.md) | P3 | errand | open | Every failed prompt is posted twice |
| [T-075](upkeep.md) | P3 | upkeep | open | live.sh's up test does nothing under systemd |
| [T-076](upkeep.md) | P3 | upkeep | open | mock-session.sh passes when the session never started |
| [T-077](errand.md) | P3 | errand | open | discord-mock has no HTTP control for delete, archive and commands |
| [T-078](upkeep.md) | P3 | upkeep | open | feature.yml applies a label that does not exist |
| [T-079](setup.md) | P1 | setup | partial | root follows a symlink under an errand-owned directory, a second route to root |
| [T-080](upkeep.md) | P1 | upkeep | done | freshness.sh runs an upstream release tag through sh -c, with the write token |
| [T-081](errand.md) | P1 | errand | done | refuse_borrowed is checked seconds before the clone, so the git directory can be swapped |
| [T-082](errand.md) | P2 | errand | open | The broker reaches egress only directly, and no session trusts a proxy's CA |
| [T-083](errand.md) | P2 | errand | open | A refused egress is silent, ERRAND_EGRESS_ALLOW replaces the defaults, and ports are global |
| [T-084](setup.md) | P2 | setup | open | doctor, setup and the shim report capability by literal, by euid, or not at all |
| [T-085](setup.md) | P2 | setup | open | errand-box is sized for a roomy host, and does not tell errand the real numbers |
| [T-086](sandbox.md) | P3 | sandbox | open | A lite engine for hosts Incus cannot serve |
| [T-087](upkeep.md) | P2 | upkeep | open | The gate does not require an entry's prove output, and no live check crosses the provider boundary |
| [T-088](upkeep.md) | P2 | upkeep | done | creds.env and errand-box.env are not ignored, and the secret check does not match them |
| [T-089](errand.md) | P2 | errand | done | The broker panics instead of returning an error when a loopback bind is denied |
