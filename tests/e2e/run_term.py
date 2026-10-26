#!/usr/bin/python3
"""Types into a real gnome-terminal (VTE) running `cat`, through the sandbox ibus, per mode.
The tty line discipline applies erase characters, so the file `cat` writes is what the shell
would have received."""
import os
import subprocess
import sys
import time

from common import *  # noqa: F401,F403

MODES = sys.argv[1].split(",") if len(sys.argv) > 1 else ["backspace", "forward-keys", "select", "preedit"]
DELAY = float(os.environ.get("GTV_E2E_DELAY", "0.04"))


def run_case(mode, typed, want):
    out = os.path.join(SANDBOX, "term-out.txt")
    open(out, "w").close()
    set_mode("vte_term", mode)
    cmd = ["/usr/bin/python3", os.path.join(HERE, "vte_term.py"), out]
    proc = subprocess.Popen(cmd, env=app_env(), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
    try:
        d = xdrive.open_display(DISPLAY)
        win = d.find("vte_term", timeout=30)
        time.sleep(1.0)
        d.focus(win)
        time.sleep(0.6)
        d.type(typed, DELAY)
        time.sleep(0.8)
        d.key("Return")
        time.sleep(0.5)
        d.key("d", ctrl=True)
        time.sleep(0.8)
        got = open(out).read().rstrip("\n")
    finally:
        try:
            os.killpg(proc.pid, 15)
        except ProcessLookupError:
            pass
        time.sleep(0.5)
    return report(got == want, f"terminal {mode}", typed, got, want)


def main():
    select_engine()
    fails = 0
    for mode in MODES:
        for typed, want in CASES + CASES_NL:
            fails += not run_case(mode, typed, want)
    print(f"\n{fails} failure(s)")
    sys.exit(1 if fails else 0)


main()
