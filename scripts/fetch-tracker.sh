#!/bin/sh
# fetch-tracker.sh - fetch one GitHub repository's tracker, read-only, into
# the corpus layout of docs/methodology/research.md section 3.1.
#
#   sh scripts/fetch-tracker.sh SLUG OUT_DIR
#
# SLUG is the owner and the name with a slash between, as QaidVoid/errand.
#
# Writes OUT_DIR/api/: repo.json, issues.json (pull requests included, both
# states), comments.json, review-comments.json, releases.json, tags.json.
# Discussions are GraphQL only and are not fetched; PROVENANCE.md says so.
#
# Every call is a GET. The default route is the read-only proxy of
# docs/hosted-sessions.md section 7, which needs no credential and sends
# none; set EB_GH_API=https://api.github.com to go direct.
# Needs: curl, jq.
# Exit codes: 0 every source fetched, 1 a source failed, 2 could not run.
set -u
die() { printf 'fetch-tracker: %s\n' "$1" >&2; exit "${2:-1}"; }
for t in curl jq; do command -v "$t" >/dev/null 2>&1 || die "$t is missing" 2; done
slug=${1:-}
out=${2:-}
case $slug in */*) ;; *) die "usage: fetch-tracker.sh SLUG OUT_DIR" 2 ;; esac
[ -n "$out" ] || die "usage: fetch-tracker.sh SLUG OUT_DIR" 2
api=${EB_GH_API:-https://api.gh.pkgforge.dev}
mkdir -p "$out/api" || exit 2
failed=0

get() { # path, file: one GET, the body to file; fails on a non-2xx answer
  code=$(curl -sS --max-time 120 -o "$2" -w '%{http_code}' "$api/$1") || return 1
  case $code in 2??) return 0 ;; *) printf 'fetch-tracker: %s answered %s\n' "$1" "$code" >&2; return 1 ;; esac
}

pages() { # path with no query, extra query, file: every page, merged into one array
  dir=$(mktemp -d) || return 1
  n=1
  while :; do
    get "$1?per_page=100&page=$n$2" "$dir/$n.json" || { rm -rf "$dir"; return 1; }
    [ "$(jq 'length' "$dir/$n.json")" -gt 0 ] || break
    n=$((n + 1))
  done
  # The pages in numeric order, so the merged array keeps the API's order.
  i=1
  : >"$dir/list"
  while [ "$i" -le "$n" ]; do printf '%s\n' "$dir/$i.json" >>"$dir/list"; i=$((i + 1)); done
  # shellcheck disable=SC2046 # one path per line, none with a space
  jq -s 'add' $(cat "$dir/list") >"$3"
  rc=$?
  rm -rf "$dir"
  return $rc
}

get "repos/$slug" "$out/api/repo.json" || failed=1
pages "repos/$slug/issues" "&state=all" "$out/api/issues.json" || failed=1
pages "repos/$slug/issues/comments" "" "$out/api/comments.json" || failed=1
pages "repos/$slug/pulls/comments" "" "$out/api/review-comments.json" || failed=1
pages "repos/$slug/releases" "" "$out/api/releases.json" || failed=1
pages "repos/$slug/tags" "" "$out/api/tags.json" || failed=1

for f in issues comments review-comments releases tags; do
  [ -s "$out/api/$f.json" ] && printf '%s\t%s\n' "$f" "$(jq 'length' "$out/api/$f.json")"
done
exit "$failed"
