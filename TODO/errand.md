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
- A pinned errand release on a host that has one: setup must replace the
  downloaded binary with the link, and `uninstall` must not delete
  errand-box when it removes `errand`.

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
**Status:** open

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

---

# T-030: The host-side push can publish another session's repository

**Source:** errand#52, reproduced with git 2.43.0 in the research of
2026-10-24
**Category:** errand
**Priority:** P1
**Effort:** S
**Status:** open

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

---

# T-031: The relay drops a connection with no log line when its target does not answer

**Source:** the research of 2026-10-24, read in errand-box's own code
**Category:** errand
**Priority:** P2
**Effort:** S
**Status:** open

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
**Status:** open

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

Open for the operator: keep or remove the web interface and `errand
threads`. The recommendation is to keep both: neither has an open defect,
and the interface is on loopback only.

## Prove

```sh
sh scripts/ci-local.sh
grep -rn 'Podman' errand/src | wc -l   # 0
```
