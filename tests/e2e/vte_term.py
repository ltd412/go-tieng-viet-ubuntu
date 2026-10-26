#!/usr/bin/python3
"""A bare VTE terminal (the widget inside gnome-terminal) running `cat > FILE`."""
import sys

import gi

gi.require_version("Gtk", "3.0")
gi.require_version("Vte", "2.91")
from gi.repository import GLib, Gtk, Vte  # noqa: E402

out = sys.argv[1]
win = Gtk.Window(title="vte-term")
win.set_default_size(700, 200)
term = Vte.Terminal()
term.spawn_async(Vte.PtyFlags.DEFAULT, None, ["/bin/bash", "--norc", "-c", f"cat > {out}"], None,
                 GLib.SpawnFlags.DEFAULT, None, None, -1, None, None)
term.connect("child-exited", lambda *_: Gtk.main_quit())
win.add(term)
win.connect("destroy", Gtk.main_quit)
win.show_all()
Gtk.main()
