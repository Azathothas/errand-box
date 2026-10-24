# errand: errand-box's hard fork

Entries about `errand/`, the chat daemon errand-box took into this tree as a
hard fork. [INDEX.md](INDEX.md) lists every entry.

---

# T-028: errand is errand-box's hard fork, built, tested and shipped from this tree

**Source:** the operator, 2026-10-24: "vendor & patch errand here, hard fork,
we will probably extend/improve it even more"; the research of that day,
[docs/history/references/2026-10-24-errand-code.md](../docs/history/references/2026-10-24-errand-code.md)
**Category:** errand
**Priority:** P1
**Effort:** L
**Status:** partial

## Problem

errand-box installs errand as a downloaded release (`pins.toml`, `[errand]`)
and leaves it unchanged. errand's direction for an Incus backend differs from
errand-box's (`docs/incus-backend.md` section 10), and a defect in errand
(T-029) can only be fixed in errand's code. A release errand-box does not
build is a release errand-box cannot fix.

## Premise

Measured on 2026-10-24, the code sweep above: errand-box uses about 76% of
errand's non-test lines; a rewrite on the same Discord library keeps about
three quarters of the crates and a C compiler; errand at `715e9b0b777a`
builds and passes its tests on errand-box's toolchain after 66 lint fixes in
test code and one test helper.

## Decision

- **A hard fork, not a vendored copy that follows upstream.** The operator's
  ruling. `errand/` is errand-box's code from the import commit on. No
  upstream release is taken; [vendoring.md](../docs/methodology/vendoring.md)
  applies to the import and its record, not to a reconciliation.
- **A workspace member, `errand/`, with its own lints.** The crate keeps
  errand's `warnings = "deny"` and `clippy::pedantic = "deny"`. The boundary
  stays visible in a listing, and the import commit is the base a diff starts
  from.
- **One binary.** errand-box links errand as a library and answers as
  `errand` when it runs under that name, as it already answers as `bailey`
  (`src/main.rs:44`). Setup links `errand` to errand-box. Two binaries in one
  release lost: a second asset, a second digest and a second download in
  `bootstrap.sh`, for no property the operator asked for. The cost is a
  larger binary and a C compiler in the static build.

**Correction, 2026-10-24: the operator's rulings change the shape.** errand
stops being a program: no `errand` link and no `errand` bin target. The
supervisor runs `errand-box serve`, which calls the errand library's daemon,
and `errand-box threads` takes over errand's thread commands. A host then
runs three external programs: errand-box, kage (or another agent harness)
and Incus. The web interface leaves the fork (T-034), so step 4 below is
moot. A release carries `THIRD-PARTY-NOTICES`, generated from `Cargo.lock`
by a pinned cargo-about. The `cross` step uses a pinned zig as the aarch64
musl C compiler when zig is present.

## Approach

1. **Import and gate.** Done on 2026-10-24: the import commit, the
   workspace, the merged `Cargo.lock` with every version errand pinned, the
   test helpers that looked for the repository root
   (`errand/src/test_util.rs`), the six stale `float_cmp` expectations, and
   `clippy::assert_is_empty` allowed in `errand/Cargo.toml`. The gate's
   `clippy` and `test` steps run `--workspace`. Two checks of the gate were
   narrowed, each with a planted positive that still fails:
   `check-placeholders.sh` asks for `{{` and an uppercase letter, because
   `{{` is Rust's escape for a brace; `check-no-secrets.sh --public` skips
   reserved domains, the user part of a URL and `git@`.
2. **Ship it.** errand becomes a library crate with a `main` errand-box
   calls; `src/main.rs` dispatches on the name `errand`; setup links
   `ERRAND_BIN` to errand-box (`src/up.rs:28`, `src/setup.rs:696`);
   `[errand]` leaves `pins.toml`, and `fetch.rs`, `freshness.sh` and
   `watch-upstreams.sh` stop reading it. `tests/errand-config-test.sh` runs
   the errand errand-box builds.
3. **The static build.** ring, aws-lc-sys and libsqlite3-sys compile C. The
   `musl` step needs a musl C compiler: `musl-tools` on the runners, which
   build natively on each architecture (`.github/workflows/release.yml`).
   The `cross` step needs a musl cross C compiler, or reports that it built
   errand-box without errand.
4. **The web interface.** errand-box's config serves it on loopback
   (`src/errandcfg.rs:152`). The release builds the bundle in
   `errand/web` with bun before the static build, or the interface is
   reported as not built (`errand/build.rs:9-12`).
5. **The licence.** errand is MIT OR Apache-2.0; errand-box takes it under
   MIT. Each release carries `errand/LICENSE-MIT` beside the binary.
6. **The pages.** AGENTS.md, RULES.md section 4, docs/design.md, the README
   and docs/operations.md say errand is errand-box's code.

## Pitfalls

- errand-box's release profile sets `panic = "abort"`. errand has no
  `catch_unwind` and no `JoinError` handling, so a panic in a task now stops
  the daemon, and the supervisor restarts it.
- A host that has a downloaded errand: `link` removes the file at
  `ERRAND_BIN` and puts the link there (`src/setup.rs`, `fn link`).
  `uninstall` removes the whole `/usr/local/lib/errand-box`, links included.
- Found on 2026-10-24: errand-box's supervisor restarts the daemon after
  every exit (`Restart=always`, `src/supervise.rs:162`; the s6 run script
  likewise), and errand's exits 2, 3 and 4 are refusals that a restart only
  repeats. errand's own service files did not restart them. Proposed to the
  operator as an entry; not filed.

## Done, and open

**Steps 1, 2, 3 and 5 done on 2026-10-24, step 6 in part.**

- Step 2: errand is a library with `errand::main`
  (`errand/src/lib.rs`); `errand/src/main.rs` calls it; errand-box
  answers as `errand` (`src/main.rs`); setup links `ERRAND_BIN` to
  `errand-box` and downloads only kage; `[errand]` left `pins.toml`, and
  `freshness.sh` and its workflow no longer name errand; `errand.version`
  is read, has no effect, and setup reports a value other than `latest`.
  `tests/errand-config-test.sh` runs errand-box as `errand`, with no
  network: PASS.
- Step 3: the `musl` step exports `CC_<target>=musl-gcc` and checks that
  the static binary answers as `errand`. Measured on this host: a cold
  static release build in 238 s, 11.8 MB, static-pie. `gates.yml`,
  `live.yml` and `release.yml` install musl-tools before it. The `cross`
  step exits 2 with the reason when no `<arch>-linux-musl-gcc` exists.
- Step 5: each release carries `LICENSE-errand`, errand's MIT notice.
- Step 6: AGENTS.md, RULES.md, the README, docs/design.md, docs/tests.md,
  docs/operations.md and docs/incus-backend.md.

**Open:**

- Step 4: the release carries no web bundle, so errand logs "continuing
  without the web interface" (`errand/src/serve.rs:1360`). Build it with bun
  in the release, or remove the interface: the operator's ruling on T-034.
- The notices of the other crates the binary now links: errand-box
  published none for its 22, and the lock now holds 280.

**Changed by the rulings, 2026-10-24.** The list above is the link shape,
kept as it was done. Since then: errand is a library with `errand::serve`
and `errand::threads` (`errand/src/lib.rs`) and no bin target;
`src/main.rs` runs them as `errand-box serve` and `errand-box threads`; the
supervisor runs `errand-box serve` (`src/up.rs`); setup removes a
downloaded `errand` or the old link (`src/setup.rs`), and takes `web` out of
a kept config (`fn drop_web`). `tests/errand-config-test.sh` runs
`errand-box serve`, and the `musl` step checks that the static binary
carries `serve`. Both open items above are closed: step 4 by T-034's removal
of the interface, and the notices by `scripts/notices.sh`, which a release
runs (`.github/workflows/release.yml`). `cross` passed every step on this
host with zig 0.16.0 from pip as the aarch64 C compiler, and qemu-user 8.2.2
with a `binfmt_misc` handler: 112 unit and 6 integration tests ran on
aarch64. zig needs `-fno-sanitize=undefined` there, because rust-lld does
not link zig's UBSan runtime (`scripts/ci-local.sh`, `zig_cc`).

**The live prove, 2026-10-24**, on this session's hosted container (Ubuntu
24.04, kernel 6.18, x86_64, cgroup v1, no init, Incus 6.0.7 from Zabbly's
6.0 LTS line): setup with the static binary of `1d184d3`, 31 changed and 0
failed; `readlink /usr/local/lib/errand-box/bin/errand` printed
`errand-box`; the errand service ran `/usr/local/lib/errand-box/bin/errand
run`, passed its config check and its probe of the shim, and stopped at the
chat connection, as T-015's test token makes it. A second setup changed
nothing. `tests/live/live.sh`: 51 of 52, the one failure `t_reap`, which
T-026 names; alone it passed 2 of 2. Then with the binary of the T-031
change: setup 0 failed, and live 52 of 52.

## Prove

```sh
sh scripts/ci-local.sh            # every step, errand included
sh tests/live/live.sh             # errand runs from errand-box's own binary
readlink /usr/local/lib/errand-box/bin/errand   # errand-box
```

---

# T-029: A guest sends the operator's provider key to a host it names

**Source:** errand#48 and errand#54 item 1, read in the research of
2026-10-24 ([2026-10-24-errand-tracker.md](../docs/history/references/2026-10-24-errand-tracker.md))
**Category:** errand
**Priority:** P0
**Effort:** S
**Status:** done

## Problem

The broker matches a provider route with `path.starts_with(prefix)`
(`errand/src/sandbox/broker/mod.rs:507`), where the prefix is
`/provider/NAME` with no trailing slash (`errand/src/sandbox/bailey.rs:182`).
It then pastes the rest of the path onto the upstream URL (`:547`) and puts
the real credential on the request (`build_forwarded`, `:657`). With a
provider whose base URL has no path, `/provider/NAME.attacker.example/v1`
goes to `https://<provider host>.attacker.example/v1` with the key. A second
spelling puts `:443`, then `@`, then a host after the name: `url_host`
(`:720`) splits on the last colon and judges the provider's host, and the
client dials the other. `provider_refusal` (`:592`) checks only that the
host is public; `sandbox.egress.allow` is not read on this path.

## Premise

Read at errand `715e9b0b777a` and at v0.3.1, which `pins.toml` installs
today. No request was sent. The guest holds the nonce by design, so a
prompt-injected agent needs nothing else.

## Approach

Match a route only at a segment boundary: the path equals the prefix or
continues with `/`. Build the target by parsing the upstream URL and joining
the rest as a path, then refuse a target whose scheme, host or port differs
from the upstream's, or whose path leaves the upstream's path. Refuse an
encoded slash or backslash in the rest. Judge the host from the parsed URL,
never from a string split. The containment check in vultrino's
`src/capability/mod.rs:616-668`, read in
[2026-10-24-brokers-b.md](../docs/history/references/2026-10-24-brokers-b.md),
is the shape.

## Prove

```sh
cargo test --locked -p errand broker   # each spelling above is refused; a normal provider call is forwarded
```

## Done, and open

**Fixed in the fork on 2026-10-24.** `under_prefix` matches a route at a
path boundary only; `provider_target` parses the joined address and refuses
another scheme, host or port, a user part, a path outside the base URL's
path, and an encoded slash or backslash; `url_host` reads the parsed
address (`errand/src/sandbox/broker/mod.rs`). Four new tests in
`errand/src/sandbox/broker/tests.rs`. The prove passed: 42 broker tests.
With the old match and the old join planted back, the end-to-end test
fails: the steered request answered `HTTP/1.1 200 OK` and the second
stand-in provider received the key.

**Closed 2026-10-24.** T-028's live test ran the fork's errand from
errand-box's own binary: 52 of 52. A host installed from v0.1.0 runs errand
v0.3.1 until it takes a release that carries the fork.

---

# T-030: The host-side push can publish another session's repository

**Source:** errand#52, reproduced with git 2.43.0 in the research of
2026-10-24
**Category:** errand
**Priority:** P1
**Effort:** S
**Status:** done

## Problem

The pull request flow clones the session's repository with `git clone
--shared --bare` (`errand/src/session/pr.rs:633-645`). `assert_contained`
(`:246`) reads only `--absolute-git-dir`. A guest that writes a host path
into `.git/objects/info/alternates`, or a `commondir` file, makes the clone
read another repository the errand account can read, and the push then
publishes it with the bot token.

## Approach

Refuse a repository whose `.git` holds `objects/info/alternates`, a
`commondir` file, or a link, before the clone. Or clone without `--shared`,
which copies the objects and reads no alternates.

## Prove

```sh
cargo test --locked -p errand pr   # an alternates file and a commondir file are each refused
```

## Done, and open

**Fixed in the fork on 2026-10-24.** Reproduced first with git 2.43.0: with
a `commondir` file naming another repository's git directory,
`--absolute-git-dir` still named the session's own `.git`, and `git clone
--shared --bare` of the session's repository had the other repository's
`main`. A clone without `--shared` is no fix, because git follows
`commondir` and alternates there too. So `assert_contained` now calls
`refuse_borrowed` (`errand/src/session/pr.rs`), which refuses a git
directory that holds `commondir` or an alternates file, or a symbolic link
anywhere in it, the git directory itself included. Two new tests in
`errand/src/session/pr/tests.rs`; the prove passed, 35 tests. With the
check switched off, both new tests fail.

**Closed 2026-10-24**, as T-029: T-028's live test ran the fork, 52 of 52.

---

# T-031: The relay drops a connection with no log line when its target does not answer

**Source:** the research of 2026-10-24, read in errand-box's own code
**Category:** errand
**Priority:** P2
**Effort:** S
**Status:** done

## Problem

`relay::unix_to_tcp` and `relay::tcp_to_tcp` (`src/relay.rs:62`, `:79`)
discard a failed connect: `if let Ok(up) = TcpStream::connect(...)`. The
client sees a closed connection, and nothing records why. The rootless
broker link uses this relay (E-23), and T-020 and T-022 build on it.

## Approach

Log each failed connect with the target and the error, rate-limited, and
count them for T-023. A relay that records "success" before it connects is
the shape to avoid: lens-sandbox-core issue 57, in
[2026-10-24-brokers-b.md](../docs/history/references/2026-10-24-brokers-b.md).

## Prove

```sh
cargo test --locked relay   # a refused target produces one log line naming it
```

## Closing

**Closed 2026-10-24.** `Refusals` in `src/relay.rs` writes one line to
stderr at the first failed connect, naming the target and the error, then
at most one line every 10 s with the count since the last; never to stdout,
which carries the agent protocol in the shim. The prove passed, 5 relay
tests. With the line silenced, the new test fails. Live: 52 of 52 with this
binary.

---

# T-032: The broker's provider route: the smaller defects errand's tracker names

**Source:** errand#44, #45, #46, #51 and #54 item 2, read in the research of
2026-10-24
**Category:** errand
**Priority:** P2
**Effort:** M
**Status:** open

## Problem

Each is confirmed by reading at `715e9b0b777a`, and each reaches errand-box:

- an upstream that echoes a request header hands the session the key: the
  answer is relayed untouched (`errand/src/sandbox/broker/mod.rs:694`,
  errand#46);
- the provider client follows redirects with its default policy
  (`errand/src/sandbox/broker/server.rs:60`), so a redirect to another host
  may carry `x-api-key` (errand#54 item 2; reqwest's handling not read);
- a brokered key can call any method on any path of its provider
  (errand#44);
- the nonce is 32 random bytes in hex, shaped like a key and not scrubbed
  from chat (errand#45);
- `is_private_v4` has no arm for 168.63.129.16 (errand#51).

## Approach

No redirects on the provider client. Scrub the credential from the answer's
headers and body with a streaming redactor that holds back only a partial
match (claw-wrap's `internal/daemon/output_redactor.go:112-160`, in
[2026-10-24-brokers-a.md](../docs/history/references/2026-10-24-brokers-a.md)).
A prefix on the nonce, and the nonce in the redaction set. The one address
arm. Method and path rules per route are a design for T-021.

## Prove

```sh
cargo test --locked -p errand broker
```

---

# T-033: Files the daemon reads from a session are read whole, and an upload is not redacted

**Source:** errand#36 (the size half) and errand#53, read in the research of
2026-10-24
**Category:** errand
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

The daemon checks a file's size, then reads it whole
(`errand/src/session/session/files.rs:230`, `:244`), and `read_beneath` has
no cap (`errand/src/sandbox/paths.rs:229-234`). A sparse file in `/state`
can make the daemon read gigabytes. An upload keeps its bytes unchanged
(`errand/src/session/redacted.rs:105-112`), so `!file` of a file holding
`GH_TOKEN` posts the token to the thread, and the redaction floor is 8 bytes
(`errand/src/config/redact.rs:17`).

## Approach

Read through a limit. Scan an upload's bytes with the redaction set before
it is posted.

## Prove

```sh
cargo test --locked -p errand files
```

---

# T-034: Remove from the fork what errand-box does not run

**Source:** the research of 2026-10-24, the code sweep's section 2
**Category:** errand
**Priority:** P3
**Effort:** M
**Status:** partial

## Problem

The fork carries a podman backend with no broker (errand#50, #55), a GitHub
issues surface errand-box does not configure, and an operator CLI. Each is
code nobody here runs and every defect in it is errand-box's.

## Approach

Delete `errand/src/sandbox/podman.rs` and its config value, and
`errand/src/issues/`, with their tests and pages. The code sweep measured
the coupling: `issues` is used from 2 files, `podman` from 1
(`errand/src/daemon.rs`). Keep the web interface, which errand-box serves on
loopback, and the CLI until the operator rules on each.

## Decision

The operator ruled on 2026-10-24: **remove the web interface** (`errand/web/`,
`errand/src/web/`, and the `web` block errand-box writes); a Cloudflare
Workers integration may come much later, as its own entry. **Move, do not
remove, the thread commands**: they become `errand-box threads`, as T-028's
correction says.

## Done, and open

**Done on 2026-10-24: the web interface and the thread commands.**
`errand/web/`, `errand/src/web/`, `errand/build.rs` and
`errand/docs/interface.md` are deleted. The `web` setting is gone from the
schema, the example and the reference page, and a config that has one is
refused ("web is not a setting"); errand-box no longer writes it, and setup
removes it from a kept config. What only the interface read went with it:
`publicUrl`, the transcript link in a pull request's footer, and the session
manager's detached start, lookups by session and resumable list. The tests
of those went too; `attach_view` stays for the tests that read a session's
notices. The thread commands run as `errand-box threads` (T-028).

**Open:** the podman backend and `errand/src/issues/`. The operator ruled on
the interface and the thread commands only; the approach above still stands
for these two.

## Prove

```sh
sh scripts/ci-local.sh
grep -rn 'Podman' errand/src | wc -l   # 0
```

---

# T-035: A Discord stand-in, so errand's chat path is tested without Discord

**Source:** the operator, 2026-10-24: "we will need this to test errand-box
and it must be quite close to the real discord apis/backend"; the research
of that day,
[2026-10-24-discord-mock.md](../docs/history/references/2026-10-24-discord-mock.md)
**Category:** errand
**Priority:** P1
**Effort:** L
**Status:** partial

## Problem

Nothing tests errand's chat path. T-015 needs a real Discord application,
and the live tests stop errand at the chat connection, so a defect between
a message and a session's reply reaches an operator first.

## Premise

Read in serenity 0.12.5 and errand, not run, in the research above; seven
of its citations were opened again at their lines and hold. errand reaches
15 REST operations, one attachment fetch, 4 client opcodes, 6 server opcodes
and 11 dispatch types, with intents 33281. serenity takes another REST base
through `HttpBuilder::proxy` with its rate limiter off, and takes the
gateway URL from `GET /gateway`, which falls back to
`wss://gateway.discord.gg` when that call fails (`client/mod.rs:364-370`).

## Approach

The research's "Implementation plan", in its order: the workspace member
`discord-mock/`, a dev-dependency of errand only; one store with REST and
gateway projections, and a 501 for any route it does not implement; the
errand patch in `errand/src/chat/endpoint.rs`, switched by the environment
variable `ERRAND_DISCORD_MOCK_URL`, with the three-part guard; then the first
five tests that section names.

## Decision

- **A focused Rust mock, not Spacebar or fauxcord.** Spacebar needs Node and
  Postgres and answers every RESUME with INVALID_SESSION; fauxcord has no
  licence.
- **An environment variable, not a config key**, so errand-box's setup and
  `up` never write it, and a guard that refuses a real token and a non-local
  gateway before any request.
- **The stand-in runs in CI and in-process tests only.** The operator's
  ruling of 2026-10-24: no release asset, no install on a test host, and
  `tests/live` does not use it. The `discord-mock` binary is for a CI job
  that runs errand against it from outside.

## Done, and open

**The plan's steps 1 to 4, the errand patch and the five tests done on
2026-10-24; steps 5 and 6 in large part; step 7 not started.**

- The stand-in: `discord-mock/`, a workspace member (0BSD,
  `publish = false`), a dev-dependency of errand only. `src/state.rs` is
  the one store, whose changes return the dispatches they cause;
  `src/rest.rs` serves the fifteen operations, the attachment CDN with
  signed URLs, and answers 501 for any other route, method or body field;
  `src/gateway.rs` holds sessions, `seq`, replay on RESUME, intents, the
  content intent, zlib payload compression, the 120-sends limit and the
  faults; `src/journal.rs` records every call and frame; `src/control.rs`
  is what a person or a moderator does; `src/main.rs` is the binary. A
  handle dropped with an unread gap fails its test. 3,651 non-test lines
  (3,042 without blank and comment lines), against the plan's estimate of
  2,000 to 3,500. New crate names: `multer` and `spin` (axum's multipart),
  `twilight-model` and five of its own, dev only: the plan's measured 8.
- The errand patch: `errand/src/chat/endpoint.rs`, called where
  `serve.rs` built the client, with the three-part guard and one
  `WARNING:` line while the stand-in is in force; unset, the builder is
  `serenity::Client::builder` as before. `FactoryAdapter` moved out of
  `run` so the tests use the adapter `run` uses. `reqwest012` is serenity's
  own reqwest 0.12.28: no crate added. `grep -rn DISCORD_MOCK src image
  bootstrap.sh examples tests` finds nothing: errand-box never sets it.
- The spec: `discord-mock/spec/errand-surface.json`, made by
  `scripts/discord-spec.sh` (with `scripts/discord-spec.py`, always
  `python3 -I`) from `discord/discord-api-spec` at `bf93257f85d6`: 15
  operations, 250 components, 164,656 bytes, the research's 164,655 and a
  newline. Its MIT licence is beside it. The commit, the whole file's
  SHA-256 and the snapshot's SHA-256 are pinned in the script. `pins.toml`
  was the plan's home and was refused: errand-box compiles it in
  (`src/fetch.rs:12`) and never downloads the spec, and `freshness.sh`
  reads every table there as a GitHub release, so it would fail every week.
- The gate: the step `discord-contract` in `scripts/ci-local.sh` and
  `gates.yml`, which installs `python3-jsonschema` first. The daily
  watchdog runs `discord-spec.sh --drift` as the check `discord-spec`.

The prove, on this session's container (Ubuntu 24.04, Python 3.13.17,
jsonschema 4.26.0, not the packaged 4.10.3):
`cargo test --locked -p discord-mock`, 26 tests pass;
`cargo test --locked -p errand --lib -- endpoint stand_in`, 13 pass;
`sh scripts/discord-spec.sh --check`, 22 answers, 0 failed;
`sh scripts/ci-local.sh`, every step passed but `cross`, which exits 2 as
before.

Each test was seen to fail, the exit code read unpiped, then restored:

| test | planted defect | what failed |
| --- | --- | --- |
| 1, the guard | condition 2 switched off | `a_token_without_the_mock_prefix_is_refused`: a real token's shape was taken; exit 101 |
| 1, the guard | condition 3 switched off | `a_failed_gateway_lookup_is_refused_not_sent_to_discord`: `Ok(())` where serenity had fallen back to `wss://gateway.discord.gg`; exit 101 |
| 2, the handshake | errand asks for `GUILD_MESSAGE_TYPING` too | IDENTIFY's intents 35329, not 33281; exit 101 |
| 3, a message opens a thread | the post without `SUPPRESS_EMBEDS` | the thread post's `flags` 0, not 4; exit 101 |
| 4, the spec | `flags` dropped from the message answer | `discord-contract` exit 1: six answers, "'flags' is a required property" |
| 5, two clients | `channel_id` dropped from the message | serenity read MESSAGE_CREATE as `Event::Unknown`; exit 101 |
| 5, the replay | RESUME replays one dispatch short | `s` rose by two; exit 101 |

Found while building it:

- errand sets an outcome reaction on the message that started a session
  by fetching it through the thread: `GET /channels/{thread}/messages/{m}`
  (`errand/src/chat/threads.rs`, `fetch`). The message is in the parent
  channel, so the stand-in answers 404 and errand skips the reaction.
  Discord's answer to that call is not in its docs: a thread's first
  message is a THREAD_STARTER_MESSAGE (`developers/topics/threads.mdx:134`),
  and its id is not stated. A capture from T-015's bot settles it.
- serenity #3575 does not hold for 0.12.5: op 1 with `"s": null` is
  answered (`discord-mock/tests/faults.rs`).
- Test 2 is in errand (`chat::endpoint::tests`), not in the stand-in's
  tests as the plan put it: errand's modules are private, and the plan's
  place needs a public test API in errand and a dev-dependency cycle.
- Running `check-todo.sh --set` also rewrote the counts line of
  `TODO/PROGRESS.md`; the gate refuses the record without it.

**Open:**

- Step 7, rate limits for route A2: the rate-limit headers are fixed
  values, and nothing answers 429.
- Step 6 in part: op 9 with `true`; errand's own state across a resume
  (`errand/src/chat/gateway.rs:434-441`); the 15-minute token, which needs
  a clock the tests can move. Done: drop and resume, op 1, op 7, op 9
  with `false` and the 5-second re-IDENTIFY, a withheld ACK, 4004, 4014
  as errand's permanent error, 4008.
- Permissions and 50013 are not modelled; THREAD_MEMBER_UPDATE after a
  thread is made is inferred, not captured; gateway fixtures from a real
  capture wait for T-015's bot.
- errand's attachment fetch (`errand/src/session/session.rs:2700-2733`) is
  not driven against the CDN; it needs the fetch injected.

## Prove

```sh
cargo test --locked -p discord-mock
cargo test --locked -p errand --lib -- endpoint stand_in   # the guard, the handshake, a thread, a command
sh scripts/discord-spec.sh --check                          # every answer against Discord's spec
sh scripts/ci-local.sh
```
