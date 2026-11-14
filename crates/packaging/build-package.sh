#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo test --workspace
cargo build --workspace --release

cmake -S "$ROOT" -B "$ROOT/build/package" -DCMAKE_BUILD_TYPE=Release
cmake --build "$ROOT/build/package" --config Release

pkg_dir="$ROOT/dist/wsb-0.2.0-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"
mkdir -p "$pkg_dir/bin" "$pkg_dir/lib" "$pkg_dir/share/wsb"

cp "$ROOT/build/package/wsb" "$pkg_dir/bin/wsb" 2>/dev/null || cp "$ROOT/build/package/Release/wsb" "$pkg_dir/bin/wsb"
cp "$ROOT/README.md" "$pkg_dir/share/wsb/README.md"

for f in target/release/libwsb_asm.so target/release/libwsb_asm.a target/release/wsb_asm.dll target/release/wsb_asm.lib; do
  [ -f "$ROOT/$f" ] && cp "$ROOT/$f" "$pkg_dir/lib/"
done

mkdir -p "$ROOT/dist"
tar -czf "$ROOT/dist/wsb-0.2.0-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m).tar.gz" -C "$ROOT/dist" "$(basename "$pkg_dir")"
echo "Package ready: $ROOT/dist"
