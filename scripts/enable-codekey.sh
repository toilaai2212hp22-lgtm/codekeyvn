#!/usr/bin/env bash
# Re-register + switch to CodeKey without rebuilding.
set -euo pipefail

PREFIX="${PREFIX:-$HOME/.local}"
COMP_DIR="$PREFIX/share/ibus/component"
METHOD="${1:-telex}" # telex | vni

if [[ ! -f "$COMP_DIR/codekey.xml" ]]; then
  echo "CodeKey chưa cài. Chạy: ./scripts/install-ibus.sh" >&2
  exit 1
fi

export PATH="$PREFIX/bin:$PATH"
export IBUS_COMPONENT_PATH="${COMP_DIR}:/usr/share/ibus/component${IBUS_COMPONENT_PATH:+:${IBUS_COMPONENT_PATH}}"

ibus write-cache 2>/dev/null || true

case "$METHOD" in
  telex|codekey) ENGINE=codekey ;;
  vni|codekey-vni) ENGINE=codekey-vni ;;
  *)
    echo "Usage: $0 [telex|vni]" >&2
    exit 1
    ;;
esac

if ! ibus list-engine 2>/dev/null | grep -q "$ENGINE"; then
  echo "Engine '$ENGINE' chưa có trong cache. Restart IBus…"
  timeout 12 ibus restart 2>/dev/null || true
  sleep 1
fi

ibus engine "$ENGINE"
echo "Active engine: $(ibus engine)"
