"""Validate and package only this project's built mods, never game/runtime files."""
from pathlib import Path
import hashlib
import io
import json
import shutil
import zipfile

ROOT = Path(__file__).resolve().parents[1]
VERSION = "2.2.0"


def inspect(path, loader):
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        for name in names:
            if name.endswith(".class") and not name.startswith("io/github/musti3214/serverbrowser/"):
                raise ValueError(f"Unexpected class in release: {name}")
            if name.startswith(("net/minecraft/", "com/mojang/", "logs/", "run/", ".local/")):
                raise ValueError(f"Private game artifact in release: {name}")
        for required in ("LICENSE", "THIRD-PARTY.md", "pack.mcmeta", "server-browser.mixins.json", "assets/server_browser/lang/en_us.json", "assets/server_browser/lang/de_de.json", "assets/server_browser/catalog.json"):
            assert required in names, required
        assert len(json.loads(archive.read("assets/server_browser/catalog.json"))["servers"]) == 2835
        if loader == "fabric":
            meta = json.loads(archive.read("fabric.mod.json"))
            assert meta["id"] == "server_browser" and meta["version"] == VERSION
            bundled = [n for n in names if n.startswith("META-INF/jars/") and n.endswith(".jar")]
            assert len(bundled) == 2
            ids = set()
            for name in bundled:
                with zipfile.ZipFile(io.BytesIO(archive.read(name))) as nested:
                    ids.add(json.loads(nested.read("fabric.mod.json"))["id"])
                    assert any(n.startswith("LICENSE") for n in nested.namelist())
            assert ids == {"fabric-api-base", "fabric-resource-loader-v1"}
        else:
            assert "META-INF/mods.toml" in names
            assert "io/github/musti3214/serverbrowser/forge/ServerBrowserForge.class" in names
            assert b"MixinConfigs: server-browser.mixins.json" in archive.read("META-INF/MANIFEST.MF")


def main():
    output = ROOT / "release"
    output.mkdir(exist_ok=True)
    manifest = {"version": VERSION, "minecraft": "26.3", "java": 25, "files": []}
    for loader in ("fabric", "forge"):
        source = ROOT / loader / "build/libs" / f"server-browser-{loader}-{VERSION}.jar"
        inspect(source, loader)
        target = output / f"server-browser-{loader}-{VERSION}+mc26.3.jar"
        shutil.copyfile(source, target)
        manifest["files"].append({"name": target.name, "loader": loader, "size": target.stat().st_size, "sha256": hashlib.sha256(target.read_bytes()).hexdigest()})
    for name in ("README.md", "LICENSE", "THIRD-PARTY.md", "VALIDATION.md"):
        shutil.copyfile(ROOT / name, output / name)
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    (output / "SHA256SUMS.txt").write_text("".join(f"{f['sha256']}  {f['name']}\n" for f in manifest["files"]), encoding="ascii")
    print(json.dumps(manifest, indent=2))


if __name__ == "__main__":
    main()
