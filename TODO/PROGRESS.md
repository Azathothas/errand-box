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

- **Last session:** the triage of 2026-10-25, attended, in a hosted
  container. It read every open issue, verified the 17 with no entry against
  the code, and filed 11 entries; it changed only `TODO/`, on `main`. It
  built nothing and changed no code.
- **Tree:** the triage commit on `main`, on top of `57231da`, pushed to
  `origin main` on the operator's word.
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
| `sh scripts/ci-local.sh --step gate` | 9 of 9, and `check-todo` reports 89 entries consistent, on the triage tree. The step covers the only checks a `TODO/`-only change can move (docs, markers, one-home, placeholders, control-bytes, changelog, attribution, secrets) |
| `sh scripts/ci-local.sh` | every step passed but `shellcheck`: gate, kill-by-name, fmt, clippy, test, discord-contract, pins, errand-config, musl and cross all exit 0 (cross with zig 0.16 from pip, qemu-aarch64-static and a `binfmt_misc` handler installed this session). `shellcheck` exits 1 on this host's shellcheck 0.9.0, which raises SC2317 on the EXIT-trap handler `cleanup` in `tests/e2e/mock-session.sh:67-69`; the file disables SC2329, newer shellcheck's name for the same false positive, but not SC2317, so CI's newer shellcheck is green. No code changed this session, so this is a pre-existing version artifact, not a defect in the tree |
| `cargo test --locked --workspace` | run this session inside `ci-local` (the `test` step, 147 s): every test passed, cross included under qemu-aarch64. The last full count was 1,243 pass, 1 ignored (Rust 1.99.1) |
| live tests | not run this session (no host change). Last run 54 of 54 on `9b27e60` ([output](../tests/live/2026-10-24-errand-library.txt)) |

The build host: a hosted container, kernel 6.18, x86_64, 4 CPUs, 16 GiB.

**89 entries: 15 done, 6 partial, 1 blocked, 67 open.**

---

## What this session did

- **Read all 58 open issues** (#2 to #59) in full, from a read-only fetch of
  the tracker.
- **Verified the 17 issues with no entry** (#43 to #59) against the tree at
  `57231da` and the errand fork at `715e9b0`: each claim re-derived at file
  and line. All confirmed, except #46 and #49, which are a design and a
  timing window the reporter reproduced in a shell, not against live errand.
- **Filed 11 entries, grouped by root cause** (the operator's ruling):
  T-079 (root follows a symlink under errand's home, #43, #44), T-080
  (freshness `sh -c`, #45), T-081 (refuse_borrowed timing, #46), T-082 (the
  broker's egress through a proxy and the CA, #56, #57), T-083 (the egress
  allowlist and its silent refusals, #58), T-084 (the capability probe and
  tier, #50, #51), T-085 (constrained-host sizing and errand's limits, #53,
  #54, #55), T-086 (the `lite` engine, #49), T-087 (the gate and the live
  boundary check, #48), T-088 (the two credential files, #47), T-089 (the
  broker panic, #52).
- **Folded #59 into T-032** (the broker's private-address arms) and
  **corrected T-029's closing evidence** (#48).
- Issues #2 to #42 already had entries T-038 to T-078 (one per issue); the
  new entries cross-link them where a root cause is shared.

## What is in progress

Nothing. The triage is complete: every open issue is in an entry, new or
existing.

---

## Start here next session

Worked by root cause, security first.

1. **The P1 security findings:** T-079 (root follows a symlink under errand's
   home, a second route to root), T-080 (the freshness workflow runs an
   upstream tag through `sh -c`), T-081 (the pull-request clone reads the
   guest-writable git directory after the check). Then T-020, the broker link
   that lets the project forbid proxy devices.
2. **The P1 host failures of T-027's run, and the capability layer under
   them:** T-040, T-041, T-046, T-038, T-039, T-042, and T-084 (probe the
   host and choose the tier, which T-044 and T-086 both need).
3. **setup proves a guest starts, and refuses where none can:** T-043, T-044,
   then T-037 with T-050 and T-089 (the broker panic that feeds the restart
   loop).
4. **The broker's provider and egress boundary:** T-032 (the provider route,
   now with the private-address arms), T-082 (egress through a proxy, the
   CA), T-083 (the allowlist and its refusals), and T-087 (the live boundary
   check and the recorded-output gate).
5. **Constrained hosts:** T-085 (sizing and errand's limits), then T-086, the
   researched `lite` engine design.
6. **The other P2 entries of T-027's run, smallest first:** T-051, T-063,
   T-047, T-048, T-058, T-059, T-060, T-061, T-062, T-064, T-065, T-045,
   T-052, T-053, T-055, T-057, T-049, T-054, T-066, and T-088.
7. **errand's chat and files:** T-035's open items, then T-033.
8. **The backend:** T-019's incus backend in the fork (needs T-020), which
   retires the `bailey` stand-in.
9. T-024, then T-021 and T-022, which wait on T-020's link.
10. T-034's podman and `issues`, T-036, T-014, T-026, T-023, T-025, and the
    P3 entries of the run.

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
