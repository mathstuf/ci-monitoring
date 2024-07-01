#!/bin/sh

set -e

readonly version="0.29.1"
readonly sha256sum="be2be986abde1f7b9544f217acce4d1699a04bc3b09e17c05b3fbd239d52456a"
readonly filename="cargo-tarpaulin-x86_64-unknown-linux-musl"
readonly tarball="$filename.tar.gz"

cd .gitlab

echo "$sha256sum  $tarball" > tarpaulin.sha256sum
curl -OL "https://github.com/xd009642/tarpaulin/releases/download/$version/$tarball"
sha256sum --check tarpaulin.sha256sum
tar xf "$tarball"
