# Progress

Read this first. It has the baseline, what the last session did, and the
work order. It carries no history: every session rewrites it. The history
is the git log and the entries.

How this repository is worked on: [`RULES.md`](RULES.md). Every entry, one
line each: [`INDEX.md`](INDEX.md). Routing for an agent:
[`AGENTS.md`](../AGENTS.md). What the last session had in flight when it
stopped: [`RESUME.md`](RESUME.md). Its summary table:
[`SUMMARY.md`](SUMMARY.md).

---

## The operator's rulings of 2026-10-25

1. **Group the new issues by root cause**, not one entry per issue. The
   review of 2026-10-25 filed issues #43 to #59 with no entries; this session
   turned them into 11 entries (T-079 to T-089) that each name their issues,
   and folded #59 into T-032.
2. **The `lite` engine (issue #49) is a researched entry, non-blocking**
   (T-086). It is a second engine for hosts Incus cannot serve, and does not
   replace the settled Incus decision.
3. **The security findings are P1**: T-079, T-080, T-081.
4. **T-029's closing evidence is corrected** (issue #48), and T-087 adds a
   live provider-boundary check and a gate that requires an entry's recorded
   prove output. The T-029 fix itself stands.

## The operator's rulings of 2026-10-24

1. **errand is errand-box's hard fork, in `errand/`** (T-028), and a
   library errand-box runs: `errand-box serve`, `errand-box threads`, no
   `errand` program. A host runs errand-box, kage and Incus.
2. **The web interface leaves the fork** (T-034).
3. **discord-mock** (T-035) runs in CI and in-process tests, and on test
   hosts as an asset of the prerelease `artefacts`; it is no asset of a
   version.
4. **A release carries `THIRD-PARTY-NOTICES`**; `cross` takes zig.
5. **T-037 is approved**: back off errand's refusals rather than stop them.
6. **T-027 ran as a standalone agent's end-to-end test** on WSL2,
   ModelScope Studio and Daytona; it filed GitHub issues and an entry per
   issue, and built nothing.

---

## State

- **Last session:** 2026-10-25, attended, in a hosted container. It triaged
  the open issues into entries (the rulings above), implemented the security
  cluster from them, filed T-090 and began it (tasks 1 and 2 landed), then
  **provisioned this host (`errand-box setup` installed Incus 6.0.7 and the
  guest image) and ran the live suite**, which closed T-079 and caught a
  regression. It then **built and live-validated T-020** (T-090 task 3's gate):
  turned CI green, measured and prototyped the unix-socket broker link live, and
  implemented it in three slices (errand's configurable `brokerPort`; errand-box
  setting a fixed port; then the switch: the disk-share link, the guest
  `systemd-socket-proxyd` relay, and `restricted.devices.proxy=block`). The live
  suite proved the broker reached over the socket with proxy devices blocked and
  both guest relay ports carrying an HTTPS CONNECT; a boot race it exposed (env
  push before `/run/errand-box` existed) was fixed. T-020 done, E-61 closed.
  Worker restarts stopped incusd twice; `errand-box up` recovered it. All on
  `main`, pushed, each commit under the operator's identity.
- **Also last session, after the above:** a strictly-technical (ASD-STE100)
  wording pass, on the operator's word. Flagged or metaphorical vocabulary was
  neutralised across the security pages and the record (`f19d1a0`) and then the
  rest of the live tree (`7752b89`): `docs/research/sandhome.md`, the
  secret-check message, errand broker and config test data (example hosts are
  now `other.example` and `elsewhere.example`), two code comments, and a test
  variable (`victim` to `target`). A pre-existing shellcheck 0.9.0 false
  positive (SC2317 on `mock-session.sh`) that blocked a clean local gate was
  fixed (`fca6d33`). No behaviour changed and no entry counts moved; only the
  cited cfrs title keeps "Sealed Sandbox", and `docs/history/` was left
  untouched as the append-only record. The `gates` CI for `7752b89` (run
  40176641403) had every confirmable job green (both `musl`; gate through
  clippy); the final `test` and trivial downstream steps were unread behind a
  stale GitHub API cache, and a self check-in is set for 09:16 UTC to confirm it.
- **This host is now a working errand-box host.** `errand-box setup` finished
  with 0 failures (7 expected degradations: cgroup v1, no init, no AppArmor,
  dir pool, lxcfs 6, the two `/dev` modes). `sh tests/live/live.sh` runs here.
  So host-touching entries are no longer blocked for want of Incus; they are
  drivable on this host. errand itself stays stopped (no Discord creds given;
  `setup` ran without `--env/--creds`), so the `errand:` live checks need the
  operator's credentials (T-015), not a code change.
- **Tree:** `main`, on top of `57231da`, through `7752b89`, pushed to
  `origin main`.
- **Deployed:** [v0.1.1](https://github.com/Azathothas/errand-box/releases/tag/v0.1.1),
  the latest release, on `2ead3fd`. The prerelease
  [`artefacts`](https://github.com/Azathothas/errand-box/releases/tag/artefacts)
  holds the test assets, rebuilt from `main` by `artefacts.yml`. errand-box
  is installed nowhere.
- **CI:** on. `gates` runs on every push to `main`.

## Baseline, as measured this session

Measure again with the same commands before you change anything.

| check | result |
| --- | --- |
| `sh scripts/ci-local.sh --step gate` | 9 of 9, `check-todo` consistent, after each entry's commit and after T-090 was filed |
| `cargo test --locked -p errand --lib` | 1,106 pass, 1 ignored |
| `cargo test --locked -p errand-box --bins` | 115 pass |
| `cargo fmt` / `cargo clippy --all-targets` | clean on both crates (`-D warnings`, pedantic) |
| `sh scripts/ci-local.sh` (full) | the affected steps run green locally (gate, kill-by-name, shellcheck, fmt, clippy, test, errand-config); `musl` ran green on CI for `7752b89` (both arches). The old `shellcheck` 0.9.0 SC2317 false positive on `tests/e2e/mock-session.sh` is now fixed: `fca6d33` extends the file's existing SC2329 suppression to SC2317 |
| `errand-box setup` (this host) | 0 failed, 30 changed, 7 degraded (all host-inherent); Incus 6.0.7, golden image `eb-golden`, `bailey doctor as errand: ok` |
| `sh tests/live/live.sh` (this host) | every sandbox check PASS: `probe`, `session` (uid, groups, no NIC, direct egress blocked, broker via proxy device, limits, instance stop, lease), `acp`, `inside` (docker, docker build, `sudo podman`, rootless podman E-30, FUSE, cpu/mem limits), `signals` (SIGTERM, SIGKILL), `reap`, `up` (restart; second up 0 changed), `maint` (after the fix below). The only non-expected FAIL was `maint --once` (the T-063 regression, fixed and re-verified `PASS`). The `errand:` checks FAIL only because errand is unconfigured (no Discord creds, T-015) |

The build host: ubuntu 24.04, kernel 6.18, x86_64, 4 CPUs, 16 GiB, cgroup v1,
no init, no AppArmor, ~15 GiB writable. Now provisioned with Incus 6.0.7 (dir
pool, no btrfs module). The agent proxy is its only egress, and incusd pulls
images through it (E-13).

**90 entries: 24 done, 8 partial, 1 blocked, 57 open.**

---

## What this session did

After the triage (above), implemented the security cluster. Each fix carries a
regression test that fails without it (the TOCTOU and panic ones were watched
to fail), and a class-prevention artifact: a `forbidden-patterns.md` row or a
mechanical check. Committed on `main` under the operator's identity.

- **T-088** (done): `creds.env` and `errand-box.env` are git-ignored and the
  secret check scans for them tracked.
- **T-080** (done): upstream tags are validated and `freshness.sh` no longer
  builds an `sh -c` string from one.
- **T-063** (done): the CLI refuses an unknown option, so `gc --dryrun` (a
  typo for `--dry-run`) cannot fall through to a real delete.
- **T-079** (partial, live pending): root opens lock and temp paths under
  errand-owned directories with `O_NOFOLLOW` + `create_new`, and chowns only a
  file it created. Unit-proven; the one step left is a live `up` run on a host.
- **T-089** (done): `start_provider` returns `Result` instead of `.expect`, so
  a denied loopback bind declines one session rather than aborting the daemon
  into T-037's restart loop.
- **T-081** (done): the pull-request push clones a snapshot of the git
  directory (`cp -a`, checked with `refuse_borrowed`), so the check and the
  clone read one immutable copy, closing T-030's borrow-swap window.
- **T-032** (done): the broker's SSRF address arms (Azure WireServer, 6to4,
  documentation, Teredo, site-local), no redirects on the provider client, the
  real credential scrubbed from the streamed answer and its headers, and a
  prefixed nonce scrubbed by shape.
- **T-033** (done): bounded file reads (`read_beneath` ceiling, `!file` read
  through the upload limit) and an upload's bytes scrubbed like every other
  report.
- **T-083** (partial): distinct host/port reasons in the 403 refusal body, and
  `ERRAND_EGRESS_ALLOW_EXTRA` extends the allowlist rather than replacing it.

Seven `forbidden-patterns.md` rows added for the classes behind these: the
TOCTOU check-then-use on a live path, a `.expect` on the runtime under
`panic = abort`, following a redirect with a credential, rewriting a proxied
body while keeping its `Content-Length`, scrubbing a stream with no carry
across the chunk boundary, an unbounded or metadata-trusting file read, and a
reported raw-bytes field that skips the redactor.

Then **authored T-090** (a design entry, not implemented, per authoring.md):
a virtual network layer. Instead of bolting an upstream HTTP proxy onto the
broker (T-082) and a per-port raw-TCP relay (T-022), one gated egress path, a
smoltcp userspace stack plus the broker's gating factored into a shared policy
gate, fed by a per-backend link device: a TUN over T-020's unix socket in the
Incus backend, and the operator's `cfrs` `LD_PRELOAD` interposition in the lite
backend (no root, no namespaces). Enforcement stays the backend's own isolation
(the netns; the sandbox's `AF_INET` block). **T-090 supersedes T-022 and T-082**,
whose rulings are kept verbatim under a pointer to it. The operator ruled this
direction over several questions this session.

Then, on the operator's word, **implemented T-090 task 1's first slice** (the
host-free part): a `gate` module (`errand/src/sandbox/broker/gate.rs`) now holds
the admission verdict (`target_admission`, lifted out of the transport's
`admit`) and the provider routing (`route_for`, data-driven), with verdict
parity (the 38 broker tests pass unchanged, plus a direct gate unit test). The
address and host predicates stay in the parent, already transport-free.

Then the operator corrected a wrong assumption: this host was never incapable,
only unprovisioned. **Provisioned it** (`errand-box setup`: Incus 6.0.7 + the
guest image, 0 failures) and **ran `sh tests/live/live.sh`**. Every sandbox
check passed. The live run **closed T-079** (its `up` re-exercise) and **caught
a T-063 regression**: the unknown-option guard whitelisted only `--loop` for
`maint`, so `maint --once` (a documented flag) was refused. Fixed (maint allows
`--once` and `--loop`), the test gained a both-sides check, re-verified `PASS
maint`, and a `forbidden-patterns.md` row names the class (an eighth row). The
`errand:` live checks fail only for want of Discord creds (T-015), not code.

Finally, on the operator's word, a strictly-technical (ASD-STE100) wording pass,
record- and docs-only in effect. Flagged or metaphorical vocabulary was
neutralised across the security pages and the record (`f19d1a0`), then the rest
of the live tree (`7752b89`), and a pre-existing shellcheck false positive was
fixed (`fca6d33`). No behaviour and no entry counts changed; `docs/history/` was
left untouched as the append-only record. The "Also last session" note under
State has the files and the CI state.

## What is in progress

Nothing mid-edit. T-090 is the open egress work; its task 1 gate module landed,
and now that this host is provisioned, the rest is drivable here:

- **T-090** (the virtual network layer, which subsumes T-082 and T-022):
  partial. Task 1's gate extraction and task 2's smoltcp stack + `LinkDevice`
  seam both landed (a loopback TCP round-trip test passes), and task 3's gate,
  **T-020, is now done** (below). Next is task 3 itself (the Incus TUN link),
  which rides T-020's unix socket; then tasks 4 to 8 (DNS + resolve-then-route,
  the provider endpoint, raw TCP, the upstream hop, the cfrs lite link); 3 to 7
  run on this host, 8 wants the isolated sandbox.
- **T-020** (the unix-socket broker link): **done this session**, closing E-61.
  The operator chose a fixed broker port; it landed in three slices (errand's
  configurable `brokerPort`, errand-box setting it, then the switch: the
  disk-share link, the guest `systemd-socket-proxyd` relay, and
  `restricted.devices.proxy=block`). Live-validated: the broker is reached over
  the socket with proxy devices blocked, both guest relay ports carry an HTTPS
  CONNECT to real registries, and rootless podman pulled a full image. See the
  entry's "Live-validated" section for the two caveats (the `errand:` and
  image-pull live checks, neither the link).
- **The rest of the host cluster** (T-021, T-024, T-025, T-019's native backend,
  and the P1 host failures of T-027's run): drivable on this host now. Caveat:
  this host has no AppArmor, so T-024/T-025's
  `apparmor_restrict_unprivileged_userns` case is not reproducible here (the live
  `inside` check confirmed rootless podman works where that restriction is
  absent).

---

## Start here next session

The host is provisioned and the live suite runs here, so host-touching work is
no longer blocked.

First, two carry-overs from last session: confirm the `gates` CI for `7752b89`
(run 40176641403) finished `success` (the 09:16 UTC self check-in may already
have noted it in chat), and see Open question 4 on whether to neutralise
`docs/history/` too. Neither blocks the work below.

1. **T-090 task 3**, the Incus TUN link, now that T-020's socket link is done.
   The guest boot step makes a TUN, routes default via it, and the guest-side
   relay pumps IP frames over T-020's same unix socket into the smoltcp stack
   (task 2); errand-box terminates the flow at the gate (task 1). The hard part
   is the async smoltcp integration (open work even in cfrs). Live-testable here.
   Reuse T-020's proven link (the fixed broker port, the disk-share, the guest
   relay pattern) and E-71 (mount on the rootfs, not `/run`). Then tasks 4 to 7
   live here, task 8 in the isolated sandbox. This closes T-022 and T-082.
2. **The rest of the host cluster, on this Incus host:** T-024, T-021, T-025,
   and T-019's native backend, which retires the `bailey` stand-in. The P1 host
   failures of T-027's run and the capability layer under them (T-040, T-041,
   T-046, T-038, T-039, T-042, T-084) also need a host.
3. **Then** the remaining P2/P3 entries of T-027's run, smallest first
   (T-051, T-047, T-048, T-058, T-059, T-060, T-061, T-062, T-064, T-065,
   T-045, T-052, T-053, T-055, T-057, T-049, T-054, T-066), T-083's open items
   (which fold into T-090's gate), T-085, T-086, and T-034's podman/`issues`
   removal.

---

## Open questions for the operator

1. **Revoke the Daytona token** that was given to the session of
   2026-10-23: it is in a session transcript.
2. **T-007's setting and secret**: the commands are in
   [`upkeep.md`](upkeep.md), T-007; they need your `gh` CLI and a token you
   create.
3. **The credentials T-015 lists**, for a real Discord application. T-027's
   run tested a real model turn through a gateway the operator named, and no
   real chat service.
4. **Whether to neutralise `docs/history/` too.** Last session made the live
   tree strictly technical (ASD-STE100) but left the dated research captures in
   `docs/history/` as they are, because that tree is append-only. They still
   hold terms like `attacker.example` and `evil.*`. Say if you want them
   neutralised as a sanctioned exception, or kept as the immutable record.

---

## Settled, and not to be raised again

See [`RULES.md`](RULES.md) section 4: Incus system containers; errand as
errand-box's hard fork; root at setup; s6 from the distribution. The `lite`
engine (T-086) is a second engine for hosts Incus cannot serve, not a
change to the Incus decision.
