"""Tiny X11 driver over ctypes: find windows, set focus, inject keys through XTest.
Only ever talks to the display given at construction (the nested Xephyr), never :0."""
import ctypes
import ctypes.util
import time

XK = {" ": 0x20, "\n": 0xFF0D, "\t": 0xFF09, "BackSpace": 0xFF08, "Return": 0xFF0D,
      "Escape": 0xFF1B, "Left": 0xFF51, "Home": 0xFF50, "End": 0xFF57}


class Display:
    def __init__(self, name):
        self.x = ctypes.CDLL("libX11.so.6")
        self.t = ctypes.CDLL("libXtst.so.6")
        self.x.XOpenDisplay.restype = ctypes.c_void_p
        self.x.XOpenDisplay.argtypes = [ctypes.c_char_p]
        self.d = self.x.XOpenDisplay(name.encode())
        if not self.d:
            raise RuntimeError(f"cannot open {name}")
        self.x.XDefaultRootWindow.restype = ctypes.c_ulong
        self.x.XDefaultRootWindow.argtypes = [ctypes.c_void_p]
        self.root = self.x.XDefaultRootWindow(self.d)
        self.x.XKeysymToKeycode.restype = ctypes.c_ubyte
        self.x.XKeysymToKeycode.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
        self.t.XTestFakeKeyEvent.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_int, ctypes.c_ulong]
        self.t.XTestFakeMotionEvent.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_ulong]
        self.t.XTestFakeButtonEvent.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_int, ctypes.c_ulong]
        self.x.XSetInputFocus.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int, ctypes.c_ulong]
        self.x.XFlush.argtypes = [ctypes.c_void_p]
        self.x.XSync.argtypes = [ctypes.c_void_p, ctypes.c_int]
        self.shift = self.x.XKeysymToKeycode(self.d, 0xFFE1)

    def _atom(self, name):
        self.x.XInternAtom.restype = ctypes.c_ulong
        self.x.XInternAtom.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_int]
        return self.x.XInternAtom(self.d, name.encode(), 0)

    def _set(self, win, prop, typ, values, fmt=32):
        self.x.XChangeProperty.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_ulong, ctypes.c_ulong,
                                           ctypes.c_int, ctypes.c_int, ctypes.c_void_p, ctypes.c_int]
        if fmt == 32:
            arr = (ctypes.c_long * len(values))(*values)  # Xlib stores format-32 data as C longs
        else:
            arr = ctypes.create_string_buffer(values)
        n = len(values)
        self.x.XChangeProperty(self.d, win, self._atom(prop), self._atom(typ) if isinstance(typ, str) else typ,
                               fmt, 0, ctypes.cast(arr, ctypes.c_void_p), n)

    def fake_wm(self):
        """Advertise an EWMH window manager so toolkits (Chrome) believe windows can be active."""
        self.x.XCreateSimpleWindow.restype = ctypes.c_ulong
        self.x.XCreateSimpleWindow.argtypes = [ctypes.c_void_p, ctypes.c_ulong] + [ctypes.c_int] * 2 + [ctypes.c_uint] * 3 + [ctypes.c_ulong] * 2
        check = self.x.XCreateSimpleWindow(self.d, self.root, -10, -10, 1, 1, 0, 0, 0)
        for w in (self.root, check):
            self._set(w, "_NET_SUPPORTING_WM_CHECK", 33, [check])
        self._set(check, "_NET_WM_NAME", "UTF8_STRING", b"gtv-fake-wm", fmt=8)
        self._set(self.root, "_NET_SUPPORTED", 4,
                  [self._atom("_NET_ACTIVE_WINDOW"), self._atom("_NET_SUPPORTING_WM_CHECK")])
        self.x.XSync(self.d, 0)

    def activate(self, wid):
        self._set(self.root, "_NET_ACTIVE_WINDOW", 33, [wid])
        self.focus(wid)

    def shot(self, path, x=0, y=0, w=900, h=200):
        """Save a crop of the root window as PNG (needs PIL)."""
        from PIL import Image

        class XImage(ctypes.Structure):
            _fields_ = [("width", ctypes.c_int), ("height", ctypes.c_int), ("xoffset", ctypes.c_int),
                        ("format", ctypes.c_int), ("data", ctypes.c_void_p), ("byte_order", ctypes.c_int),
                        ("bitmap_unit", ctypes.c_int), ("bitmap_bit_order", ctypes.c_int),
                        ("bitmap_pad", ctypes.c_int), ("depth", ctypes.c_int),
                        ("bytes_per_line", ctypes.c_int), ("bits_per_pixel", ctypes.c_int)]

        self.x.XGetImage.restype = ctypes.POINTER(XImage)
        self.x.XGetImage.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int, ctypes.c_int,
                                     ctypes.c_uint, ctypes.c_uint, ctypes.c_ulong, ctypes.c_int]
        img = self.x.XGetImage(self.d, self.root, x, y, w, h, 0xFFFFFFFF, 2)
        im = img.contents
        raw = ctypes.string_at(im.data, im.bytes_per_line * im.height)
        Image.frombuffer("RGB", (im.width, im.height), raw, "raw", "BGRX", im.bytes_per_line, 1).save(path)

    def toplevels(self):
        """(window id, WM_CLASS, WM_NAME) for each mapped child of the root."""
        import subprocess, re, os
        env = dict(os.environ, DISPLAY=self.name_env)
        out = subprocess.run(["xwininfo", "-root", "-children"], capture_output=True, text=True, env=env).stdout
        res = []
        for m in re.finditer(r"(0x[0-9a-f]+) (\"[^\"]*\"|\(has no name\)):", out):
            wid = int(m.group(1), 16)
            prop = subprocess.run(["xprop", "-id", hex(wid), "WM_CLASS", "WM_NAME", "_NET_WM_NAME"],
                                  capture_output=True, text=True, env=env).stdout
            cls = re.search(r'WM_CLASS\(STRING\) = (.*)', prop)
            name = re.search(r'_NET_WM_NAME\(UTF8_STRING\) = "(.*)"', prop) or re.search(r'WM_NAME\(\w+\) = "(.*)"', prop)
            res.append((wid, cls.group(1).lower() if cls else "", name.group(1) if name else ""))
        return res

    def viewable(self, wid):
        import subprocess, os
        env = dict(os.environ, DISPLAY=self.name_env)
        out = subprocess.run(["xwininfo", "-id", hex(wid)], capture_output=True, text=True, env=env).stdout
        return "IsViewable" in out

    def find(self, cls_substr, timeout=20):
        end = time.time() + timeout
        while time.time() < end:
            for wid, cls, name in self.toplevels():
                if cls_substr in cls and self.viewable(wid):
                    return wid
            time.sleep(0.3)
        raise RuntimeError(f"no window with class {cls_substr!r}")

    def title(self, wid):
        import subprocess, re, os
        env = dict(os.environ, DISPLAY=self.name_env)
        prop = subprocess.run(["xprop", "-id", hex(wid), "_NET_WM_NAME", "WM_NAME"], capture_output=True, text=True, env=env).stdout
        m = re.search(r'_NET_WM_NAME\(UTF8_STRING\) = "(.*)"', prop) or re.search(r'WM_NAME\(\w+\) = "(.*)"', prop)
        return m.group(1).encode().decode("unicode_escape").encode("latin-1").decode("utf-8") if m else ""

    def focus(self, wid):
        self.x.XSetInputFocus(self.d, wid, 1, 0)  # RevertToPointerRoot
        self.x.XSync(self.d, 0)

    def click(self, x, y):
        self.t.XTestFakeMotionEvent(self.d, -1, x, y, 0)
        self.t.XTestFakeButtonEvent(self.d, 1, 1, 0)
        self.t.XTestFakeButtonEvent(self.d, 1, 0, 0)
        self.x.XSync(self.d, 0)

    def _kc(self, keysym):
        return self.x.XKeysymToKeycode(self.d, keysym)

    def key(self, keysym, shift=False, ctrl=False, hold=0.0):
        if isinstance(keysym, str):
            keysym = XK.get(keysym, ord(keysym) if len(keysym) == 1 else 0)
        kc = self._kc(keysym)
        ctrl_kc = self._kc(0xFFE3)
        if ctrl:
            self.t.XTestFakeKeyEvent(self.d, ctrl_kc, 1, 0)
        if shift:
            self.t.XTestFakeKeyEvent(self.d, self.shift, 1, 0)
        self.t.XTestFakeKeyEvent(self.d, kc, 1, 0)
        self.x.XFlush(self.d)
        if hold:
            time.sleep(hold)
        self.t.XTestFakeKeyEvent(self.d, kc, 0, 0)
        if shift:
            self.t.XTestFakeKeyEvent(self.d, self.shift, 0, 0)
        if ctrl:
            self.t.XTestFakeKeyEvent(self.d, ctrl_kc, 0, 0)
        self.x.XFlush(self.d)

    def type(self, text, delay=0.01):
        import os
        pause = float(os.environ.get("GTV_E2E_BSDELAY", "0"))  # a human pauses around BackSpace
        for ch in text:
            if ch == "\b" and pause:
                time.sleep(pause)
            self.key("BackSpace" if ch == "\b" else "Return" if ch == "\n" else ch, shift=ch.isupper() or ch in '~!@#$%^&*()_+{}|:"<>?')
            time.sleep(delay + (pause if ch == "\b" else 0))

    name_env = ""


def open_display(name):
    d = Display(name)
    d.name_env = name
    return d
