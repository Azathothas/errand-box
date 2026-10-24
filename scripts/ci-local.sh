#!/bin/sh
# ci-local.sh - run every step of .github/workflows/gates.yml on this machine.
#
# The defect this exists to stop: a push that nobody checked. gates.yml calls
# this script one step at a time, so the two cannot drift, and a CI job stops
# at its first failed step, so the steps after it are not checked there. CI
# runs every step but `cross`. Its runner image moves (ubuntu-latest), and a
# newer tool can refuse what an older one passed: shellcheck 0.11 did.
#
# Usage:
#   sh scripts/ci-local.sh                 every step, in order
#   sh scripts/ci-local.sh --step NAME     one step
#   sh scripts/ci-local.sh --list          the step names
#
# Steps: gate kill-by-name shellcheck fmt clippy test pins errand-config musl
# cross. `musl` builds the static release binary for this machine's
# architecture. `cross` builds for the other architecture (x86_64 and
# aarch64) and runs the unit and integration tests there under qemu-user.
# It needs qemu-<arch>-static, and a binfmt_misc handler for the integration
# tests; without them it runs what it can and exits 2.
# The live tests (tests/live/live.sh) need a host with Incus and are not a
# step here; docs/tests.md says when to run them.
#
# Needs: cargo (the toolchain of rust-toolchain.toml), shellcheck, curl.
# CARGO_BUILD_JOBS defaults to 4.
# Exit codes: 0 every step passed, 1 a step failed, 2 could not run.
set -u
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root" || exit 2
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"
STEPS="gate kill-by-name shellcheck fmt clippy test pins errand-config musl cross"

# The shell files this repository ships. research/ is a record of what was
# run and is not linted.
shell_files() {
  printf '%s\n' bootstrap.sh image/provision.sh image/launch image/guest-init tests/fake-incus \
    tests/pins-test.sh tests/errand-config-test.sh tests/live/live.sh scripts/ci-local.sh scripts/pins.sh \
    scripts/check-no-kill-by-name.sh scripts/check-todo.sh scripts/track-issue.sh \
    scripts/freshness.sh scripts/watch-upstreams.sh scripts/build-image.sh scripts/fetch-tracker.sh scripts/notices.sh tests/hosts/host.sh \
    tests/setup-failed-only-errand.sh
}

# The C compiler for target $1, exported as the cc crate reads it, or a
# message and 2. errand compiles C (ring, aws-lc-sys, SQLite; T-028), and a
# static musl binary needs a musl C compiler: `musl-gcc` from musl-tools for
# this machine's own target, `ARCH-linux-musl-gcc` for another, else zig
# when $3 is `zig`. A CC_ the caller set wins.
musl_cc() {
  cv=CC_$(printf '%s' "$1" | tr '-' '_')
  if env | grep -q "^$cv="; then return 0; fi
  for cc in "$2" "${1%%-*}-linux-musl-gcc"; do
    if [ -n "$cc" ] && command -v "$cc" >/dev/null 2>&1; then
      export "$cv=$cc"
      return 0
    fi
  done
  if [ "${3:-}" = zig ] && zig_cc "$1"; then return 0; fi
  echo "no musl C compiler for $1: set $cv (musl-tools gives musl-gcc for this machine's own target only; zig serves any, see docs/tests.md)" >&2
  return 2
}

# zig as the C compiler and archiver for target $1: `zig` on PATH, or the
# ziglang package of pip (`python3 -m ziglang`). It writes two wrappers under
# target/zig, because the cc crate names a compiler with one path, and drops
# the Rust triple the cc crate passes as --target, which zig does not read.
zig_cc() {
  if command -v zig >/dev/null 2>&1; then
    zig=zig
  elif python3 -m ziglang version >/dev/null 2>&1; then
    zig="python3 -m ziglang"
  else
    return 1
  fi
  d=$PWD/target/zig
  mkdir -p "$d" || return 1
  zt=${1%%-*}-linux-musl
  # shellcheck disable=SC2016 # the wrapper's own text: its $@ and $a expand when it runs
  {
    printf '#!/bin/sh\n# Written by scripts/ci-local.sh: %s cc for %s.\n' "$zig" "$zt"
    printf 'for a in "$@"; do shift; case $a in --target=*) ;; *) set -- "$@" "$a" ;; esac; done\n'
    printf 'exec %s cc -target %s "$@"\n' "$zig" "$zt"
  } >"$d/$zt-cc" || return 1
  printf '#!/bin/sh\nexec %s ar "$@"\n' "$zig" >"$d/$zt-ar" || return 1
  chmod +x "$d/$zt-cc" "$d/$zt-ar" || return 1
  tv=$(printf '%s' "$1" | tr '-' '_')
  export "CC_$tv=$d/$zt-cc" "AR_$tv=$d/$zt-ar"
  echo "zig ($zig version $($zig version)) compiles C for $1" >&2
}

musl_target() {
  case $(uname -m) in
    x86_64 | amd64) echo x86_64-unknown-linux-musl ;;
    aarch64 | arm64) echo aarch64-unknown-linux-musl ;;
    *) echo "" ;;
  esac
}

step() {
  case $1 in
    gate) sh scripts/common/check-gate.sh --strict && sh scripts/check-todo.sh ;;
    kill-by-name) sh scripts/check-no-kill-by-name.sh ;;
    shellcheck)
      command -v shellcheck >/dev/null 2>&1 || { echo "shellcheck is missing" >&2; return 2; }
      # shellcheck disable=SC2046 # one file per word; no names have spaces
      shellcheck -s sh $(shell_files)
      ;;
    fmt) cargo fmt --all -- --check ;;
    clippy) cargo clippy --locked --workspace --all-targets -- -D warnings ;;
    test) cargo test --locked --workspace ;;
    pins) sh tests/pins-test.sh ;;
    # Runs errand-box as errand against the config errand-box writes.
    errand-config) cargo build --locked && sh tests/errand-config-test.sh ;;
    musl)
      t=$(musl_target)
      [ -n "$t" ] || { echo "no musl target for $(uname -m)" >&2; return 2; }
      musl_cc "$t" musl-gcc || return 2
      cargo build --locked --release --target "$t" || return 1
      bin="target/$t/release/errand-box"
      # A static binary has no interpreter; `file` says so where it exists.
      if command -v file >/dev/null 2>&1; then
        file "$bin" | grep -q 'static' || { echo "$bin is not static: $(file "$bin")" >&2; return 1; }
      fi
      "$bin" version || return 1
      # The supervisor runs errand as `errand-box serve` (T-028): the static
      # binary must carry it. A word serve does not take is refused by
      # errand's own parser, which proves the library is there.
      said=$("$bin" serve --not-a-flag 2>&1)
      case $said in
        *"serve takes -v"*) ;;
        *) echo "$bin does not carry errand's serve: $said" >&2; return 1 ;;
      esac
      ;;
    cross)
      case $(uname -m) in
        x86_64 | amd64) arch=aarch64 ;;
        aarch64 | arm64) arch=x86_64 ;;
        *) echo "no cross target for $(uname -m)" >&2; return 2 ;;
      esac
      t=$arch-unknown-linux-musl
      rustup target list --installed 2>/dev/null | grep -qx "$t" || { echo "rust target $t is not installed: rustup target add $t" >&2; return 2; }
      # rust-lld links a static musl binary for any target; no cross gcc.
      tv=$(printf '%s' "$t" | tr 'a-z-' 'A-Z_')
      export "CARGO_TARGET_${tv}_LINKER=rust-lld"
      musl_cc "$t" "" zig || return 2
      cargo build --locked --release --target "$t" || return 1
      # A test that was not run is not a pass: each path that runs less
      # than every test says what it left out and returns 2.
      if ! command -v "qemu-$arch-static" >/dev/null 2>&1; then
        echo "qemu-$arch-static is missing: built for $t, tests not run" >&2
        return 2
      fi
      export "CARGO_TARGET_${tv}_RUNNER=qemu-$arch-static"
      # The runner starts the test binaries. tests/shim.rs starts the shim
      # as a child, and only a binfmt_misc handler lets the kernel run that.
      if [ ! -e "/proc/sys/fs/binfmt_misc/qemu-$arch" ]; then
        cargo test --locked --target "$t" --bins || return 1
        echo "no binfmt_misc handler for $arch: the unit tests passed; the integration tests start the shim as a child and were not run" >&2
        return 2
      fi
      cargo test --locked --target "$t"
      ;;
    *) echo "unknown step: $1 (steps: $STEPS)" >&2; return 2 ;;
  esac
}

case ${1:-} in
  --list) for s in $STEPS; do echo "$s"; done; exit 0 ;;
  --step)
    [ -n "${2:-}" ] || { echo "usage: ci-local.sh --step NAME" >&2; exit 2; }
    step "$2"
    rc=$?
    [ "$rc" -le 1 ] || exit 2
    exit "$rc"
    ;;
  "") ;;
  *) echo "usage: ci-local.sh [--step NAME | --list]" >&2; exit 2 ;;
esac

failed=0
summary=""
for s in $STEPS; do
  printf '== %s\n' "$s"
  t0=$(date +%s)
  step "$s"
  rc=$?
  summary="$summary$(printf '%-14s exit %s  %4s s' "$s" "$rc" "$(($(date +%s) - t0))")
"
  [ "$rc" = 0 ] || failed=1
done
printf '\n%s' "$summary"
[ "$failed" = 0 ] && echo "ci-local: every step passed" || echo "ci-local: a step failed"
exit "$failed"
