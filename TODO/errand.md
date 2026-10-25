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
**Status:** done

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
  likewise), and errand's exits 2, 3 and 4 are refusals that a restart
  repeats every 2 s. errand's own service files did not restart them. Not
  restarting is not the fix either: a sandbox that does not answer yet is
  also exit 2 (`errand/src/serve.rs:818`), which is E-34's race at boot.
  Proposed to the operator as an entry, with a backoff; not filed.

## Done, and open

**Done on 2026-10-24, every step; step 4 by T-034.** Shipped in
[v0.1.1](https://github.com/Azathothas/errand-box/releases/tag/v0.1.1),
the tag on `2ead3fd`, after `gates` passed there. Its assets were downloaded
again and checked: both binaries match `SHA256SUMS`, the x86_64 one reports
`errand-box 0.1.1` and carries `serve`, and `bootstrap.sh --dry-run` from
the tag fetches and checks them. The notes below are the order it was done
in.

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

**The live prove of the library shape, 2026-10-24**, on the same kind of
host, after a full teardown of the first: setup with the static binary of
`9b27e60`, 30 changed and 0 failed, reporting errand-box "which runs errand
as `serve`"; a second setup changed nothing. The errand service's run
script execs `errand-box __as errand -- errand-box serve`, and
`/usr/local/lib/errand-box/bin` holds errand-box and kage only.
`tests/live/live.sh`: 54 of 54, two of them the new checks of that shape
([output](../tests/live/2026-10-24-errand-library.txt)). `errand-box
threads` as root ran as errand and answered "no threads are remembered".
Torn down after: uninstall, the 87 packages the run added (the purge
simulated first and compared with the list), the accounts, the bridges, the
dnsmasq that T-036 names (stopped by PID), the subordinate ids, the device
modes and the Zabbly repository; the package list matches the one before.

## Prove

```sh
sh scripts/ci-local.sh            # every step, errand included
sh tests/live/live.sh             # errand runs as errand-box serve
```

The prove first named `readlink /usr/local/lib/errand-box/bin/errand`, for
the link shape the rulings replaced; `live.sh`'s `errand` test now checks
that no `errand` program is there.

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

**Correction, 2026-10-25 ([issue #48](https://github.com/Azathothas/errand-box/issues/48)).**
The proof of this fix is the broker unit tests, not the live run cited above.
The four new tests in `errand/src/sandbox/broker/tests.rs` send each
boundary-crossing spelling and assert it is refused, and the planted-defect
run (the old match and join) shows the steered request reaching the second
stand-in provider with the key; that is the evidence. The live tests of
`tests/live/live.sh` send no `/provider/` path, so the 52 of 52 run does not
exercise this path on a host. A live check that does, and a gate that
requires an entry's recorded prove output, are T-087; the fix itself stands.

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
2026-10-24; the address arms extended by [issue #59](https://github.com/Azathothas/errand-box/issues/59), read on `57231da`
**Category:** errand
**Priority:** P2
**Effort:** M
**Status:** done

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
- `is_private_address` judges several internal destinations public, so a
  name that resolves to one is dialled with the default allowlist. Confirmed
  by compiling the functions as they are on `57231da`
  (`errand/src/sandbox/broker/mod.rs:243-269`, `:345-385`): `is_private_v4`
  has no arm for 168.63.129.16, Azure's WireServer (errand#51); and
  `is_private_v6` has no arm for 6to4 (`2002::/16`, which wraps an IPv4
  address in bits 16 to 48), documentation (`2001:db8::/32`), site-local
  (`fec0::/10`) or Teredo (`2001::/32`). The v4 arm for the WireServer must
  be inside `is_private_v4`, not only the literal branch of
  `public_addresses`, or a name resolving to it still dials (issue #59).

## Approach

No redirects on the provider client. Scrub the credential from the answer's
headers and body with a streaming redactor that holds back only a partial
match (claw-wrap's `internal/daemon/output_redactor.go:112-160`, in
[2026-10-24-brokers-a.md](../docs/history/references/2026-10-24-brokers-a.md)).
A prefix on the nonce, and the nonce in the redaction set. The address arms:
168.63.129.16 inside `is_private_v4`, and `2002::/16` (judging the embedded
v4), `2001:db8::/32`, `fec0::/10` and `2001::/32` inside `is_private_v6`.
Method and path rules per route are a design for T-021.

## Prove

```sh
cargo test --locked -p errand broker
```

A table test of `is_private_address` over the addresses in issue #59: the
WireServer, the 6to4, documentation, site-local and Teredo spellings each
read private.

## Closing

**Closed 2026-10-25.** All four in-scope defects fixed; the fifth (per-route
method and path rules, errand#44) was always T-021's design, as the Approach
says, and stays there.

Address arms (`errand/src/sandbox/broker/mod.rs`, issues #51 and #59):
`is_private_v4` now judges `168.63.129.16` (Azure's WireServer) private, so a
name resolving to it is caught, not only the literal. `is_private_v6` judges
6to4 (`2002::/16`) by the v4 it wraps, and the documentation (`2001:db8::/32`),
Teredo (`2001::/32`) and site-local (`fec0::/10`) ranges. A table test,
`the_metadata_and_v6_transition_ranges_of_issue_59_are_refused`, covers each
and keeps an ordinary `2001::` allocation and a 6to4 of a public v4 public.
`public_addresses` judges every resolved address through these, so a name is
covered as well as a literal.

Redirects (errand#54 item 2): the provider client is built with
`redirect(reqwest::redirect::Policy::none())` (`server.rs`), so a 30x from an
upstream is not followed with the real credential to a host the allowlist never
cleared.

Echoed credential (errand#46): `stream_answer` scrubs the real credential from
the answer's headers and, through `redacting_stream`, from the streamed body,
so an upstream that reflects the key cannot hand it back to the session; it
stays out of the session's memory, not just its chat output (which the
`Redacting` layer already covered). The streaming redactor holds back the tail
of each chunk so a value split across a boundary is still caught; `Content-Length`
is dropped because redaction changes the body's length and it is re-framed.
Tests: `redact_bytes_replaces_every_complete_occurrence`,
`the_answer_redactor_catches_a_secret_split_across_chunks`, and
`the_real_credential_is_scrubbed_from_the_answer` (an upstream that echoes the
key in a header and the body).

Nonce (errand#45): `provider_nonce` (`serve.rs`) mints the nonce with a fixed
`errand-nonce-` prefix (`config::redact::BROKER_NONCE_PREFIX`), and
`redact_text` scrubs any value of that shape (`redact_nonces`), so a nonce a
session echoes is hidden though it is minted per run and never a config secret.
Judging by shape rather than threading each per-run nonce into the secret list
keeps it self-contained and also catches a truncated or stale-run nonce; it is
missed only if an agent re-encodes it, the same limit the verbatim-secret layer
states. Test: `a_broker_nonce_is_scrubbed_by_its_shape`.

Three `forbidden-patterns.md` rows name the classes: following a redirect while
holding a credential, rewriting a proxied body while forwarding its
`Content-Length`, and scrubbing a stream chunk by chunk with no carry across
the boundary.

Prove, run 2026-10-25:

```sh
cargo test --locked -p errand --lib sandbox::broker
cargo test --locked -p errand --lib config::redact
```

```text
sandbox::broker: ok. 37 passed; 0 failed
config::redact:  ok. 9 passed; 0 failed
```

fmt and clippy are clean. The change is in the fork's broker and redaction
logic, not setup, `up`, the shim or the image, so RULES.md's live gate does not
bind it.

---

# T-033: Files the daemon reads from a session are read whole, and an upload is not redacted

**Source:** errand#36 (the size half) and errand#53, read in the research of
2026-10-24
**Category:** errand
**Priority:** P2
**Effort:** S
**Status:** done

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

## Closing

**Closed 2026-10-25.** Both halves fixed.

Bounded reads. `paths::read_beneath` reads through a ceiling
(`MAX_BENEATH_BYTES`, 16 MiB) counted from the bytes read, not the metadata, so
a sparse or oversized file a session planted in a writable tree is an error
rather than a gigabyte read; the ceiling is far above the largest control file
errand reads this way (a diff snapshot, already guarded at 512 KiB by its
caller). The `!file` upload (`session/files.rs`) reads `max_upload_bytes + 1`
through `Read::take` and refuses a file that reached the limit, so one that
grew between the size check and the read is caught rather than streamed whole.

Redacted uploads. `redact_event`'s `Upload` arm now scrubs `bytes` through a new
`config::redact::redact_bytes`, so a key in a file handed back by `!file` is
removed like every text field, keeping it off the thread. `redact_bytes` reuses
the shared byte primitive `replace_bytes` (the broker's streaming redactor was
moved onto the same primitive, so there is one byte-level scrubber, not two) and
also scrubs a broker nonce by shape when the bytes read as text.

The 8-byte redaction floor (errand#36's other note) is left as it is: it is a
deliberate guard against mangling prose with a too-short "secret", documented at
`redact.rs`, and lowering it trades a real leak-surface question for noise. The
provider credentials and the chat and GitHub tokens are all well above it.

Tests: `redact_bytes_scrubs_secrets_and_nonces_in_a_files_bytes`
(`config/redact`), `an_uploads_bytes_are_scrubbed` (`session/redacted`),
`read_beneath_refuses_a_file_over_its_limit` (`sandbox/paths`), and the broker's
`replace_bytes_replaces_every_complete_occurrence` over the shared primitive.
Two `forbidden-patterns.md` rows name the classes: an unbounded or
metadata-trusting file read, and a reported field of raw bytes that skips the
redactor.

Prove, run 2026-10-25:

```sh
cargo test --locked -p errand --lib
```

```text
test result: ok. 1105 passed; 0 failed; 1 ignored
```

fmt and clippy are clean. The change is in the fork's read and redaction paths,
not setup, `up`, the shim or the image, so RULES.md's live gate does not bind
it.

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

  **Correction, 2026-10-24, the operator:** the binary goes to test hosts
  too, as an asset of the prerelease `artefacts`
  (`.github/workflows/artefacts.yml`), for T-027's end-to-end run. It is
  still no asset of a version. `tests/e2e/mock-session.sh` runs errand
  against it on a host errand-box set up with `tests/e2e/mock.env`; it runs
  errand by hand, because `errand-box __as` clears the environment
  (`src/procs.rs`) and nothing hands the supervised errand
  `ERRAND_DISCORD_MOCK_URL`. Measured on a hosted container with the
  assets of `e3121d0` (v0.1.1 installed, the image imported from
  `artefacts`): PASS, errand opened a thread (201) and posted three
  messages in it, with no gap; kage then refused the first turn, "no
  provider credentials found", with the `mock.env` gateway.

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

---

# T-041: ERRAND_PROVIDER with a custom gateway never reaches kage

**Source:** [issue #5](https://github.com/Azathothas/errand-box/issues/5), T-027's end-to-end run, 2026-10-24
**Category:** errand
**Priority:** P1
**Effort:** S
**Status:** open

## Problem

errand-box writes a provider's credential, base URL and kind, and no
models. errand leaves a provider kage does not know out of kage's config
unless it has models, so kage finds no provider. A name the pinned kage
knows and errand's list does not is served by kage's own endpoint, and the
base URL is ignored.

## Premise

Measured on WSL2 (Ubuntu 24.04) with a real Anthropic-compatible gateway:
`gateway` and `kenari` failed (the second went to kage's own endpoint and
the broker refused it); `anthropic` with the same base URL answered a model
turn. Read: `src/errandcfg.rs:101-129`, `errand/src/sandbox/agent_config.rs:38-59`, `:265-298`.

## Approach

1. Write `models: [{id: ERRAND_MODEL}]` into the provider.
2. Keep errand's list of kage's providers in step with the pinned kage, by
   a test.
3. setup's errand row says what kage will see; examples/errand-box.env
   shows a custom gateway.

## Prove

```sh
cargo test --locked errandcfg:: -p errand-box
cargo test --locked -p errand --lib -- agent_config
```

With `ERRAND_PROVIDER=gateway` and a real gateway, a session's kage config
holds `[providers.custom.gateway]` with its model, and a turn is answered.

---

# T-049: A `mock.` token without the stand-in's URL goes to Discord

**Source:** [issue #13](https://github.com/Azathothas/errand-box/issues/13), T-027's end-to-end run, 2026-10-24
**Category:** errand
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

With tests/e2e's inputs the supervised errand uses Discord itself, since
`__as` clears the environment and no setting names the stand-in. Discord
refuses the token and errand exits and restarts; where Discord is not
reachable it waits 30 s each time. tests/e2e/mock-creds.env claims the
token only goes to a loopback stand-in.

## Premise

Measured on WSL2 (Ubuntu 24.04): "Sent invalid authentication", exit 2,
88 restarts; on Daytona, a 31 s loop. Read:
`errand/src/chat/endpoint.rs:97-105`, `src/procs.rs:217-219`.

## Approach

1. Refuse a `mock.` token at start when the variable is unset.
2. An optional ERRAND_DISCORD_MOCK_URL in errand-box.env that setup passes
   to the service, under errand's three conditions.
3. Correct the comment in the kit.

## Prove

```sh
cargo test --locked -p errand --lib -- endpoint
```

With the tests/e2e inputs, the supervised errand either serves discord-mock
or refuses once with the variable's name.

---

# T-059: The message that starts a thread never gets its reaction

**Source:** [issue #23](https://github.com/Azathothas/errand-box/issues/23), T-027's end-to-end run, 2026-10-24
**Category:** errand
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

errand fetches a message through the thread's channel before it reacts to
it. The message that started the thread lives in the parent channel, so
the fetch returns 404 and the reaction is skipped without a log line.

## Premise

Measured against discord-mock on WSL2 and Daytona: two GET 404 per new
thread, no reaction on the first prompt, reactions on every reply. Read:
`errand/src/chat/threads.rs:198-226`, `:576-583`.

## Approach

Fetch and react to the starter through the thread's parent; log a skipped
reaction; a stand-in test.

## Prove

```sh
cargo test --locked -p errand --lib -- stand_in
```

mock-session's calls.txt shows a reaction on the first prompt and no 404.

---

# T-060: `!allow` confirms access for an account errand ignores

**Source:** [issue #24](https://github.com/Azathothas/errand-box/issues/24), T-027's end-to-end run, 2026-10-24
**Category:** errand
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

`!allow` of an account outside allowedUserIds answers that it can now
prompt the session, and errand still drops every message from it. Such an
account also gets no reply and no reaction, though threads.md says it is
told once.

## Premise

Measured against discord-mock with a second seeded person. Read:
`errand/src/chat/inbound.rs:218-222`, `errand/src/session/session/answering.rs:122-150`.

## Approach

Refuse `!allow` for an account that is not permitted, or let guests past
the gate for their thread; react once on the ignore path.

## Prove

```sh
cargo test --locked -p errand --lib -- inbound answering
```

---

# T-074: Every failed prompt is posted twice

**Source:** [issue #38](https://github.com/Azathothas/errand-box/issues/38), T-027's end-to-end run, 2026-10-24
**Category:** errand
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

One provider failure gives two thread messages, "the agent reported an
error" and "the agent refused the prompt". `!compact` with no provider
reports success and then the error.

## Premise

Measured against discord-mock. Read: `errand/src/session/session.rs:895-920`.

## Approach

One message per failure; `!compact` reports after the agent answers.

## Prove

```sh
cargo test --locked -p errand --lib -- session
```

---

# T-077: discord-mock has no HTTP control for delete, archive and commands

**Source:** [issue #41](https://github.com/Azathothas/errand-box/issues/41), T-027's end-to-end run, 2026-10-24
**Category:** errand
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

The binary's `/_mock/` routes post messages, inject faults and read the
journal. Deleting a message, archiving or deleting a thread, a slash
command and expiring attachments exist on `MockHandle` and nowhere else, so
an end-to-end test cannot reach those paths of errand.

## Premise

Read: `discord-mock/src/control.rs:231-239`, `:101-190`. Measured: every
gateway fault over HTTP left errand's session intact.

## Approach

`/_mock/` routes for the existing methods, and steps that use them.

## Prove

```sh
cargo test --locked -p discord-mock
```

---

# T-081: refuse_borrowed is checked seconds before the clone, so the git directory can be swapped in between

**Source:** [issue #46](https://github.com/Azathothas/errand-box/issues/46), read on `57231da`
**Category:** errand
**Priority:** P1
**Effort:** M
**Status:** done

## Problem

T-030 added `refuse_borrowed` to refuse a session git directory that borrows
another repository through `commondir`, an alternates file or a symlink
(`errand/src/session/pr.rs:266-279`). It runs in `assert_contained` at the
start of `open_pull_request` (`errand/src/session/pr.rs:248-266`). The clone
is made later, in `push_work` (`errand/src/session/pr.rs:696`, `git clone
--shared --bare`), after `current_branch`, `origin_remote`, a GitHub API
call, and `fork_of`, which polls for up to `FORK_WAIT_MS` of 30 s
(`errand/src/session/pr.rs:506`, `:619`). The project is bind-mounted into
the guest, read and write, the whole time. So the guarded directory and the
cloned directory are one live path, read seconds apart, and a guest process
can plant the borrow after the check and before the clone. This reopens the
T-030 threat (the host-side push publishes another repository under the bot
token) through a time window.

## Premise

Read on `57231da`. The window was reproduced by the reporter in a shell on
plain git, not against a live errand: a loop that writes and removes an
alternates file every few milliseconds, against a check and clone seconds
apart, carried another repository's objects into about one clone in six. The
timing against a live guest and the real push were not run.

## Approach

Checking and cloning one live, guest-writable directory cannot close the
window. Copy the git directory to a path the guest cannot write
(`cp -a --reflink=auto`), run `refuse_borrowed` on the copy, and clone from
the copy. For the agent-requested path, stopping the guest's processes before
the clone is the other route.

## Prove

```sh
cargo test --locked -p errand pr
```

A test whose fake runner swaps the git directory between the guard and the
clone fails on the current code and passes once the clone reads a snapshot.

## Closing

**Closed 2026-10-25.** `push_work` (`errand/src/session/pr.rs`) no longer
clones the live project. It copies the git directory `assert_contained`
resolved to a staging path the guest cannot write
(`cp -a --reflink=auto <git_dir> <staging>/source.git`), runs `refuse_borrowed`
on that copy, and clones the copy (`git clone --shared --bare <staging>/source.git ...`).
The check and the clone now read one snapshot nothing can swap under them, so
the window between the guard and the clone is closed: a borrow planted before
the copy is caught on the snapshot, and one planted after it cannot reach the
clone. `cp -a` copies a symlink as a symlink rather than following it, so a
`.git` (or a path inside it) turned into a link is caught by `refuse_borrowed`,
not chased by `cp`. `assert_contained` still runs the live `refuse_borrowed`
as an early, cheap refusal before the fork and the API calls, and now returns
the resolved git directory for the snapshot step.

The snapshot route (not the "stop the guest's processes" alternative) was
taken, since it needs nothing of the engine. The copy and the clone run on the
host as the errand account, where the staging directory (`tempfile::tempdir`)
is outside every session's bind mount.

Two regression tests (`errand/src/session/pr/tests.rs`), with the fake runner
performing the `cp` for real so the snapshot and `refuse_borrowed` act on a
real tree:

- `a_borrow_planted_after_the_guard_is_caught_on_the_snapshot` plants a
  `commondir` at the `remote` call, within the window and after the early
  guard read a clean tree, and asserts the request is refused and nothing is
  pushed. It fails when the snapshot check is removed (verified by neutering
  `refuse_borrowed(&source)` and watching it fail), so it guards the window,
  not just the static case.
- `the_push_clones_a_snapshot_not_the_live_tree` asserts the clone's source is
  the copy in staging, never the session's tree or its git directory.

A `forbidden-patterns.md` row names the class: checking a path an untrusted
party can write, then acting on that same live path instead of on an immutable
copy.

Prove, run 2026-10-25:

```sh
cargo test --locked -p errand --lib session::pr
```

```text
test result: ok. 48 passed; 0 failed
```

fmt and clippy are clean. The change is in the fork's push path, not setup,
`up`, the shim or the image, so RULES.md's live gate does not bind it; the
host primitives it adds (`cp -a --reflink=auto`, then
`git clone --shared --bare` from the copy) were exercised on this host.

---

# T-082: the broker reaches egress only directly, so a proxy-only host gets no session egress and a TLS-intercepting proxy breaks every session

**Source:** [issue #56](https://github.com/Azathothas/errand-box/issues/56) and [issue #57](https://github.com/Azathothas/errand-box/issues/57), read on `57231da`
**Category:** errand
**Priority:** P2
**Effort:** M
**Status:** open

**Superseded by T-090 (2026-10-25).** The upstream HTTP proxy and `proxy_ca`
this entry adds become the virtual network layer's upstream-hop policy (T-090
task 7): the gate dials direct or CONNECTs via the proxy, refuses an internal
target before the proxy, and carries the CA into the dialer and the session
trust store. The problem statement below is kept verbatim as the record; the
work now lives in T-090, and this stays open until T-090 closes it.

## Problem

errand's broker is the only way out of a session, and it reaches its targets
by a direct connection. On a host whose only egress is an HTTP proxy, every
session `CONNECT` fails. This is E-22, recorded as "errand-box does not
change errand"; the fork is errand-box's now, so that is no longer true.

1. The broker dials each target with `TcpStream::connect`
   (`errand/src/sandbox/broker/server.rs:272`). There is no upstream proxy.
   The provider route uses `reqwest::Client::builder` (`:59`), which reads
   the proxy environment, so the model provider can work through a proxy
   while every session fetch fails: a session that talks to the model and
   cannot fetch a package.
2. `network.proxy_ca` reaches only the image build (`src/image.rs:321-324`);
   nothing else in `src/` or `errand/src/` reads it (E-15 covers the build).
   On a network with a TLS-intercepting proxy the build works and a session
   fails TLS as soon as it goes through anything that re-signs, and errand's
   own provider call fails the same way.

## Premise

Read on `57231da`. Not run: no proxy, session or broker was started. Through
a proxy the broker resolves nothing itself, so the address-level check on a
`CONNECT` target is lost and the name check is all that remains; the proxy's
own address is usually private, so it alone is exempt from the private-address
refusal.

## Approach

1. `sandbox.egress.upstreamProxy` (a URL, with credentials from the
   environment, never the config file) and `noProxy`: for a `CONNECT
   host:port`, send `CONNECT host:port` to the proxy and relay the tunnel,
   keeping the allowlist and port checks on the name. Point the provider
   route at the same proxy, so the two stop differing.
2. When `network.proxy_ca` is set, pass `SSL_CERT_FILE`, `CURL_CA_BUNDLE`,
   `NODE_EXTRA_CA_CERTS`, `REQUESTS_CA_BUNDLE` and `GIT_SSL_CAINFO` into the
   session environment, and give the broker's provider client the CA
   (`reqwest::Client::builder().add_root_certificate`). Verify each variable
   name against its tool's current documentation before relying on it.
3. Update E-22 and E-15: the column that says errand-box does not change
   errand is stale.

## Prove

```sh
cargo test --locked -p errand broker
```

A test with a fake proxy listener: with `upstreamProxy` set, an allowlisted
`CONNECT` reaches the proxy as `CONNECT host:port` and the broker resolves
nothing itself; a host not on the allowlist still gets 403 and never reaches
the proxy. A test builds the provider client with a CA and reaches a local
TLS fixture signed by it.

---

# T-083: a refused egress is silent, ERRAND_EGRESS_ALLOW replaces the defaults, and a port opens for every host

**Source:** [issue #58](https://github.com/Azathothas/errand-box/issues/58), read on `57231da`
**Category:** errand
**Priority:** P2
**Effort:** M
**Status:** partial

## Problem

The egress allowlist is hard to set and its refusals are invisible, so a
session fails a fetch and nobody waiting learns why or how to fix it.

1. The broker answers `403 not on the egress allowlist` for both a host that
   is not allowed and a port that is not allowed, one reason, one info-level
   log line (`errand/src/sandbox/broker/server.rs`). Nothing reaches the
   chat.
2. `ERRAND_EGRESS_ALLOW` replaces the default allowlist rather than extending
   it: `let mut allow = list(...); if allow.is_empty() { allow = DEFAULT_ALLOW }`
   (`src/errandcfg.rs:127-130`). An operator who adds one host loses
   github.com, pypi.org, crates.io and the rest, with no word.
3. `sandbox.egressPorts` applies to every allowed host
   (`errand/src/config/schema.rs`), so allowing ssh on port 22 for one forge
   opens port 22 for all. T-022 is the raw-TCP egress entry; this is the
   allowlist and its visibility.

## Premise

Read on `57231da`. Not run. The refusal body and the single test above it
are the one reason both cases get.

## Approach

1. Put the host, the port and the reason in the refusal body, and post one
   rate-limited line in the session's thread with the host, the port and the
   reason, no credentials. This is the refusal-audit side of errand#47.
2. `ERRAND_EGRESS_ALLOW_EXTRA` extends the defaults; document that
   `ERRAND_EGRESS_ALLOW` replaces them; accept `host:port` entries for a
   per-host port.
3. Consider `errand-box egress test HOST` (dial through the real broker
   configuration and say which layer refused) and `errand-box egress suggest`
   (read lockfiles and print the registries they need); both suggestion only.

## Prove

```sh
cargo test --locked errandcfg:: -p errand-box
cargo test --locked -p errand broker
```

A broker test refuses a port and a host and expects two different reasons in
the body; a config test sets `ERRAND_EGRESS_ALLOW_EXTRA=example.org` and
expects the defaults plus `example.org`.

## Done, and open

**Done 2026-10-25: the distinct refusal reasons and the extending allowlist,
the two the Prove names.**

The broker's `admit` (`errand/src/sandbox/broker/server.rs`) checks the host
and the port separately and puts one reason per cause in the 403 body: the
host named when it is off the allowlist, the port named when it is not open,
or both when both. A session that was refused now reads which to fix rather
than one "not on the egress allowlist" for either. Test:
`a_refusal_says_whether_the_host_or_the_port_was_the_problem`.

`ERRAND_EGRESS_ALLOW_EXTRA` adds to whatever allowlist is in force, the
defaults or the set `ERRAND_EGRESS_ALLOW` replaces them with
(`src/errandcfg.rs`), so an operator who needs one more host keeps github.com,
pypi.org, crates.io and the rest. `examples/errand-box.env` documents both,
that `ERRAND_EGRESS_ALLOW` replaces and `ERRAND_EGRESS_ALLOW_EXTRA` extends.
Test: `allow_extra_extends_the_defaults_and_allow_still_replaces_them`. fmt,
clippy and the `errand-config` check are clean.

**Open, each its own change:**

1. **The refusal in the thread.** A rate-limited line posted into the session's
   thread with the host, the port and the reason (no credentials), for when the
   agent does not surface the 403 itself. The broker has only a `Logger`, no
   handle to the session's views, so this is broker-to-session wiring, not a
   line in `admit`. The informative 403 body (done) already reaches the agent,
   so this is the belt-and-suspenders half.
2. **Per-host ports (`host:port`).** `sandbox.egressPorts` is global, so a port
   opened for one host is open for all. Doing it properly is end-to-end: accept
   `host:port` in the allowlist, match the host part, and enforce the port per
   entry with the global list as the fallback, in the broker and the schema.
   Not started rather than half-wired (a `host:port` the engine parsed but did
   not enforce would be the "setting nobody reads" the conventions forbid). The
   default list is 443 only, so this bites only a non-standard port an operator
   adds.
3. **`errand-box egress test` / `egress suggest`.** Suggestion-only tooling the
   Approach marks "consider"; no behaviour depends on it.

## Prove, run 2026-10-25

```sh
cargo test --locked -p errand-box --bins errandcfg::
cargo test --locked -p errand --lib sandbox::broker
```

Both green (the two tests above among them).

---

# T-089: the broker panics instead of returning an error when a loopback bind is denied

**Source:** [issue #52](https://github.com/Azathothas/errand-box/issues/52), read on `57231da`
**Category:** errand
**Priority:** P2
**Effort:** S
**Status:** done

## Problem

`start_provider` binds the provider endpoint's loopback listener with
`.expect("a loopback port for the provider endpoint")`, and the next line is
another `.expect` (`errand/src/sandbox/broker/server.rs:114-117`). The other
listener in the same file returns its error and `serve.rs` logs it. So a host
that forbids a loopback bind turns a configuration problem into a panic in the
daemon's main task. The release profile sets `panic = "abort"`
(`Cargo.toml`), so the process aborts, and the supervisor restarts it, the
two-second loop of T-037. The fork's own rules say to fail loudly with an
actionable message, not to panic with a bare string.

## Premise

Read on `57231da`. The denied bind was observed in a review sandbox
(`bind(("127.0.0.1", 0))` returned `Permission denied`); errand was not run.
Sandboxes and hosted containers with a network policy are the realistic case.

## Approach

Return `std::io::Result<u16>` from `start_provider`, propagate it from
`listen`, and let `serve.rs` refuse with the message it already uses for the
main listener. The refusal is exit 2, which T-037's backoff then covers.

## Prove

```sh
cargo test --locked -p errand broker
```

A test starts the broker with a provider route and a bind that fails (an
injected listener, or a port already taken) and expects an error return, not
a panic.

## Closing

**Closed 2026-10-25.** `start_provider`
(`errand/src/sandbox/broker/server.rs`) now returns `std::io::Result<u16>`:
the provider endpoint's `TcpListener::bind` and `.local_addr()` propagate with
`?` instead of `.expect`, and `listen` propagates that with `?` at the call
site, so a denied loopback bind returns an error the way the main listener
already does. `serve.rs` already matches on `listen(..).await` and logs the
error, so a bind a host forbids declines that one session rather than aborting
the daemon into T-037's restart loop. The serve future spawned on the runtime
no longer `.expect`s either: a later serve error is logged (`the provider
endpoint stopped serving`) through a cloned `Logger`, because an `.expect`
there runs after `listen` returned and could not be caught by the caller.

A regression test,
`listen_returns_an_error_rather_than_panicking_when_it_cannot_bind`
(`errand/src/sandbox/broker/tests.rs`), builds a broker with a provider route
(so the provider path runs first) and calls `listen("192.0.2.1")`. That is
TEST-NET-1 (RFC 5737), never assigned to an interface, so the bind fails with
`EADDRNOTAVAIL` on every host, and the test asserts `listen` returns `Err`, not
a panic. A `forbidden-patterns.md` row names the class: `.expect()`/`.unwrap()`
on a fallible call in a long-lived daemon path under `panic = "abort"`.

Prove, run 2026-10-25:

```sh
cargo test --locked -p errand --lib sandbox::broker
```

```text
test sandbox::broker::tests::listen_returns_an_error_rather_than_panicking_when_it_cannot_bind ... ok
test result: ok. 33 passed; 0 failed
```

fmt and clippy are clean. No host is needed: the change is entirely in the
fork's logic, so the entry closes here.
