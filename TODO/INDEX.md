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
Those three are done and shown live on one host. The fourth, T-029, is a
guest that can send the operator's provider key off the host, through the
broker of the errand errand-box installs. It is P0 because it loses a
credential, and it is first in the work order.

The open P1 entries are errand as errand-box's own build (T-028), the
host-side push that can publish another session's repository, a broker link
that does not need proxy devices,
which closes the path from the errand account to the admin socket, a
session with a real chat service, rootless podman on Ubuntu 24.04 hosts,
errand-box used as a user uses it on three hosts the operator named (WSL2,
Daytona, ModelScope Studio), and the one path of the scheduled workflows
that has not run: a pull request.
systemd and musl are shown live, the first release is out, and every
workflow has run on GitHub. A host type is P1 rather than P0 because the
code paths exist and the failure, if any, is at setup, before errand runs.

P2 and P3 make errand-box cheaper or clearer: rootless setup, a published
image, a pool size that fits the disk, errand's wording, a docs check, a
narrower AppArmor route, two live checks that depend on timing, the smaller
defects in errand's broker and file reads, a relay that says why it dropped
a connection, a smaller fork, and a native Incus backend in the fork, which
retires the `bailey` stand-in.

---

## Counts

**34 items. 18 open, 5 partial, 1 blocked, 10 done.**

| priority | open | partial | blocked | done | total |
| --- | --- | --- | --- | --- | --- |
| P0 | 1 | 0 | 0 | 3 | 4 |
| P1 | 4 | 3 | 1 | 5 | 13 |
| P2 | 10 | 2 | 0 | 2 | 14 |
| P3 | 3 | 0 | 0 | 0 | 3 |
| **All** | 18 | 5 | 1 | 10 | 34 |

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
| [T-027](setup.md) | P1 | setup | open | Use errand-box as a user does, on three hosts, and file what it finds |
| [T-028](errand.md) | P1 | errand | partial | errand is errand-box's hard fork, built, tested and shipped from this tree |
| [T-029](errand.md) | P0 | errand | open | A guest sends the operator's provider key to a host it names |
| [T-030](errand.md) | P1 | errand | open | The host-side push can publish another session's repository |
| [T-031](errand.md) | P2 | errand | open | The relay drops a connection with no log line when its target does not answer |
| [T-032](errand.md) | P2 | errand | open | The broker's provider route: the smaller defects errand's tracker names |
| [T-033](errand.md) | P2 | errand | open | Files the daemon reads from a session are read whole, and an upload is not redacted |
| [T-034](errand.md) | P3 | errand | open | Remove from the fork what errand-box does not run |
