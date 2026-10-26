#!/usr/bin/python3
"""Types into a textarea in a real Chrome through the sandbox ibus, per delivery mode.
The page mirrors the textarea into document.title, which Chrome publishes as an X property."""
import os
import shutil
import subprocess
import sys
import time

from common import *  # noqa: F401,F403

MODES = sys.argv[1].split(",") if len(sys.argv) > 1 else ["select", "forward-keys", "backspace", "surrounding"]
DELAY = float(os.environ.get("GTV_E2E_DELAY", "0.04"))
PROFILE = os.path.join(SANDBOX, "chrome-profile")


def read_value(d, win, quiet=0.8, limit=12.0):
    last, since, end = None, time.time(), time.time() + limit
    while time.time() < end:
        t = d.title(win)
        cur = t[2:].rsplit(" - Google Chrome", 1)[0] if t.startswith("v:") else ""
        if cur != last:
            last, since = cur, time.time()
        elif time.time() - since >= quiet:
            break
        time.sleep(0.15)
    return last or ""


def main():
    select_engine()
    fails = 0
    for mode in MODES:
        set_mode("google-chrome", mode)
        shutil.rmtree(PROFILE, ignore_errors=True)
        env = app_env()
        # Chrome only enables its input method when a session bus exists.
        if os.environ.get("GTV_E2E_PORTAL") != "1":
            env["DBUS_SESSION_BUS_ADDRESS"] = os.environ.get("GTV_REAL_DBUS", "unix:path=/run/user/%d/bus" % os.getuid())
        proc = subprocess.Popen(
            ["google-chrome", f"--user-data-dir={PROFILE}", "--no-first-run", "--no-default-browser-check",
             "--disable-extensions", "--disable-gpu", "--ozone-platform=x11", "--window-size=900,500",
             "--window-position=0,0", "--password-store=basic", f"file://{os.path.join(HERE, 'page.html')}"],
            env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
        try:
            d = xdrive.open_display(DISPLAY)
            d.fake_wm()
            win = d.find("google-chrome", timeout=40)
            time.sleep(3)
            d.activate(win)
            time.sleep(1)
            d.click(60, 40)
            time.sleep(1)
            for typed, want in CASES + CASES_NL:
                d.key("a", ctrl=True)
                d.key("BackSpace")
                time.sleep(0.8)
                d.type(typed, DELAY)
                got = read_value(d, win)
                # document.title cannot hold a newline; Chrome turns it into a space.
                want = want.replace("\n", " ").rstrip()  # document.title also trims the end
                fails += not report(got == want, f"chrome {mode}", typed, got, want)
        finally:
            try:
                os.killpg(proc.pid, 15)
            except ProcessLookupError:
                pass
            time.sleep(1.5)
    print(f"\n{fails} failure(s)")
    sys.exit(1 if fails else 0)


main()
