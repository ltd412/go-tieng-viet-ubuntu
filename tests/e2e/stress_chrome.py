#!/usr/bin/python3
"""Fast-typing stress in a real Chrome textarea: random phrases of common hard words.
Usage: stress_chrome.py <mode> <cases> [seed]. Types into a textarea in a real Chrome through the sandbox ibus.
The page mirrors the textarea into document.title, which Chrome publishes as an X property."""
import os
import shutil
import subprocess
import sys
import time

from common import *  # noqa: F401,F403

MODE, N = sys.argv[1], int(sys.argv[2])
SEED = int(sys.argv[3]) if len(sys.argv) > 3 else 5
import random
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


WORDS = [("dduwocj", "được"), ("neeus", "nếu"), ("nguoiwf", "người"), ("nhuwngx", "những"), ("khoong", "không"),
         ("ddax", "đã"), ("cuar", "của"), ("vowis", "với"), ("muoons", "muốn"), ("tieengs", "tiếng"),
         ("thuwowng", "thương"), ("ddang", "đang"), ("hocj", "học"), ("laapj", "lập"), ("trinhf", "trình")]


def main():
    select_engine()
    rng = random.Random(SEED)
    fails = 0
    set_mode("google-chrome", MODE)
    shutil.rmtree(PROFILE, ignore_errors=True)
    env = app_env()
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
        for _ in range(N):
            picks = [rng.choice(WORDS) for _ in range(rng.randint(4, 7))]
            typed, want = " ".join(t for t, _ in picks), " ".join(w for _, w in picks)
            d.key("a", ctrl=True)
            d.key("BackSpace")
            time.sleep(0.6)
            d.type(typed, DELAY)
            got = read_value(d, win, quiet=0.6)
            if got != want:
                fails += 1
                print(f"FAIL {typed!r} -> {got!r} (want {want!r})", flush=True)
    finally:
        try:
            os.killpg(proc.pid, 15)
        except ProcessLookupError:
            pass
    print(f"{fails} failure(s) of {N}", flush=True)


main()
