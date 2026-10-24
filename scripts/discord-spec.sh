#!/bin/sh
# discord-spec.sh - Discord's REST spec, trimmed to what errand reaches, and
# the check that the Discord stand-in answers in its shapes.
#
# The defect this exists to stop: a stand-in tuned until serenity accepts
# its answers is not yet shaped like Discord. serenity reads a message with
# no `flags`; Discord's spec requires the field. The check drives the
# fifteen operations errand reaches through serenity
# (discord-mock/tests/contract.rs) and validates every answer against
# discord/discord-api-spec at a pinned commit.
#
# Usage:
#   sh scripts/discord-spec.sh            regenerate the snapshot from the pin
#   sh scripts/discord-spec.sh --check    the gate step `discord-contract`
#   sh scripts/discord-spec.sh --drift    compare the snapshot with upstream main
#
# The snapshot, discord-mock/spec/errand-surface.json, is generated and never
# edited; --check refuses one whose digest is not the pinned one. To move
# the pin, set PINNED_REF to the new commit, set both digests to what the
# regeneration prints, run --check, and record the move in TODO/errand.md.
#
# Needs: python3, with python3-jsonschema for --check; cargo for --check;
# curl for a regeneration and --drift. Every Python run is `python3 -I`.
# Exit codes: 0 passed, 1 an answer does not match the spec, the snapshot
# does not match its pin, or upstream drifted; 2 could not run.
set -u
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root" || exit 2

# The spec's commit, the digest of its whole openapi.json there, and the
# digest of the snapshot this script makes from it.
PINNED_REF=bf93257f85d62054a0a6982d32f6bcb724abd015
SPEC_PINNED_SHA256=8ce3450e18f3381dd6b2936db2172ecb5bf8a7f6549ecb51ec3cd01b7c30fa3b
SNAPSHOT_PINNED_SHA256=cbbcbe4700f0eaba7822510c36151914aad7bfcb82db9f11916c9c7daad72b75

raw=https://raw.githubusercontent.com/discord/discord-api-spec
snapshot=discord-mock/spec/errand-surface.json
licence=discord-mock/spec/LICENSE-discord-api-spec
tool=scripts/discord-spec.py

say() { printf 'discord-spec: %s\n' "$*"; }
die() { say "$1" >&2; exit "${2:-1}"; }

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | cut -d' ' -f1
  else shasum -a 256 "$1" | cut -d' ' -f1
  fi
}

command -v python3 >/dev/null 2>&1 || die "python3 is missing" 2

# Downloads the spec at URL $1 into a new directory, and prints the directory.
fetch_spec() {
  tmp=$(mktemp -d) || die "cannot make a temporary directory" 2
  curl -fsSL --retry 3 -o "$tmp/openapi.json" "$1" || { rm -rf "$tmp"; die "download failed: $1"; }
  printf '%s\n' "$tmp"
}

case ${1:-} in
  "")
    command -v curl >/dev/null 2>&1 || die "curl is missing" 2
    tmp=$(fetch_spec "$raw/$PINNED_REF/specs/openapi.json") || exit $?
    got=$(sha256_of "$tmp/openapi.json")
    if [ "$got" != "$SPEC_PINNED_SHA256" ]; then
      rm -rf "$tmp"
      die "openapi.json at $PINNED_REF is $got, not the pinned $SPEC_PINNED_SHA256"
    fi
    curl -fsSL --retry 3 -o "$tmp/LICENSE" "$raw/$PINNED_REF/LICENSE" || { rm -rf "$tmp"; die "download failed: the licence"; }
    mkdir -p "$(dirname "$snapshot")"
    python3 -I "$tool" trim "$tmp/openapi.json" "$snapshot"
    rc=$?
    cp "$tmp/LICENSE" "$licence"
    rm -rf "$tmp"
    [ "$rc" = 0 ] || exit "$rc"
    now=$(sha256_of "$snapshot")
    say "snapshot sha256 $now"
    [ "$now" = "$SNAPSHOT_PINNED_SHA256" ] || die "the snapshot is not the pinned $SNAPSHOT_PINNED_SHA256: move SNAPSHOT_PINNED_SHA256 if the pin moved on purpose"
    say "the snapshot matches its pin"
    ;;
  --check)
    if ! python3 -I -c 'import jsonschema' 2>/dev/null; then
      die "python3-jsonschema is missing, so no answer was checked: apt-get install -y python3-jsonschema" 2
    fi
    command -v cargo >/dev/null 2>&1 || die "cargo is missing" 2
    [ -r "$snapshot" ] || die "$snapshot is missing: run sh scripts/discord-spec.sh" 2
    now=$(sha256_of "$snapshot")
    [ "$now" = "$SNAPSHOT_PINNED_SHA256" ] || die "$snapshot is $now, not the pinned $SNAPSHOT_PINNED_SHA256: it is generated, never edited"
    corpus=$(mktemp -d) || die "cannot make a temporary directory" 2
    trap 'rm -rf "$corpus"' EXIT INT TERM
    DISCORD_MOCK_CORPUS=$corpus cargo test --locked -p discord-mock --test contract || die "the contract test failed before any answer was checked"
    python3 -I "$tool" check "$snapshot" "$corpus"
    rc=$?
    [ "$rc" -le 1 ] || exit 2
    exit "$rc"
    ;;
  --drift)
    command -v curl >/dev/null 2>&1 || die "curl is missing" 2
    tmp=$(fetch_spec "$raw/main/specs/openapi.json") || exit $?
    python3 -I "$tool" trim "$tmp/openapi.json" "$tmp/upstream.json" >/dev/null || { rm -rf "$tmp"; exit 1; }
    python3 -I "$tool" diff "$snapshot" "$tmp/upstream.json"
    rc=$?
    rm -rf "$tmp"
    [ "$rc" -le 1 ] || exit 2
    exit "$rc"
    ;;
  *) die "usage: discord-spec.sh [--check | --drift]" 2 ;;
esac
