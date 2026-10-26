#!/bin/bash
# Starts a private ibus-daemon (own bus address, own config/data dirs) that only knows the
# GTV engine from target/debug. Never touches the ibus-daemon of the desktop session.
#
#   eval "$(tests/start-isolated-ibus.sh)"     # exports IBUS_ADDRESS & friends
#   tests/stop-isolated-ibus.sh
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${GTV_BIN:-$ROOT/target/debug/ibus-engine-gtv}"
SANDBOX="${GTV_SANDBOX:-${TMPDIR:-/tmp}/gtv-sandbox}"
mkdir -p "$SANDBOX"/{config/gtv,home/.local/share/ibus/component,cache,run}
"$ROOT/scripts/render-component.sh" "$BIN" "$ROOT/data/gtv.svg" 0.0.0-test > "$SANDBOX/home/.local/share/ibus/component/gtv.xml"
export XDG_CONFIG_HOME="$SANDBOX/config" XDG_DATA_HOME="$SANDBOX/data" XDG_CACHE_HOME="$SANDBOX/cache"
export IBUS_ADDRESS="unix:path=$SANDBOX/run/ibus.sock"
if [ -f "$SANDBOX/run/pid" ] && kill -0 "$(cat "$SANDBOX/run/pid")" 2>/dev/null; then :; else
  rm -f "$SANDBOX/run/ibus.sock"
  # setsid -f: a plain background job is reaped when the calling shell exits.
  # IBUS_COMPONENT_PATH replaces the component search path, so keep the system dir in it.
  env -u DISPLAY -u WAYLAND_DISPLAY ${GTV_DISPLAY:+DISPLAY=$GTV_DISPLAY} IBUS_COMPONENT_PATH="$SANDBOX/home/.local/share/ibus/component:/usr/share/ibus/component" GTV_LOG="${GTV_LOG:-trace}" GTV_LOG_FILE="$SANDBOX/run/engine.log" setsid -f ibus-daemon ${GTV_DISPLAY:+-x} -s -t none -a "$IBUS_ADDRESS" >"$SANDBOX/run/daemon.log" 2>&1 < /dev/null
  sleep 0.3; pgrep -nx ibus-daemon > "$SANDBOX/run/pid" || true
  for _ in $(seq 50); do [ -S "$SANDBOX/run/ibus.sock" ] && break; sleep 0.1; done
fi
cat <<ENV
export XDG_CONFIG_HOME='$XDG_CONFIG_HOME' XDG_DATA_HOME='$XDG_DATA_HOME' XDG_CACHE_HOME='$XDG_CACHE_HOME'
export IBUS_ADDRESS='$IBUS_ADDRESS' GTV_SANDBOX='$SANDBOX'
ENV
