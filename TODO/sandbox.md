# The sandbox: the shim and the guest

Entries about the `bailey` shim and the guest image. [INDEX.md](INDEX.md)
lists every entry.

---

# T-002: errand runs unchanged, each session in its own system container

**Source:** the operator (next-session.md section 2 of the agent-box research)
**Category:** sandbox
**Priority:** P0
**Effort:** L
**Status:** done

## Problem

errand's own sandboxes are host processes (bailey, with Landlock) or
hardened app containers (podman). Neither runs systemd, Docker or a package
manager.

## Premise

READ and measured: errand reaches its sandbox only through the program
`bailey` on its PATH ([docs/bailey-contract.md](../docs/bailey-contract.md));
its podman backend asks for flags a system container cannot keep (section
1.1 there).

## Approach

`src/shim.rs`, `src/session.rs`, `src/trust.rs`: the bailey calls, mapped to
an Incus instance per session, copied from a golden instance.

## Prove

```sh
cargo test --locked
sh tests/live/live.sh probe session acp errand
```

## Closing

**Closed 2026-10-23T08:40Z.** `tests/shim.rs` passes against the fake Incus;
live, errand v0.3.1 under the supervisor runs its own probe through the
shim:

```text
PASS session: session exit 0 in 4273 ms
PASS acp: initialize: result (3.33 s);session/new: result (3.34 s);launcher exit: 0 (4.41 s)
PASS errand: errand's own probe of the bailey shim passed: the backend enforces every configured guarantee
```

---

# T-004: A session's processes cannot outlive it

**Source:** docs/bailey-contract.md section 8: errand has no orphan cleanup
for this backend
**Category:** sandbox
**Priority:** P1
**Effort:** M
**Status:** done

## Approach

The shim stops the instance on SIGTERM; a watchdog in its own session stops
it when the shim's pipe closes; `maint` reaps what both miss; the profile
sets `boot.autostart=false` (E-27, E-28).

## Prove

```sh
sh tests/live/live.sh signals reap
```

## Closing

**Closed 2026-10-23T08:40Z.**

```text
PASS signals: SIGTERM to the launcher's process group: instance STOPPED after 1015 ms
PASS signals: SIGKILL to the launcher's process group: instance STOPPED after 1126 ms
PASS reap: launcher and watchdog killed: instance still RUNNING, as after a crash
PASS reap: errand-box reap: instance STOPPED
```

---

# T-005: Docker, podman and FUSE work inside a session

**Source:** the operator; found broken by the live `inside` check, 2026-10-23
**Category:** sandbox
**Priority:** P1
**Effort:** M
**Status:** done

## Problem

The first live run: `docker` got "permission denied" on its socket, and
rootless podman failed in newuidmap, then in pasta.

## Premise

Measured: `incus exec --user` gives no supplementary groups (E-29); the
guest's 65536 ids had no subordinate range, and pasta copies a default route
the guest does not have (E-30).

## Approach

`image/launch` starts as the guest's root and drops to the agent with
`setpriv --init-groups`; `image/provision.sh` gives the agent ids 1001 to
65535 and makes slirp4netns the rootless network. The image records a digest
of its inputs, so every host rebuilds it (E-31).

## Prove

```sh
sh tests/live/live.sh inside
```

## Closing

**Closed 2026-10-23T08:40Z.**

```text
PASS inside: docker run: 13.7
PASS inside: docker build and run: built
PASS inside: sudo podman run: 13.7
PASS inside: rootless podman as the agent (E-30): 13.7
PASS inside: FUSE mount as the agent: fuse-ok fuse.rclone
PASS inside: CPUs seen: 3, limit 3
PASS inside: memory seen: 12571 MiB of 16091 MiB on the host (limits.memory 80%)
```

An image file owned by an id above 64535 still needs `sudo podman`; that is
the size of the restricted project's range, stated in E-30.

---

# T-014: A distrobuilder image, built once and published

**Source:** the operator: "a distrobuilder image is in scope"
**Category:** sandbox
**Priority:** P2
**Effort:** M
**Status:** partial

## Problem

Each host builds the guest image itself: 220 to 502 s, and the result can
differ by the day of the build.

## Done, and open

[scripts/build-image.sh](../scripts/build-image.sh) builds the image with
distrobuilder (`image/distrobuilder.yaml`), running `image/provision.sh`
unchanged, and writes `OUT_DIR/inputs`, the digest `errand-box image
import` records so setup takes the image as current (E-31). A unit test
proves the shell and the Rust compute the same digest
(`image::tests::the_distrobuilder_build_records_the_same_inputs`). Built
and imported on the build host 2026-10-23 (a 395 MiB rootfs; `image import`
checked its SHA256SUMS, recorded the inputs and moved the alias). Open:
building it in a GitHub workflow and publishing it with its digest in
`pins.toml`, so a host imports a release instead of building (waits on
Actions, T-009).

## Prove

```sh
sudo sh scripts/build-image.sh OUT_DIR && errand-box image import OUT_DIR
sh tests/live/live.sh
```

---

# T-015: A live session with a real chat service and model gateway

**Source:** this session
**Category:** sandbox
**Priority:** P1
**Effort:** S
**Status:** blocked

## Problem

The live tests stop where errand needs a real chat service: a fake token
passes errand's probe and fails at the chat connection.

## Premise

Read at errand v0.3.1 (commit `97cea7de0265`):

- Discord is the only chat service. `src/chat/gateway.rs:243-247` asks for
  `GUILDS`, `GUILD_MESSAGES` and `MESSAGE_CONTENT`, and `Cargo.toml:31`
  depends on serenity.
- A message authored by a bot is ignored (`src/chat/inbound.rs:187`), so the
  trigger must come from a person.
- There is no chat endpoint override. `src/serve.rs` builds the client with
  no base URL, and serenity and reqwest validate TLS against roots compiled
  into the binary, so a local certificate in `/etc/ssl/certs` is not read.
- The broker refuses a host-local model gateway unless `allowInternal` is
  true. The default is false (`src/config/schema.rs:600`), and errand-box
  writes neither `allowInternal` nor a `models` list
  (`src/errandcfg.rs:90-155`).
- errand-box writes `"forwardToolOutput": false` (`src/errandcfg.rs:150`),
  so a thread shows the diff and the tool line, not the tool's output body.

One fact is inferred, not measured: errand's WebSocket client
(`tokio-tungstenite`) has no proxy support, so a host without direct TCP 443
to `gateway.discord.gg` should fail at `wait_ready` whatever the credentials.
The preflight below decides it in seconds. That is a second blocker, not
recorded before this research.

## Decision

| route | verdict | evidence |
| --- | --- | --- |
| A. A dedicated Discord application in a private test server, plus a hosted model key with a hard limit | recommended | The only route that needs no patch, no fork and no hand-edited errand config. |
| B. A real Discord bot plus a local OpenAI-compatible gateway | possible, unsupported | `kind: openai` forwards to `http://`, but the broker needs `allowInternal` and a gateway kage does not know needs a `models` list. errand-box writes neither; the workaround is a hand edit of errand's `config.json`, which this repository does not support. |
| C. A fully local Discord stand-in | refuted | No endpoint override, and the TLS roots are compiled in, so it needs a patched serenity or a patched errand. |
| D. A cheaper route in the existing config | refuted | Discord is the only transport. The web interface starts a session but still needs a token and does not exercise the message path. |

Take A. It costs a test application and a capped key, and it uses the
configuration errand-box already writes.

## Plan

Run on the build host, as root, after `setup` has finished. Values in
`$VARIABLE` form are the operator's. They go in files, never on a command
line.

1. **Preflight, before creating a credential.**

```sh
curl -sS -o /dev/null -w 'rest %{http_code}\n' https://discord.com/api/v10/gateway
timeout 5 bash -c 'exec 3<>/dev/tcp/gateway.discord.gg/443' && echo 'gateway ok'
```

Any HTTP status from the first line proves REST reachability. If the second
line prints nothing, stop: routes A and B both fail, and the host needs
direct egress or a transparent redirect before this entry can move.

2. **Create the Discord application.** New Application. Bot, Reset Token,
copy the token once into `creds.env`. Bot, Privileged Gateway Intents, enable
Message Content Intent and save. Public Bot off. OAuth2 URL Generator:
scopes `bot` and `applications.commands`; permissions View Channel, Send
Messages, Read Message History, Add Reactions, Attach Files, Create Public
Threads, Send Messages in Threads, Manage Threads. Open the URL, choose the
private test server, authorize. Create a text channel `errand-test`; a forum
or announcement channel does not work. Confirm the bot's role holds the
eight permissions in that channel. Enable Developer Mode and copy the
channel id and the user id.

3. **Create the model key.** Pick a provider id from errand's known list in
`src/sandbox/agent_config.rs:38-59`. Create a key for this test only, with a
hard limit, and note the model id. Leave the base URL empty for a known
provider.

4. **Write two files under `/root/t015`, mode 0600.**

| variable | file | value |
| --- | --- | --- |
| `ERRAND_CHAT_TOKEN` | `creds.env` | the bot token |
| `ERRAND_PROVIDER_CREDENTIAL` | `creds.env` | the model key |
| `ERRAND_CHANNEL_ID` | `errand-box.env` | the text channel id |
| `ERRAND_ALLOWED_USER_IDS` | `errand-box.env` | the operator's user id |
| `ERRAND_OPERATOR_USER_IDS` | `errand-box.env` | the same user id |
| `ERRAND_PROVIDER` | `errand-box.env` | a known provider id |
| `ERRAND_PROVIDER_BASE_URL` | `errand-box.env` | empty for a known provider |
| `ERRAND_PROVIDER_KIND` | `errand-box.env` | `openai` or `anthropic` |
| `ERRAND_PROVIDER_CREDENTIAL_NAME` | `errand-box.env` | the variable kage reads |
| `ERRAND_MODEL` | `errand-box.env` | the model id |

Do not set `ERRAND_GITHUB_TOKEN`. It enters every session as `GH_TOKEN`.
Leave `ERRAND_START_ON_MENTION` unset so a plain message starts a session.

5. **Configure and restart.**

```sh
errand-box setup --env /root/t015/errand-box.env --creds /root/t015/creds.env
errand-box up
errand-box status
errand-box logs errand -n 100
```

The log must show `[info] connected`. A rejected token and an unreachable
gateway give different messages, so read that line before the thread.

6. **Prove the bot, then the message path.** Set `ERRAND_CHAT_TOKEN` and
`ERRAND_CHANNEL_ID` in the current shell from the two files. Write the
authorization header to a file mode 0600, so the token is not in the process
list:

```sh
install -m 600 /dev/null /root/t015/auth
printf 'Authorization: Bot %s\n' "$ERRAND_CHAT_TOKEN" > /root/t015/auth
curl -sS -H @/root/t015/auth https://discord.com/api/v10/users/@me
curl -sS -H @/root/t015/auth "https://discord.com/api/v10/channels/$ERRAND_CHANNEL_ID"
```

The first answers `"bot": true`; the second answers `"type": 0` with the
guild. Post `!model` and fix `ERRAND_MODEL` if the model is absent. Then post
from the operator's own account, because bot messages are ignored:

> Create a file `hello.txt` in the workspace containing the single line
> `hello from errand`. Then run `docker run hello-world` and quote its
> output. Report both.

7. **Record the evidence.** Save the live output, the redacted API answers
and the thread transcript in `tests/live/YYYY-MM-DD-t015.txt`; replace the
user, channel and thread ids. Update `## Prove` below with the run, then
`sh scripts/check-todo.sh --set T-015 done`, update `TODO/PROGRESS.md` and
`docs/tests.md` section 3, and run `sh scripts/ci-local.sh` and
`sh tests/live/live.sh`.

8. **Clean up.** Post `!stop`. Run `errand-box gc --dry-run`, then
`errand-box gc`. Remove `/root/t015`. Reset the bot token or delete the
application, delete the test server, and revoke the model key.

## Blocker

The operator's credentials: a Discord bot token, a channel id and a user id,
and a model gateway key and base URL. [SECURITY.md](../SECURITY.md) lists the
permissions each needs. They go in the environment's secrets, not in a chat.
The preflight adds a second condition: direct TCP 443 to
`gateway.discord.gg`. The message path also needs a person to post the
trigger, because bot messages are ignored.

## Prove

A message in the channel starts a session; the agent edits a file in the
workspace and runs `docker run hello-world`; the thread shows the diff and
the tool line. The expected lines:

```text
PASS chat: errand connected to the chat service
PASS chat: a message started a session
PASS chat: the thread shows the file and the docker run
PASS chat: hello.txt is in the host workspace
```

The raw tool output body is not posted unless `forwardToolOutput` becomes
true. Either accept the diff and the tool line as "both", or add that key as
its own entry.

---

# T-016: errand's description of this backend is partly wrong

**Source:** errand's log on the build host, 2026-10-23
**Category:** sandbox
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

errand logs, for errand-box: "sessions run as confined host processes using
the host's own tools" and "outbound access is not restricted by
destination". Both are errand's fixed text for bailey. A session is a
guest with its own tools, and its only way out is errand's broker.

## Approach

Find which `bailey doctor` lines errand reads for these two notes
(`src/sandbox/bailey.rs` at the pinned errand), and whether a line the shim
can print truthfully changes them. errand stays unchanged.

## Prove

```sh
grep -c "confined host processes" /var/log/errand-box/errand/current
```

0 after the change.

---

# T-019: An incus backend in errand

**Source:** the operator, 2026-10-23; the specification is
[docs/incus-backend.md](../docs/incus-backend.md)
**Category:** sandbox
**Priority:** P2
**Effort:** L
**Status:** blocked

## Problem

errand reaches Incus only through errand-box's stand-in for `bailey`. That
costs a policy file round trip, a digest check on a file the guest can
write, doctor lines written for Landlock, and one limit for every session,
because errand's bailey policy carries no memory, CPU or process limit.
errand then describes the sandbox with bailey's words, which are wrong here
(T-016).

## Premise

Read at errand v0.3.1 (commit `97cea7de0265`) and on `main`
(`715e9b0b777a`):

- `SandboxBackend` is a closed enum of two values, `Podman` and `Bailey`
  (`src/config/schema.rs:17-22`).
- `Backend` is an enum, not a trait (`src/sandbox/mod.rs:245-250`), and the
  module says "It is not a plugin system" (`src/sandbox/backend.rs:5`).
- The backend string is parsed in two arms
  (`src/config/validate.rs:1527-1533`).
- `incus` does not appear in `src/`, `docs/` or `README.md`, and no upstream
  issue or pull request mentions it. The author has not been asked and has
  not ruled.
- errand's configuration already holds `memory`, `cpus` and `pids`
  (`src/config/schema.rs:411-416`).
- The policy writer drops them: it writes only `file_max`, `tmp_size` and
  `shm_size` (`src/sandbox/policy.rs:292-311`).
- `bailey` already accepts and enforces them: `RawResources` at
  `bailey/crates/bailey/src/config.rs:342-353`, and the cgroup writes at
  `bailey/crates/bailey/src/backend/enforce.rs:2136-2143`.

The four `Backend` operations still fit a child process and HTTP, so the
backend is plausible. It is errand's change, and the closed enum is why it is
not errand-box's.

The limits are the part that is wrong today: `sandbox.memory`, `sandbox.cpus`
and `sandbox.pids` are read from errand's configuration, never named in the
policy, and never reported as inert. A session runs on the backend default
while the configuration says otherwise.

## Decision

| route | verdict | evidence |
| --- | --- | --- |
| A. A full incus backend in errand | plausible, blocked | The four operations fit, but the closed enum and "not a plugin system" make it errand's change. No upstream work exists and the author has not ruled. |
| B. A smaller generic seam | refuted | errand's own `AGENTS.md` forbids an abstraction for a second implementation that does not exist. |
| C. Keep the bailey stand-in and fix the limits | recommended now | The stand-in works, and the limits can be carried without the backend. |
| D. Implement the podman command surface | refuted | The podman backend refuses a non-rootless runtime and asserts `--read-only`, `--cap-drop=ALL` and `--userns=keep-id`; a system container cannot honour those. |

Take C now. Leave A blocked on the author. Record B and D as refused.

## Approach

Two parts, in this order.

### Part 1: the limits errand already holds

errand-box's policy parser rejects an unknown `resources` key
(`src/policy.rs:271-283`), so errand-box must accept the three keys before
errand writes them. The order is fixed.

1. In errand-box, add `memory: Option<u64>`, `pids: Option<u32>` and
   `cpu_percent: Option<u32>` to `Policy`, and accept `memory`, `pids_max`
   and `cpu_percent` in `resources`.
2. In errand, add `memory`, `cpus` and `pids` to `PolicyOptions`
   (`src/sandbox/policy.rs`), write `memory`, `pids_max` and
   `cpu_percent = round(cpus * 100).max(1)` in `policy_contents`, and pass
   the three at the session and probe sites in `src/sandbox/bailey.rs`.
   Add two policy tests: `6g`, `1.5` and `768` must produce
   `memory = "6g"`, `pids_max = 768` and `cpu_percent = 150`; a value below
   one percent must produce `1`.
3. In errand-box, apply the parsed values in `src/session.rs`:
   `limits.memory`, `limits.processes`, and `limits.cpu` for whole cores or
   `limits.cpu.allowance` as `<n>ms/100ms` for a fraction. The allowance
   spelling is inferred, not measured; the live test confirms it.
4. In `src/shim.rs`, report which limit came from errand's policy and which
   from the profile, so a mismatch is visible.
5. Add parser tests, a `tests/fake-incus` case for the three `incus config
   set` calls, and live checks that compare against errand's values and add
   `limits.processes`.

The errand half is 145 lines in four files and applies cleanly to v0.3.1 and
`main` (`git apply --check` exits 0 on both). It is not compiled here: this
host is Windows and the crate is Linux-only. Do not pin an errand version
that writes the keys until errand-box's parser has shipped.

### Part 2: the backend, if the author accepts

Add `SandboxBackend::Incus` and `IncusConfig { golden, guest_uid, root_size }`
to `src/config/schema.rs`; parse `"incus"` in `src/config/validate.rs`; add
`src/sandbox/incus.rs` with `probe`, `launch`, `list_orphans` and
`remove_orphans`, an `incus exec` child process (`-T`, `--cwd /workspace`,
no `--user`), environment through `incus file push`, and orphan listing
through `/1.0/instances?recursion=1`; add `Backend::Incus` and
`SandboxHandle::Incus` in `src/sandbox/mod.rs`; gate on the exact errand
versions that carry it. Add `sha2` only if the instance-name suffix is
hashed. Regenerate the config schema and the reference page. Port the
section 11 acceptance checks, and start the session through errand: today
`tests/live/live.sh` writes the policy and runs the shim itself, so it
proves errand-box, not errand.

## Decisions for errand's author

| decision | recommendation | who decides |
| --- | --- | --- |
| D1, who performs the host prerequisites | an installer outside errand, so errand never needs root; report what the probe cannot verify as a gap | the author, in part: whether errand may name a root-run daemon as a prerequisite |
| D2, one instance per session or one per launch | per session; drop `keepDays`, because `sweep_orphans` must not treat a kept instance as an orphan | the implementer |
| D3, a rootless incusd | later, after T-020, which is the unix-socket relay it needs | not the author, until D1 settles the root daemon |
| D4, the client for the exec | the `incus` client as a child process, so errand owns its pipes as it does `podman run`; the REST API for the rest | the implementer |

## Blocker

errand's author: whether errand takes an `incus` backend, and the decisions
D1 to D4. Unblock: an errand release with the backend, or a ruling against
it, in which case this entry records the ruling and errand-box keeps the
stand-in. Part 1 is blocked on nobody.

## Prove

Part 1:

```sh
sh tests/live/live.sh session
```

The session reads the memory and process limits from errand's configuration,
not from the profile.

Part 2, the entry's acceptance:

```sh
sh tests/live/live.sh   # with sandbox.backend = "incus" and no `bailey` on errand's PATH
```

Every check PASS.

---

# T-020: A broker link that does not need proxy devices

**Source:** the review of 2026-10-23 (security lens), measured in build
**Category:** sandbox
**Priority:** P1
**Effort:** M
**Status:** open

## Problem

errand's project allows proxy devices, because the broker link is two
proxy devices per session (`src/session.rs` `desired_devices`; setup sets
`restricted.devices.proxy=allow`, `src/setup.rs`). Incus has no setting
that limits what a proxy device may connect to, so the errand account can
attach one whose host end is any host socket, the Incus admin socket
included. That grants the errand account more than the restricted project
is meant to (E-61). A guest cannot add devices, so this is not reachable
from inside a session; it matters when the errand account itself is
compromised.

## Premise

Measured in build on Incus 6.0.6: with `restricted.devices.proxy=allow`,
the errand account attached such a device to a throwaway instance. With
`restricted.devices.proxy=block`, Incus refused the device for the errand
account and for root alike ("Proxy devices are forbidden"), so root cannot
add the broker devices on errand's behalf in a project that forbids them.

## Approach

Carry the broker over a unix socket instead: the shim already relays the
broker through a unix socket for the rootless engine (`relay::unix_to_tcp`,
E-23). Share that socket into the guest with a disk device, which the
project already allows under errand's home, and have the guest's boot step
listen on 169.254.169.1 and relay to it, so kage and the guest's services
see the same address. Then set `restricted.devices.proxy=block`. The guest
image changes, so every host rebuilds it (E-31).

## Prove

```sh
sh tests/live/live.sh   # the broker and system-proxy checks pass with proxy devices blocked
incus project get user-UID restricted.devices.proxy   # block
```

---

# T-021: Operator-chosen session credentials, safely

**Source:** the operator, 2026-10-23: "I want agents to use any [credential]
I choose via env vars, for instance an ssh key the agent can use to ssh into
another machine"
**Category:** sandbox
**Priority:** P2
**Effort:** M
**Status:** open

## Problem

What reaches the agent is entirely errand's per-session policy, which
errand-box trust-verifies (`src/shim.rs:262`) and then relays faithfully:
`[env] pass` names are read from bailey's own environment
(`src/shim.rs:283`, `src/policy.rs:409`) and `set` is applied verbatim, into
a 0600 file the launcher sources and deletes (`src/session.rs:390`). Two
consequences: errand's defaults flow through unfiltered (the GitHub token
reaches the guest verbatim, `docs/design.md:76`), and the operator has no
errand-box-side way to add a credential errand's policy does not already
grant. errand-box cannot simply bolt extra env or mounts on top, because the
policy is a trusted artifact signed as errand's; doing so silently would
break that trust.

## Approach: research and resolve the best balance between safety and usability

The task is to find where operator-chosen secrets belong without weakening
the trust model. Options to measure and compare, not to assume:
- An errand-box overlay the operator signs, merged with errand's policy
  under its own trust record, so the agent's environment and mounts are the
  union and the provenance of each is auditable.
- A secrets directory (e.g. `/etc/errand-box/secrets/`, mode 0700) whose
  files errand-box mounts read-only into the session at operator-named
  guest paths (an ssh key at `/state/home/.ssh/id_ed25519`, 0600), named in
  the host config rather than in errand's per-session policy.
- Deciding what, if anything, errand-box should redact or refuse to relay
  from errand's defaults (the verbatim git token), versus leaving that to
  errand.

Each option is a point on the safety/usability line; the entry resolves to
the one that lets the operator add arbitrary credentials while keeping every
grant verifiable and least-privilege. An ssh key is the worked example: it
is useless without T-022, so the two are resolved together.

## Prove

```sh
# the chosen mechanism puts an operator secret in a session, 0600, not in
# any argv or log, and absent from a session the operator did not grant it to
sh tests/live/live.sh
```

---

# T-022: Egress a session can use beyond the HTTP broker

**Source:** the operator, 2026-10-23: the ssh-into-another-machine case
**Category:** sandbox
**Priority:** P2
**Effort:** M
**Status:** open

## Problem

A session has no network device; its only path out is the two proxy devices
to errand's broker at `169.254.169.1:P`, under errand's host-and-port
allowlist (`src/session.rs` `desired_devices`, `docs/design.md:73`, measured:
a direct connection fails with errno 101). The guest's clients are pointed
at it as an HTTP proxy (`HTTPS_PROXY` in the policy `set`). Raw-TCP protocols
the agent may legitimately need - ssh to a machine it is diagnosing, a
database connection, a git+ssh remote - have no path: ssh is TCP/22, not
HTTP, and whether the broker tunnels it is errand's to decide, not
errand-box's.

## Approach: research and resolve the best balance between safety and usability

Measure what each egress shape costs in containment and buys in usability,
rather than opening the network:
- A per-session allowlist of raw-TCP host:port the broker (or an errand-box
  relay beside it) tunnels, so ssh to a named host works and nothing else
  does. This is largely errand's broker; errand-box's part is the transport
  and the allowlist plumbing.
- An operator-opt-in egress profile per session (still no general NIC) that
  errand-box carries into the policy errand already shapes with
  `sandbox.network` and `--proxy-net` (`docs/bailey-contract.md:268-272`).
- Keeping the default exactly as it is (HTTP allowlist only), so broader
  egress is never silent.

The entry resolves to the narrowest mechanism that makes the operator's
named outbound connection work while every other destination still fails
closed. It is resolved with T-021 (the credential is useless without the
path, and the path is dangerous without a scoped credential).

## Prove

```sh
# with an operator-granted ssh target, the agent reaches host:22 and no other
# host; every ungranted raw-TCP connection still fails
sh tests/live/live.sh
```
