# Upkeep: CI, releases and the scheduled workflows

Entries about how this repository keeps itself current. [INDEX.md](INDEX.md)
lists every entry.

---

# T-007: Scheduled workflows that open pull requests and issues

**Source:** the operator: "self-maintaining, as mirror-archlinux does"
**Category:** upkeep
**Priority:** P1
**Effort:** M
**Status:** partial

## Problem

errand, kage, Rust and the upstreams setup reads move without notice.

## Approach

`.github/workflows/freshness.yml` (weekly) runs `scripts/freshness.sh`: a
pull request per newer errand, kage or stable Rust, with the pin moved by
`scripts/pins.sh`, which checks each digest against upstream's `.sha256`
and GitHub's asset digest. `.github/workflows/watchdog.yml` (daily) runs
`scripts/watch-upstreams.sh` and keeps one issue per missing upstream with
`scripts/track-issue.sh`. Each workflow keeps the other's schedule on.

## Prove

```sh
sh tests/pins-test.sh
sh scripts/watch-upstreams.sh
GITHUB_API=https://api.github.com sh scripts/freshness.sh --dry-run
```

and one run of each workflow on GitHub.

## Closing

Partial, 2026-10-23T08:40Z. The local half passes: `tests/pins-test.sh` 5 of
5; `scripts/watch-upstreams.sh` 4 of 4 PASS; `scripts/freshness.sh
--dry-run` reports kage and errand current and Rust 1.99.1 newer than the
pinned 1.99.0; this session then moved the pin to 1.99.1. A bump of errand
from v0.3.0 to v0.3.1 reproduced the pinned
digests exactly, and matched both upstream's `.sha256` and GitHub's digest.
`actionlint` passes on every workflow.

On GitHub, 2026-10-23T12:57Z, each dispatched by hand on `main`: `watchdog`
(run 39957224850) passed 4 of 4 checks, found no `upstream-*` issue to
clear, and turned the freshness schedule back on. `freshness` with
`dry_run` (run 39957228817) and without it (run 39957441157) found kage
v0.5.0, errand v0.3.1 and Rust 1.99.1 current, opened nothing, cleared no
issue, and turned the watchdog schedule back on.

Open: the step that opens a pull request has not run, because no upstream
is newer. It needs the repository setting "Allow GitHub Actions to create
and approve pull requests", which this session cannot read. It proves
itself on the first newer errand, kage or Rust, or on a test bump.

## Decision

The operator ruled on 2026-10-23: an agent with the operator's `gh` CLI
sets the repository setting and stores `FRESHNESS_TOKEN`, when the operator
asks for it. That is the delegation [secrets.md](../docs/security/secrets.md)
requires. The operator creates the token: fine-grained, this repository
only, Contents and Pull requests read and write. The agent:

```sh
gh api repos/Azathothas/errand-box/actions/permissions/workflow
gh api -X PUT repos/Azathothas/errand-box/actions/permissions/workflow \
  -f default_workflow_permissions=read -F can_approve_pull_request_reviews=true
gh secret list --repo Azathothas/errand-box
gh secret set FRESHNESS_TOKEN --repo Azathothas/errand-box < "$TOKEN_FILE"
```

The first command reads the current value; pass the
`default_workflow_permissions` it shows, not a guess. Set the secret only if
the list shows no `FRESHNESS_TOKEN`, so no working value is overwritten. The
value goes from a file the operator writes, outside every repository,
through stdin: never on a command line or in the chat. Delete the file
after. Then dispatch `freshness.yml` with a test bump, or wait for the
first newer upstream, and record the pull request it opens.

---

# T-009: The first release

**Source:** this session; `bootstrap.sh` downloads a release that does not
exist yet
**Category:** upkeep
**Priority:** P1
**Effort:** S
**Status:** done

## Problem

`bootstrap.sh` outside a checkout fetches `errand-box-<arch>-linux-musl`
and `SHA256SUMS` from the latest release. There is none.

## Approach

Dispatch `.github/workflows/release.yml` on `main` with the version
`v0.1.0`, or push that tag. It builds both architectures through
`scripts/ci-local.sh --step musl` and publishes the release; a dispatch
creates the tag on the commit it built. The dispatch exists because a
session's git route can push `main` and refuses a tag: measured
2026-10-23, HTTP 403 on the tag, while `git push --dry-run` accepts it.

## Prove

```sh
sh bootstrap.sh --dry-run && sh bootstrap.sh
```

on a host with no checkout: the digest line, then `installed`.

## Closing

**Closed 2026-10-23T12:56Z.** A dispatch of `release.yml` on `main` (run
39956908005, commit `7363d89`) built both architectures and published
[v0.1.0](https://github.com/Azathothas/errand-box/releases/tag/v0.1.0) with
the tag on that commit. The three assets, downloaded: `sha256sum -c
SHA256SUMS` passes for both binaries, each digest matches the one GitHub
records, both are static, and both print `errand-box 0.1.0`, the aarch64 one
under qemu-user. Then `bootstrap.sh`, fetched from the tag into an empty
directory, on a host with no errand-box installed. The digest below is cut to
its first 12 characters; the release's `SHA256SUMS` holds all of it.

```text
bootstrap: fetching errand-box-x86_64-linux-musl and SHA256SUMS from https://github.com/Azathothas/errand-box/releases/latest/download
bootstrap: digest checked: 589e100bd83d...
bootstrap: installed /usr/local/lib/errand-box/bin/errand-box; /usr/local/bin/errand-box links to it
```

Exit 0, after `--dry-run` exited 0 and named each step it would run.

---

# T-017: A documentation check: every command, key and path a page names exists

**Source:** docs/research/sandhome.md (sandhome's `tests/docs.sh`)
**Category:** upkeep
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

A page can name an `errand-box` command, a config key or a path that the
code no longer has, and nothing notices.

## Approach

A script in the gate that extracts `errand-box <word>`, the keys of
`examples/errand-box.toml` and the paths in `docs/operations.md`, and checks
each against `errand-box help`, `src/config.rs` and the tree.

## Prove

```sh
sh scripts/ci-local.sh --step gate
```

fails on a page that names `errand-box frobnicate`.

---

# T-026: Live checks that read a state once, at a moment they do not control

**Source:** the live runs on GitHub's runners, 2026-10-23 (run 39968392861,
and job 120468853678 of a later run)
**Category:** upkeep
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

Two checks in `tests/live/live.sh` read a state once, so the same code
passes or fails by timing:

1. `t_errand` (`tests/live/live.sh:355`) reads `systemctl is-active` once.
   Under test credentials errand exits at the chat connection and systemd
   restarts it, so the check reads `active` or `activating` by chance. In
   run 39968392861 the x86_64 job read `active` and the aarch64 job read
   `activating`.
2. `t_reap` (`tests/live/live.sh:299`) kills the launcher and the watchdog,
   sleeps 2 s, and expects RUNNING. In job 120468853678, on aarch64, it read
   STOPPED.

## Premise

The first is measured. The cause of the second is read, not measured: if
`pgrep` finds no watchdog, the watchdog outlives the kill and stops the
instance as designed, which reads as STOPPED.

## Approach

1. `t_errand`: read `systemctl show -p ActiveState,SubState,NRestarts`.
   Under test credentials, pass when errand is active, or restarting with
   the chat failure in its log, and name which. With T-015's credentials,
   require `active`.
2. `t_reap`: wait until `pgrep` finds the watchdog instead of `sleep 2`, and
   fail with its own message when it never does, so the two causes read
   differently.

## Prove

```sh
sh tests/live/live.sh errand
sh tests/live/live.sh reap
```

On both runners, through live.yml; and neither check sleeps where it can
wait on a condition.

---

# T-075: live.sh's up test does nothing under systemd

**Source:** [issue #39](https://github.com/Azathothas/errand-box/issues/39), T-027's end-to-end run, 2026-10-24
**Category:** upkeep
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

On a systemd host `t_up` passes at once and stops nothing. By hand, `up`
restored every unit, and the same pass found doctor's wrong golden row
(T-068) and two `up` runs that disagreed (T-050).

## Premise

Measured on WSL2 (Ubuntu 24.04). Read: `tests/live/live.sh`, `t_up`.

## Approach

Under systemd stop the units, reset the device modes, run `up`, and check
each unit and a second `up` with 0 changed.

## Prove

```sh
sh tests/live/live.sh up   # on a systemd host: units stopped and restored, no early PASS
```

---

# T-076: mock-session.sh passes when the session never started

**Source:** [issue #40](https://github.com/Azathothas/errand-box/issues/40), T-027's end-to-end run, 2026-10-24
**Category:** upkeep
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

mock-session.sh passes on any post in the thread, so "this session could
not start" counts as a pass.

## Premise

Measured on Daytona, where no instance can start: PASS. Read:
`tests/e2e/mock-session.sh:126-148`.

## Approach

Pass on errand's ready line, fail on "could not start", and print the
thread's posts.

## Prove

```sh
sh tests/e2e/mock-session.sh "demo: say hello"; echo "exit $?"   # 1 where sessions cannot start
```

---

# T-078: feature.yml applies a label that does not exist

**Source:** [issue #42](https://github.com/Azathothas/errand-box/issues/42), T-027's end-to-end run, 2026-10-24
**Category:** upkeep
**Priority:** P3
**Effort:** S
**Status:** open

## Problem

The Feature form labels its issues `feature`, and the repository has no
such label, so they arrive unlabelled and `--label feature` fails.

## Premise

Read: `.github/ISSUE_TEMPLATE/feature.yml`; `gh label list` has
`enhancement` and no `feature`.

## Approach

Create the label, or use `enhancement` in the template.

## Prove

```sh
gh label list --repo Azathothas/errand-box | grep -c '^feature'   # 1, or the template names enhancement
```

---

# T-080: freshness.sh runs an upstream release tag through sh -c, with the workflow's write token

**Source:** [issue #45](https://github.com/Azathothas/errand-box/issues/45), read on `57231da`
**Category:** upkeep
**Priority:** P1
**Effort:** S
**Status:** open

## Problem

The weekly freshness workflow builds a shell string from an upstream release
tag it does not validate, and runs it with `sh -c`, holding a write token.
`pins.sh upstream` takes the newest tag from the GitHub API with no check
(`scripts/pins.sh:87`, a `grep -o '"tag_name"...'`). `freshness.sh` passes it
to `update` as part of a command string (`scripts/freshness.sh:99`), and
`update` runs that string with `sh -c "$3"` (`scripts/freshness.sh:55`). A
git tag name may hold `;`, `$`, `(`, `)` and backticks, so a tag such as
`v1.0;command` runs `command`. `pins.sh bump` validates its own argument
(`scripts/pins.sh:94`), but the injection runs in `sh -c` before bump is
reached, so that check does not help. The job runs with `contents: write`,
`pull-requests: write` and `actions: write`
(`.github/workflows/freshness.yml`). The Rust channel path is the same shape:
`sed -i` with an unvalidated version (`scripts/freshness.sh:110`).

## Premise

Read on `57231da`, and the injection reproduced by the reporter with a stub
`pins.sh`: `git check-ref-format` accepts a tag holding shell syntax, and
`sh -c "sh scripts/pins.sh bump kage $tag"` ran the injected command. Whether
GitHub's release API accepts such a tag was not tested; git accepts it as a
ref name, and an upstream maintainer, or anyone who can publish a release in a
watched repository, sets the tag.

## Approach

The root cause is a value from outside run as code. Accept only
`^v?[0-9][0-9A-Za-z.+-]*$` in `pins.sh upstream` and again in `update`, and
pass the version as an argument rather than in a shell string:
`sh scripts/pins.sh bump "$tool" "$up"`, not `sh -c`. Validate the Rust
version the same way before the `sed`.

## Prove

```sh
sh tests/pins-test.sh
```

A test runs `update` with a tag such as `v1;command` and asserts it is
refused and the command did not run; it fails on the current code.

---

# T-087: the gate does not require an entry's prove output, and no live check crosses the broker's provider boundary

**Source:** [issue #48](https://github.com/Azathothas/errand-box/issues/48), read on `57231da`
**Category:** upkeep
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

`scripts/check-todo.sh` is structural: it checks the rows, the statuses and
the counts, and never reads an entry's Prove section or requires a recorded
output. So an entry can be `done` with no evidence on file, which is how
T-029, T-030 and T-031 record "the prove passed, N tests" with no output
kept. T-029 (P0) was then closed citing a live run, `tests/live/live.sh` 52
of 52, that sends no `/provider/` path and so does not exercise the boundary
the fix is about; the real proof is the broker unit tests. The live tests
have no check that crosses the provider boundary at all: `tests/live/live.sh`
reports `direct egress: blocked` and passes on any failure, not a specific
result (`tests/live/live.sh:179`).

## Premise

Read on `57231da`: the live artefacts, the closing records of T-029, T-030
and T-031, and `check-todo.sh`, which reads only the index, the entries'
status and category lines, and the counts. T-029's closing evidence is
corrected under that entry in this change.

## Approach

1. A live check in `tests/live/live.sh` that sends `/provider/NAME.other.example/`
   and the `:443@other.example` spelling through the broker and expects a 4xx,
   so a host run exercises T-029's boundary.
2. Make `check-todo.sh --set T-NNN done` require a recorded prove output, or a
   pointer to one under `tests/`, before it moves a status to `done`; or a
   gate step that refuses a `done` entry whose Prove names a command with no
   recorded output.
3. Where a claim quotes a number another file measures (the live counts), keep
   the artefact, so `direct egress: blocked` becomes a recorded result rather
   than prose.

## Prove

```sh
sh scripts/ci-local.sh --step gate
sh tests/live/live.sh session
```

The gate step refuses a `done` entry with no recorded prove output; the live
`session` test's broker check sends a boundary-crossing path and expects a
4xx.

---

# T-088: creds.env and errand-box.env are not ignored, and the secret check does not match them

**Source:** [issue #47](https://github.com/Azathothas/errand-box/issues/47), read on `57231da`
**Category:** upkeep
**Priority:** P2
**Effort:** S
**Status:** open

## Problem

The docs tell the operator to keep `creds.env` (mode 0600) and
`errand-box.env` beside the command (README quick start,
docs/security/secrets.md). `.gitignore` ignores `.env`, `.env.*` and the rest
(`.gitignore:10-12`), but not `creds.env` or `errand-box.env`: a run with
this repository's `.gitignore` showed both staged by `git add -A`.
`scripts/common/check-no-secrets.sh` scans for secret shapes and for file
names, but its name pattern matches `.env` preceded by a slash or the start,
not the `creds.env` and `errand-box.env` spellings
(`scripts/common/check-no-secrets.sh:94`). So a checkout where the operator
wrote the two files stages the credentials, and no check refuses them.
docs/security/secrets.md already says to list the ignore rule before the file
exists, and to re-exclude credentials by name.

## Premise

Read on `57231da`, and reproduced in a scratch repository with this tree's
`.gitignore`: `creds.env` and `errand-box.env` both show as untracked and
stage. No secret is in the tree or its history by a pattern sweep; this is
about the next commit, not a past one.

## Approach

Add `creds.env` and `errand-box.env` (or `creds.*` and `/*.env` with an
exception for the example twins) to `.gitignore`, re-excluded by name after
every re-inclusion rule, and make `check-no-secrets.sh` refuse the two names
when tracked.

## Prove

```sh
sh scripts/ci-local.sh --step gate
```

In a scratch tree with the new `.gitignore`, `git status --short` prints
nothing for `creds.env` or `errand-box.env`, and `check-no-secrets.sh`
refuses either when it is tracked.
