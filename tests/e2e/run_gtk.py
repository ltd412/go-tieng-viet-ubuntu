#!/usr/bin/python3
"""Types into a real GTK3 widget through the real ibus-daemon + engine, per delivery mode."""
import os
import subprocess
import sys
import time

from common import *  # noqa: F401,F403

MODES = sys.argv[1].split(",") if len(sys.argv) > 1 else ["surrounding", "backspace", "select", "forward-keys", "preedit"]
DELAY = float(os.environ.get("GTV_E2E_DELAY", "0.04"))


def run(kind, mode):
    out = os.path.join(SANDBOX, f"gtk-{kind}.txt")
    fails = 0
    for typed, want in CASES + (CASES_NL if kind == "textview" else []):
        set_mode("gtk_edit", mode)
        proc = subprocess.Popen(["/usr/bin/python3", os.path.join(HERE, "gtk_edit.py"), kind, out], env=app_env(),
                                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            d = xdrive.open_display(DISPLAY)
            win = d.find("gtk_edit")
            time.sleep(0.5)
            d.focus(win)
            time.sleep(0.4)
            d.type(typed, DELAY)
            got = settle(out)
        finally:
            kill(proc)
        fails += not report(got == want, f"gtk-{kind} {mode}", typed, got, want)
    return fails


def main():
    select_engine()
    total = 0
    for kind in (("entry", "search", "textview") if os.environ.get("GTV_E2E_GTK4") == "1" else ("entry", "textview")):
        for mode in MODES:
            total += run(kind, mode)
    print(f"\n{total} failure(s)")
    sys.exit(1 if total else 0)


main()
