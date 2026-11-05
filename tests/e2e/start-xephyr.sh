#!/bin/bash
# Nested X server for end-to-end tests; its input is fully separate from the desktop session.
# Hidden by default: Xephyr renders into a host window that is never mapped (hidden_parent.py), so
# (-parent has to come before -screen) nothing appears on the desktop and no focus is taken. GTV_E2E_VISIBLE=1 shows a normal window.
DISP="${GTV_E2E_DISPLAY:-:77}"
SANDBOX="${GTV_SANDBOX:?set GTV_SANDBOX}"
HERE="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$SANDBOX/run"
if [ -f "$SANDBOX/run/xephyr.pid" ] && kill -0 "$(cat "$SANDBOX/run/xephyr.pid")" 2>/dev/null; then exit 0; fi
PARENT=()
if [ "${GTV_E2E_VISIBLE:-0}" != 1 ]; then
  setsid -f /usr/bin/python3 "$HERE/hidden_parent.py" 1100 700 > "$SANDBOX/run/parent.id" 2>"$SANDBOX/run/parent.log" < /dev/null
  for _ in $(seq 30); do [ -s "$SANDBOX/run/parent.id" ] && break; sleep 0.1; done
  PARENT=(-parent "$(head -1 "$SANDBOX/run/parent.id")")
  pgrep -nf "hidden_parent.py 1100 700" > "$SANDBOX/run/parent.pid"
fi
setsid -f nice -n 19 Xephyr "$DISP" "${PARENT[@]}" -screen 1100x700 -ac -no-host-grab -title "GTV e2e (safe to ignore)" >"$SANDBOX/run/xephyr.log" 2>&1 < /dev/null
sleep 1
pgrep -nx Xephyr > "$SANDBOX/run/xephyr.pid"
