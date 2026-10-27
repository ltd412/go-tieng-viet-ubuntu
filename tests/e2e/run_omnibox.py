#!/usr/bin/python3
"""Types into Chrome's address bar, where inline autocomplete selects a suggested suffix after the
caret: a BackSpace aimed at the last typed letter deletes that selection instead.

The profile is seeded with a typed-history entry for http://viet.localhost:8123/ so that typing
"vi" makes Chrome complete the rest of the host. The text read back is the address bar content
with the inline suggestion removed (Delete) and copied through the X clipboard."""
import http.server
import os
import shutil
import sqlite3
import subprocess
import sys
import threading
import time

from common import *  # noqa: F401,F403

MODES = sys.argv[1].split(",") if len(sys.argv) > 1 else ["default"]
DELAY = float(os.environ.get("GTV_E2E_DELAY", "0.04"))
PROFILE = os.path.join(SANDBOX, "chrome-omnibox-profile")
HOST = "viet.localhost:8123"

# (keys typed, text expected in the address bar). Every word starts with the prefix of HOST so the
# inline suggestion is alive while the tone / mark keys arrive.
OMNIBOX_CASES = [
    ("vietj", "vietj"),
    ("vieets", "viết"),
    ("viets", "viets"),
    ("vieetj", "việt"),
    ("viee", "viê"),
    ("vif", "vì"),
    ("vieetj nam", "việt nam"),
    ("viet nam", "viet nam"),
    ("dduwocj looxi", "được lỗi"),
]


# GTV_E2E_TAB=1: type the first letters of HOST, press Tab to accept the inline suggestion.
TAB_CASES = [("vi", "viet.localhost:8123/"), ("viet", "viet.localhost:8123/")]


class Quiet(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        body = b"<title>viet</title><p>viet"
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *a):
        pass


def chrome(env, *extra):
    return subprocess.Popen(
        ["google-chrome", f"--user-data-dir={PROFILE}", "--no-first-run", "--no-default-browser-check",
         "--disable-extensions", "--disable-gpu", "--ozone-platform=x11", "--window-size=900,500",
         f"--window-position={os.environ.get('GTV_E2E_WINPOS', '0,0')}", "--password-store=basic", "--disable-features=Translate", *extra],
        env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)


def stop(proc):
    try:
        os.killpg(proc.pid, 15)
    except ProcessLookupError:
        pass
    time.sleep(2.5)


def clipboard():
    code = ("import gi;gi.require_version('Gtk','3.0');from gi.repository import Gtk,Gdk;"
            "print(Gtk.Clipboard.get(Gdk.SELECTION_CLIPBOARD).wait_for_text() or '',end='')")
    env = dict(os.environ, DISPLAY=DISPLAY)
    return subprocess.run(["/usr/bin/python3", "-c", code], env=env, capture_output=True, text=True,
                          timeout=10).stdout


def seed(env):
    """Let Chrome create its profile, then write a typed-history row so the history provider
    inlines the host while the first letters are typed."""
    proc = chrome(env, "about:blank")
    d = xdrive.open_display(DISPLAY)
    d.fake_wm()
    d.find("google-chrome", timeout=40)
    time.sleep(6)
    stop(proc)
    db = sqlite3.connect(os.path.join(PROFILE, "Default", "History"))
    # Chrome time: microseconds since 1601-01-01.
    now = int((time.time() + 11644473600) * 1e6)
    cur = db.execute("INSERT INTO urls (url, title, visit_count, typed_count, last_visit_time, hidden)"
                     " VALUES (?, 'viet', 9, 9, ?, 0)", (f"http://{HOST}/", now))
    db.execute("INSERT INTO visits (url, visit_time, from_visit, transition, segment_id, visit_duration)"
               " VALUES (?, ?, 0, ?, 0, 1000000)", (cur.lastrowid, now, 0x30000001))
    db.commit()
    print("seeded history:", db.execute("SELECT url, typed_count FROM urls").fetchall())
    db.close()


def main():
    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 8123), Quiet)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    select_engine()
    fails = 0
    for mode in MODES:
        set_mode("google-chrome", mode)
        shutil.rmtree(PROFILE, ignore_errors=True)
        env = app_env()
        if os.environ.get("GTV_E2E_PORTAL") != "1":
            env["DBUS_SESSION_BUS_ADDRESS"] = os.environ.get(
                "GTV_REAL_DBUS", "unix:path=/run/user/%d/bus" % os.getuid())
        seed(env)
        proc = chrome(env, "about:blank")
        try:
            d = xdrive.open_display(DISPLAY)
            d.fake_wm()
            win = d.find("google-chrome", timeout=40)
            time.sleep(3)
            d.activate(win)
            time.sleep(1)
            for typed, want in (TAB_CASES if os.environ.get("GTV_E2E_TAB") else OMNIBOX_CASES):
                d.key("l", ctrl=True)
                time.sleep(0.4)
                d.key("BackSpace")
                time.sleep(0.8)
                d.type(typed, DELAY)
                time.sleep(1.2)
                if os.environ.get("GTV_E2E_SHOT"):
                    d.shot(os.path.join(SANDBOX, f"omnibox-{mode}-{typed.replace(' ', '_')}.png"),
                           0, 0, 900, 140)
                if os.environ.get("GTV_E2E_TAB"):
                    d.key("\t")
                    time.sleep(0.8)
                    if os.environ.get("GTV_E2E_SHOT"):
                        d.shot(os.path.join(SANDBOX, f"omnibox-tab-{mode}-{typed}.png"), 0, 0, 900, 330)
                    d.key("l", ctrl=True)  # Tab may have moved focus; Ctrl+L selects the address bar again
                else:
                    d.key("Delete")  # drops the inline suggestion; no-op when there is none
                time.sleep(0.3)
                d.key("a", ctrl=True)
                d.key("c", ctrl=True)
                time.sleep(0.5)
                got = clipboard()
                fails += not report(got == want, f"omnibox {mode}", typed, got, want)
                d.key("Escape")
                d.key("Escape")
        finally:
            stop(proc)
    print(f"\n{fails} failure(s)")
    sys.exit(1 if fails else 0)


main()
