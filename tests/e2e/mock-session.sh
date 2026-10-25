#!/bin/sh
# mock-session.sh - one errand session against the Discord stand-in, on a
# host that errand-box has set up with tests/e2e/mock.env and
# tests/e2e/mock-creds.env.
#
#   sh mock-session.sh [PROMPT]      # as root; PROMPT defaults to "demo: say hello"
#
# It starts discord-mock on loopback, stops the supervised errand, runs
# `errand-box serve -v` as the errand account with ERRAND_DISCORD_MOCK_URL
# naming the stand-in, and posts PROMPT as the person the stand-in seeds.
# It waits until errand has opened a thread for it and posted in that
# thread, then stops both and starts the supervised errand again.
#
# errand runs by hand here because the supervised errand cannot reach the
# stand-in: `errand-box __as` clears the environment (src/procs.rs), and no
# setting names a stand-in.
#
# Environment:
#   DISCORD_MOCK   the stand-in (default: discord-mock on PATH, else
#                  ./discord-mock-<arch>-linux-musl)
#   EB_E2E_USER    errand's account (default errand)
#   EB_E2E_WAIT    seconds to wait for the thread and its first post (default 300)
#   EB_E2E_OUT     where the logs go (default ./mock-session-<UTC time>)
# Needs: root, errand-box set up, curl, python3 (to read the journal).
# Output: OUT/mock.log, OUT/errand.log, OUT/journal.json (every request
# errand made), OUT/gaps.json (what it asked for that the stand-in does not
# implement), OUT/calls.txt, and a summary.
# Exit codes: 0 errand opened a thread and posted in it, 1 it did not,
#   2 could not run.
set -u
die() { printf 'mock-session: %s\n' "$1" >&2; exit "${2:-2}"; }

prompt=${1:-demo: say hello}
user=${EB_E2E_USER:-errand}
wait_s=${EB_E2E_WAIT:-300}
out=${EB_E2E_OUT:-./mock-session-$(date -u +%Y%m%dT%H%M%SZ)}
eb=/usr/local/lib/errand-box/bin/errand-box
scan=/var/lib/errand-box/s6

[ "$(id -u)" = 0 ] || die "run as root"
[ -x "$eb" ] || die "$eb is missing: install errand-box and run setup first"
for t in curl python3; do command -v "$t" >/dev/null 2>&1 || die "$t is missing"; done
case $(uname -m) in
  x86_64 | amd64) arch=x86_64 ;;
  aarch64 | arm64) arch=aarch64 ;;
  *) die "no discord-mock for $(uname -m)" ;;
esac
mock=${DISCORD_MOCK:-$(command -v discord-mock || echo "./discord-mock-$arch-linux-musl")}
[ -x "$mock" ] || die "no discord-mock at $mock: set DISCORD_MOCK"
mkdir -p "$out" || die "cannot make $out"

# Stops (down) or starts (up) the supervised errand: through s6 where
# errand-box runs its own, else through systemd.
svc=$(command -v s6-svc || echo /usr/bin/s6-svc)
sup() {
  if [ -d "$scan/errand" ]; then
    case $1 in down) "$svc" -d "$scan/errand" ;; *) "$svc" -u "$scan/errand" ;; esac
  else
    case $1 in down) systemctl stop errand-box-errand.service ;; *) systemctl start errand-box-errand.service ;; esac
  fi
}

mock_pid=""
errand_pid=""
# shellcheck disable=SC2329,SC2317 # the EXIT trap calls it, so its body is reached
cleanup() {
  [ -n "$errand_pid" ] && kill "$errand_pid" 2>/dev/null && wait "$errand_pid" 2>/dev/null
  [ -n "$mock_pid" ] && kill "$mock_pid" 2>/dev/null && wait "$mock_pid" 2>/dev/null
  sup up || printf 'mock-session: could not start the supervised errand again; run errand-box up\n' >&2
}
trap cleanup EXIT
trap 'exit 2' INT TERM

"$mock" --listen 127.0.0.1:0 --port-file "$out/port" >"$out/mock.log" 2>&1 &
mock_pid=$!
i=0
until [ -s "$out/port" ]; do
  i=$((i + 1))
  [ "$i" -le 20 ] || die "discord-mock wrote no port in 20 s: $(tail -n 3 "$out/mock.log")"
  kill -0 "$mock_pid" 2>/dev/null || die "discord-mock stopped: $(tail -n 3 "$out/mock.log")"
  sleep 1
done
port=$(tr -d ' \n' <"$out/port")
base=http://127.0.0.1:$port
printf 'discord-mock: %s\n' "$base"

sup down || die "cannot stop the supervised errand"
# The environment the service gives errand (src/supervise.rs): its PATH,
# with the bailey shim first, and the proxy settings setup wrote.
# shellcheck source=/dev/null
if [ -r /etc/errand-box/proxy.env ]; then set -a; . /etc/errand-box/proxy.env; set +a; fi
PATH=/usr/local/lib/errand-box/shim:/usr/local/lib/errand-box/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  "$eb" __as "$user" -- /usr/bin/env ERRAND_DISCORD_MOCK_URL="$base" "$eb" serve -v >"$out/errand.log" 2>&1 &
errand_pid=$!
i=0
until grep -q 'accepting messages' "$out/errand.log" 2>/dev/null; do
  i=$((i + 1))
  kill -0 "$errand_pid" 2>/dev/null || die "errand stopped before it accepted messages: $(tail -n 5 "$out/errand.log")" 1
  [ "$i" -le 120 ] || die "errand did not accept messages in 120 s: $(tail -n 5 "$out/errand.log")" 1
  sleep 1
done
printf 'errand: accepting messages\n'

# PROMPT as a JSON string: backslashes and double quotes escaped.
json_prompt=$(printf '%s' "$prompt" | sed 's/\\/\\\\/g; s/"/\\"/g')
curl -fsS --noproxy '*' -H 'Content-Type: application/json' \
  -d "{\"content\":\"$json_prompt\"}" "$base/_mock/messages" >"$out/posted.json" ||
  die "the stand-in refused the message: $(cat "$out/posted.json" 2>/dev/null)"
printf 'posted: %s\n' "$prompt"

# The journal's REST calls, one line each: METHOD ROUTE STATUS PATH.
calls() {
  python3 -I -c '
import json, sys
for e in json.load(open(sys.argv[1])):
    if e.get("kind") == "rest":
        print(e["method"], e.get("route") or "(no route)", e["status"], e["path"])
' "$out/journal.json" >"$out/calls.txt"
}

# Wait for a thread, then for a post in a channel other than the served
# one, which is the channel the stand-in put the message in.
channel=$(grep -o '"channel_id":"[0-9]*"' "$out/posted.json" | head -n 1 | tr -dc '0-9')
[ -n "$channel" ] || die "the stand-in's answer names no channel: $(cat "$out/posted.json")"
thread=""
posted=""
i=0
while [ "$i" -lt "$wait_s" ]; do
  curl -fsS --noproxy '*' "$base/_mock/journal" >"$out/journal.json" 2>/dev/null && calls
  thread=$(grep -E '^POST [^ ]*/threads 20[01] ' "$out/calls.txt" | head -n 1)
  if [ -n "$thread" ]; then
    posted=$(grep -E '^POST /api/v10/channels/\{channel_id\}/messages 200 ' "$out/calls.txt" |
      grep -v " /api/v10/channels/$channel/messages\$" | head -n 1)
    [ -n "$posted" ] && break
  fi
  kill -0 "$errand_pid" 2>/dev/null || break
  i=$((i + 2))
  sleep 2
done
curl -fsS --noproxy '*' "$base/_mock/gaps" >"$out/gaps.json" 2>/dev/null

printf '\nREST calls errand made, by method, route and status:\n'
cut -d' ' -f1-3 "$out/calls.txt" | sort | uniq -c
printf '\nwhat errand asked for that the stand-in does not implement: %s\n' "$(cat "$out/gaps.json")"
printf '\nthe last lines errand logged:\n'
tail -n 15 "$out/errand.log"
if [ -n "$thread" ] && [ -n "$posted" ]; then
  printf '\nPASS errand opened a thread and posted in it; logs in %s\n' "$out"
  exit 0
fi
[ -n "$thread" ] || printf '\nFAIL no thread was opened in %s s; logs in %s\n' "$wait_s" "$out"
[ -z "$thread" ] || printf '\nFAIL a thread was opened, and nothing was posted in it in %s s; logs in %s\n' "$wait_s" "$out"
exit 1
