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
