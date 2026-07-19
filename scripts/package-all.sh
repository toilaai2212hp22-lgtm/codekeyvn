#!/usr/bin/env bash
# Build packages for the current distro family.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
source "$HOME/.cargo/env" 2>/dev/null || true

echo "==> Release binaries…"
cargo build --release -p codekey-cli -p codekey-ibus -p codekey-tray -p codekey-ffi

if command -v dpkg-deb >/dev/null 2>&1; then
  echo "==> Debian/Ubuntu .deb"
  ./scripts/build-deb.sh
fi

if command -v makepkg >/dev/null 2>&1; then
  echo "==> Arch PKGBUILD (local)"
  echo "    cd packaging/arch && CODEKEY_ROOT=$ROOT makepkg -sf"
fi

if command -v rpmbuild >/dev/null 2>&1; then
  echo "==> Fedora RPM (local)"
  echo "    rpmbuild -ba --define \"codekey_root $ROOT\" packaging/fedora/codekey.spec"
fi

echo
echo "Done. Artifacts:"
ls -lh dist/*.deb 2>/dev/null || true
echo
echo "Install matrix:"
echo "  Ubuntu/Debian/Mint/Pop  →  sudo dpkg -i dist/codekey_*.deb"
echo "  Arch                    →  makepkg -si  (packaging/arch)"
echo "  Fedora                  →  rpmbuild / dnf install local rpm"
echo "  KDE (any of above)      →  use Fcitx5: ./scripts/install-fcitx5.sh"
