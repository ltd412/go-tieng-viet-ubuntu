#!/usr/bin/python3
"""Fast-typing stress in Chrome's address bar with a history that inline-autocompletes most first
letters (as a real profile does). One Vietnamese word per case; the inline suggestion is removed
with Delete and the field read back through the X clipboard.
Usage: stress_omnibox.py <mode> <cases> [seed]"""
import os
import random
import shutil
import sqlite3
import subprocess
import sys
import time

from common import *  # noqa: F401,F403

MODE, N = (sys.argv[1], int(sys.argv[2])) if __name__ == "__main__" else (None, 0)
SEED = int(sys.argv[3]) if __name__ == "__main__" and len(sys.argv) > 3 else 5
DELAY = float(os.environ.get("GTV_E2E_DELAY", "0.04"))
PROFILE = os.path.join(SANDBOX, "chrome-omnibox-profile")
HOSTS = ["dantri.com.vn", "nhatot.com", "nguoiduatin.vn", "khoahoc.vn", "cafef.vn", "vnexpress.net",
         "muasamcong.vn", "tinhte.vn", "thanhnien.vn", "hocmai.vn", "laodong.vn", "baomoi.com",
         "quanly.vn", "giaitri.vn", "tuoitre.vn", "kenh14.vn", "zingnews.vn", "oto.com.vn"]
WORDS = [("dduwocj", "được"), ("neeus", "nếu"), ("nguoiwf", "người"), ("nhuwngx", "những"), ("khoong", "không"),
         ("ddax", "đã"), ("cuar", "của"), ("vowis", "với"), ("muoons", "muốn"), ("tieengs", "tiếng"),
         ("thuwowng", "thương"), ("hocj", "học"), ("quanr", "quản"), ("tinhf", "tình"), ("baos", "báo"),
         ("laapj", "lập"), ("giuwx", "giữ"), ("kenhf", "kềnh")]


def chrome(env, *extra):
    return subprocess.Popen(
        ["google-chrome", f"--user-data-dir={PROFILE}", "--no-first-run", "--no-default-browser-check",
         "--disable-extensions", "--disable-gpu", "--ozone-platform=x11", "--window-size=900,500",
         "--window-position=0,0", "--password-store=basic", "--disable-features=Translate", *extra],
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
    return subprocess.run(["/usr/bin/python3", "-c", code], env=dict(os.environ, DISPLAY=DISPLAY),
                          capture_output=True, text=True, timeout=10).stdout


def seed(env):
    proc = chrome(env, "about:blank")
    d = xdrive.open_display(DISPLAY)
    d.fake_wm()
    d.find("google-chrome", timeout=40)
    time.sleep(6)
    stop(proc)
    db = sqlite3.connect(os.path.join(PROFILE, "Default", "History"))
    now = int((time.time() + 11644473600) * 1e6)
    for h in HOSTS:
        cur = db.execute("INSERT INTO urls (url, title, visit_count, typed_count, last_visit_time, hidden)"
                         " VALUES (?, ?, 9, 9, ?, 0)", (f"http://{h}/", h, now))
        db.execute("INSERT INTO visits (url, visit_time, from_visit, transition, segment_id, visit_duration)"
                   " VALUES (?, ?, 0, ?, 0, 1000000)", (cur.lastrowid, now, 0x30000001))
    db.commit()
    db.close()


def main():
    select_engine()
    set_mode("google-chrome", MODE)
    shutil.rmtree(PROFILE, ignore_errors=True)
    env = app_env()
    seed(env)
    rng = random.Random(SEED)
    proc = chrome(env, "about:blank")
    fails = 0
    try:
        d = xdrive.open_display(DISPLAY)
        d.fake_wm()
        win = d.find("google-chrome", timeout=40)
        time.sleep(3)
        d.activate(win)
        time.sleep(1)
        for _ in range(N):
            typed, want = rng.choice(WORDS)
            d.key("l", ctrl=True)
            time.sleep(0.3)
            d.key("BackSpace")
            time.sleep(0.5)
            d.type(typed, DELAY)
            time.sleep(1.0)
            d.key("Delete")
            time.sleep(0.2)
            d.key("a", ctrl=True)
            d.key("c", ctrl=True)
            time.sleep(0.4)
            got = clipboard()
            if got != want:
                fails += 1
                print(f"FAIL {typed!r} -> {got!r} (want {want!r})", flush=True)
                log = os.path.join(SANDBOX, "run", "engine.log")
                tail = subprocess.run(["tail", "-n", "160", log], capture_output=True, text=True).stdout
                open(os.path.join(SANDBOX, f"om-fail-{fails}.log"), "w").write(f"{typed!r} -> {got!r}\n" + tail)
            d.key("Escape")
            d.key("Escape")
    finally:
        stop(proc)
    print(f"{fails} failure(s) of {N}", flush=True)


if __name__ == "__main__":
    main()
