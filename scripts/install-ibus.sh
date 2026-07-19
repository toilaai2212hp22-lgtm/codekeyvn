#!/usr/bin/env bash
# Build and install CodeKey IBus engine (user-local, no root required).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PREFIX="${PREFIX:-$HOME/.local}"
cd "$ROOT"

source "$HOME/.cargo/env" 2>/dev/null || true

echo "==> Building CodeKey (release)…"
cargo build --release -p codekey-cli -p codekey-ibus -p codekey-tray

BIN_DIR="$PREFIX/bin"
COMP_DIR="$PREFIX/share/ibus/component"
APP_DIR="$PREFIX/share/applications"
AUTO_DIR="$PREFIX/etc/xdg/autostart"
mkdir -p "$BIN_DIR" "$COMP_DIR" "$APP_DIR"
mkdir -p "$HOME/.config/autostart"

install -m755 target/release/codekey "$BIN_DIR/codekey"
install -m755 target/release/ibus-engine-codekey "$BIN_DIR/ibus-engine-codekey"
install -m755 target/release/codekey-tray "$BIN_DIR/codekey-tray"
install -m644 "$ROOT/data/desktop/codekey-tray.desktop" "$APP_DIR/codekey-tray.desktop"
# User autostart (no root)
sed "s|^Exec=.*|Exec=$BIN_DIR/codekey-tray|" \
  "$ROOT/data/desktop/codekey-tray.desktop" > "$HOME/.config/autostart/codekey-tray.desktop"

ICON_NAME="input-keyboard"
EXE="$BIN_DIR/ibus-engine-codekey"
sed -e "s|@EXE@|$EXE|g" -e "s|@ICON@|$ICON_NAME|g" \
  "$ROOT/data/ibus/codekey.xml.in" > "$COMP_DIR/codekey.xml"

# Optional system-wide component (helps some DEs without IBUS_COMPONENT_PATH).
SYS_COMP="/usr/share/ibus/component/codekey.xml"
if [[ -w /usr/share/ibus/component ]] 2>/dev/null; then
  cp "$COMP_DIR/codekey.xml" "$SYS_COMP"
  echo "==> System component: $SYS_COMP"
elif command -v sudo >/dev/null 2>&1; then
  if sudo -n true 2>/dev/null; then
    sudo cp "$COMP_DIR/codekey.xml" "$SYS_COMP"
    echo "==> System component: $SYS_COMP"
  fi
fi

# Register with IBus cache (user path + system path).
export IBUS_COMPONENT_PATH="${COMP_DIR}:/usr/share/ibus/component${IBUS_COMPONENT_PATH:+:${IBUS_COMPONENT_PATH}}"
echo "==> Refreshing IBus registry (IBUS_COMPONENT_PATH)…"
ibus write-cache 2>/dev/null || true

# Ensure ~/.local/bin is on PATH for this session.
export PATH="$BIN_DIR:$PATH"

# Soft-restart IBus so it picks up the new component.
if command -v ibus >/dev/null 2>&1; then
  echo "==> Restarting IBus…"
  timeout 12 ibus restart 2>/dev/null || true
  sleep 1
fi

# Preload Telex engine alongside US English (non-destructive if already set).
if command -v gsettings >/dev/null 2>&1; then
  current="$(gsettings get org.freedesktop.ibus.general preload-engines 2>/dev/null || echo "[]")"
  if [[ "$current" != *codekey* ]]; then
    # Keep existing engines, append codekey.
    if [[ "$current" == "[]" || "$current" == "@as []" ]]; then
      gsettings set org.freedesktop.ibus.general preload-engines "['xkb:us::eng', 'codekey']" || true
    else
      # shell-friendly: use Python for list merge
      python3 - <<'PY' || true
import subprocess, ast, re
raw = subprocess.check_output(
    ["gsettings", "get", "org.freedesktop.ibus.general", "preload-engines"],
    text=True,
).strip()
# gsettings prints like ['a', 'b']
try:
    engines = ast.literal_eval(raw)
except Exception:
    engines = ["xkb:us::eng"]
if "codekey" not in engines:
    engines.append("codekey")
out = "[" + ", ".join(repr(e) for e in engines) + "]"
subprocess.check_call(
    ["gsettings", "set", "org.freedesktop.ibus.general", "preload-engines", out]
)
print("preload-engines =", out)
PY
    fi
  fi

  # GNOME / Cinnamon / Mint input sources
  if gsettings list-keys org.gnome.desktop.input-sources &>/dev/null; then
    sources="$(gsettings get org.gnome.desktop.input-sources sources 2>/dev/null || echo "")"
    if [[ "$sources" != *codekey* ]]; then
      python3 - <<'PY' || true
import subprocess, ast
raw = subprocess.check_output(
    ["gsettings", "get", "org.gnome.desktop.input-sources", "sources"],
    text=True,
).strip()
try:
    sources = list(ast.literal_eval(raw))
except Exception:
    sources = [("xkb", "us")]
if ("ibus", "codekey") not in sources:
    sources.append(("ibus", "codekey"))
# gsettings format: [('xkb', 'us'), ('ibus', 'codekey')]
parts = []
for a, b in sources:
    parts.append(f"('{a}', '{b}')")
out = "[" + ", ".join(parts) + "]"
subprocess.check_call(
    ["gsettings", "set", "org.gnome.desktop.input-sources", "sources", out]
)
print("input-sources =", out)
PY
    fi
  fi
fi

echo
echo "==> Installed:"
echo "    $BIN_DIR/codekey"
echo "    $BIN_DIR/ibus-engine-codekey"
echo "    $BIN_DIR/codekey-tray"
echo "    $COMP_DIR/codekey.xml"
echo "    ~/.config/autostart/codekey-tray.desktop"
echo

# Start tray if not already running
if ! pgrep -x codekey-tray >/dev/null 2>&1; then
  nohup "$BIN_DIR/codekey-tray" >/dev/null 2>&1 &
  echo "==> Started codekey-tray"
fi
echo

if ibus list-engine 2>/dev/null | grep -q codekey; then
  echo "✓ CodeKey đã có trong IBus:"
  ibus list-engine 2>/dev/null | grep codekey
  echo
  if timeout 5 ibus engine codekey 2>/dev/null; then
    echo "✓ Đã chuyển engine hiện tại → $(ibus engine 2>/dev/null)"
  else
    echo "  (chưa switch được ngay — thử: ibus engine codekey)"
  fi
else
  echo "! Chưa thấy engine trong list. Thử:"
  echo "    export IBUS_COMPONENT_PATH=\"$COMP_DIR:/usr/share/ibus/component\""
  echo "    ibus write-cache && ibus restart"
  echo "    ibus list-engine | grep codekey"
fi

echo
echo "Cách dùng:"
echo "  • Super+Space  — đổi input method (nếu DE đã add CodeKey)"
echo "  • ibus engine codekey      — bật Telex"
echo "  • ibus engine codekey-vni  — bật VNI"
echo "  • ibus engine xkb:us::eng  — về English"
echo "  • codekey transform \"xin chaof\"  — test CLI"
echo
echo "Nên thêm vào ~/.profile (để IME + PATH ổn định):"
echo "  export PATH=\"$BIN_DIR:\$PATH\""
echo "  export GTK_IM_MODULE=ibus QT_IM_MODULE=ibus XMODIFIERS=@im=ibus"
echo "  export IBUS_COMPONENT_PATH=\"$COMP_DIR:/usr/share/ibus/component\${IBUS_COMPONENT_PATH:+:\$IBUS_COMPONENT_PATH}\""
