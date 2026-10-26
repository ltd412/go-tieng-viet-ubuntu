#!/usr/bin/python3
"""Minimal GTK3 text widget that mirrors its content into a file, so a test can read it."""
import sys

import gi

gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

kind, out = sys.argv[1], sys.argv[2]
win = Gtk.Window(title="gtk-edit")
win.set_default_size(500, 120)
if kind in ("entry", "password"):
    w = Gtk.Entry()
    if kind == "password":
        w.set_visibility(False)
    get = w.get_text
    w.connect("changed", lambda *_: open(out, "w").write(get()))
else:
    w = Gtk.TextView()
    buf = w.get_buffer()
    get = lambda: buf.get_text(buf.get_start_iter(), buf.get_end_iter(), True)
    buf.connect("changed", lambda *_: open(out, "w").write(get()))
win.add(w)
win.connect("destroy", Gtk.main_quit)
win.show_all()
open(out, "w").write("")
Gtk.main()
