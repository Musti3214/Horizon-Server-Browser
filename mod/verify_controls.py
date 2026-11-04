"""Targeted native controller keyboard smoke test in an independent home."""
import argparse
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / "tools"))
import native_fixture as fixture


def main():
    p = argparse.ArgumentParser()
    for name in ("game", "ctl", "resources", "out"):
        p.add_argument("--" + name, required=True, type=Path)
    args = p.parse_args()
    for name in ("game", "ctl", "resources", "out"):
        setattr(args, name, getattr(args, name).resolve())
    args.out.mkdir(parents=True, exist_ok=True)
    args.large = 3
    args.mod = None
    report = {"result": "FAIL"}
    d = fixture.Driver(args, args.out, 1280, 720, 65020)
    try:
        d.wait(lambda: d.node("Multiplayer"), "title", seconds=180)
        d.click("Multiplayer")
        d.click(fixture.label("search"), "field")
        # Current Horizon initializes pad focus on the first registered
        # header button, independently of mouse focus. Navigate through Clear
        # to the adjacent search field with actual controller input.
        d.call("pad", "up", "tap")
        d.call("pad", "down", "tap")
        d.call("pad", "left", "tap")
        d.wait(lambda: (d.call("pad", "status")["menu"].get("focus") or {}).get("kind") == "field", "controller search focus")
        d.call("pad", "south", "tap")
        d.wait(lambda: d.call("pad", "status")["menu"]["keyboard"], "controller keyboard open")
        d.call("pad", "south", "tap")  # Horizon keyboard starts on q.
        d.wait(lambda: d.node("Quiet sandbox", "row") and not d.node("Local friends", "row"), "controller typed q")
        d.snapshot(args.out / "controller-keyboard.png")
        d.call("pad", "west", "tap")
        d.wait(lambda: d.node("Local friends", "row"), "controller backspace")
        d.call("pad", "east", "tap")
        d.wait(lambda: not d.call("pad", "status")["menu"]["keyboard"], "controller keyboard closed")
        d.call("pad", "release")
        assert (d.home / "servers.dat").read_bytes() == d.original
        report.update(result="PASS", checks=["Controller focuses native search, opens Horizon keyboard, types q, backspaces and closes; servers.dat unchanged"])
    except Exception:
        try:
            d.snapshot(args.out / "failed.png")
            report["pad"] = d.call("pad", "status")
            report["ui"] = d.ui()
        except Exception:
            pass
        raise
    finally:
        d.close()
        (args.out / "report.json").write_text(json.dumps(report, indent=2), encoding="utf8")
        print(json.dumps(report, indent=2), flush=True)


if __name__ == "__main__":
    main()
