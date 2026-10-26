#!/bin/bash
# Private session bus + ibus-portal bound to the sandbox daemon, so apps can be tested the way
# a desktop with IBUS_USE_PORTAL=1 runs them. Writes the bus address to $GTV_SANDBOX/run/portal-bus.
SANDBOX="${GTV_SANDBOX:?set GTV_SANDBOX}"
mkdir -p "$SANDBOX/run"
if [ -f "$SANDBOX/run/portal-bus" ] && [ -f "$SANDBOX/run/portal.pid" ] && kill -0 "$(cat "$SANDBOX/run/portal.pid")" 2>/dev/null; then exit 0; fi
ADDR=$(dbus-daemon --session --fork --print-address 2>/dev/null)
echo "$ADDR" > "$SANDBOX/run/portal-bus"
DBUS_SESSION_BUS_ADDRESS="$ADDR" setsid -f /usr/libexec/ibus-portal >"$SANDBOX/run/portal.log" 2>&1 < /dev/null
sleep 1
pgrep -nx ibus-portal > "$SANDBOX/run/portal.pid"
