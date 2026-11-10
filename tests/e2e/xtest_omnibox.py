#!/usr/bin/python3
"""Feasibility check for editing with XTest: BackSpace and the new letter (through a remapped spare
keycode) go to the server as one ordered stream. Chrome's address bar, engine off for Chrome, history
that inline-autocompletes the first letters. Each word is typed the way the engine would rewrite it.
Usage: xtest_omnibox.py <cases> [seed]   env: GTV_E2E_DELAY (between keys), GTV_E2E_MAPWAIT (after remap)"""
import ctypes
import os
import random
import shutil
import sys
import time

from common import *  # noqa: F401,F403
import stress_omnibox as so

N = int(sys.argv[1])
SEED = int(sys.argv[2]) if len(sys.argv) > 2 else 5
DELAY = float(os.environ.get("GTV_E2E_DELAY", "0.08"))
MAPWAIT = float(os.environ.get("GTV_E2E_MAPWAIT", "0.03"))
BS = "<"
# (steps, expected): ASCII = a plain key, "<" = BackSpace, anything else = a letter injected with a remapped keycode.
WORDS = [(list("kho") + ["<", "ô"] + list("ng"), "không"),
         (["d", "<", "đ", "a", "<", "ã"], "đã"),
         (list("tie") + ["<", "ê", "n", "<", "<", "ế", "n"], "tiến"),
         (list("hoc") + ["<", "<", "ọ", "c"], "học"),
         (list("nguoi") + ["<", "<", "<", "ư", "ơ", "i", "<", "<", "ờ", "i"], "người")]


class Injector:
    def __init__(self, d):
        self.d = d
        x = d.x
        lo, hi = ctypes.c_int(), ctypes.c_int()
        x.XDisplayKeycodes(d.d, ctypes.byref(lo), ctypes.byref(hi))
        self.kc = hi.value  # last keycode: spare on a normal keymap
        x.XChangeKeyboardMapping.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_int,
                                             ctypes.POINTER(ctypes.c_ulong), ctypes.c_int]
        self.mapped = None

    def letter(self, ch):
        sym = 0x01000000 + ord(ch)
        if self.mapped != sym:
            arr = (ctypes.c_ulong * 1)(sym)
            self.d.x.XChangeKeyboardMapping(self.d.d, self.kc, 1, arr, 1)
            self.d.x.XSync(self.d.d, 0)
            self.mapped = sym
            time.sleep(MAPWAIT)  # let the client see MappingNotify before the key arrives
        self.d.t.XTestFakeKeyEvent(self.d.d, self.kc, 1, 0)
        self.d.t.XTestFakeKeyEvent(self.d.d, self.kc, 0, 0)
        self.d.x.XFlush(self.d.d)


def main():
    select_engine()
    set_mode("google-chrome", "off")
    shutil.rmtree(so.PROFILE, ignore_errors=True)
    env = app_env()
    so.seed(env)
    rng = random.Random(SEED)
    proc = so.chrome(env, "about:blank")
    fails = 0
    try:
        d = xdrive.open_display(DISPLAY)
        d.fake_wm()
        win = d.find("google-chrome", timeout=40)
        time.sleep(3)
        d.activate(win)
        time.sleep(1)
        inj = Injector(d)
        for _ in range(N):
            steps, want = rng.choice(WORDS)
            d.key("l", ctrl=True)
            time.sleep(0.3)
            d.key("BackSpace")
            time.sleep(0.5)
            for s in steps:
                if s == BS:
                    d.key("BackSpace")
                elif ord(s) < 128:
                    d.key(s)
                else:
                    inj.letter(s)
                time.sleep(DELAY)
            time.sleep(1.0)
            d.key("Delete")
            time.sleep(0.2)
            d.key("a", ctrl=True)
            d.key("c", ctrl=True)
            time.sleep(0.4)
            got = so.clipboard()
            if got != want:
                fails += 1
                print(f"FAIL {''.join(steps)!r} -> {got!r} (want {want!r})", flush=True)
            d.key("Escape")
            d.key("Escape")
    finally:
        so.stop(proc)
    print(f"{fails} failure(s) of {N}", flush=True)


main()
