#!/bin/bash
# Stops what start-isolated-ibus.sh / e2e/start-xephyr.sh / e2e/start-portal.sh started.
# The engine exits by itself a few seconds after its daemon disappears.
SANDBOX="${GTV_SANDBOX:-${TMPDIR:-/tmp}/gtv-sandbox}"
for f in pid xephyr.pid portal.pid; do
  [ -f "$SANDBOX/run/$f" ] && kill "$(cat "$SANDBOX/run/$f")" 2>/dev/null
done
rm -rf "$SANDBOX/run"
exit 0
