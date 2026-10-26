#!/bin/bash
# Nested X server for end-to-end tests; its input is fully separate from the desktop session.
DISP="${GTV_E2E_DISPLAY:-:77}"
SANDBOX="${GTV_SANDBOX:?set GTV_SANDBOX}"
mkdir -p "$SANDBOX/run"
if [ -f "$SANDBOX/run/xephyr.pid" ] && kill -0 "$(cat "$SANDBOX/run/xephyr.pid")" 2>/dev/null; then exit 0; fi
setsid -f Xephyr "$DISP" -screen 1100x700 -ac -no-host-grab -title "GTV e2e (safe to ignore)" >"$SANDBOX/run/xephyr.log" 2>&1 < /dev/null
sleep 1
pgrep -nx Xephyr > "$SANDBOX/run/xephyr.pid"
