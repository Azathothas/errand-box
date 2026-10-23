#!/bin/sh
# setup-failed-only-errand.sh - did a setup report fail only where the test
# credentials make it fail?
#
# The test credentials are not real, so errand's daemon fails at the chat
# connection (T-015); every other FAILED row is a real defect. One script
# holds that rule for tests/hosts/host.sh and .github/workflows/live.yml.
#
# A row name is followed by the padding and FAILED: "errand " alone is also
# the start of "errand project" and "errand config", which must not fail.
#
# Usage: sh tests/setup-failed-only-errand.sh LOG [ROW]
#   ROW  one more row allowed to fail, such as "guest image"
# Exit codes: 0 no FAILED row but errand's (or ROW's), 1 another row failed,
# 2 could not run.
set -u
[ -r "${1:-}" ] || { echo "usage: setup-failed-only-errand.sh LOG [ROW]" >&2; exit 2; }
others=$(grep -E '  FAILED ' "$1" | grep -vE "^(errand|errand-box-errand|bailey doctor as errand${2:+|$2})  +FAILED ")
[ -z "$others" ] && exit 0
printf '%s\n' "$others"
exit 1
