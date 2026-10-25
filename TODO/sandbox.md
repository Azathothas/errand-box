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
checked its SHA256SUMS, recorded the inputs and moved the alias).

The digest that build wrote was not errand-box's. The script's check for a
single quote in a setting reused the variable that held the kage version,
so `OUT_DIR/inputs` hashed the proxy URL, and setup took every imported
image as stale and rebuilt it (E-31). The unit test ran `--print-inputs`,
which exited before that check. Fixed 2026-10-23T13:23Z: the script takes
the digest once, and `--print-inputs` prints that value. Measured: the build
writes `d2d5988b7438`, the digest setup records for the image it builds.

distrobuilder has no Ubuntu 24.04 package. Measured 2026-10-23: the Go
module at commit `232e78c93721` (distrobuilder 3.3) builds in its own module
tree with `github.com/cyphar/filepath-securejoin` replaced by v0.5.2 and the
build tags `exclude_graphdriver_btrfs exclude_graphdriver_devicemapper
containers_image_openpgp`. Without the replace, `go.podman.io/storage`
v1.62.0 calls a function that filepath-securejoin v0.6.1 moved to
`pathrs-lite`. A build took 3 min 38 s on the build host.

Open: building it in a GitHub workflow and publishing it with its digest in
`pins.toml`, so a host imports a release instead of building. Actions runs
now (T-009 is done).

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

The operator ruled on 2026-10-23: run it on every host that passes T-027,
as root, after `setup` has finished. A host passes when setup exits 0 and
`tests/live/live.sh` has no FAIL that an E-row does not explain. Each host
gets its own run and its own evidence file. Values in `$VARIABLE` form are
the operator's. They go in files, never on a command line.

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
| `ERRAND_PROVIDER_KIND` | `errand-box.env` | empty for a provider kage knows; `openai` or `anthropic` for another endpoint |
| `ERRAND_PROVIDER_CREDENTIAL_NAME` | `errand-box.env` | optional: the variable kage reads the key from |
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
and the thread transcript in `tests/live/YYYY-MM-DD-t015-HOST.txt`, one file
per host; replace the user, channel and thread ids. Update `## Prove` below with the run, then
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

The steps of this plan are the list the operator follows. Unblock: the
files of step 4 on the host, and the preflight passing there. T-027's run
covers the chat path against discord-mock, and a model turn where the
operator gives it a provider key; a real Discord application stays this
entry's.

**Correction, 2026-10-24: the chat half can be tested without Discord.**
The research of 2026-10-24 finds that serenity can be pointed at a local
stand-in with no serenity patch, which reverses route C above for the chat
half. T-035 builds it. The model half still needs a gateway errand-box can
configure.

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

**Correction, 2026-10-24: errand is errand-box's code now (T-028).** The
two notes are fixed text in `errand/src/sandbox/bailey.rs:435` and
`:452-454`, so the fix is an edit there, or nothing once T-019's backend
writes its own notes.

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
**Status:** open

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

**Correction, 2026-10-23T15:00Z: the author has been asked.**
[QaidVoid/errand#41](https://github.com/QaidVoid/errand/issues/41), opened
by the operator at 13:12Z, proposes the backend with this entry's design and
names the two broker transports and the five project exceptions it needs.
Its one comment, at 13:15Z, is a triage note from an account with no role in
errand, not the maintainer's. It confirms the v0.3.1 references and lists
corrections. The maintainer has not answered. The corrections are claims
read from the comment, not measured here, and each is checked where it
lands before it is acted on:

| claim in the triage comment | where it lands | how to check it |
| --- | --- | --- |
| `limits.cpu` is a count or a cpuset, not a cap on use; a fraction of `cpus` maps to `limits.cpu.allowance` as a time quota (`150ms/100ms`), not set together with `limits.cpu` | Part 1, step 3 | the live test of Part 1, reading the cgroup's quota in a session |
| `limits.memory.swap` defaults to `true`; `false` makes `limits.memory` a ceiling | Part 1, and errand-box's own profile today | `incus config show` of a session, on a host that has swap |
| `limits.kernel.fsize` is a per-process limit, and is set before the start | Part 2, the limits table of [docs/incus-backend.md](../docs/incus-backend.md) | the Incus documentation at the pinned Incus |
| an instance name must not start with a digit or a dash, nor end with a dash | Part 2 | errand-box's own names: `session::tests::names` |
| on errand `main` (`715e9b0`), the agent command passes `--yolo` and an images endpoint exists | Part 2 | re-pin the line references before implementing |

**Correction, 2026-10-24: one citation holds for v0.3.1 only.** The two arms
of `parse_backend` are at `src/config/validate.rs:1527-1533` in v0.3.1 and at
`:1592-1598` on `main` (`715e9b0b777a`). Read in the sweep of
[docs/history/references/2026-10-24-errand-code.md](../docs/history/references/2026-10-24-errand-code.md).

## Decision

| route | verdict | evidence |
| --- | --- | --- |
| A. A full incus backend in errand | accepted, errand's to build | The four operations fit, but the closed enum and "not a plugin system" make it errand's change. Proposed as #41; the maintainer accepted it on 2026-10-23, with changes (below). |
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

**Correction, 2026-10-23T15:10Z: the maintainer has ruled.** QaidVoid, the
owner, answered every open question on #41 at 15:02Z. The table in section
10 of [docs/incus-backend.md](../docs/incus-backend.md) lists each ruling
and where it differs from the specification. The four that change this
entry:

- Yes, a third backend, in errand. Part 2 is errand's to build; this
  repository does not write to errand's.
- Proxy devices stay forbidden, and the broker reaches the guest over a unix
  socket on a disk device, with a guest-side relay. That is T-020's design,
  so errand-box needs no second transport.
- One instance per launch (D2), not one per session.
- The incusd runs as root, and `probe` requires the `incus` client as root
  or in `incus-admin` (D1, D3). errand's account in `incus-admin` is equal
  to root, which errand-box's restricted project exists to avoid
  ([SECURITY.md](../SECURITY.md), "What each compromise reaches").

The operator ruled on 2026-10-23, after reading this, on what errand-box
does when errand ships the backend: section 10 of
[docs/incus-backend.md](../docs/incus-backend.md) has the ruling. The opt-in
key goes in `errand-box.toml`, and `doctor`, `setup` and `up` name the
reason with the DEGRADED line.

## Blocker

An errand release that carries the `incus` backend. Then errand-box adds the
opt-in key, pins that release, and runs this entry's Part 2 acceptance with
the key on. Part 1 needs errand to write the three limits into the bailey
policy, which is also errand's change, and errand-box's parser must accept
them first (step 1, blocked on nobody).

**Correction, 2026-10-24: the blocker is gone.** The operator ruled that
errand-box takes errand into this tree as a hard fork (T-028). Part 2 is
errand-box's to build, in `errand/src/sandbox/`, and the maintainer's ruling
in section 10 of [docs/incus-backend.md](../docs/incus-backend.md) no longer
binds it: the errand account stays in errand-box's restricted project, not in
`incus-admin`, and the broker link is T-020's. Part 1 folds into Part 2,
because the backend reads `sandbox.memory`, `cpus` and `pids` itself. When
Part 2 passes, the `bailey` stand-in (`src/shim.rs`, `src/trust.rs`,
`src/policy.rs`, `tests/shim.rs`) and docs/bailey-contract.md move to
history.

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

## Decision

The operator ruled on 2026-10-23: the unix socket, in a per-session host
directory that the guest cannot write, shared into the guest read-only by a
disk device. Not in the session's state directory, which the guest writes
(SECURITY.md rule 1). If errand's maintainer chooses another transport for
the incus backend (#41, T-019), errand-box adds that transport as a
configuration option beside this one. On 2026-10-23 the maintainer chose
the same transport, a unix socket on a disk device with a guest-side relay,
so no second option is needed.

To measure first, before the design is built on it:

- that a guest connects to a unix socket on a read-only disk device;
- the socket's owner and mode, so that the agent (guest uid 1000, mapped to
  the errand account) and the guest's services (the guest's root, an isolated
  uid) both connect, and no other host account does;
- the guest-side relay: a systemd socket unit with `systemd-socket-proxyd`,
  or `socat`, listening on 169.254.169.1 for the broker port and for
  `guest_proxy_port`.

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

## Decision

The operator ruled on 2026-10-23: the secrets directory. Each secret is a
file in a directory only root can read (`/etc/errand-box/secrets/`, 0700).
`errand-box.toml` names it, the guest path it appears at, and which sessions
are granted it. errand-box mounts it read-only into a granted session, owned
by the agent and mode 0600. It is never in errand's policy, an argument
list or a log. A session that is not granted it does not see it. The signed
overlay lost: it is more machinery for the same grant, and it mixes the
operator's grants into errand's trusted policy.

To settle in the implementation: what a grant names a session by (all
sessions, or errand's project path, which the policy carries), and what
errand-box does with errand's own defaults, the GitHub token included.

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

## Decision

The operator ruled on 2026-10-23: a raw-TCP allowlist that errand-box
relays. `errand-box.toml` names the `host:port` pairs a session may reach,
off by default. The shim relays each pair over the same unix-socket link
T-020 builds, and the guest reaches each one at its own port on
169.254.169.1. The guest still has no network device, and every destination
that is not named fails. An HTTP-only default lost because it leaves the
operator's own example, ssh to a named machine, unreachable. This waits on
T-020's link.

To settle in the implementation: how a session finds a target by name (an
`/etc/hosts` entry in the guest, or an ssh `Host` alias the boot step
writes), and whether a grant is per session, as T-021's is.

## Prove

```sh
# with an operator-granted ssh target, the agent reaches host:22 and no other
# host; every ungranted raw-TCP connection still fails
sh tests/live/live.sh
```

---

# T-024: Rootless podman under AppArmor's user namespace restriction, and on cgroup v2

**Source:** measured 2026-10-23 on GitHub's ubuntu-24.04 runners, both
architectures (live.yml runs 39968392861 and 39970746432); the operator's
ruling of 2026-10-23
**Category:** sandbox
**Priority:** P1
**Effort:** M
**Status:** open

## Problem

On a host with `kernel.apparmor_restrict_unprivileged_userns = 1`, the
agent's `podman run` as guest uid 1000 fails with "cannot re-exec process"
(E-69). With the restriction off, on cgroup v2, it stops at "dbus: couldn't
determine address of session bus" (E-70). Docker and `sudo podman` pass on
the same hosts. T-005 says podman works inside a session, and on such a host
the rootless half does not. The check that fails is "inside: rootless
podman as the agent (E-30)" in `tests/live/live.sh`.

## Premise

Measured in run 39970746432: the audit line `apparmor="DENIED"
operation="userns_create"` with the sysctl at 1. With `sysctl -w
kernel.apparmor_restrict_unprivileged_userns=0` on the runner, podman gets
past it and stops at the dbus error. Every cgroup v1 host passes, because the
guest's boot step runs podman with cgroups disabled there
(`image/guest-init:12-23`, E-04).

Read, not measured: the agent has no systemd user session in the guest. The
launcher drops to uid 1000 with `setpriv` (`image/launch:30`), and nothing
enables lingering for the agent, so no systemd user manager for uid 1000
and no session bus exist.

## Decision

The operator ruled on 2026-10-23: opt in for now, and the research for a
narrower route is its own entry (T-025).

- A key in `errand-box.toml`, off by default, makes `up` set
  `kernel.apparmor_restrict_unprivileged_userns = 0`.
- The sysctl is host-wide: every unprivileged process on the host may then
  create user namespaces, which the restriction exists to stop. So while the
  key lifts it, `doctor`, `setup` and `up` report the host as DEGRADED and
  name the sysctl.
- With the key off on such a host, they report rootless podman in guests as
  unavailable, with E-69 and the key's name, and change nothing.

## Approach

1. The key, in `Engine` (`src/config.rs:23`), `examples/errand-box.toml` and
   [docs/operations.md](../docs/operations.md). A proposed name:
   `engine.lift_userns_restriction`. Check it against the other keys' names
   before it ships, because a key name is an interface.
2. In `src/up.rs`, beside `sysctls` (`src/up.rs:357`), called from
   `src/up.rs:1241`. With the key on and the sysctl at 1: write 0, record the
   value found in errand-box's state, and report DEGRADED. With the key off
   and a recorded value: write the recorded value back, so turning the key
   off undoes it. With the key off and nothing recorded: report and change
   nothing. A host without the sysctl reports nothing.
3. The E-70 fix, in the guest. Two routes, to measure in this order:
   - A. Lingering for the agent at image build (the file
     `/var/lib/systemd/linger/agent`), and the launcher sets
     `XDG_RUNTIME_DIR=/run/user/1000`, so podman's systemd cgroup manager
     reaches the agent's systemd user manager. Recommended: the agent's containers keep
     real cgroup limits inside the guest's own.
   - B. An agent-only `containers.conf` with `cgroup_manager = "cgroupfs"`,
     if A fails.

   The image changes, so every host rebuilds it (E-31).
4. `tests/live/live.sh`: the rootless podman check passes on a host with the
   key on. On a host with the restriction and the key off, it reports SKIP,
   with E-69, instead of FAIL.
5. Update E-69 and E-70 in [docs/edge-cases.md](../docs/edge-cases.md),
   [docs/design.md](../docs/design.md) and the README.

## Pitfalls

- A sysctl write fails on a host where `/proc/sys` is read-only (a hosted
  container). That is a DEGRADED line with the cause, not a failed setup.
- `up` runs at every boot. The key's write must hold across restarts and not
  fight another tool that sets the same sysctl; read the value each time.

## Measured on WSL2, 2026-10-24

T-027's run, WSL2 Ubuntu 24.04 with systemd and cgroup v2 (no AppArmor in
that kernel): live.sh `inside`, rootless podman as the agent, failed with
"dbus: couldn't determine address of session bus": E-70 on a second kind
of host. Docker, sudo podman and FUSE passed.

## Prove

```sh
cargo test --locked up
sh tests/live/live.sh inside   # on ubuntu-24.04, key on: rootless podman PASS
sh tests/live/live.sh inside   # on ubuntu-24.04, key off: SKIP with E-69
```

and live.yml on both runners, with the key on.

---

# T-025: An AppArmor route for user namespaces in guests, without the host-wide sysctl

**Source:** the operator's ruling on E-69, 2026-10-23: "opt in for now and
leave the research as a new entry"
**Category:** sandbox
**Priority:** P2
**Effort:** M
**Status:** open

## Problem

T-024's key lifts the user namespace restriction for every process on the
host. Ubuntu's design is per program: a profile that grants `userns` lets
that program create user namespaces while the restriction holds for the
rest. If a profile can cover the guests' processes and nothing else, the
operator gets rootless podman in sessions without weakening the host.

## Premise

Read, not measured. Incus confines each guest with a profile it generates
per instance. errand's restricted project forbids `raw.apparmor`, because
`restricted.containers.lowlevel` blocks it, and that rule is what keeps the
errand account from raw LXC and AppArmor settings. So a route that needs
`raw.apparmor` in errand's project widens the project, and loses for the
same reason proxy devices are a finding (E-61, T-020).

## Approach: research and measure

Measure each route on a host with the sysctl at 1 (GitHub's ubuntu-24.04
runners have it), and record the commands and the output in
`docs/research/userns.md`:

- A. Read the profile Incus generates for a session instance, on the stable
  line and on 6.0 LTS (`/var/lib/incus/security/apparmor/profiles/`). Does
  it grant `userns`, or a setting that adds it? If a newer Incus grants it,
  the route is a version, not a profile.
- B. A profile errand-box installs for the processes Incus starts, without
  a key in errand's project: whether AppArmor's stacking lets a host
  profile grant `userns` under the generated one.
- C. `raw.apparmor` set by root on the profile errand's project uses: whether
  a restricted project refuses it from root as it refuses proxy devices.
  Expected refused, and recorded as measured either way.

## Prove

```sh
# with the sysctl at 1 and T-024's key off
sh tests/live/live.sh inside   # rootless podman PASS through the chosen route
```

Or `docs/research/userns.md` shows each route refused, with the command and
its output, and T-024's key stays the only route.

---

# T-051: bailey doctor reports every limit as unset

**Source:** [issue #15](https://github.com/Azathothas/errand-box/issues/15), T-027's end-to-end run, 2026-10-24
**Category:** sandbox
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

`bailey doctor` always prints `per-instance limits: memory unset, cpu unset,
processes unlimited`, while the profile holds all three and sessions
enforce them. E-63 says doctor reports the limits the profile holds.

## Premise

Measured on WSL2 (Ubuntu 24.04) and Daytona against `incus profile show`.
Read: `src/shim.rs:104-108`: the JSON pointer escapes `.` as `~1`, which
RFC 6901 decodes to `/`, so the key looked up is `limits/memory`.

## Approach

Look the key up with `get`, and add a unit test with a profile JSON.

## Prove

```sh
cargo test --locked shim::
errand-box __as errand -- /usr/local/lib/errand-box/shim/bailey doctor | grep 'per-instance limits'   # memory 80%, cpu N, processes 8192
```

---

# T-054: The image build is fsync-bound and prints nothing for half an hour

**Source:** [issue #18](https://github.com/Azathothas/errand-box/issues/18), T-027's end-to-end run, 2026-10-24
**Category:** sandbox
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

The guest image built on a WSL2 host in 1894 s, and setup printed nothing
between the build bridge row and the result. docs/operations.md says about
10 minutes.

## Premise

Measured on WSL2 (Ubuntu 24.04, 20 CPUs): packages downloaded in about 2
minutes; dpkg then ran 25 minutes with I/O pressure near 50% and the CPU
idle. A second build on the same machine took 775 s. Read:
`image/provision.sh:53-58` (no unsafe-io), `src/image.rs` (output kept for
failures only).

## Approach

dpkg with `force-unsafe-io` in the build; provision's progress lines shown
as they come; measured times per kind of host in the page, and image
route 2 named for slow hosts.

## Prove

```sh
time errand-box image build   # on the same WSL2 host: well under the 1894 s measured, with progress rows
```

---

# T-058: A session with no broker loses a race with the guest's tmpfiles

**Source:** [issue #22](https://github.com/Azathothas/errand-box/issues/22), T-027's end-to-end run, 2026-10-24
**Category:** sandbox
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

The shim pushes `/run/errand-box/env.sh` as soon as `incus exec true`
works, and the directory is made later by systemd-tmpfiles. Without
`--egress-proxy` every run fails with "Failed to open target file", and an
immediate retry hits the dead run's lock.

## Premise

Measured on WSL2 (Ubuntu 24.04, Incus 7.5.1): exec ready after 269 to
357 ms, the directory after 659 to 728 ms; 3 of 3 runs without a broker
failed, 3 of 3 with one passed, and live.sh `session` passed. Read:
`src/session.rs:238-255`, `:386-396`, `image/provision.sh:90`.

## Approach

Wait for the directory in `wait_ready`, or push it first; a live check that
starts an instance and pushes at once.

## Prove

```sh
cargo test --locked session::
sh tests/live/live.sh session
```

---

# T-061: A session that cannot start tells the chat only Incus's hint

**Source:** [issue #25](https://github.com/Azathothas/errand-box/issues/25), T-027's end-to-end run, 2026-10-24
**Category:** sandbox
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

When a session instance cannot start, the shim's last stderr line is
Incus's "Try `incus info --show-log ...`", and errand posts that line as
the reason. The error itself reaches errand's log only.

## Premise

Measured on Daytona with mock-session.sh: the thread got "this session
could not start: the agent exited with code 125: Try `incus info ...`".
Read: `src/session.rs:299-300`, `errand/src/session/session.rs:1127`.

## Approach

The shim prints one summary line last, with the first line of the Incus
error; or errand quotes the first `errand-box:` line.

## Prove

```sh
cargo test --locked session::
```

On a host where instances cannot start, mock-session's thread names the
Incus error.

---

# T-066: `image.source = "release"` downloads the published image

**Source:** [issue #30](https://github.com/Azathothas/errand-box/issues/30), T-027's end-to-end run, 2026-10-24
**Category:** sandbox
**Priority:** P2
**Effort:** M
**Status:** open

## Problem

`image.source = "release"` refuses because no release publishes the image,
but the prerelease `artefacts` does, for both architectures, and the run
used it by hand on Daytona and WSL2 Arch, where the build is impossible.

## Premise

Measured: imported images were taken as current (inputs `3ddf6025ef90`).
Read: `src/setup.rs:1488-1495`, `.github/workflows/artefacts.yml:85-95`.
T-014 holds the published-image plan.

## Approach

Download, check against SHA256SUMS and the inputs digest, and import;
publish the image with every tagged release.

## Prove

```sh
printf '[image]\nsource = "release"\n' > r.toml && errand-box setup --config r.toml --skip-image   # guest image ok, imported
```

---

# T-086: a lite engine for hosts Incus cannot serve

**Source:** [issue #49](https://github.com/Azathothas/errand-box/issues/49), a spec and plan read on `57231da`
**Category:** sandbox
**Priority:** P3
**Effort:** L
**Status:** open

## Problem

A host with about 1 GiB of RAM or less, no user, mount, PID or network
namespaces for the account, no FUSE, and possibly no root or cgroup
delegation, cannot run the Incus engine: setup refuses it or no guest starts
(T-042, T-039, T-044). The rootless routes need user namespaces or FUSE, and
emulation is out of scope by the operator's instruction. So errand-box serves
none of these hosts, and nothing says why.

## Premise

Read on `57231da`, and probed by the reporter in a review sandbox (uid 0,
`CapEff` 0, no user namespaces, an existing seccomp filter):
`unshare -U` returned `Operation not permitted`, `ptrace(PTRACE_TRACEME)`
returned EPERM, `landlock_create_ruleset` returned ABI 10, and the seccomp
user notification and `ADDFD` were available. So the closest existing thing is
bailey's no-namespace fallback, which errand's own check refuses (T-084,
issue #51) and whose egress lock is weaker than errand's proxy mode needs.
A prototype of the egress mediation ran in that sandbox (the issue carries it);
a real TCP connect to a real broker, throughput, other architectures and
Landlock integration were not run.

## Approach

A second errand sandbox engine, `lite`, that runs a session as a supervised
host process with Landlock for the filesystem, seccomp for the syscall
surface, a subreaper and tree kill for the lifetime, and an emulated
`connect` (seccomp user notification) that reaches only errand's broker
address, so egress stays locked without a network namespace. It is an engine
of the existing `bailey` stand-in (`engine.mode = "lite"` in runtime.toml,
chosen in `src/shim.rs`), not a new errand backend. It reuses `src/policy.rs`
to map the policy to Landlock rules. The one change inside errand is the path
model: the four path constants in `errand/src/sandbox/backend.rs:19-32` become
fields of `SandboxLaunch`, which the Incus, bailey and podman backends set to
the current values and `lite` sets to the real host paths, since nothing can
appear at `/workspace` without a mount namespace. The issue carries the full
spec: the kernel ladder, the seccomp rule table, the egress prototype, the
memory and process handling without cgroups, and the milestones M0 to M5.

Depends on T-084, which adds the capability probe, the tier chooser and the
change that lets errand carry `user namespaces: no` as a gap rather than a
refusal; `lite` needs all three.

## Decision

The operator ruled on 2026-10-25: file this as a researched entry a future
session can evaluate and build, not blocking, and not now. It is a **second**
engine, for hosts the Incus engine cannot serve. It does not replace the
settled decisions (RULES.md section 4: Incus system containers, root at
setup); those stand for every host Incus can run on, and `lite` gives up the
kernel boundary, Docker and podman inside, package installs and a disk quota,
which the spec states plainly. The security model is weaker than a container's
and the spec says where. Before it is built, a session re-measures the
prototype's open questions (the issue's section 13): Landlock on the target
boards, the emulated socket under a model stream, and a pre-existing seccomp
listener.

## Prove

The entry is a design and a plan; its acceptance is the milestones' own exit
criteria in the issue. A first implementing session's acceptance:

```sh
cargo test --locked -p errand-box lite::
```

M1's criterion: a `lite` session runs the agent against a project, cannot read
a file outside its grants, and leaves no process after SIGTERM and after
SIGKILL of the launcher.
