#!/usr/bin/python3
"""Drives the real engine through a real ibus-daemon (see start-isolated-ibus.sh) and replays
the signals it emits against a model application, for every delivery mode."""
import os
import sys
import time

import gi

gi.require_version("IBus", "1.0")
from gi.repository import GLib, IBus  # noqa: E402

RELEASE = 1 << 30
SHIFT = 1
BACKSPACE, LEFT, RETURN = 0xFF08, 0xFF51, 0xFF0D
MODES = ["surrounding", "backspace", "select", "forward-keys", "preedit"]


class App:
    """Text before the cursor + selection + preedit, fed by the IC's signals."""

    def __init__(self):
        self.text, self.sel, self.preedit = [], 0, ""

    def drop_sel(self):
        for _ in range(self.sel):
            self.text.pop()
        self.sel = 0

    def native(self, keyval, state):
        if keyval == BACKSPACE:
            if self.sel:
                self.drop_sel()
            elif self.text:
                self.text.pop()
        elif keyval == LEFT and state & SHIFT:
            self.sel = min(self.sel + 1, len(self.text))
        elif keyval == RETURN:
            self.drop_sel()
            self.text.append("\n")
        else:
            ch = chr(keyval) if 0x20 <= keyval < 0x7F or 0xA0 <= keyval <= 0xFF else (
                chr(keyval - 0x01000000) if keyval >= 0x01000000 else None)
            if ch:
                self.drop_sel()
                self.text.append(ch)

    def on_commit(self, _ic, text):
        self.drop_sel()
        self.preedit = ""
        self.text.extend(text.get_text())

    def on_forward(self, _ic, keyval, keycode, state):
        if not state & RELEASE:
            self.native(keyval, state)

    def on_delete(self, _ic, offset, n):
        for _ in range(n):
            if self.text:
                self.text.pop()

    def on_preedit(self, _ic, text, cursor, visible):
        self.preedit = text.get_text() if visible else ""

    def on_hide_preedit(self, _ic):
        self.preedit = ""

    def shown(self):
        return "".join(self.text) + self.preedit


def pump():
    ctx = GLib.MainContext.default()
    deadline = time.time() + 0.05
    while time.time() < deadline:
        while ctx.iteration(False):
            deadline = time.time() + 0.02


def write_config(mode):
    d = os.path.join(os.environ["XDG_CONFIG_HOME"], "gtv")
    os.makedirs(d, exist_ok=True)
    with open(os.path.join(d, "config.toml"), "w") as f:
        f.write(f'default_mode = "{mode}"\n')


def new_session(bus, mode, engine):
    write_config(mode)
    time.sleep(0.05)  # distinct mtime so the engine notices the change
    app = App()
    ic = bus.create_input_context("gtv-test")
    ic.connect("commit-text", app.on_commit)
    ic.connect("forward-key-event", app.on_forward)
    ic.connect("delete-surrounding-text", app.on_delete)
    ic.connect("update-preedit-text", app.on_preedit)
    ic.connect("hide-preedit-text", app.on_hide_preedit)
    ic.set_capabilities(IBus.Capabilite.PREEDIT_TEXT | IBus.Capabilite.FOCUS | IBus.Capabilite.SURROUNDING_TEXT)
    ic.focus_in()  # the daemon only starts the engine for a focused context
    ic.set_engine(engine)
    for _ in range(100):
        pump()
        desc = ic.get_engine()
        if desc and desc.get_name() == engine:
            break
        time.sleep(0.05)
    else:
        sys.exit(f"engine {engine} did not start; see $GTV_SANDBOX/run/engine.log")
    pump()
    return ic, app


def press(ic, app, keyval, state=0):
    handled = ic.process_key_event(keyval, 0, state)
    pump()
    if not handled:
        app.native(keyval, state)
    rel = ic.process_key_event(keyval, 0, state | RELEASE)
    pump()
    assert rel == handled, f"release of {keyval:#x} must mirror press (press={handled}, release={rel})"


def type_str(ic, app, s):
    for c in s:
        press(ic, app, ord(c), SHIFT if c.isupper() else 0)


def run_case(bus, mode, engine, typed, want, keys_after=None):
    ic, app = new_session(bus, mode, engine)
    type_str(ic, app, typed)
    if keys_after:
        keys_after(ic, app)
    # Real clients reset the context before losing focus; IBus drops signals sent after that.
    ic.reset()
    pump()
    ic.focus_out()
    pump()
    ic.destroy()
    got = app.shown()
    ok = got == want
    print(f"{'ok  ' if ok else 'FAIL'} {engine:10} {mode:13} {typed!r} -> {got!r}" + ("" if ok else f"  (want {want!r})"))
    return ok


def main():
    IBus.init()
    bus = IBus.Bus()
    if not bus.is_connected():
        sys.exit("no ibus-daemon at $IBUS_ADDRESS")
    cases = [
        ("gtv-telex", "Vieetj Nam", "Việt Nam"),
        ("gtv-telex", "xin chaof cacs banj, ddi hocj.", "xin chào các bạn, đi học."),
        ("gtv-telex", "Nguoiwf Vieetj ddang hocj laapj trinhf", "Người Việt đang học lập trình"),
        ("gtv-telex", "class off error www.example.com", "class of eror ww.example.com"),
        ("gtv-telex", "sudo apt update && git push origin main", "sudo apt update && git push origin main"),
        ("gtv-telex", "hoaf thuyr khoer ngoaif", "hòa thủy khỏe ngoài"),
        ("gtv-telex", "dod dad dodo dduwocj", "đo đa đô được"),
        ("gtv-vni", "Vie6t5 Nam 2024 d9i", "Việt Nam 2024 đi"),
    ]
    failures = 0
    for mode in MODES:
        for engine, typed, want in cases:
            failures += not run_case(bus, mode, engine, typed, want)
        # backspace in the middle of a composed word, then keep typing
        def bs(ic, app):
            press(ic, app, BACKSPACE)
            type_str(ic, app, "t")
        failures += not run_case(bus, mode, "gtv-telex", "vieetj", "việt", bs)
    print(f"\n{failures} failure(s)")
    sys.exit(1 if failures else 0)


if __name__ == "__main__":
    main()
