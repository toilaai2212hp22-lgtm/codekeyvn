#!/usr/bin/env bash
# Build Rust engine + Fcitx5 addon and install.
# Supports: Ubuntu/Debian/Mint, Fedora, Arch (detects package manager).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
source "$HOME/.cargo/env" 2>/dev/null || true

need_pkgs() {
  echo
  echo "Thiếu Fcitx5 dev headers. Cài theo distro:"
  echo "  Debian/Ubuntu/Mint/Pop:  sudo apt install fcitx5 libfcitx5core-dev libfcitx5utils-dev cmake g++ pkg-config extra-cmake-modules"
  echo "  Fedora:                  sudo dnf install fcitx5 fcitx5-devel cmake gcc-c++ pkgconf-pkg-config extra-cmake-modules"
  echo "  Arch:                    sudo pacman -S fcitx5 fcitx5-qt fcitx5-gtk fcitx5-configtool cmake extra-cmake-modules base-devel"
  echo
}

install_deps_hint() {
  if command -v apt-get >/dev/null 2>&1; then
    echo "Gợi ý: sudo apt install -y fcitx5 libfcitx5core-dev libfcitx5utils-dev cmake g++ pkg-config"
  elif command -v dnf >/dev/null 2>&1; then
    echo "Gợi ý: sudo dnf install -y fcitx5 fcitx5-devel cmake gcc-c++ pkgconf-pkg-config"
  elif command -v pacman >/dev/null 2>&1; then
    echo "Gợi ý: sudo pacman -S --needed fcitx5 fcitx5-qt fcitx5-gtk cmake extra-cmake-modules base-devel"
  fi
}

echo "==> cargo build libcodekey + CLI…"
cargo build --release -p codekey-ffi -p codekey-cli

# Detect fcitx5 cmake package
if ! cmake --find-package -DNAME=Fcitx5Core -DCOMPILER_ID=GNU -DLANGUAGE=CXX -DMODE=EXIST 2>/dev/null; then
  if [[ ! -d /usr/lib/cmake/Fcitx5Core && ! -d /usr/lib64/cmake/Fcitx5Core && ! -d /usr/share/cmake/Fcitx5Core ]]; then
    echo "Chưa tìm thấy Fcitx5Core CMake package."
    need_pkgs
    install_deps_hint
    echo "libcodekey đã build: $ROOT/target/release/libcodekey.so"
    exit 1
  fi
fi

BUILD="$ROOT/fcitx5-addon/build"
rm -rf "$BUILD"
cmake -S "$ROOT/fcitx5-addon" -B "$BUILD" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX=/usr \
  -DCODEKEY_ROOT="$ROOT" \
  -DCODEKEY_LIB_DIR="$ROOT/target/release" \
  -DCODEKEY_INCLUDE_DIR="$ROOT/include"

cmake --build "$BUILD" -j"$(nproc 2>/dev/null || echo 2)"

echo "==> Install (cần quyền ghi /usr)…"
if [[ "${EUID:-}" -eq 0 ]]; then
  cmake --install "$BUILD"
else
  sudo cmake --install "$BUILD"
fi

# ldconfig so libcodekey.so is found
if command -v ldconfig >/dev/null 2>&1; then
  sudo ldconfig 2>/dev/null || true
fi

echo
echo "✓ Fcitx5 CodeKey đã cài."
echo
echo "Bật Fcitx5 làm IM mặc định:"
echo "  # Debian/Ubuntu"
echo "  im-config -n fcitx5"
echo "  # hoặc trong KDE System Settings → Input Method → Fcitx 5"
echo
echo "Env (thêm ~/.pam_environment hoặc /etc/environment):"
echo "  GTK_IM_MODULE=fcitx"
echo "  QT_IM_MODULE=fcitx"
echo "  XMODIFIERS=@im=fcitx"
echo "  SDL_IM_MODULE=fcitx"
echo
echo "Khởi động lại session, rồi:"
echo "  fcitx5 -r"
echo "  fcitx5-configtool   # Add → CodeKey / CodeKey VNI"
echo "  # phím tắt mặc định thường Super+Space hoặc Ctrl+Space"
echo
echo "Test engine (không cần Fcitx):"
echo "  $ROOT/target/release/codekey transform \"cuar\""
