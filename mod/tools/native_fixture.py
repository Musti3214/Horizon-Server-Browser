#!/usr/bin/env python3
"""Drive the browser through horizon-ctl in disposable homes, with loopback status fixtures.

python tools/server_browser_e2e.py --game target/fast/horizon-client.exe \
  --ctl target/fast/horizon-ctl.exe --resources PATH --out .local/server-browser-ui

--large 10000 also checks the virtualized list with ten thousand saved entries.
No public server is contacted. Discovery remains off and no account is used.
"""
import argparse
import json
import os
import shutil
from pathlib import Path
import socketserver
import socket
import struct
import subprocess
import tempfile
import threading
import time
class lab:
    @staticmethod
    def game_env(base):
        env = dict(base)
        for key in ("HORIZON_HOME", "HORIZON_LAUNCHER_HOME", "XDG_DATA_HOME", "XDG_CONFIG_HOME",
                    "HORIZON_LAUNCHER_TRASH", "HORIZON_LAUNCHER_CLIPBOARD", "HORIZON_LAUNCHER_DESKTOP"):
            if not env.get(key):
                raise RuntimeError("An isolated test folder is required: " + key)
        env["HORIZON_NO_AUDIO"] = "1"
        env["HORIZON_LAUNCHER_NO_SELF_UPDATE"] = "1"
        env.pop("HORIZON_SESSION_FILE", None)
        return env


FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)
ROOT = Path(__file__).resolve().parents[1]
LANG = json.loads((ROOT / "src/lang/en_us.json").read_text(encoding="utf-8"))


def label(key):
    return LANG["horizon.browser." + key]


def varint(number):
    result = bytearray()
    while number > 127:
        result.append((number & 127) | 128)
        number >>= 7
    result.append(number)
    return bytes(result)


def exact(stream, size):
    result = bytearray()
    while len(result) < size:
        data = stream.recv(size - len(result))
        if not data:
            raise EOFError()
        result.extend(data)
    return bytes(result)


def frame(stream):
    size = 0
    for shift in range(0, 21, 7):
        byte = exact(stream, 1)[0]
        size |= (byte & 127) << shift
        if byte < 128:
            if size > 262144:
                raise ValueError("oversized fixture request")
            return exact(stream, size)
    raise ValueError("bad frame")


class StatusHandler(socketserver.BaseRequestHandler):
    def handle(self):
        try:
            self.request.settimeout(3)
            frame(self.request)
            frame(self.request)
            status = {
                "version": {"name": "26.3", "protocol": 777},
                "description": {"text": "Local ", "color": "gold", "extra": [
                    {"text": "JSON MOTD", "bold": True, "color": "aqua"},
                    {"text": "\nA fixture for the server browser", "color": "gray"}]},
                "players": {"online": 2, "max": 20, "sample": [
                    {"name": "Fixture_Alex"}, {"name": "Fixture_Steve"}]},
            }
            text = json.dumps(status).encode()
            packet = b"\0" + varint(len(text)) + text
            self.request.sendall(varint(len(packet)) + packet)
            ping = frame(self.request)
            self.request.sendall(varint(len(ping)) + ping)
        except (OSError, EOFError, ValueError):
            pass


class StatusServer(socketserver.ThreadingTCPServer):
    daemon_threads = True
    allow_reuse_address = True


def nbt_string(value):
    data = value.encode("utf-8")  # test fixtures deliberately contain only ASCII
    return struct.pack(">H", len(data)) + data


def tag(kind, name, body):
    return bytes([kind]) + nbt_string(name) + body


def seed(home, port, large):
    entries = [("Local friends", f"127.0.0.1:{port}"),
               ("Quiet sandbox", "127.0.0.1:65010"),
               ("Offline fixture", "127.0.0.1:65011")]
    entries += [(f"Fixture {i:05}", f"127.0.0.1:{40000+i}") for i in range(max(0, large-len(entries)))]
    compounds = [tag(8, "name", nbt_string(name)) + tag(8, "ip", nbt_string(address))
                 + tag(3, "foreignField", struct.pack(">i", 42)) + b"\0" for name, address in entries]
    hidden = tag(8, "name", nbt_string("Hidden fixture")) + tag(8, "ip", nbt_string("127.0.0.1:65012")) + tag(1, "hidden", b"\1") + b"\0"
    compounds.append(hidden)
    raw = tag(10, "", tag(3, "foreignRoot", struct.pack(">i", 123))
              + tag(9, "servers", b"\x0a" + struct.pack(">i", len(compounds)) + b"".join(compounds)) + b"\0")
    (home / "servers.dat").write_bytes(raw)
    state = {"schema": 1, "settings": {"online_directory": False}, "metadata": {f"127.0.0.1:{port}": {
        "favorite": True, "tags": ["survival", "EU"], "notes": "friends local notes",
        "auth": "offline", "language": "en", "region": "EU"}}}
    (home / "horizon-server-browser.json").write_text(json.dumps(state), encoding="utf-8")
    (home / "options.txt").write_text("guiScale:2\nlang:en_us\nrenderDistance:2\nsimulationDistance:2\n", encoding="utf-8")
    (home / "horizon-extras.txt").write_text("agentControl:true\nnoAi:false\ndiscordPresence:false\n", encoding="utf-8")
    return raw


class Driver:
    def __init__(self, args, out, width, height, port):
        self.args = args
        self.scratch = Path(tempfile.mkdtemp(prefix=f"browser-{width}x{height}-", dir=out))
        self.home = self.scratch / "home"
        self.home.mkdir()
        self.original = seed(self.home, port, args.large)
        self.endpoint = self.scratch / "control.json"
        self.env = os.environ.copy()
        for key in ("OLLIE_DESKTOP", "OLLIE_TEST_INPUT", "HORIZON_SHADER_PACK", "HORIZON_CONTROL", "HORIZON_SESSION_FILE"):
            self.env.pop(key, None)
        for key, value in {
            "HORIZON_HOME": self.home, "HORIZON_LAUNCHER_HOME": self.scratch / "launcher",
            "XDG_DATA_HOME": self.scratch / "data", "XDG_CONFIG_HOME": self.scratch / "config",
            "HORIZON_LAUNCHER_TRASH": self.scratch / "trash",
            "HORIZON_LAUNCHER_CLIPBOARD": self.scratch / "clipboard",
            "HORIZON_LAUNCHER_DESKTOP": self.scratch / "desktop",
            "HORIZON_NO_DISCORD": "1", "HORIZON_NO_HEARTBEAT": "1",
            "HORIZON_TELEMETRY_URL": "off", "PYTHONUTF8": "1",
        }.items():
            self.env[key] = str(value)
        self.env = lab.game_env(self.env)
        self.log = (self.scratch / "game.log").open("w", encoding="utf-8")
        self.game = subprocess.Popen([
            str(args.game), "--offscreen", "--control", "--control-socket", str(self.endpoint),
            "--resources", str(args.resources), "--width", str(width), "--height", str(height),
            "--render-distance", "2", "--name", "BrowserFixture"],
            cwd=self.home, env=self.env, stdout=self.log, stderr=subprocess.STDOUT, creationflags=FLAGS)

    def call(self, *words, timeout=30):
        result = subprocess.run([str(self.args.ctl), "--socket", str(self.endpoint), *map(str, words)],
                                env=self.env, cwd=self.home, capture_output=True, text=True,
                                encoding="utf-8", errors="replace", timeout=timeout, creationflags=FLAGS)
        if result.returncode:
            raise RuntimeError(f"horizon-ctl {words}: {result.stdout[-1000:]} {result.stderr[-500:]}")
        reply = json.loads(result.stdout)
        if isinstance(reply, dict) and "error" in reply:
            raise RuntimeError(reply["error"])
        return reply

    def wait(self, check, message, seconds=10):
        end = time.monotonic() + seconds
        last = None
        while time.monotonic() < end:
            if self.game.poll() is not None:
                raise RuntimeError(f"Game exited ({self.game.returncode}); see {self.scratch / 'game.log'}")
            try:
                last = check()
                if last:
                    return last
            except (RuntimeError, OSError, ValueError, subprocess.TimeoutExpired):
                pass
            time.sleep(.1)
        raise RuntimeError(f"Timed out: {message}; last result: {last}")

    def ui(self):
        return self.call("ui")["widgets"]

    def node(self, name, kind=None):
        return next((w for w in self.ui() if w.get("label") == name
                     and (kind is None or w.get("kind") == kind) and w.get("enabled", True)), None)

    def click(self, name, kind=None):
        widget = self.wait(lambda: self.node(name, kind), f"widget {name}")
        return self.call("click", widget["id"])

    def field(self, name, value):
        self.click(name, "field")
        self.call("key", "ctrl", "down")
        self.call("key", "a", "tap")
        self.call("key", "ctrl", "up")
        self.call("type", value)

    def snapshot(self, path):
        self.call("screenshot", path)
        self.wait(lambda: path.exists(), "screenshot file")

    def close(self):
        try:
            if self.game.poll() is None:
                self.call("quit", timeout=15)
                self.game.wait(timeout=20)
        finally:
            if self.game.poll() is None:
                self.game.terminate()
                self.game.wait(timeout=10)
            self.log.close()


def dedicated(args, report):
    """Real server status integration, only after the caller explicitly accepts its EULA."""
    fixture_args = argparse.Namespace(**vars(args))
    fixture_args.large = 0
    d = Driver(fixture_args, args.out, 854, 480, 65020)
    server = None
    try:
        server_home = d.scratch / "dedicated"
        server_home.mkdir()
        with socket.socket() as available:
            available.bind(("127.0.0.1", 0))
            port = available.getsockname()[1]
        (server_home / "eula.txt").write_text("eula=true\n", encoding="utf-8")
        (server_home / "server.properties").write_text(
            f"server-ip=127.0.0.1\nserver-port={port}\nonline-mode=false\n"
            "white-list=false\nenforce-whitelist=false\n"
            "level-name=BrowserFixture\nlevel-type=minecraft:flat\nview-distance=2\n"
            "simulation-distance=2\nmax-players=5\nmotd=Horizon dedicated browser fixture\n",
            encoding="utf-8")
        env = d.env.copy()
        env["HORIZON_HOME"] = str(server_home)
        server_log = (server_home / "server.log").open("w", encoding="utf-8")
        server = subprocess.Popen([str(args.game), "--resources", str(args.resources), "--dedicated", "--nogui"],
                                  cwd=server_home, env=env, stdin=subprocess.PIPE,
                                  stdout=server_log, stderr=subprocess.STDOUT, text=True, creationflags=FLAGS)
        def ready():
            if server.poll() is not None:
                raise AssertionError(f"Dedicated server exited; see {server_home / 'server.log'}")
            with socket.create_connection(("127.0.0.1", port), timeout=.2):
                return True
        d.wait(ready, "real dedicated server status listener", seconds=180)
        d.wait(lambda: d.node("Multiplayer"), "title", seconds=180)
        d.click("Multiplayer")
        d.click(label("add"))
        d.field(label("name"), "Real Horizon dedicated")
        d.field(label("address"), f"127.0.0.1:{port}")
        d.click(label("apply"))
        d.field(label("search"), "Real Horizon dedicated")
        d.click("Real Horizon dedicated", "row")
        d.click(label("filters"))
        d.click(label("status") + ": " + label("any"))
        d.click(label("version") + ": " + label("any"))
        d.click(label("apply"))
        d.wait(lambda: d.node("Real Horizon dedicated", "row"), "real server answers modern status")
        d.snapshot(args.out / "browser-real-dedicated.png")
        # The compact quick-sort bar hides chips. Use the wide layout for the
        # chip-removal assertion; the matrix separately covers every GUI scale.
        d.call("options-set", "guiScale", 1)
        d.click(label("status") + " ×")
        d.wait(lambda: d.node(label("version") + " ×"), "removing one chip retains the version filter")
        d.call("options-set", "guiScale", 2)
        d.click(label("join"))
        def joined():
            remote = d.call("status").get("server") or {}
            return remote.get("joined") and remote.get("loaded")
        d.wait(joined, "Join enters the real dedicated server", seconds=120)
        d.call("disconnect")
        d.wait(lambda: d.node(label("search"), "field"), "disconnect returns to the modern browser")
        d.click(label("direct"))
        d.field(label("address"), f"127.0.0.1:{port}")
        d.click(label("apply"))
        d.wait(joined, "Direct Connect enters the real dedicated server", seconds=120)
        report["checks"].append("real isolated horizon-client --dedicated: online/compatible filters, Join and Direct Connect")
    finally:
        if server is not None:
            if server.poll() is None:
                try:
                    server.communicate("stop\n", timeout=30)
                except subprocess.TimeoutExpired:
                    server.terminate()
                    server.wait(timeout=15)
            server_log.close()
        d.close()


