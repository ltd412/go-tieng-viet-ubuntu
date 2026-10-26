import os
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import xdrive  # noqa: E402

DISPLAY = os.environ.get("GTV_E2E_DISPLAY", ":77")
SANDBOX = os.environ["GTV_SANDBOX"]

CASES = [
    ("Vieetj Nam ddang hocj laapj trinhf", "Việt Nam đang học lập trình"),
    ("xin chaof cacs banj, ddi hocj.", "xin chào các bạn, đi học."),
    ("Nguoiwf Vieetj ddang hocj", "Người Việt đang học"),
    ("hoaf thuyr khoer ngoaif nhuwngx", "hòa thủy khỏe ngoài những"),
    ("class off error www.example.com", "class of eror ww.example.com"),
    ("sudo apt update && git push origin main", "sudo apt update && git push origin main"),
    ("Tooi yeeu Vieetj Nam", "Tôi yêu Việt Nam"),
    ("dduwocj loox looxi", "được lỗ lỗi"),
    ("confil config confirm conf file", "confil config confirm còn file"),
    ("conff eee\b m", "conf e m"),
    ("eee\b ", "e "),
    ("eeee\b ", "ee "),
    ("eeee\b m", "ee m"),
    ("conffil\b\b m", "conf m"),
    ("eee\b m", "e m"),
    ("eee\b\b m", " m"),
    ("tieengs\b\b m", "tiế m"),
    ("Vieetj\bm nam", "Việm nam"),
]


# Enter right after a word that was just rewritten: the key must not overtake the delayed text.
CASES_NL = [
    ("Chaof\nbanj\nddi hocj", "Chào\nbạn\nđi học"),
    ("Vieetj\nNam\nTooi yeeu Vieetj Nam", "Việt\nNam\nTôi yêu Việt Nam"),
]


_TRAIL = os.environ.get("GTV_E2E_TRAIL", "")
if _TRAIL:  # a space after the last word, for engines that hold the final word until a boundary
    CASES = [(t + _TRAIL, w + _TRAIL) for t, w in CASES]
    CASES_NL = [(t + _TRAIL, w + _TRAIL) for t, w in CASES_NL]


def app_env():
    env = dict(os.environ)
    env.update(DISPLAY=DISPLAY, GTK_IM_MODULE="ibus", QT_IM_MODULE="ibus", XMODIFIERS="@im=ibus",
               IBUS_ADDRESS=os.environ["IBUS_ADDRESS"])
    env.pop("WAYLAND_DISPLAY", None)
    # The desktop session exports IBUS_USE_PORTAL=1, which would route the toolkit to the real
    # ibus-portal (and the real input method) instead of the sandbox daemon.
    env.pop("IBUS_USE_PORTAL", None)
    env["DBUS_SESSION_BUS_ADDRESS"] = "disabled:"
    if os.environ.get("GTV_E2E_PORTAL") == "1":
        # Same wiring as a session with IBUS_USE_PORTAL=1: apps talk to a private ibus-portal.
        env["IBUS_USE_PORTAL"] = "1"
        env["DBUS_SESSION_BUS_ADDRESS"] = open(os.path.join(SANDBOX, "run", "portal-bus")).read().strip()
    env["IBUS_ENABLE_SYNC_MODE"] = os.environ.get("GTV_E2E_SYNC", env.get("IBUS_ENABLE_SYNC_MODE", "1"))
    return env


def set_mode(app_key, mode):
    d = os.path.join(os.environ["XDG_CONFIG_HOME"], "gtv")
    os.makedirs(d, exist_ok=True)
    with open(os.path.join(d, "config.toml"), "w") as f:
        if "GTV_E2E_PAUSE" in os.environ:  # otherwise the engine's own default is what gets tested
            f.write(f'pause_ms = {os.environ["GTV_E2E_PAUSE"]}\n')
        if mode != "default":  # "default" exercises the built-in policy
            f.write(f'[apps]\n"{app_key}" = "{mode}"\n')
    time.sleep(0.05)


def select_engine(name=None):
    name = name or os.environ.get("GTV_E2E_ENGINE", "gtv-telex")
    import gi
    gi.require_version("IBus", "1.0")
    from gi.repository import IBus
    IBus.init()
    bus = IBus.Bus()
    assert bus.set_global_engine(name), f"cannot select {name}"
    return bus


def report(ok, label, typed, got, want):
    print(f"{'ok  ' if ok else 'FAIL'} {label:34} {typed!r} -> {got!r}" + ("" if ok else f"  (want {want!r})"), flush=True)
    return ok


def kill(proc):
    try:
        proc.terminate()
        proc.wait(timeout=5)
    except Exception:
        proc.kill()


def settle(path, quiet=0.8, limit=15.0):
    """Content of `path` once it has stopped changing for `quiet` seconds."""
    last, since, end = None, time.time(), time.time() + limit
    while time.time() < end:
        cur = open(path).read()
        if cur != last:
            last, since = cur, time.time()
        elif time.time() - since >= quiet:
            break
        time.sleep(0.1)
    return last
