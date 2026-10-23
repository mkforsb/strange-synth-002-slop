#!/bin/sh
# Release web build that can be served from any subfolder, not just the web root.
#
# dx hardcodes root-absolute `/{base_path}/assets/...` URLs into index.html and
# the wasm-bindgen loader, so rewrite them to page-relative `./assets/...`.
#
# usage: tools/build_web.sh [extra dx build args]
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
public="$root/target/dx/oxide-app/release/web/public"

(cd "$root/crates/app" && dx build --platform web --release "$@")

sed -i 's#"/\./assets/#"./assets/#g' "$public/index.html" "$public"/assets/*.js

if grep -q '"/\./assets/' "$public/index.html" "$public"/assets/*.js; then
    echo "build_web.sh: root-absolute asset paths remain in $public" >&2
    exit 1
fi
echo "Relocatable build in $public"
