#!/usr/bin/python3
"""Minimal GTK text widget (GTK3, or GTK4 with GTV_E2E_GTK4=1) that mirrors its content into a
file, so a test can read it."""
import os
import sys

import gi

kind, out = sys.argv[1], sys.argv[2]
gtk4 = os.environ.get("GTV_E2E_GTK4") == "1"
gi.require_version("Gtk", "4.0" if gtk4 else "3.0")
from gi.repository import GLib, Gtk  # noqa: E402

GLib.set_prgname("gtk_edit")  # GTK4 sets WM_CLASS from this; the harness finds the window by it
win = Gtk.Window(title="gtk-edit")
win.set_default_size(500, 120)
if kind in ("entry", "password", "search"):
    w = Gtk.SearchEntry() if kind == "search" and gtk4 else Gtk.Entry()
    if kind == "password":
        w.set_visibility(False)
    get = w.get_text
    w.connect("changed", lambda *_: open(out, "w").write(get()))
else:
    w = Gtk.TextView()
    buf = w.get_buffer()
    get = lambda: buf.get_text(buf.get_start_iter(), buf.get_end_iter(), True)
    buf.connect("changed", lambda *_: open(out, "w").write(get()))
open(out, "w").write("")
if gtk4:
    win.set_child(w)
    loop = GLib.MainLoop()
    win.connect("close-request", lambda *_: loop.quit())
    win.present()
    loop.run()
else:
    win.add(w)
    win.connect("destroy", Gtk.main_quit)
    win.show_all()
    Gtk.main()
