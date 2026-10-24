#!/bin/sh
# notices.sh - write THIRD-PARTY-NOTICES: the licence of every crate the
# release binary links, generated from Cargo.lock.
#
#   sh scripts/notices.sh OUT_FILE
#
# The binary links errand (T-028) and its crates, each under a licence that
# asks for its notice to go with the binary. A release publishes the file
# this writes beside the binaries.
#
# It downloads cargo-about for this machine, refuses bytes whose SHA-256 is
# not the pinned one, and runs it with scripts/notices/about.toml and
# about.hbs. A crate under a licence about.toml does not accept stops it.
# cargo-about reads Cargo.lock and the crates' licence files; it builds
# nothing. A new cargo-about is a deliberate edit of the three pins below,
# each read from the digest GitHub records for the release asset.
#
# Needs: curl, tar, sha256sum, cargo, and the network.
# Exit codes: 0 written, 1 generation failed, 2 could not run.
set -u
root=$(cd "$(dirname "$0")/.." && pwd)
out=${1:-}
[ -n "$out" ] || { echo "usage: notices.sh OUT_FILE" >&2; exit 2; }
for t in curl tar sha256sum cargo; do
  command -v "$t" >/dev/null 2>&1 || { echo "notices: $t is missing" >&2; exit 2; }
done

version=0.9.2
PINNED_SHA256_X86_64=9099a59e820c38a68b9d65f300662a567d56562f9a10f6aa4c7e86c17c2566af
PINNED_SHA256_AARCH64=af5169282fb6f84e13471493f405437e43ac517744c9ae12fbe2cdf0a6f0e5a8
case $(uname -m) in
  x86_64 | amd64) arch=x86_64 want=$PINNED_SHA256_X86_64 ;;
  aarch64 | arm64) arch=aarch64 want=$PINNED_SHA256_AARCH64 ;;
  *) echo "notices: no cargo-about for $(uname -m)" >&2; exit 2 ;;
esac
name=cargo-about-$version-$arch-unknown-linux-musl
url=https://github.com/EmbarkStudios/cargo-about/releases/download/$version/$name.tar.gz

t=$(mktemp -d) || exit 2
trap 'rm -rf "$t"' EXIT INT TERM
curl -fsSL --retry 3 -o "$t/a.tar.gz" "$url" || { echo "notices: cannot download $url" >&2; exit 2; }
got=$(sha256sum "$t/a.tar.gz" | cut -d' ' -f1)
[ "$got" = "$want" ] || { echo "notices: $url has sha256 $got, not the pinned $want" >&2; exit 1; }
tar -xzf "$t/a.tar.gz" -C "$t" || exit 2
bin=$(find "$t" -type f -name cargo-about -perm -u+x | head -n 1)
[ -n "$bin" ] || { echo "notices: no cargo-about in $url" >&2; exit 2; }

cd "$root" || exit 2
"$bin" about generate --locked --fail \
  -c scripts/notices/about.toml -o "$t/notices" scripts/notices/about.hbs || exit 1
[ -s "$t/notices" ] || { echo "notices: cargo-about wrote nothing" >&2; exit 1; }
cp "$t/notices" "$out" || exit 2
printf 'notices: %s, %s lines\n' "$out" "$(wc -l < "$out" | tr -d ' ')"
