#!/usr/bin/python3
"""Fast-typing stress: random phrases of hard Vietnamese words into one
GTK widget per case. Usage: stress_gtk.py <entry|textview> <mode> <cases> [seed]"""
import os
import random
import subprocess
import sys
import time

from common import *  # noqa: F401,F403

KIND, MODE, N = sys.argv[1], sys.argv[2], int(sys.argv[3])
SEED = int(sys.argv[4]) if len(sys.argv) > 4 else 7
DELAY = float(os.environ.get("GTV_E2E_DELAY", "0.015"))

WORDS = [("nguoiwf", "người"), ("dduwocj", "được"), ("thuyr", "thủy"), ("khoer", "khỏe"), ("ngoaif", "ngoài"),
         ("nhuwngx", "những"), ("tieengs", "tiếng"), ("Vieetj", "Việt"), ("quanr", "quản"), ("hocj", "học"),
         ("trinhf", "trình"), ("ddang", "đang"), ("cuar", "của"), ("muoons", "muốn"), ("truwowngf", "trường"),
         ("giuwx", "giữ"), ("thuwowng", "thương"), ("yeeu", "yêu"), ("nghieepj", "nghiệp"), ("hoaf", "hòa")]


def case(rng):
    words, want = [], []
    for _ in range(rng.randint(3, 6)):
        t, w = rng.choice(WORDS)
        words.append(t)
        want.append(w)
    return " ".join(words), " ".join(want)


def main():
    select_engine()
    rng = random.Random(SEED)
    fails = 0
    out = os.path.join(SANDBOX, f"stress-{KIND}.txt")
    for i in range(N):
        typed, want = case(rng)
        set_mode("gtk_edit", MODE)
        proc = subprocess.Popen(["/usr/bin/python3", os.path.join(HERE, "gtk_edit.py"), KIND, out], env=app_env(),
                                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            d = xdrive.open_display(DISPLAY)
            win = d.find("gtk_edit")
            time.sleep(0.4)
            d.focus(win)
            time.sleep(0.3)
            d.type(typed, DELAY)
            got = settle(out, quiet=0.6)
        finally:
            kill(proc)
        if got != want:
            fails += 1
            print(f"FAIL {typed!r} -> {got!r} (want {want!r})", flush=True)
            if len(got) < len(want) - 3:  # cut short: keep the engine's trace for the post-mortem
                log = os.path.join(SANDBOX, "run", "engine.log")
                tail = subprocess.run(["tail", "-n", "300", log], capture_output=True, text=True).stdout
                open(os.path.join(SANDBOX, f"cut-{i}.log"), "w").write(f"{typed!r} -> {got!r}\n" + tail)
    print(f"{fails} failure(s) of {N}", flush=True)


main()
