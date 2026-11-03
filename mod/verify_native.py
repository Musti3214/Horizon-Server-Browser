"""Real native-package UI checks; every client/server uses a fresh scratch home."""
import argparse
import json
from pathlib import Path
import sys
import threading

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / "tools"))
import native_fixture as fixture


def workflow(d, report):
    label = fixture.label
    d.wait(lambda: d.node("Multiplayer"), "title", seconds=180)
    d.click("Multiplayer")
    d.wait(lambda: d.node(label("search"), "field"), "native browser")
    assert len([w for w in d.ui() if w.get("kind") == "row"]) < 40
    d.snapshot(d.args.out / "native-browser-start.png")
    d.field(label("search"), "Local friends")
    d.wait(lambda: d.node("Local friends", "row"), "search")
    d.click(label("clear"))
    d.click(label("add"))
    d.field(label("name"), "Native Added")
    d.field(label("address"), "127.0.0.1:65001")
    d.field(label("tags"), "friends, testing")
    d.field(label("notes"), "native mod note")
    d.click(label("apply"))
    d.field(label("search"), "Native Added")
    d.click("Native Added", "row")
    d.click(label("favorite"))
    d.click(label("filters"))
    d.click(label("status") + ": " + label("any"))
    d.click(label("apply"))
    d.wait(lambda: not d.node("Native Added", "row"), "online filter")
    d.click(label("reset"))
    d.field(label("search"), "Native Added")
    d.click("Native Added", "row")
    d.click(label("remove"))
    d.click(label("confirm_remove"))
    d.wait(lambda: not d.node("Native Added", "row"), "remove")
    d.click(label("undo"))
    d.click("Native Added", "row")
    d.click(label("up"))
    d.click("…")
    d.click(label("presets"))
    d.field(label("preset_name"), "Native preset")
    d.click(label("apply"))
    d.click(label("clear"))
    d.click("…")
    d.click(label("presets"))
    d.click("Native preset")
    d.wait(lambda: d.node("Native Added", "row"), "preset")
    d.click(label("clear"))
    assert (d.home / "servers.dat.before-modern-browser").read_bytes() == d.original
    assert not (d.home / "server-directory-cache").exists()
    for button, tab in [("rb", "discover"), ("rb", "favorites"), ("lb", "discover"), ("lb", "my_servers")]:
        d.call("pad", button, "tap")
        d.wait(lambda: d.node("• " + label(tab)), "controller tab " + tab)
    d.call("pad", "release")
    d.click(label("classic"))
    d.wait(lambda: any(w.get("label")=="Join Server" for w in d.ui()), "Classic")
    assert not d.node(label("search"), "field")
    d.click(label("title"))
    d.wait(lambda: d.node(label("search"), "field"), "modern restored from Classic")
    d.call("key", "ctrl", "down")
    d.call("key", "f", "tap")
    d.call("key", "ctrl", "up")
    d.call("type", "Local friends")
    d.wait(lambda: d.node("Local friends", "row"), "Ctrl+F")
    d.click(label("clear"))
    d.snapshot(d.args.out / "native-browser-workflows.png")
    d.click("💬")
    d.click(label("feedback_server"))
    d.field(label("feedback_server_prompt"),"public.example.org | Public suggestion fixture")
    assert not any("m.kabo" in str(w.get("label","")) or "outlook.de" in str(w.get("label","")) for w in d.ui())
    d.snapshot(d.args.out / "native-feedback.png")
    d.click(label("feedback_save"))
    assert "public.example.org" in (d.home/"server_submissions.log").read_text(encoding="utf8")
    assert not (d.home/"feedback-last-attempt.json").exists(), "Saving a draft must not submit anything"
    report["checks"].append("10000 saved entries: virtualized UI, search, add, fields, favorite, filter, remove/undo, reorder, preset, controller tabs, Classic/modern and Ctrl+F")
    report["checks"].append("Public server proposal draft saved locally without starting a mail request; recipient absent from game UI")


def lifecycle(args, report):
    original_seed = fixture.seed
    def disabled_seed(home, port, large):
        raw = original_seed(home, port, large)
        (home / "horizon-mods.txt").write_text("mod.server_browser:false\n", encoding="utf8")
        return raw
    fixture.seed = disabled_seed
    d = fixture.Driver(args, args.out, 854, 480, 65020)
    fixture.seed = original_seed
    try:
        d.wait(lambda: d.node("Multiplayer"), "disabled mod title", seconds=180)
        d.click("Multiplayer")
        d.wait(lambda: any(w.get("label")=="Join Server" for w in d.ui()), "disabled mod Classic")
        assert not d.node(fixture.label("search"), "field")
        assert (d.home / "servers.dat").read_bytes() == d.original
        d.click("Back")
        d.click("Options...")
        d.click("Mods...")
        row = d.wait(lambda: next((w for w in d.ui() if w.get("label", "").startswith("Modern Server Browser 2.1.0")), None), "native mod toggle")
        d.call("click", row["id"])
        d.click("Done")
        d.click("Done")
        d.click("Multiplayer")
        d.wait(lambda: d.node(fixture.label("search"), "field"), "enabled native mod")
        assert (d.home / "servers.dat").read_bytes() == d.original
        d.click(fixture.label("back"))
        d.click("Options...")
        d.click("Mods...")
        row = d.wait(lambda: next((w for w in d.ui() if w.get("label", "").startswith("Modern Server Browser 2.1.0")), None), "native mod off toggle")
        d.call("click", row["id"])
        d.click("Done")
        d.click("Done")
        d.click("Multiplayer")
        d.wait(lambda: any(w.get("label")=="Join Server" for w in d.ui()), "disabled native mod restores Classic")
        assert (d.home / "servers.dat").read_bytes() == d.original
        report["checks"].append("Options > Mods toggles native browser on/off; disabled mod keeps Classic and byte-identical servers.dat")
    finally:
        d.close()


def matrix(args, report):
    for width, height in [(854, 480), (1920, 1080), (3840, 2160)]:
        d = fixture.Driver(args, args.out, width, height, 65020)
        try:
            d.wait(lambda: d.node("Multiplayer"), "matrix title", seconds=180)
            d.click("Multiplayer")
            for scale in [1, 2, 3, 4, 0]:
                d.call("options-set", "guiScale", scale)
                d.wait(lambda: d.node(fixture.label("search"), "field"), "matrix browser")
                path = args.out / f"native-{width}x{height}-scale{scale}.png"
                d.snapshot(path)
                report["screenshots"].append(str(path))
            d.call("options-set", "lang", "de_de")
            path = args.out / f"native-{width}x{height}-de.png"
            d.snapshot(path)
            report["screenshots"].append(str(path))
        finally:
            d.close()
    report["checks"].append("18 screenshots: 854x480, 1920x1080, 3840x2160; GUI scales 1/2/3/4/auto and German")


def main():
    p = argparse.ArgumentParser()
    for name in ["game", "ctl", "resources", "out"]:
        p.add_argument("--" + name, type=Path, required=True)
    p.add_argument("--accept-eula", action="store_true")
    p.add_argument("--quick", action="store_true")
    args = p.parse_args()
    for name in ["game", "ctl", "resources", "out"]:
        setattr(args, name, getattr(args, name).resolve())
    args.out.mkdir(parents=True, exist_ok=True)
    args.mod = None
    args.large = 10000
    report = {"result": "FAIL", "checks": [], "screenshots": []}
    try:
        with fixture.StatusServer(("127.0.0.1", 0), fixture.StatusHandler) as status:
            thread=threading.Thread(target=status.serve_forever, daemon=True)
            thread.start()
            try:
                d = fixture.Driver(args, args.out, 854, 480, status.server_address[1])
                try:
                    workflow(d, report)
                except Exception:
                    try:
                        d.snapshot(args.out / "failed.png")
                        (args.out / "failed-ui.json").write_text(json.dumps(d.ui(), indent=2), encoding="utf8")
                    except Exception:
                        pass
                    raise
                finally:
                    d.close()
            finally:
                status.shutdown()
                thread.join(timeout=2)
        lifecycle(args, report)
        if not args.quick:
            matrix(args, report)
            if args.accept_eula:
                fixture.dedicated(args, report)
        report["result"] = "PASS"
    finally:
        (args.out / "report.json").write_text(json.dumps(report, indent=2), encoding="utf8")
        print(json.dumps(report, indent=2), flush=True)


if __name__ == "__main__":
    main()
