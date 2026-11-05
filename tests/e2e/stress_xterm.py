#!/usr/bin/python3
"""Fast-typing stress in xterm.js (the terminal of VS Code, Cursor, Orca...) inside a real Chrome.
The page logs what xterm.js would send to the shell; BackSpace (0x7f) is then applied like a tty does.
Usage: stress_xterm.py <mode> <lines> [seed]"""
import os
import random
import shutil
import subprocess
import sys
import time

from common import *  # noqa: F401,F403

MODE, N = sys.argv[1], int(sys.argv[2])
SEED = int(sys.argv[3]) if len(sys.argv) > 3 else 5
DELAY = float(os.environ.get("GTV_E2E_DELAY", "0.04"))
PROFILE = os.path.join(SANDBOX, "chrome-profile")
WORDS = [("dduwocj", "được"), ("neeus", "nếu"), ("nguoiwf", "người"), ("nhuwngx", "những"), ("khoong", "không"),
         ("ddax", "đã"), ("cuar", "của"), ("vowis", "với"), ("muoons", "muốn"), ("tieengs", "tiếng")]


def tty_lines(data):
    lines, cur = [], []
    for ch in data:
        if ch == "\x7f":
            if cur:
                cur.pop()
        elif ch == "\r":
            lines.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    return lines


def main():
    select_engine()
    rng = random.Random(SEED)
    set_mode("google-chrome", MODE)
    shutil.rmtree(PROFILE, ignore_errors=True)
    proc = subprocess.Popen(
        ["google-chrome", f"--user-data-dir={PROFILE}", "--no-first-run", "--no-default-browser-check",
         "--disable-extensions", "--disable-gpu", "--ozone-platform=x11", "--window-size=900,500",
         "--window-position=0,0", "--password-store=basic", "--allow-file-access-from-files",
         f"file://{os.path.join(HERE, 'xterm.html')}" + ("#events" if os.environ.get("GTV_E2E_EVENTS") else "")],
        env=app_env(), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
    wanted, fails = [], 0
    try:
        d = xdrive.open_display(DISPLAY)
        d.fake_wm()
        win = d.find("google-chrome", timeout=40)
        time.sleep(3)
        d.activate(win)
        time.sleep(1)
        d.click(60, 60)
        time.sleep(1)
        for _ in range(N):
            picks = [rng.choice(WORDS) for _ in range(rng.randint(3, 5))]
            typed, want = " ".join(t for t, _ in picks), " ".join(w for _, w in picks)
            wanted.append((typed, want))
            d.type(typed + "\n", DELAY)
            time.sleep(0.3)
        t = ""
        end = time.time() + 6
        last = None
        while time.time() < end:
            t = d.title(win)
            if t != last:
                last, end = t, time.time() + 1.5
            time.sleep(0.2)
        raw = t[2:].rsplit(" - Google Chrome", 1)[0] if t.startswith("v:") else ""
        events = ""
        if "#" in raw:
            raw, events = raw.split("#", 1)
            print("events:", events[:1500])
        if os.environ.get("GTV_E2E_RAW"):
            print("raw:", repr("".join(chr(int(h, 16)) for h in raw.split(",") if h)))
        got = tty_lines("".join(chr(int(h, 16)) for h in raw.split(",") if h)) if raw else []
        for (typed, want), line in zip(wanted, got):
            fails += not report(line == want, f"xterm.js {MODE}", typed, line, want)
        if len(got) < len(wanted):
            print(f"only {len(got)} of {len(wanted)} lines arrived (title={t[:80]!r})")
            fails += 1
    finally:
        try:
            os.killpg(proc.pid, 15)
        except ProcessLookupError:
            pass
    print(f"{fails} failure(s) of {N}", flush=True)


main()
