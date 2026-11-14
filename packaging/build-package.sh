#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT/binaric.list"
BUILD_DIR="$ROOT/build/package"
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

if [[ ! -f "$MANIFEST" ]]; then
  echo "error: missing package descriptor: $MANIFEST" >&2
  exit 1
fi

name=""
version=""
while IFS='=' read -r key value; do
  case "$key" in
    name) name="$value" ;;
    version) version="$value" ;;
  esac
done < "$MANIFEST"
if [[ -z "$name" || -z "$version" ]]; then
  echo "error: binaric.list must define name and version" >&2
  exit 1
fi

for tool in cargo cmake tar; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "error: required build tool not found: $tool" >&2
    exit 1
  fi
done

cd "$ROOT"
echo "==> testing Rust workspace"
cargo test --workspace

echo "==> building Rust workspace (release)"
cargo build --workspace --release

echo "==> building CMake CLI (release)"
cmake -S "$ROOT" -B "$BUILD_DIR" -DCMAKE_BUILD_TYPE=Release
cmake --build "$BUILD_DIR" --config Release

cli=""
for candidate in "$BUILD_DIR/wsb" "$BUILD_DIR/wsb.exe" "$BUILD_DIR/Release/wsb.exe" "$BUILD_DIR/Release/wsb"; do
  if [[ -f "$candidate" ]]; then
    cli="$candidate"
    break
  fi
done
if [[ -z "$cli" ]]; then
  echo "error: CMake build succeeded but the wsb executable was not found" >&2
  exit 1
fi

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"
package_id="$name-$version-$os-$arch"
dist="$ROOT/dist"
stage="$(mktemp -d "${TMPDIR:-/tmp}/wsb-package.XXXXXX")"
trap 'rm -rf "$stage"' EXIT
pkg="$stage/$package_id"
mkdir -p "$pkg/bin" "$pkg/lib" "$pkg/share/wsb"
cp "$cli" "$pkg/bin/$(basename "$cli")"
cp "$ROOT/README.md" "$pkg/share/wsb/README.md"
cp "$MANIFEST" "$pkg/share/wsb/binaric.list"

# Bundle only the Rust assembler library artifacts that Cargo actually produced.
# The assembler emits bytes and does not execute them or provide a BSD runtime.
shopt -s nullglob
lib_patterns=(
  "$TARGET_DIR/release/libwsb_asm.so"
  "$TARGET_DIR/release/libwsb_asm.dylib"
  "$TARGET_DIR/release/libwsb_asm.a"
  "$TARGET_DIR/release/wsb_asm.dll"
  "$TARGET_DIR/release/wsb_asm.lib"
)
for artifact in "${lib_patterns[@]}"; do
  if [[ -f "$artifact" ]]; then
    cp "$artifact" "$pkg/lib/"
  fi
done
shopt -u nullglob

cat > "$pkg/share/wsb/PACKAGE-INFO.txt" <<INFO
$name $version
Built for: $os-$arch
Contents: C++ CLI, available wsb-asm Rust library artifacts, and project README.
This package does not include a BSD userland, shell runtime, or host server.
INFO

mkdir -p "$dist"
archive="$dist/$package_id.tar.gz"
tar -czf "$archive" -C "$stage" "$package_id"
if command -v sha256sum >/dev/null 2>&1; then
  (cd "$dist" && sha256sum "$(basename "$archive")" > "$(basename "$archive").sha256")
elif command -v shasum >/dev/null 2>&1; then
  (cd "$dist" && shasum -a 256 "$(basename "$archive")" > "$(basename "$archive").sha256")
fi

echo "Package created: $archive"
