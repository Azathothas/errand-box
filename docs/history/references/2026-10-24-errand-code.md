# Reference sweep: errand's code, vendor and patch or rewrite

errand, read and built on 2026-10-24 for one question: is vendoring errand
into this tree and patching it cheaper and safer than rewriting the parts
errand-box needs, measured in code size, dependencies, patch surface and risk?
The procedure is [`../../methodology/research.md`](../../methodology/research.md)
section 3. Citations are `path:line` at errand `715e9b0b777a` unless a tag is
named. "Estimate" marks every number that was not measured.

**What would make me recommend the rewrite instead**, written before the
measurements: (1) errand-box needs less than about a third of errand's
non-test code; (2) a rewrite keeps errand-box's build posture (no C compiler,
about 20 crates) where vendoring cannot; (3) the operator wants errand's
releases taken routinely. Section "Candidates" says how each came out.

| a reader with | reads |
| --- | --- |
| two minutes | the verdict |
| ten minutes | the two lists below, the verdict, "Candidates" |
| the implementation to do | sections 3 and 4, then 8 |
| a reason to distrust this page | "How to re-check", then the provenance table |

## What this sweep did NOT establish

- **errand was never run.** No Discord token, no kage, no Incus. Builds and
  unit tests ran; nothing exercised a session.
- **No incus backend was written.** Every patch size in section 3 and every
  rewrite size in section 4 is an estimate, with its basis stated.
- **The integration tests under `tests/` did not run.** `cargo test` stopped at
  the first failing target (the 5 layout failures in section 5).
- **aarch64 was not built.** x86_64 musl was built only with clang standing in
  for a musl C compiler, and it failed at link (section 5).
- **Upstream issues 42 to 55 were skimmed, not verified**, except 48, which I
  confirmed by reading the code. Nothing was executed against it.
- **No ACP specification is in the corpus.** Which methods are standard ACP is
  from memory where section 1 says so.
- **Churn covers five weeks of a young Rust codebase** (ported 2026-09-18). It
  may not predict the next five.
- **This page has no earlier revision**, so there is no error rate to quote.
  Assume more errors remain.

## The claims I am least sure of

1. The rewrite estimate, 7,600 to 12,900 non-test lines (section 4). It is a
   judgement per feature, not a measurement. If it is low by half, the
   verdict holds more strongly; if high by half, see the flip conditions.
2. That a serenity-only stack links for musl with clang. ring compiled and
   reported no undefined symbols in the failed link; that is an inference.
3. The incus backend estimate, 900 to 1,350 non-test lines. Basis: errand-box
   does the same work in 905 raw lines (`src/session.rs` 555, `src/incus.rs`
   350), scaled by the two trees' code density (errand 73% code lines,
   errand-box 86%).
4. That the 66 clippy errors are the whole toolchain cost. They are the cost of
   one step, 1.98.0 to 1.99.1, measured once.

## Verdict

**Vendor errand at `715e9b0b777a` as a fork and patch it. Do not rewrite.**
Plausible, with the conditions below, not confirmed: no backend was built.

1. **errand-box needs most of errand.** The needed modules hold 25,752 of
   errand's 33,768 non-test lines (76%; 23,716 or 70% without the command
   surfaces). A rewrite of that set is estimated at 7,600 to 12,900 lines,
   against an estimated 1,300 to 2,200 lines of patches.
2. **The dependency argument for a rewrite fails.** serenity 0.12.5 with
   errand's features, alone with tokio and serde_json, resolves to 194 lock
   entries, 150 unique normal and build crates, and ring (C). errand resolves
   to 273 and 198. A serenity rewrite keeps about three quarters of the crate
   set and a C compiler in the static build.
3. **The paid-for security behaviour comes with it, and the bailey stand-in
   goes.** errand carries fixes for at least seven closed upstream reports
   (2, 6, 7, 9, 12, 17 and 18; section 4) and 1,119 passing unit tests.
   A native backend retires `src/shim.rs`, `src/trust.rs` and `src/policy.rs`
   (865 non-test, 316 test lines), `tests/shim.rs` (372) and the policy
   round trip that `src/trust.rs` exists to guard.

The conditions. **Do not track errand's releases**: the files a backend
touches are its most-changed files, and upstream issue 41 records an accepted
`incus` backend with different rules that will collide by name (section 6).
Reconcile only by reading, per [`vendoring.md`](../../methodology/vendoring.md).
**Fix upstream issue 48 on the day of vendoring**: it is a credential leak in
the broker, and it is also in the errand v0.3.1 that `pins.toml` runs today.

## 1. Anatomy

Counted with the script in "How to re-check". `raw` is `wc -l`, blank lines
included. `code` is non-blank lines minus lines that start with `//`; errand
has no line-leading block comments. Test code is every `*/tests.rs`, every
file under a `tests/` directory, `src/test_util.rs`, `src/docs.rs`,
`src/config/example_test.rs`, and `src/main.rs` from line 269 to its end.
172 `.rs` files, 78 of them test-only.

| module | non-test raw | non-test code | test raw | what it does |
| --- | --- | --- | --- | --- |
| `session/` | 10,554 | 7,758 | 11,110 | lifecycle, thread binding, commands, PR flow, views, records (`session/mod.rs:1-2`) |
| `sandbox/` | 4,218 | 2,944 | 3,458 | backends, bailey policy, broker, spawn, path containment (`sandbox/mod.rs:1-2`) |
| `config/` | 3,564 | 2,552 | 2,793 | schema, hand-written validator, live reload, redaction (`config/mod.rs:1-7`) |
| `chat/` | 3,483 | 2,530 | 2,367 | serenity gateway, threads, outbox, rendering (`chat/mod.rs:1-2`) |
| `agent/` | 2,965 | 2,206 | 2,487 | ACP client to `kage rpc`, framing, delegation (`agent/mod.rs:1`) |
| `provider/` | 1,986 | 1,337 | 2,331 | usage windows, image generation, vision, model store (`provider/mod.rs:1-2`) |
| `serve.rs` | 1,682 | 1,310 | 458 | start order: config, lock, probe, sweep, broker, connect (`serve.rs:1-6`) |
| `web/` | 1,294 | 1,033 | 1,268 | local browser interface (`web/mod.rs:1-2`) |
| `daemon.rs` | 964 | 721 | 2,233 | wiring, backend choice, startup report (`daemon.rs:1-5`) |
| `issues/` | 903 | 700 | 744 | GitHub issues as a chat surface (`issues/mod.rs:1-6`) |
| `admission/` | 496 | 373 | 331 | caps on turns and sessions, provider backoff (`admission/scheduler.rs:1-10`) |
| `cli/` | 482 | 392 | 621 | `errand threads` operator commands (`cli/threads.rs:1-6`) |
| `memory/` | 479 | 339 | 429 | facts store in SQLite (`memory/store.rs:1-6`) |
| `log.rs`, `main.rs`, `lock.rs` | 698 | 495 | 333 | ASCII log, command line, one daemon per state dir |
| `docs.rs`, `test_util.rs` | 0 | 0 | 893 | reference pages checked against code; test helper |
| **total** | **33,768** | **24,690** | **31,856** | |

Outside `src/`: `tests/` 408 lines, `web/` 30 files (Svelte, 5,097 lines
without `bun.lock`), `docs/` 22 files, `skills/` 4 files, `build.rs`. errand-box
for comparison: 25 files, 8,689 non-test raw, 1,865 test raw, plus
`tests/shim.rs` 372. errand's Rust is 6.0 times errand-box's
(66,032 against 10,926 lines, `src/` and `tests/` together).

Persistence is a JSON index, `threads.json`, written by rename with a
directory flush (`session/registry.rs:1-11, 23, 218-227`). SQLite is used only
for memory (`memory/store.rs:3-6`).

**Is the ACP client kage-specific? Verdict: plausible that the wire core
works with any ACP agent; the launch and several features are kage's.**
Generic: newline framing on bytes (`agent/framing.rs:1-9`), `initialize`,
`session/new`, `session/prompt`, `session/cancel` (`agent/protocol.rs:133-202`),
and a turn that ends on the prompt's answer, not on a kage event
(`agent/client.rs:1393-1418`). Only `session/request_permission` is served;
every other agent request gets method-not-found (`agent/client.rs:1072-1073`).
kage-specific: the command `kage rpc --yolo -m --system`
(`sandbox/backend.rs:168, 180-200`), kage's config file
(`sandbox/agent_config.rs:1-7`), `_kage/session/compact`
(`agent/protocol.rs:217-225`), `_kage/turn`, `_kage/compaction` and
`_kage/notice` (`agent/protocol.rs:512-531`), plan review and token totals
under `_meta.kage` (`agent/protocol.rs:546-548, 670`). `session/resume`, with a
fallback to `session/new` (`agent/client.rs:517-533`), and the `delivery`
field (`agent/protocol.rs:177-193`) are not in the ACP methods I know of;
unverified. kage stays a dependency under either option.

## 2. What errand-box uses

Groups are sums of the files named; the script is in "How to re-check". The
groups add to the totals above exactly.

| group | files | non-test raw | code | test raw | class |
| --- | --- | --- | --- | --- | --- |
| chat transport | `chat/{gateway,threads,inbound,outbox,mod}` | 1,913 | 1,373 | 1,458 | needed |
| chat rendering | `chat/{render,chars}` | 1,032 | 756 | 594 | needed |
| chat slash commands, diffs | `chat/{commands,diff}` | 538 | 401 | 315 | needed |
| session core | `session/manager`, `session/session.rs`, `session/session/{answering,files}`, `session/mod.rs` | 5,002 | 3,945 | 6,175 | needed |
| session records | `session/{registry,record,ids,projects}` | 789 | 492 | 729 | needed |
| session views | `session/{views,event,transcript,redacted}` | 1,195 | 785 | 1,161 | needed |
| session commands | `session/{commands,model,rules,disk,files,attachments}` | 1,498 | 1,025 | 1,069 | needed |
| PR flow | `session/pr.rs`, `session/github`, `session/session/pulls` | 1,360 | 974 | 1,289 | needed |
| ACP client | `agent/{client,protocol,framing,mod}` | 2,390 | 1,801 | 1,843 | needed |
| broker | `sandbox/broker/` | 1,100 | 805 | 871 | needed |
| kage config | `sandbox/agent_config` | 518 | 380 | 826 | needed |
| sandbox seam | `sandbox/{mod,backend,spawn,paths}` | 1,013 | 632 | 264 | needed |
| config | `config/` | 3,564 | 2,552 | 2,793 | needed |
| daemon | `daemon`, `serve`, `main`, `lock`, `log` | 3,344 | 2,526 | 3,024 | needed |
| admission | `admission/` | 496 | 373 | 331 | needed |
| bailey backend | `sandbox/{bailey,policy,runtime}` | 1,232 | 844 | 1,126 | replace |
| podman backend | `sandbox/podman` | 355 | 283 | 371 | drop |
| web interface | `web/` | 1,294 | 1,033 | 1,268 | drop |
| GitHub issues surface | `issues/` | 903 | 700 | 744 | drop |
| operator CLI | `cli/` | 482 | 392 | 621 | drop, optional |
| providers: usage, images, vision | `provider/` | 1,986 | 1,337 | 2,331 | keep first; woven in |
| memory and recall | `memory/`, `session/session/memory`, `session/recalling` | 970 | 714 | 605 | keep first; woven in |
| delegation | `agent/{delegate,delegation,requests}`, `session/delegating` | 794 | 567 | 1,155 | keep first; woven in |

Needed: 25,752 non-test raw, 18,820 code. Coupling decides what can be
dropped cheaply. Outside their own module, `web` is referenced from 1 file
(`serve.rs`), `issues` from 2, `sandbox::podman` from 1 (`daemon.rs`), and
`provider` from 9 and `memory` from 6. So `web/`, `issues/`, `cli/` and
`podman` come out cleanly; `provider`, `memory` and delegation do not. The
images feature in `provider/images.rs` came from upstream issue 34 and pull
request 35, opened from the account that owns this repository, so I class it
as wanted. `bailey.rs` also holds the broker glue that a new backend reuses
(`EGRESS_MAP_ADDRESS`, `provider_prefix`, `ProviderBrokering`,
`brokered_env`: `sandbox/bailey.rs:176-256`), so it is replaced last.

## 3. Vendor-and-patch plan

**Shape.** Copy errand at `715e9b0b777a` into one directory with its two
licence files; record the base commit. Exclude `AGENTS.md` and
`skills/*/SKILL.md`, which are agent instruction files
([`vendoring.md`](../../methodology/vendoring.md), "What is never vendored").
Make it a workspace member beside errand-box. Delete `web/` (and the `bun`
step), `issues/`, `podman`; keep the rest until the native backend passes.

| patch | where | estimate |
| --- | --- | --- |
| lints on errand-box's toolchain | 66 test sites (60 `assert_is_empty`, 6 unfulfilled `float_cmp` expectations), measured | 66 to 130 lines |
| tests that find the repository root by `.git` | `src/test_util.rs:7-21`, `tests/ascii.rs:29`, `tests/docs.rs:23` | 10 to 30 |
| musl C compiler for ring, aws-lc-sys, libsqlite3-sys | `scripts/ci-local.sh` steps `musl` and `cross`; the release workflow | 10 to 40, plus a CI package |
| native incus backend, T-019 Part 2 with errand-box's restricted project | new `sandbox/incus.rs`; `config/schema.rs:16-21, 423-445`; `config/validate.rs:1594-1595` and its key checks; `sandbox/mod.rs:99-104, 245-293`; `daemon.rs:76-102, 110-119`; `config.schema.json`, `docs/reference` | 900 to 1,350 non-test, 600 to 1,000 test |
| limits, T-019 Part 1 | read `sandbox.memory`, `cpus`, `pids` (`config/schema.rs:438-442`) in the backend; the bailey policy writer (`sandbox/policy.rs:292-311`) is not on this path | 10 to 30 |
| wording, T-016 | moot: the new backend writes its own notes. Under bailey: `sandbox/bailey.rs:435, 452-454` | 0, or 2 to 6 |
| unix-socket broker link, T-020 | `handle` takes a `TcpStream` (`sandbox/broker/server.rs:149`); make it generic and accept on a per-session socket, or keep TCP and relay per session in the backend | 60 to 150 |
| operator secrets, T-021 | a config key, its validation, read-only disk devices in the backend; the GitHub token default at `session/session.rs:2389-2399` becomes patchable | 100 to 250 |
| raw-TCP allowlist, T-022 | the broker already tunnels CONNECT for any host in `allow` times any port in `egressPorts` (`sandbox/broker/server.rs:186-196`); make it host:port pairs, and decide the internal-address rule (`sandbox/broker/mod.rs:428-458`) | 80 to 200 |
| upstream issue 48, confirmed by reading | prefix match with no boundary (`sandbox/broker/mod.rs:500-508`), then `upstream + rest` (`:547-549`); `provider_refusal` judges only whether the host is internal (`:592-654`) | 10 to 30 |
| upstream issue 52 and the rest of 42 to 55 | `--shared` clone (`session/pr.rs:628-645`); one nonce per provider for all sessions (`serve.rs:419-436, 487-495`); others unverified | unknown |

Total, estimate: 1,300 to 2,200 non-test lines, plus tests. T-020, T-021 and
T-022 are errand-box work today too; vendoring moves their errand-side half
into errand and lets the broker do per-session work it cannot do now.

**What errand-box drops: yes, all of the bailey impersonation.** `src/shim.rs`
(323 non-test, 118 test), `src/trust.rs` (114, 57), `src/policy.rs` (428,
141), `tests/shim.rs` (372) and the `bailey` dispatch in `src/main.rs`;
`docs/bailey-contract.md` (687 lines) moves to history. `src/session.rs` (555,
203) stays: `src/maint.rs` and `src/ops.rs` use its names and keys, and it is
the basis of the new backend. A thinner variant, estimated at 250 to 400 lines
in errand, hands a typed launch spec to `errand-box` and keeps
`src/session.rs` whole; it keeps a process boundary but no policy file.

## 4. Rewrite plan

| feature | errand today, non-test raw | rewrite, estimate |
| --- | --- | --- |
| Discord gateway, threads, inbound rules, outbox, rendering | 2,945 | 1,200 to 2,000 |
| session lifecycle, records, views, commands, attachments | 8,484 | 2,500 to 4,000 |
| ACP client to kage | 2,390 | 900 to 1,500 |
| broker with nonce swap and allowlist, kage config | 1,618 | 900 to 1,400 |
| PR push | 1,360 | 500 to 900 |
| config, with serde refusing unknown keys | 3,564 | 500 to 1,000 |
| daemon wiring, log, lock, admission | 3,840 | 800 to 1,500 |
| sandbox seam, path containment | 1,013 | 300 to 600 |
| total | 25,214 (25,752 with slash commands) | 7,600 to 12,900 |

The incus side is not in the table: errand-box has it. A rewrite also needs a
test volume like errand's, which runs about 0.94 test lines per non-test line.

Behaviour a rewrite would have to find again, each paid for once:

| behaviour | where errand has it | paid for by |
| --- | --- | --- |
| judge the address dialled, not the name; v4 special ranges; v6 by value, including v4-mapped and NAT64 | `sandbox/broker/mod.rs:230-391, 411-458`; `server.rs:201-227` | issue 9; `e2808ca285ad`, `d2898006478f` |
| tell "did not resolve" from "internal" | `sandbox/broker/mod.rs:462-470` | issue 12; `5d61aafbbe61` |
| bounded lookup and dial, all addresses, one budget | `sandbox/broker/mod.rs:396-409`; `server.rs:255-290` | issue 17; `6ec654af1ca7`, `ace9ba71f513` |
| plain `http://` through the proxy | `sandbox/broker/mod.rs:110`; `server.rs:165-173` | issue 18; `ffe4c72d6646` |
| constant-time nonce compare, either key header | `sandbox/broker/mod.rs:196-228` | (no issue found) |
| host git: no hooks, no fsmonitor, no helper, no `ext::`, no signature programs, no system or global config | `session/pr.rs:170-218` | issues 2 and 6; `5811ab256754` |
| push and read from a bare clone, never the session's tree; the git dir must be inside the project | `session/pr.rs:246, 608-700, 707` | issues 2 and 7; `e2808ca285ad` |
| token only through a credential helper reading the environment | `session/pr.rs:161-167, 661-662` | (design) |
| daemon file access with `openat2` beneath, no symlinks | `sandbox/paths.rs:120-147` | `088ee6a0c923`, `6c4cafab783e` |
| redaction, structural and by value | `config/redact.rs:1-17, 80-89`; `session/redacted.rs:31` | (design; issue 53 says it is incomplete) |
| per-thread ordered outbox that survives a disconnect, bounded at 500 | `chat/outbox.rs:1-35` | (design) |
| reconnect give-up after 10 failed attempts; serenity keeps the socket | `chat/gateway.rs:8-36` | issue 4, inferred: it closed with no comment |
| caps on turns and sessions, pause on provider backoff | `admission/scheduler.rs:1-10, 137` | (design) |
| ACP framing on bytes, LF only, 8 MiB ceiling | `agent/framing.rs:1-12, 31-35` | (design: U+2028) |
| a turn ends on the prompt's answer | `agent/client.rs:1393-1418` | (kage emits a turn end per iteration) |
| 2,000-character messages split by code point, fences reopened | `chat/render.rs:1-39` | (design) |

## 5. Build and dependency cost

Measured on one host: Linux 6.18 x86_64, 4 vCPUs, 15 GB, `CARGO_BUILD_JOBS=4`,
crates already downloaded, a copy of the tree made by `git archive`.

| measure | errand | errand-box |
| --- | --- | --- |
| `Cargo.toml` dependencies | 15 (`Cargo.toml:19-34`) | 4 (`Cargo.toml:15-19`) |
| `Cargo.lock` packages | 273 | 22 |
| unique normal and build crates, gnu target | 198 | not measured |
| C built from source | ring, aws-lc-sys (cmake), libsqlite3-sys (bundled) | none |
| toolchain | 1.98.0 (`rust-toolchain.toml`), `rust-version = "1.98"` | 1.99.1, `rust-version = "1.99"` |
| lints | `unsafe_code = "forbid"`, `warnings = "deny"`, `clippy::pedantic = "deny"` (`Cargo.toml:36-41`) | none in `Cargo.toml`; the gate runs clippy with `-D warnings` |
| `cargo check --locked`, cold, 1.99.1 | pass, 85 s | pass, 6 s |
| `cargo clippy --locked --all-targets`, 1.99.1 | **fail, 66 errors, all in test code**, 19 s | not run |
| the same, 1.98.0 (control) | pass, 93 s | not run |
| `cargo clippy --locked`, non-test, 1.99.1 | pass | not run |
| `cargo fmt --check`, 1.99.1 | pass | not run |
| `cargo test --locked`, 1.98.0 | 1,119 pass, 5 fail, 1 ignored, 133 s | not run |
| release, x86_64 musl, 1.99.1, no musl C compiler | fail in 6 s: ring needs `x86_64-linux-musl-gcc` | no crate with C sources; the gate's `musl` step (`scripts/ci-local.sh:66-76`) |
| the same with clang as the C compiler | fail at link after 244 s: `open64`, `stat64` and others from libsqlite3-sys, `__isoc23_strtol` from aws-lc-sys | n/a |

The 5 test failures are all `could not find the repository root`
(`src/test_util.rs:15`): the copy has no `.git`. Vendored inside errand-box the
same helper finds errand-box's root instead, so the patch is needed either
way. errand's own release builds on a native runner per architecture with
`musl-tools` and `CC=musl-gcc` (`.github/workflows/artifacts.yml:22-27, 31-34,
50-53`), and builds the web bundle with bun first (`:41-48`); errand-box's
`cross` step builds aarch64 from x86_64 with rust-lld and no C compiler
(`scripts/ci-local.sh:77-87`). Vendoring breaks that step unless a musl cross
C compiler is added.

Where the crates come from: in errand's graph, `serenity` reaches 163 of the
198 crates. Resolved alone (serenity 0.12.5 with errand's features, tokio,
serde_json), it is 194 lock entries and 150 crates, with ring and without
aws-lc-sys or SQLite. aws-lc-sys comes in through rustls 0.23 under reqwest
(`cargo tree -i aws-lc-rs`); libsqlite3-sys through rusqlite for memory.

## 6. Churn

Full history read: 230 commits from `aa775a9db96a` (2026-09-02) to
`715e9b0b777a` (2026-10-23); `origin/main` had not moved at fetch time. The
Rust port is `9c8a91d66505` (2026-09-18); earlier commits are TypeScript.

| ISO week | W38 | W39 | W40 | W41 | W42 | W43 |
| --- | --- | --- | --- | --- | --- | --- |
| commits, from the port | 22 | 41 | 21 | 4 | 6 | 7 |

Non-test raw lines in `src/`: 25,523 at the port, 29,839 at v0.2.2, 32,223
at v0.3.0, 32,352 at v0.3.1, 33,768 at `715e9b0b777a`.

| range (tags reachable) | commits | `git diff --shortstat -- src` |
| --- | --- | --- |
| port to v0.2.2 | 61 | 90 files, +9,370, -1,108 |
| v0.2.2 to v0.2.3 | 6 | 30 files, +2,466, -325 |
| v0.2.3 to v0.2.4 | 6 | 18 files, +650, -110 |
| v0.2.4 to v0.2.5 | 2 | 4 files, +148, -5 |
| v0.2.5 to v0.3.0 | 12 | 37 files, +5,270, -1,619 |
| v0.3.0 to v0.3.1 | 6 | 19 files, +418, -110 |
| v0.3.1 to `715e9b0b777a` | 7 | 37 files, +4,137, -248 |

Commits since the port, and since v0.3.1, on the files a patch touches:
`session/session.rs` 30 and 4; `config/schema.rs` 27 and 5; `serve.rs` 24 and
2; `config/validate.rs` 15 and 2; `sandbox/bailey.rs` 13 and 0; `daemon.rs` 11
and 3; `session/manager.rs` 10 and 1; `agent/client.rs` 9 and 1;
`sandbox/policy.rs` 7 and 0; `sandbox/broker/mod.rs` 6 and 0;
`session/pr.rs` 5 and 0; `sandbox/mod.rs` 4 and 0. 114 paths under `src/`
changed after the port; `src/` tracks 173 files. The agent itself changed in
that window, from pi to kage ACP (`7402932e6189`, `c4110abf80c0`).

The cost this sets: the schema, validator and daemon wiring that a backend
must touch change in most releases. Upstream issue 41 records an accepted
`incus` backend whose rules differ from errand-box's design: the errand
account in `incus-admin`, one instance per launch, no proxy devices
([`../../incus-backend.md`](../../incus-backend.md) section 10). When it
ships, its `SandboxBackend::Incus`, `Backend::Incus` and `sandbox/incus.rs`
meet ours at the same names.

## 7. Licence

errand is `MIT OR Apache-2.0` (`Cargo.toml:9`), with `LICENSE-MIT`
(copyright 2026 Rabindra Dhakal) and `LICENSE-APACHE`; there is no `NOTICE`
file. errand-box is 0BSD (`Cargo.toml:6`, `LICENSE`). This is a reading of the
licence texts, not legal advice.

- The `OR` lets errand-box take errand under MIT: keep the copyright and
  permission notice with the vendored source and with every release binary.
  errand-box releases bare binaries and `SHA256SUMS`
  (`.github/workflows/release.yml:4`), so a release gains a notice file.
- Under Apache-2.0, each modified file needs a changed-file notice (section
  4(b)); the derived patch series of `vendoring.md` serves that.
- 0BSD does not reach errand's files: the tree becomes mixed-licence by
  directory, and the binary carries 273 lock entries' crate licences, not 22.

## 8. What breaks each option

**Vendoring:**

- **Inherited defects.** Upstream issues 42 to 55, opened 2026-10-24, report
  defects against this commit, and `vendoring.md` makes each confirmed one a
  same-session fix. Issue 48 is real as read, for a provider whose base URL
  has no path: `/provider/<name>.attacker.example/...` passes the
  `starts_with` match and is forwarded to
  `https://<provider host>.attacker.example/...` with the real key.
- **Reconciliation.** The touched files churn most; the name collision is
  certain if errand's own backend ships.
- **Toolchain drift.** The deny lints turned one toolchain step into 66 errors.
- **Build and size.** A musl C compiler for both architectures; 85 s against
  6 s for a cold check; 66,032 lines of Rust nobody here wrote.
- **Discord drift.** serenity 0.12 against Discord's API. Shared with a rewrite.

**Rewriting:**

- **The fixes in section 4's table**, each re-found or ported by reading.
  Issues 48 and 52 show the same families recurring after their first fix.
- **kage drift.** errand tracked kage's resume answer (`28f78e60c44e`) and
  token totals (`f47a0c31f99d`); a rewrite takes that on alone.
- **Discord surface.** Message and thread-name limits, attachments, stickers,
  slash commands, reconnect.
- **Parity.** 1,119 unit tests and the images feature to rebuild.

## Candidates

| | candidate | refuted if | found | verdict |
| --- | --- | --- | --- | --- |
| A | vendor and track releases | the touched files churn, or a known upstream change collides | both hold (section 6) | refuted as routine; reconcile only by reading |
| B | vendor as a fork, drop the edges, patch | errand-box needs little of errand, or a rewrite avoids the build costs | needs 76%; serenity alone brings 150 crates and ring | plausible, recommended |
| C | rewrite on errand-box's Incus code | the needed set is small, or the stack shrinks | neither; estimated 7,600 to 12,900 lines | plausible, costlier |
| D | stay with the pinned errand and the bailey stand-in | errand-box can fix T-016, T-019 Part 1 and issue 48 from outside | it cannot: fixed text at `sandbox/bailey.rs:435, 452-454`; issue 48's match is at v0.3.1 `src/sandbox/broker/mod.rs:507` | refuted |

Flip conditions, against the three stated at the top: (1) 76% needed, not a
third; (2) refuted for any serenity stack; (3) the operator ruled against
depending on errand's releases. If the operator wants errand's releases after
all, or if the first reconciliation by reading touches more patched files
than the backend itself, take C.

Second pass, looking only for what is not above: the web bundle's absence is
handled by `build.rs:9-12`, so dropping `web/` needs no build patch; the
`server.rs:3-4` comment says one broker per session while `serve.rs:654-660`
builds one per daemon, which is issue 49's subject.

## Corrections to pages in this tree

- `TODO/sandbox.md` T-019 cites `src/config/validate.rs:1527-1533` for both
  v0.3.1 and `main`. At `715e9b0b777a` the arms are at 1594-1595; 1530 is
  v0.3.1's.
- `references/QaidVoid__errand/PROVENANCE.md` gives the licence as Apache-2.0;
  `Cargo.toml:9` says `MIT OR Apache-2.0`. It also says depth 50: this session
  deepened the clone to its full 230 commits.

## How to re-check

Run against a `git archive` copy of each tree. Line counts, per file, then
summed per row of sections 1 and 2; `src/main.rs` and errand-box's files split
at their `#[cfg(test)] mod tests {` line:

```sh
find src -name '*.rs' | sort | while read -r f; do
  case "$f" in */tests.rs|*/tests/*|src/test_util.rs|src/docs.rs|src/config/example_test.rs) k=test ;; *) k=src ;; esac
  nb=$(grep -cv '^[[:space:]]*$' "$f"); cm=$(grep -c '^[[:space:]]*//' "$f")
  echo "$f $k $(wc -l < "$f") $nb $((nb - cm))"
done
```

Builds, each under `timeout 1500`, one at a time, `CARGO_BUILD_JOBS=4`:

```sh
cargo +1.99.1 check --locked
cargo +1.99.1 clippy --locked --all-targets     # 66 errors
cargo +1.98.0 clippy --locked --all-targets     # control: pass
cargo +1.98.0 test --locked                     # 1119 pass, 5 fail
cargo +1.99.1 build --locked --release --target x86_64-unknown-linux-musl
CC_x86_64_unknown_linux_musl=clang cargo +1.99.1 build --locked --release --target x86_64-unknown-linux-musl
cargo tree --locked -e normal,build --target x86_64-unknown-linux-gnu --prefix none -p serenity
```

Churn: `git log --format='%ad' --date=format:'%G-W%V' 9c8a91d66505^..HEAD`,
`git diff --shortstat vA vB -- src`, `git log --oneline 9c8a91d66505..HEAD -- <file>`.

## Provenance

| source | commit | depth reached | not reached |
| --- | --- | --- | --- |
| QaidVoid/errand, code | `715e9b0b777a`, 2026-10-23 | full history, 230 commits; every `src/` file's module comment; each cited file opened at its lines; built and tested from a `git archive` copy, which restores the deleted `AGENTS.md`, removed again before use | `web/` source, `docs/`, the TypeScript history, most of `session/session.rs` |
| QaidVoid/errand, tracker | fetched 2026-10-24 | issues 2, 6, 7, 9, 11, 12, 17, 18, 34, 45, 48, 49 and 52 skimmed for the code they explain | the rest; discussions were not fetched |
| errand-box | this session's working tree | `src/` counted; the pages in the brief read in full | live tests |
| QaidVoid/bailey, QaidVoid/kage | not opened | | kage's ACP extensions are known here only from errand's side |
