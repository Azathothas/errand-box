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
  the open issues into entries (the rulings above), then implemented the
  security cluster from them: eight entries fixed, one carried partway, on
  `main`, pushed to `origin main` on the operator's word, each commit under
  the operator's identity.
- **Tree:** `main`, on top of `57231da`, through `47be7f1`, pushed to
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
| `sh scripts/ci-local.sh` (full) | not run to completion this session. The `musl` and `cross` steps were not re-run: no build-config changed, and the code changes cross-compile (std, `futures-util`, `libc` already in use). The `shellcheck` 0.9.0 SC2317 artifact on `tests/e2e/mock-session.sh:67-69` still stands (the file disables SC2329, the newer name; CI's newer shellcheck is green), and it is not this session's |
| live tests | not run: this container has no Incus, so no entry that touches a host could be driven |

The build host: a hosted container, kernel 6.18, x86_64, 4 CPUs, 16 GiB. No
Incus, no proxy.

**90 entries: 22 done, 9 partial, 1 blocked, 58 open.**

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
backend (no root, no namespaces). Enforcement stays the backend's own seal (the
netns; the sandbox's `AF_INET` block). **T-090 supersedes T-022 and T-082**,
whose rulings are kept verbatim under a pointer to it. The operator ruled this
direction over several questions this session.

Then, on the operator's word, **implemented T-090 task 1's first slice** (the
host-free part): a `gate` module (`errand/src/sandbox/broker/gate.rs`) now holds
the admission verdict (`target_admission`, lifted out of the transport's
`admit`) and the provider routing (`route_for`, data-driven), with verdict
parity (the 38 broker tests pass unchanged, plus a direct gate unit test). The
address and host predicates stay in the parent, already transport-free.

## What is in progress

Nothing mid-edit. The egress work is now T-090, whose task 1 is partly done
(the gate module above); its remaining stages need an Incus host or the sealed
sandbox. The rest of the open security and host work has a real blocker, not an
oversight:

- **T-090** (the virtual network layer, which subsumes T-082 and T-022):
  partial. Task 1's gate extraction landed this session; tasks 2 to 8 (the
  smoltcp stack, the Incus TUN link, the live egress checks, the cfrs lite link)
  need a host or the sealed sandbox.
- **The host cluster** (T-020, T-021, T-024, T-025, T-019's native backend, and
  the P1 host failures of T-027's run): needs an Incus host and kernel features
  (AppArmor, user namespaces, rootless podman) this container does not have.

---

## Start here next session

The host-free security cluster is done; what is left needs a host or the sealed
sandbox.

1. **Close T-079:** one `sh tests/live/live.sh up` on a host with Incus; no
   code work remains, only the live re-exercise of `up`'s `write_if_changed`.
2. **T-090**, the virtual network layer, staged as the entry lays out. Task 1's
   first slice (the `gate` module: admission verdict + provider routing, with
   verdict parity) landed this session; finish task 1 by gathering the predicates
   and shaping the flow-level API, then task 2 (the smoltcp stack, host-free and
   unit-testable). Tasks 3 to 7 need an Incus host, task 8 the sealed sandbox.
   This closes T-022 and T-082 as it lands.
3. **The host cluster, on an Incus host:** T-020 (the broker link that needs
   no proxy devices, which T-090's Incus link rides), then T-024, T-021, T-025,
   and T-019's native backend, which retires the `bailey` stand-in. The P1 host
   failures of T-027's run and the capability layer under them (T-040, T-041,
   T-046, T-038, T-039, T-042, T-084) also need a host.
4. **Then** the remaining P2/P3 entries of T-027's run, smallest first
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

---

## Settled, and not to be raised again

See [`RULES.md`](RULES.md) section 4: Incus system containers; errand as
errand-box's hard fork; root at setup; s6 from the distribution. The `lite`
engine (T-086) is a second engine for hosts Incus cannot serve, not a
change to the Incus decision.
