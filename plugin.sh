#!/bin/sh
here="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
sys="$(uname -s)"
case "$sys" in
MINGW* | MSYS* | CYGWIN*) rel="x86_64-pc-windows-msvc/gray-loop.exe" ;;
Darwin*)
    case "$(uname -m)" in
    arm64) rel="aarch64-apple-darwin/gray-loop" ;;
    *) rel="x86_64-apple-darwin/gray-loop" ;;
    esac ;;
*)
    case "$(uname -m)" in
    aarch64 | arm64) rel="aarch64-unknown-linux-musl/gray-loop" ;;
    *) rel="x86_64-unknown-linux-musl/gray-loop" ;;
    esac ;;
esac
if [ -x "$here/$rel" ]; then exec "$here/$rel" "$@"; fi
for cand in "$here/target/release/gray-loop.exe" "$here/target/release/gray-loop" "$here/target/debug/gray-loop.exe" "$here/target/debug/gray-loop"; do
    if [ -x "$cand" ]; then exec "$cand" "$@"; fi
done
echo "gray-loop binary not found in $here" >&2
exit 1
