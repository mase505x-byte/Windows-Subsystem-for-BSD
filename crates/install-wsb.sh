#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="${WSB_BUILD_DIR:-$ROOT/build/install}"
INSTALL_DIR="${WSB_INSTALL_DIR:-${HOME:?HOME must be set}/.local/bin}"

if ! command -v cmake >/dev/null 2>&1; then
  echo "error: CMake is required to build WSB." >&2
  exit 1
fi

if [[ ! -f "$ROOT/CMakeLists.txt" || ! -f "$ROOT/cli/main.cpp" ]]; then
  echo "error: run this script from a WSB source checkout." >&2
  exit 1
fi

echo "==> configuring WSB CLI"
cmake -S "$ROOT" -B "$BUILD_DIR" -DCMAKE_BUILD_TYPE=Release

echo "==> building WSB CLI"
cmake --build "$BUILD_DIR" --config Release

binary=""
for candidate in \
  "$BUILD_DIR/wsb" \
  "$BUILD_DIR/wsb.exe" \
  "$BUILD_DIR/Release/wsb.exe" \
  "$BUILD_DIR/Release/wsb"
do
  if [[ -f "$candidate" ]]; then
    binary="$candidate"
    break
  fi
done

if [[ -z "$binary" ]]; then
  echo "error: build completed, but the wsb executable was not found." >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"
installed_name="wsb"
[[ "$binary" == *.exe ]] && installed_name="wsb.exe"
install -m 755 "$binary" "$INSTALL_DIR/$installed_name"

echo "==> checking installed CLI"
"$INSTALL_DIR/$installed_name" version

echo "Installed to: $INSTALL_DIR/$installed_name"
case ":${PATH:-}:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "Add it to PATH if needed: export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac
