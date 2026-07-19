#!/usr/bin/env bash
# Build a simple .deb for Ubuntu/Debian/Mint (amd64).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
source "$HOME/.cargo/env" 2>/dev/null || true

VERSION="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
ARCH="amd64"
PKG="codekey_${VERSION}_${ARCH}"
DIST="$ROOT/dist/$PKG"

echo "==> Building release binaries…"
cargo build --release -p codekey-cli -p codekey-ibus -p codekey-tray -p codekey-ffi

echo "==> Staging $PKG …"
rm -rf "$DIST"
mkdir -p "$DIST/DEBIAN"
mkdir -p "$DIST/usr/bin"
mkdir -p "$DIST/usr/lib"
mkdir -p "$DIST/usr/libexec"
mkdir -p "$DIST/usr/share/ibus/component"
mkdir -p "$DIST/usr/share/applications"
mkdir -p "$DIST/etc/xdg/autostart"
mkdir -p "$DIST/usr/share/doc/codekey"
mkdir -p "$DIST/usr/include"

install -m755 target/release/codekey "$DIST/usr/bin/codekey"
install -m755 target/release/codekey-tray "$DIST/usr/bin/codekey-tray"
install -m755 target/release/ibus-engine-codekey "$DIST/usr/libexec/ibus-engine-codekey"
# Shared engine for Fcitx5 (install fcitx addon separately via install-fcitx5.sh)
if [[ -f target/release/libcodekey.so ]]; then
  install -m755 target/release/libcodekey.so "$DIST/usr/lib/libcodekey.so"
  install -m644 include/codekey.h "$DIST/usr/include/codekey.h"
fi

# Component XML points at libexec path
sed -e "s|@EXE@|/usr/libexec/ibus-engine-codekey|g" \
    -e "s|@ICON@|input-keyboard|g" \
    data/ibus/codekey.xml.in > "$DIST/usr/share/ibus/component/codekey.xml"

install -m644 data/desktop/codekey-tray.desktop "$DIST/usr/share/applications/codekey-tray.desktop"
install -m644 data/desktop/codekey-tray.desktop "$DIST/etc/xdg/autostart/codekey-tray.desktop"

cat > "$DIST/usr/share/doc/codekey/copyright" <<EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: codekey
Source: https://github.com/example/codekey

Files: *
Copyright: CodeKey Contributors
License: MIT
EOF

cat > "$DIST/usr/share/doc/codekey/changelog.Debian" <<EOF
codekey ($VERSION) unstable; urgency=medium

  * Release $VERSION — IBus engine, tray, CLI.

 -- CodeKey Contributors <codekey@localhost>  $(date -R)
EOF
gzip -9n -f "$DIST/usr/share/doc/codekey/changelog.Debian"

# Control + scripts
SIZE_KB="$(du -sk "$DIST/usr" | cut -f1)"
cat > "$DIST/DEBIAN/control" <<EOF
Package: codekey
Version: $VERSION
Section: utils
Priority: optional
Architecture: $ARCH
Depends: ibus, libc6
Recommends: gir1.2-ayatanaappindicator3-0.1 | gir1.2-appindicator3-0.1
Maintainer: CodeKey Contributors <codekey@localhost>
Installed-Size: $SIZE_KB
Homepage: https://github.com/example/codekey
Description: Vietnamese input method for Linux (UniKey-like)
 CodeKey is a Vietnamese Telex/VNI input method for IBus with a system
 tray toggle and a terminal CLI. Works on Ubuntu, Debian, and Linux Mint.
EOF

cat > "$DIST/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
if command -v ibus >/dev/null 2>&1; then
  # Merge user+system components into cache (best-effort).
  export IBUS_COMPONENT_PATH="${IBUS_COMPONENT_PATH:-}:/usr/share/ibus/component"
  ibus write-cache 2>/dev/null || true
fi
if [ -x /usr/bin/update-desktop-database ]; then
  update-desktop-database -q /usr/share/applications 2>/dev/null || true
fi
echo "CodeKey installed. Run: ibus engine codekey   or open CodeKey from the menu/tray."
echo "You may need: ibus restart   (or log out/in)"
exit 0
EOF
chmod 755 "$DIST/DEBIAN/postinst"

cat > "$DIST/DEBIAN/prerm" <<'EOF'
#!/bin/sh
set -e
# Switch away from CodeKey if it is active
if command -v ibus >/dev/null 2>&1; then
  cur=$(ibus engine 2>/dev/null || true)
  case "$cur" in
    codekey|codekey-vni) ibus engine xkb:us::eng 2>/dev/null || true ;;
  esac
fi
exit 0
EOF
chmod 755 "$DIST/DEBIAN/prerm"

echo "==> dpkg-deb …"
mkdir -p "$ROOT/dist"
dpkg-deb --root-owner-group --build "$DIST" "$ROOT/dist/${PKG}.deb"

echo
echo "✓ Built: $ROOT/dist/${PKG}.deb"
echo "  Install:  sudo dpkg -i dist/${PKG}.deb"
echo "  Remove:   sudo dpkg -r codekey"
ls -lh "$ROOT/dist/${PKG}.deb"
