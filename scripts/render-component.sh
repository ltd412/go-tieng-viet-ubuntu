#!/bin/sh
# Usage: render-component.sh <engine-binary> <icon> <version> [setup-program] > gtv.xml
set -eu
sed -e "s|@EXEC@|$1|" -e "s|@ICON@|$2|" -e "s|@VERSION@|$3|" -e "s|@SETUP@|${4:-$(dirname "$1")/ibus-setup-gtv}|" "$(dirname "$0")/../data/gtv.xml.in"
