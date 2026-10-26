#!/usr/bin/python3
"""Types into a VS Code editor (Electron) through the sandbox ibus and saves the file."""
import os
import shutil
import subprocess
import sys
import time

from common import *  # noqa: F401,F403

DELAY = float(os.environ.get("GTV_E2E_DELAY", "0.04"))
WS = os.path.join(SANDBOX, "vscode-ws")
FILE = os.path.join(WS, "test.txt")


def main():
    select_engine()
    set_mode("code", "default")
    for d in ("vscode-data", "vscode-ext", "vscode-ws"):
        shutil.rmtree(os.path.join(SANDBOX, d), ignore_errors=True)
    os.makedirs(WS)
    open(FILE, "w").close()
    env = app_env()
    if os.environ.get("GTV_E2E_PORTAL") != "1":
        env["DBUS_SESSION_BUS_ADDRESS"] = "unix:path=/run/user/%d/bus" % os.getuid()
    proc = subprocess.Popen(
        ["code", f"--user-data-dir={os.path.join(SANDBOX, 'vscode-data')}",
         f"--extensions-dir={os.path.join(SANDBOX, 'vscode-ext')}", "--disable-extensions", "--disable-gpu",
         "--disable-workspace-trust", "--skip-welcome", "--skip-release-notes", "--new-window", FILE],
        env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
    fails = 0
    try:
        d = xdrive.open_display(DISPLAY)
        d.fake_wm()
        win = d.find("code", timeout=60)
        time.sleep(8)
        d.activate(win)
        time.sleep(1)
        d.click(600, 300)
        time.sleep(1)
        for typed, want in CASES + CASES_NL:
            d.key("a", ctrl=True)
            d.key("BackSpace")
            time.sleep(0.6)
            d.type(typed, DELAY)
            time.sleep(1.0)
            d.key("s", ctrl=True)
            time.sleep(1.2)
            got = open(FILE).read()
            fails += not report(got == want, "vscode default", typed, got, want)
    finally:
        # The `code` launcher hands over to Electron, so killing its group is not enough;
        # match on the sandbox profile path to leave the user's own VS Code alone.
        marker = os.path.join(SANDBOX, "vscode-data")
        for pid in filter(str.isdigit, os.listdir("/proc")):
            try:
                if marker in open(f"/proc/{pid}/cmdline").read():
                    os.kill(int(pid), 15)
            except OSError:
                pass
        time.sleep(3)
    print(f"\n{fails} failure(s)")
    sys.exit(1 if fails else 0)


main()
