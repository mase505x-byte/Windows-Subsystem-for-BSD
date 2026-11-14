#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="${ROOT}/dist/wsb-package"
PKG_NAME="wsb-0.2.0-linux-x86_64"

mkdir -p "${OUT_DIR}/bin" "${OUT_DIR}/lib" "${OUT_DIR}/share/wsb"

echo "==> building Rust workspace"
cargo build --workspace --release

echo "==> configuring CMake CLI"
cmake -S "${ROOT}" -B "${ROOT}/build" -DCMAKE_BUILD_TYPE=Release

echo "==> building CLI"
cmake --build "${ROOT}/build" --config Release

echo "==> staging package content"
cp "${ROOT}/build/wsb" "${OUT_DIR}/bin/wsb"

if [ -f "${ROOT}/target/release/libwsb_asm.so" ]; then
  cp "${ROOT}/target/release/libwsb_asm.so" "${OUT_DIR}/lib/"
fi

if [ -f "${ROOT}/target/release/libwsb_asm.a" ]; then
  cp "${ROOT}/target/release/libwsb_asm.a" "${OUT_DIR}/lib/"
fi

cp "${ROOT}/README.md" "${OUT_DIR}/share/wsb/README.md"

cat > "${OUT_DIR}/bin/wsb-env.sh" <<'EOF'
#!/usr/bin/env bash
export WSB_HOME="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export LD_LIBRARY_PATH="${WSB_HOME}/lib:${LD_LIBRARY_PATH:-}"
EOF
chmod +x "${OUT_DIR}/bin/wsb-env.sh"

echo "==> creating archive"
mkdir -p "${ROOT}/dist"
tar -czf "${ROOT}/dist/${PKG_NAME}.tar.gz" -C "${OUT_DIR}" .

echo "Package created at: ${ROOT}/dist/${PKG_NAME}.tar.gz"
