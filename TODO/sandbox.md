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
**Status:** done

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

## Measured (2026-10-25, on the provisioned host)

All three items, on Incus 6.0.7 in project `user-994`, against a throwaway copy
of `eb-golden`. Evidence, not assumption (the scripts are in the session's
scratch; the findings are here):

- **A guest connects to a unix socket on a read-only disk device: yes.** With the
  disk device `readonly=true`, a guest `curl --unix-socket` to the shared socket
  reports `num_connects=1`; the result is identical with `readonly=false`. A
  read-only disk device does not block `connect()`. This is the property T-090's
  Incus link also rides (its TUN carries IP frames over this same socket).
- **Owner and mode: socket and directory owned by the errand account
  (994:993), socket mode 0660, directory mode 0750.** The restricted project
  pins a split idmap (`restricted.idmap.uid=994`, `gid=993`): guest uid 1000
  maps to host 994, guest root (uid 0) to a per-instance base (1196608 here,
  volatile). A socket owned by host 994 appears inside as uid 1000, and both
  guest uid 1000 (as owner) and guest root (via its DAC override, since 994 is
  inside the idmap) connect at 0660, no world bit. The alternatives were
  measured and rejected: a host-root-owned socket appears as `nobody` (unmapped)
  and connects only at mode 0666 (world), because guest root has no override
  over an unmapped file; a socket owned by the mapped guest-root uid connects
  for guest root but not guest 1000, and that uid is per-instance volatile. "No
  other host account" is the parent `/var/lib/errand` (0750 errand:errand):
  other host accounts (ubuntu, nixbld*) cannot enter it. The source must sit
  under `/var/lib/errand` regardless (`restricted.devices.disk.paths`), which is
  why a source under `/run` was refused.
- **The guest-side relay: `systemd-socket-proxyd`, already in the image.** The
  golden image ships it (`/lib/systemd/` and `/usr/lib/systemd/`) and has
  `curl`, `setpriv`, `ip`, but not `socat` or `nc`. So the relay is a
  socket-activated `systemd-socket-proxyd` on
  `169.254.169.1:{broker, guest_proxy_port}`, run as guest root (which reaches
  the 994-owned host socket), and no new package is added.

So the design is grounded: a per-session directory `/var/lib/errand/links/<id>/`
(994:993, 0750) holds the broker and sysproxy unix sockets (994:993, 0660);
errand-box relays each to errand's broker port (the shim's `relay::unix_to_tcp`,
E-23, already does this for rootless); the directory is shared read-only by a
disk device; the guest's `systemd-socket-proxyd` units listen on 169.254.169.1
and connect to the shared sockets; then `restricted.devices.proxy=block`.

## Prototyped (2026-10-25, on the provisioned host)

The full guest-relay path, proven live end to end against a stand-in broker
(scripts in the session's scratch):

```
guest curl 169.254.169.1:PORT
  -> a systemd .socket unit (FreeBind, ListenStream=169.254.169.1:PORT)
    -> systemd-socket-proxyd (guest root), upstream a unix socket path
      -> the shared socket on a read-only disk device
        -> relay::unix_to_tcp on the host -> errand's broker tcp port
```

Both the broker port and the `guest_proxy_port` listener returned the broker's
answer (curl exit 0), and the path survives an instance restart. Findings that
shape the build:

- **The share must mount at a rootfs path, not under `/run`.** A disk device
  targeted at `/run/errand-box/link` is gone after a restart: the guest's
  systemd mounts a fresh tmpfs over `/run` during boot, shadowing the bind mount
  Incus placed there (the socket vanishes, the relay fails). Mounted at
  `/var/lib/errand-box/link` (on the rootfs) the bind mount survives (`findmnt`
  shows `/dev/vda[...]`, the socket is present, the relay works). An edge case to
  record with a row and a test when the build lands. The env file at
  `/run/errand-box/env.sh` is unaffected: the shim writes it through the Incus
  API after boot, onto the live tmpfs.
- **One host socket, two guest listeners.** Both proxy devices today connect to
  the same host target (errand's single broker port), so the build needs one
  per-session host socket, relayed to the broker, and two guest proxyd units
  (the broker port and `guest_proxy_port`) pointing at it.
- **The guest relay adds no package:** `systemd-socket-proxyd` is in the image;
  the units are enabled in `provision.sh` the way `errand-box-guest.service` is.

## Open design point (2026-10-25): the dynamic broker port

The prototype used a fixed port; it did not settle the one piece the guest side
turns on. errand's broker binds a kernel-picked `127.0.0.1` port that changes at
each errand restart (`docs/bailey-contract.md:586-588`;
`errand/src/serve.rs:702` `broker_instance.listen("127.0.0.1")`), and the shim
passes it per run as `--egress-proxy 169.254.169.1:<P>`; the agent's
`HTTPS_PROXY` (errand's policy) points at that same `<P>`. Today a per-session
proxy device listens on exactly `<P>` inside the guest. A guest-side relay baked
into the static image cannot know `<P>` at build time. The host side is
unaffected: `relay::unix_to_tcp` dials `127.0.0.1:<P>` resolved per run.

Both `<P>` and `guest_proxy_port` (3128) connect to the SAME host target (the
one broker multiplexes the agent protocol and the HTTP proxy;
`src/session.rs:94-105`), so the guest needs only "169.254.169.1 -> the shared
socket", not a per-port fan-out.

Options, each needing a live check, to decide before building:

1. **A fixed broker port (recommended).** errand is errand-box's fork; have its
   broker bind a configured fixed `127.0.0.1` port instead of `:0`, so the guest
   units are static (`169.254.169.1:<fixed>` and `:3128`). A contract change
   (`docs/bailey-contract.md:586-588`) and an errand change
   (`serve.rs` `start_broker`), both in scope. Cleanest; also the simplest base
   for T-090 task 3 (the TUN rides the same fixed mapping).
2. **A per-session guest listener on `<P>`,** set up by errand-box each run via a
   systemd template socket unit (`eb-broker@.socket`, `ListenStream=
   169.254.169.1:%i`) started with `incus exec ... systemctl start
   eb-broker@<P>.socket`. Keeps the port dynamic; adds an exec-systemctl step
   whose bus reachability over `incus exec` must be confirmed (an early
   prototype hit "Failed to connect to system scope bus" running systemctl
   through `incus exec`, though boot-time start worked).
3. **A guest nftables REDIRECT** of all `169.254.169.1` TCP to one local proxyd
   -> the shared socket. Static and port-agnostic, but relies on nat in the
   guest's own netns (an unprivileged container); needs a live check.

Only this choice gates the build; the rest is mapped below.

**Decided 2026-10-25: option 1, a fixed broker port.** Build in progress, in
slices that keep `main` green:

- **Slice 1 (done):** errand's broker port is configurable. `EgressConfig`
  gains `brokerPort` (default `0`, the kernel picks one, so standalone errand is
  unchanged); `Broker::listen(host, port)` binds it; `serve.rs` passes
  `config.sandbox.egress.broker_port`. Tests for the default, a set value and an
  out-of-range value; the generated config reference regenerated. Non-breaking:
  errand-box does not set it yet, so every host stays on an ephemeral port until
  the switch.
- **Slice 2 (done):** errand-box sets the fixed port. `network.broker_port`
  (default 3129) is written into errand's config (`errandcfg` adds `brokerPort`,
  the `errand-config` gate confirms the fork accepts it) and documented in the
  example. Still over proxy devices: errand binds 127.0.0.1:3129 and reports it
  on `--egress-proxy`, which `desired_devices` wires unchanged. `runtime.toml`
  does not carry it yet; that lands with slice 3, where the shim relay needs it.
- **Slice 3 (the switch, atomic). Implemented and live-validated 2026-10-25.**
  All the code and docs below landed, unit-green (errand and errand-box tests,
  fmt, clippy, shellcheck, errand-config, gate) and proven live (see
  "Live-validated"). The disk-share link, the guest relay units,
  `restricted.devices.proxy=block`, the image rebuild, the edge-case row (E-71),
  and the live run. It is one atomic change because
  the socket-mode guest proxyd units listen on the SAME `169.254.169.1` ports the
  tcp-mode proxy devices use inside the guest, so the image is mode-specific and
  there is no half-deployed state (image.rs must pass the mode, e.g.
  `EB_BROKER_LINK`, so `provision.sh` installs the proxyd units only for the
  socket build). The pieces, all mapped and proven:
  - `runtime.rs`: add `broker_port` (serde default 3129, so old `runtime.toml`
    parses); `setup.rs:1577` writes it; the `Runtime` struct literals
    (`src/shim.rs`, `src/session.rs`, test fixtures) gain the field.
  - `session.rs` `desired_devices`: for `broker_link == "socket"`, emit one
    `eb-link` disk device (source the per-session link dir, `path=/var/lib/
    errand-box/link` on the rootfs, `readonly=true`) and no proxy devices; add a
    `link_dir` parameter threaded from `prepare`.
  - `session.rs` lifecycle: `prepare` creates `/var/lib/errand/links/<name>/`
    (994:993, 0750, idempotent) so the disk source exists for both `check`
    (`shim.rs:310`) and `run`; `run_inner` creates `broker.sock` (994:993, 0660)
    and `relay::unix_to_tcp(sock, "127.0.0.1:<rt.broker_port>")`; `gc` removes the
    link dir right after `inc.delete` (`maint.rs:90`). `relay::unix_to_tcp`
    already clears a stale socket (`relay.rs:103`), so re-runs are safe.
  - `setup.rs`: `restricted.devices.proxy` -> `block` (`:1146`); write
    `broker_link="socket"` for the root Incus backend.
  - `image/provision.sh`: the two `systemd-socket-proxyd` `.socket`/`.service`
    units (FreeBind, `ListenStream=169.254.169.1:{broker_port, guest_proxy_port}`,
    upstream `/var/lib/errand-box/link/broker.sock`), enabled the way
    `errand-box-guest.service` is, plus the `/var/lib/errand-box/link` mountpoint;
    gated on the socket build. `image/guest-init:5-7` comment and
    `docs/design.md:50,73,77` updated to drop the proxy-device description.
  - `docs/bailey-contract.md:586-588`: the broker port is now a fixed configured
    port, not kernel-picked. New `docs/edge-cases.md` row for the `/run`-tmpfs
    shadow (fallback: mount on the rootfs; test: a live restart-and-reconnect).
  - Prove: `sh tests/live/live.sh` with `restricted.devices.proxy` at `block`.

## Touchpoints for the build (mapped 2026-10-25)

- `src/session.rs:73-108` `desired_devices`: for the socket mode, emit one disk
  device (source the per-session dir under `/var/lib/errand`, path the ROOTFS
  target `/var/lib/errand-box/link`, `readonly`) and no proxy devices.
- `src/session.rs:321-332` `run_inner`: a branch like the `unix` one, but the
  socket lives in `/var/lib/errand/links/<id>/` (dir 0750, socket 0660, both
  994:993), not `state_dir()` (the guest writes state; SECURITY.md rule 1).
  `reconcile` reruns each launch, so both `check()` (`src/shim.rs:310`,
  `--isolate`) and `run()` must create the dir before the disk device is added.
- Lifecycle: remove the per-session link dir with the instance (`run_inner`
  end, and `reap`/`gc` for orphans); the relay thread already dies with the
  launcher (`src/relay.rs:105`, `docs/bailey-contract.md:577-579`).
- `src/setup.rs:1137-1156`: `restricted.devices.proxy` -> `block` (was `allow`);
  set `broker_link` to the socket mode in `runtime.toml`; fix
  `docs/operations.md:64`.
- `image/provision.sh:86-120`: add the guest relay units and the
  `/var/lib/errand-box/link` mountpoint, enabled the way
  `errand-box-guest.service` is. `image/guest-init:5-7` and
  `docs/design.md:50,73,77` describe the removed proxy devices; update them in
  the same change. Changing `image/` triggers an E-31 rebuild on every host.
- A new edge-case row (`docs/edge-cases.md`): a boot-time disk-device mount
  under `/run` is shadowed by systemd's `/run` tmpfs; fallback a rootfs target;
  test a live restart-and-reconnect.
- Prove: `sh tests/live/live.sh` (broker and sysproxy pass with proxy blocked);
  `incus project get user-994 restricted.devices.proxy` is `block`.

## Live-validated (2026-10-25, on the provisioned host)

`errand-box setup` rebuilt the socket-mode image and set
`restricted.devices.proxy=block` and `broker_link=socket`; `sh tests/live/
live.sh` ran against it.

- **The broker link works over the socket, proxy devices blocked.** The session
  check's "broker reached" passes (the guest reaches the broker on
  169.254.169.1 through the guest relay and the shared socket), "direct egress
  blocked" passes, and the project is `restricted.devices.proxy=block`. T-020's
  prove, met.
- **Both guest relay ports carry a real HTTPS CONNECT.** A direct probe (guest
  `curl -x http://169.254.169.1:<port>` to `mirror.gcr.io` and
  `registry-1.docker.io`, through the socket link) returns the registry's 401 on
  the broker port (3129) and the guest-proxy port (3128), one connection each.
  So the system-proxy path apt and Docker use carries CONNECT.
- **A full image pull succeeds through the link:** rootless podman (as the agent)
  pulled and ran `docker.io/library/debian:trixie-slim` (E-30).
- 46 checks pass. The rest are not the link: the three `errand:` checks need
  Discord credentials (T-015, no code), and `docker run` / `docker build` /
  `sudo podman run` fail on registry auth and rate limits through this host's
  crude socat stand-in for the broker (not errand's allowlisting one), the same
  link that pulled a full image for rootless podman and returns 401 on a direct
  CONNECT to those registries.

A boot race surfaced and was fixed: `exec true` (the old `wait_ready` probe)
succeeds a moment before systemd-tmpfiles makes `/run/errand-box`, so the env
push opened a missing target on the heavier socket-mode image. `wait_ready` now
waits for that directory.

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

**Superseded by T-090 (2026-10-25).** The virtual network layer carries raw TCP
as one more flow through the shared gate, so the per-port relay this entry rules
on is subsumed by T-090 task 6. The 2026-10-23 ruling below is kept verbatim as
the record of why raw-TCP egress has the shape it does; the work now lives in
T-090, and this stays open until T-090 closes it.

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

---

# T-090: a virtual network layer, one gated egress path for every backend

**Source:** the operator, 2026-10-25, choosing a virtual network layer over an
upstream HTTP proxy bolted onto the broker (T-082) and a per-port raw-TCP relay
(T-022). The lite-backend link is the operator's `cfrs` design, "Userspace
Virtual Networking in a Sealed Sandbox", read on 2026-10-25 at
`https://raw.githubusercontent.com/talaria0101/cfrs/refs/heads/main/VIRTUAL-NETWORK.md`.
**Category:** sandbox
**Priority:** P2
**Effort:** XL
**Status:** partial

## What it is, and what it is not

**Is:** one egress path for a session. errand-box runs a userspace TCP/IP stack
(smoltcp) and a policy gate. The guest's traffic reaches the stack as a
packet-or-flow stream over a per-backend link device; errand-box terminates
every flow, resolves DNS itself, enforces the allowlist, the ports and the
`is_private_address` check on every flow (resolve-then-route), and relays to the
real destination, direct or through an upstream HTTP proxy for the external hop.
The gate and the smoltcp stack are shared across backends; only the link device
differs.

**Is not:**
- Not a host NIC or bridge. In the Incus backend the TUN rides T-020's
  unix-socket link, so the guest stays device-less and
  `restricted.devices.proxy=block` holds. In the lite backend there is no
  device: the link is `AF_UNIX`.
- Not "open the network". Every flow is gated as the broker gates HTTP today;
  the default stays fail-closed.
- Not TLS interception. HTTPS tunnels opaquely, as CONNECT does now; the
  provider-credential swap stays an explicit terminating HTTP path, never a
  man in the middle.
- Not the enforcement itself. Enforcement is the backend's existing seal (the
  network namespace in Incus; the sandbox's `AF_INET` block in lite). The
  virtual layer is the usable, policed path through that seal.

## What already exists (audited)

- **The gate, in the broker** (`errand/src/sandbox/broker/`): `host_allowed`,
  `is_private_address` (with T-032's arms), the port check, `public_addresses`
  (resolve then judge), the provider route (`serve_provider_request`, the
  nonce-to-key swap), and T-032's answer redaction. This is the policy core
  T-090 factors out and reuses; only the transport in front of it changes.
- **The Incus link** (T-020, ruled 2026-10-23): a unix socket on a read-only
  disk device with a guest-side relay on 169.254.169.1, replacing proxy
  devices. T-090's Incus link carries IP packets (a TUN) over this same socket,
  so T-020's property is kept.
- **TUN availability:** the guest already has `/dev/net/tun` (doctor checks it,
  `src/up.rs:348`, `src/main.rs:561`).
- **Raw-TCP egress** (T-022, ruled): a per-port relay over T-020's link. T-090
  subsumes it; raw TCP is one more flow through the gate.
- **The upstream hop** (T-082, open): an upstream HTTP proxy and `proxy_ca`.
  T-090 absorbs it as the gate's upstream-hop policy (direct, or CONNECT via the
  proxy; an internal target is refused at resolve-then-route and never sent to
  the proxy; `proxy_ca` into the gate's dialer and the session trust store).
- **The lite transport** (`cfrs`, the operator's, WIP): an `LD_PRELOAD`
  `libcfrsnet.so` rewrites `AF_INET` sockets to `AF_UNIX` abstract sockets
  (`\0cfrsnet/4/<ip>/<port>`); a smoltcp stack on `10.66.0.0/24` with a
  transport-agnostic link (an in-process queue or a length-prefixed tunnel); a
  SOCKS5/HTTP proxy mode for programs that cannot be preloaded. cfrs's open
  work: Switch mode, the control server, a single-waker race, unbounded queues,
  a 1280-byte MTU floor, and no interposition for statically-linked programs.
- **No userspace netstack dependency** in the tree today; smoltcp is the Rust
  candidate, and cfrs already carries it.

## Premise, and how it was checked

Read, not measured (no Incus host and no sealed sandbox this session):

- The gate functions are the broker's current code (T-032 touched them this
  session).
- The Incus link and its ruling are T-020's.
- The lite transport is cfrs's design doc. cfrs is WIP; its claims (`AF_INET`
  gives `EACCES`, namespaces give `EPERM`, `LD_PRELOAD` plus `AF_UNIX` works)
  are the operator's measurements in that sandbox, not re-run here.

Each is measured before the stage that builds on it (the checkpoints below).

## Decisions

1. **smoltcp, in errand-box's process**, not a sidecar: the policy stays in the
   trusted Rust process, one language, reusing the gate. A sidecar stays the
   fallback only if smoltcp cannot carry the flow volume, measured not assumed.
2. **DNS is answered by errand-box on the virtual link.** The guest's resolver
   points at the gate; the gate resolves, applies the allowlist and the address
   check to the answer, and returns only addresses that pass. This is where
   resolve-then-route lives, so the check covers every flow. A literal-IP dial
   that skipped DNS is checked again at dial time, so DNS is not the only gate.
3. **Provider flows route to the existing terminating HTTP endpoint**
   (`serve_provider_request`): the gate recognises the provider's virtual
   address and hands those flows to it; everything else tunnels. The netstack is
   a router in front of the broker's two existing roles.
4. **The gate is a backend-agnostic library; the link device is per-backend.**
   Incus: a TUN over T-020's unix socket. Lite: cfrs's `LD_PRELOAD`
   interposition. Both feed the same gate and stack. That is the point: one
   gate, two links.
5. **Enforcement is the backend's existing seal, not the virtual layer.** Incus:
   the netns. Lite: the sandbox's `AF_INET` block, so a static binary or a
   shim-evader fails closed. T-090 does not claim to seal a backend that is not
   already sealed; the lite seal is cfrs's premise, recorded as such.
6. **T-090 supersedes T-022 and T-082.** Their rulings are kept verbatim in
   those entries (history.md: append, never edit) under a pointer here; the
   work moves to T-090's stages. T-083's done parts (the refusal reasons,
   `ERRAND_EGRESS_ALLOW_EXTRA`) stay; its open parts (per-host `host:port`, the
   in-thread refusal line) fold into the gate.

## Tasks

Staged, so each stage leaves a working egress and proves against the last.

1. **The gate as a library.** Factor the broker's gating (allowlist, ports,
   `is_private_address`, `public_addresses`, the provider route, the redaction)
   into a backend-agnostic `gate` that answers a flow (src, dst name or ip,
   port) with allow, refuse or route-to-provider, plus the dialer
   (direct or upstream-proxy). Invariant: verdicts identical to today's broker
   on the same inputs. Reuse: the broker functions, moved not rewritten. Prove:
   the current broker tests pass against the gate unchanged,
   `cargo test --locked -p errand broker`.

   **Started 2026-10-25 (first slice).** `errand/src/sandbox/broker/gate.rs` now
   holds the admission verdict (`target_admission`, the host/port decision and
   its T-083 reason, lifted out of the transport's `admit`) and the provider
   routing (`route_for`, now data-driven on `&[ProviderRoute]`, with
   `under_prefix`). `admit` and `serve_provider_request` call the gate; the 38
   broker tests pass unchanged (verdict parity), and a direct unit test reads
   the verdict off the gate with no transport. Still in the parent module, where
   they are already transport-free: the host and address predicates
   (`host_allowed`, `is_private_v4/v6`, `is_private_address`, `public_addresses`)
   and the redaction. Gathering those physically into `gate`, and the flow-level
   `allow/refuse/route` + dialer API the netstack calls, is the rest of the task
   and lands with the stack (task 2) and the Incus link (task 3).
2. **The smoltcp stack and a link-device seam.** A smoltcp stack on a private
   subnet with a `LinkDevice` trait (cfrs's "in-process queue or tunnel").
   Invariant: a userspace TCP handshake and transfer over an in-process link
   round-trips. Prove: `cargo test --locked -p errand netstack`.

   **Done 2026-10-25.** `errand/src/sandbox/netstack.rs`: the `LinkDevice` trait
   (the per-backend seam that carries raw IP frames, `recv_frame`/`send_frame`,
   with an MTU) and `Phy<L>`, a smoltcp `phy::Device` adapter over any
   `LinkDevice` (`Medium::Ip`, RX/TX tokens). smoltcp 0.14 is pinned in
   `Cargo.toml` with only `std`, `medium-ip`, `proto-ipv4`, `socket-tcp` (pure
   Rust, cross-compatible; the phy backends are this crate's own). The module is
   `#[allow(dead_code)]` scaffolding until task 3 feeds it from a transport. A
   test cross-wires two in-process links and two stacks and round-trips a TCP
   handshake plus a two-way transfer (`ping`/`pong`): `PASS`. fmt and clippy
   (pedantic) clean. The flow-level `allow/refuse/route` API that joins this
   stack to the task-1 gate, and the predicate gathering, land with task 3.
3. **The Incus link: a TUN over T-020's socket.** The guest boot step makes a
   TUN, routes default via it, and the guest-side relay pumps packets over
   T-020's unix socket; errand-box feeds the stack. Invariant: one allowlisted
   HTTPS flow reaches its host and a non-allowlisted one is refused, with
   `restricted.devices.proxy=block`. Prove (live): `sh tests/live/live.sh`.
4. **DNS plus resolve-then-route plus the address check on every flow.**
   Invariant: a literal metadata IP and an allowlisted name resolving internal
   are both refused; an external name is relayed. Prove (live): the live egress
   checks with an internal-target case added.
5. **The provider endpoint over the stack.** Route provider flows to
   `serve_provider_request`. Invariant: the nonce is swapped for the key, the
   answer is redacted (T-032), and the session never holds the key. Prove: the
   provider tests through the stack.
6. **Raw TCP (closes T-022).** A granted `host:port` flow tunnels; an ungranted
   one fails. Invariant: ssh to a granted host works, every other raw-TCP
   destination fails closed. Prove (live): T-022's prove.
7. **The upstream hop (closes T-082).** Direct, or CONNECT via an upstream
   proxy; `proxy_ca` into the dialer and the session trust store. Invariant: on
   a proxy-only host an allowlisted external flow reaches its host through the
   proxy, an internal target is refused before the proxy, and a
   TLS-intercepting proxy's CA is trusted. Prove (live): a host behind a proxy.
8. **The lite link (cfrs).** The `LD_PRELOAD` interposition feeding the same
   gate and stack, with the sandbox's `AF_INET` block as the backstop.
   Invariant: a dynamically-linked client reaches an allowlisted host and
   nothing else; a static binary fails closed, recorded not silently bypassing.
   Prove (sealed sandbox): cfrs's verification plus the gate's allowlist test.
   Depends on cfrs's open work; may land after the Incus stages.

## Checkpoints

- Before task 3: measure T-020's link (its three listed items: a guest connects
  to a unix socket on a read-only disk device; the owner and mode; the
  guest-side relay). T-090's Incus link cannot be built before T-020's is real.
  **Done 2026-10-25** (see T-020's "Measured" section): a guest connects to a
  unix socket on a read-only disk device (`num_connects=1`); the socket and its
  directory are owned by errand (994:993), socket 0660, under `/var/lib/errand`;
  the guest ships `systemd-socket-proxyd` for the relay. The reachability is
  proven; T-020's link is not yet built (that implementation is the remaining
  gate for task 3).
- Before task 8: measure cfrs's premises in the target sandbox (`AF_INET`
  `EACCES`, namespaces `EPERM`, `LD_PRELOAD` honoured) and its open work
  (unbounded queues, the MTU floor) against the flow sizes errand needs.
- After task 1: the gate-as-library matches the broker verdict for verdict
  before any transport rides it.

## Pitfalls

- A userspace stack parses untrusted packets from the guest, a new
  security-sensitive surface. Bounded buffers, a fuzz target, and the
  forbidden-patterns rules on buffering whole bodies and trusting declared
  lengths all apply. cfrs's "unbounded queues" open item is this class.
- The address check must fire on a literal-IP dial, not only on a DNS answer
  (decision 2).
- The MTU floor and the framing must not silently truncate a flow
  (forbidden-patterns: padding or truncating on a length mismatch).
- Performance: userspace packet processing for all session egress; measure
  before claiming parity with the direct broker.
- `LD_PRELOAD` is usability, not enforcement (decision 5): the entry must not
  present the lite backend as sealed by the shim.
- Keep T-020's property: the Incus link rides the unix socket, never a host NIC.

## Acceptance gate (three parts)

- (a) `sh scripts/ci-local.sh` green, plus the gate-as-library and netstack
  unit tests.
- (b) The live tests on an Incus host (tasks 3 to 7) and, for task 8, cfrs's
  verification in the sealed sandbox.
- (c) The deep reviews: the packet parsing read adversarially; the gate's
  verdict parity with the old broker audited; the lite backend's fail-closed
  behaviour for a static binary confirmed.

## Self-review questions (for the implementing session)

- Does the gate give byte-identical verdicts to today's broker (task 1)?
- Is there any flow that reaches the network without the address check: a
  literal-IP dial, a DNS answer, a provider flow? Each named.
- Does a static binary in the lite backend fail closed, or silently reach the
  network?
- Does the Incus link keep `restricted.devices.proxy=block`?

## Only the operator can do

- Provide or name a host with Incus for tasks 3 to 7, and a host behind an HTTP
  proxy for task 7.
- Provide or name the sealed-sandbox environment for task 8, and rule on how
  errand-box consumes cfrs (vendored in-tree per the third-party convention, or
  a crate dependency) when task 8 is reached.
- Rule on the lite backend's static-binary gap (accept fail-closed, or require
  a static-interposition route) before task 8 ships.

## Prove

```sh
sh scripts/ci-local.sh
sh tests/live/live.sh            # on an Incus host: tasks 3 to 7
# task 8: cfrs's verification in the sealed sandbox, plus the gate allowlist test
```
