#!/bin/sh

set -e

readonly version="0.31.2"
readonly sha256sum="47781f68fd98db830983a59020bbbaf0841322b362c8d7a7634b7d88128a22ba"
readonly filename="cargo-tarpaulin-x86_64-unknown-linux-musl"
readonly tarball="$filename.tar.gz"

cd .gitlab

echo "$sha256sum  $tarball" > tarpaulin.sha256sum
curl -OL "https://github.com/xd009642/tarpaulin/releases/download/$version/$tarball"
sha256sum --check tarpaulin.sha256sum
tar xf "$tarball"
