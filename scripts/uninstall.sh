#!/bin/bash
set -eu
PREFIX="${PREFIX:-/usr}"
if [ -d "$PREFIX/lib/ibus-gtv" ] && [ ! -w "$PREFIX/lib/ibus-gtv" ]; then echo "Chạy: sudo $0" >&2; exit 1; fi
rm -f "$PREFIX/share/ibus/component/gtv.xml" "$PREFIX/share/applications/ibus-setup-gtv.desktop" "$PREFIX/share/applications/ibus-setup-gtv-telex.desktop" "$PREFIX/share/applications/ibus-setup-gtv-vni.desktop"
rm -f "$PREFIX/lib/systemd/user/gtv-update.service" "$PREFIX/lib/systemd/user/gtv-update.timer" "$PREFIX/lib/systemd/user/timers.target.wants/gtv-update.timer"
rm -rf "$PREFIX/lib/ibus-gtv" "$PREFIX/share/ibus-gtv"
echo "Đã gỡ GTV. Chạy 'ibus restart' và bỏ GTV khỏi Input Sources."
