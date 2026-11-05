#!/usr/bin/python3
"""Creates a host window that is never mapped, prints its id, then sleeps.
Xephyr -parent <id> renders into it, so the nested test X server never shows on the desktop and
cannot take focus. Usage: hidden_parent.py <width> <height>"""
import ctypes
import sys
import time

x = ctypes.CDLL("libX11.so.6")
x.XOpenDisplay.restype = ctypes.c_void_p
x.XOpenDisplay.argtypes = [ctypes.c_char_p]
x.XDefaultRootWindow.restype = ctypes.c_ulong
x.XDefaultRootWindow.argtypes = [ctypes.c_void_p]
x.XCreateSimpleWindow.restype = ctypes.c_ulong
x.XCreateSimpleWindow.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int, ctypes.c_int,
                                  ctypes.c_uint, ctypes.c_uint, ctypes.c_uint, ctypes.c_ulong, ctypes.c_ulong]
x.XFlush.argtypes = [ctypes.c_void_p]
d = x.XOpenDisplay(None)
if not d:
    sys.exit("cannot open the host display")
w = x.XCreateSimpleWindow(d, x.XDefaultRootWindow(d), -5000, -5000, int(sys.argv[1]), int(sys.argv[2]), 0, 0, 0)
x.XFlush(d)
print(w, flush=True)
while True:
    time.sleep(3600)
