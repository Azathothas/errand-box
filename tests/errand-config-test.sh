#!/bin/sh
# errand-config-test.sh - the config.json errand-box writes is one that the
# errand built into errand-box accepts.
#
# errand v0.3.1 refused a key that the generator wrote for a later errand
# (`output.postImages`), and its supervisor restarted it every second. This
# check finds that class of defect before setup does: it builds the config
# from tests/data with `errand-box errand-config`, and runs `errand-box
# serve`, errand's daemon from the library built in (T-028), with that config
# for a few seconds. errand checks its config before it does anything
# else. The pass condition is the line errand logs after the check ("egress
# is brokered"); with no bailey on PATH it then stops.
#
# Environment: ERRAND_BOX, the errand-box binary (default target/debug).
# Needs: timeout. No network.
# Exit codes: 0 errand accepts the config, 1 it refuses it, 2 could not run.
set -u
root=$(cd "$(dirname "$0")/.." && pwd)
eb=${ERRAND_BOX:-$root/target/debug/errand-box}
[ -x "$eb" ] || { echo "errand-config-test: $eb is missing; run cargo build" >&2; exit 2; }
t=$(mktemp -d) || exit 2
trap 'rm -rf "$t"' EXIT INT TERM

v=fork

"$eb" errand-config --env "$root/tests/data/errand.env" --creds "$root/tests/data/creds.env" \
  --home "$t/errand-home" --out "$t/h/.config/errand/config.json" || exit 1
# errand gets the environment errand-box gives it (procs::exec_as): cleared,
# then HOME and PATH. errand reads XDG_CONFIG_HOME before HOME, and a GitHub
# runner sets it, so an inherited environment hides the config from errand.
out=$(cd "$t" && env -i HOME="$t/h" PATH=/usr/bin:/bin timeout 20 "$eb" serve 2>&1)
case $out in
  *ConfigError* | *"configuration rejected"*)
    printf 'FAIL errand %s refuses the config errand-box writes:\n%s\n' "$v" "$(printf '%s\n' "$out" | grep -m 3 -i 'config')"
    exit 1
    ;;
  *"egress is brokered"*)
    printf 'PASS errand %s accepts the config errand-box writes\n' "$v"
    exit 0
    ;;
  *)
    printf 'FAIL errand %s stopped before its config check passed:\n%s\n' "$v" "$(printf '%s\n' "$out" | tail -n 5)"
    exit 1
    ;;
esac
