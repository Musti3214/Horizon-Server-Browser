"""Opt-in developer check of the actual public directory from a fresh game home."""
import argparse
import json
from pathlib import Path
import sys
import time

ROOT = Path(__file__).resolve().parent
sys.path.insert(0,str(ROOT / "tools"))
import native_fixture as fixture


def main():
    parser=argparse.ArgumentParser()
    for name in ("game","ctl","resources","out"):
        parser.add_argument("--"+name,type=Path,required=True)
    parser.add_argument("--allow-public-directory",action="store_true",required=True)
    args=parser.parse_args()
    for name in ("game","ctl","resources","out"):
        setattr(args,name,getattr(args,name).resolve())
    args.out.mkdir(parents=True,exist_ok=True)
    args.mod=None
    args.large=3
    original_seed=fixture.seed
    def fresh_seed(home,port,large):
        raw=original_seed(home,port,large)
        path=home/"horizon-server-browser.json"
        state=json.loads(path.read_text(encoding="utf8"))
        state.pop("settings",None)
        path.write_text(json.dumps(state),encoding="utf8")
        return raw
    fixture.seed=fresh_seed
    d=fixture.Driver(args,args.out,1920,1080,65020)
    fixture.seed=original_seed
    report={"result":"FAIL"}
    try:
        d.wait(lambda:d.node("Multiplayer"),"title",seconds=180)
        d.click("Multiplayer")
        d.click(fixture.label("discover"))
        labels=[w.get("label") for w in d.ui()]
        assert fixture.label("load_directory") not in labels
        assert fixture.label("directory_save") not in labels
        def cached_page():
            for path in (d.home/"server-directory-cache").glob("*.json"):
                value=json.loads(path.read_text(encoding="utf8"))
                if value.get("page",{}).get("servers"):
                    return value["page"]
            return None
        page=d.wait(cached_page,"real public directory response parsed and cached",seconds=35)
        assert page["page"]==1 and len(page["servers"])>0
        def modrinth_page():
            for path in (d.home/"modrinth/server-directory-cache").glob("*.json"):
                value=json.loads(path.read_text(encoding="utf8"))
                if value.get("page",{}).get("servers"):
                    return value["page"]
            return None
        extra=d.wait(modrinth_page,"real Modrinth Java page parsed and cached",seconds=35)
        assert len(extra["servers"])>0
        assert all(s["metadata"]["group"]=="Modrinth" for s in extra["servers"])
        assert not extra["statuses"], "Third-party ping metadata is not our own current ping"
        d.wait(lambda:len([w for w in d.ui() if w.get("kind")=="row"])>0,"discovered rows")
        d.call('options-set', 'lang', 'de_de')
        de=json.loads((ROOT/'src/lang/de_de.json').read_text(encoding='utf8'))
        d.wait(lambda:d.node(de['horizon.browser.search'],'field'),'German directory UI')
        d.call('click-at', 1, 1)
        d.ui()
        time.sleep(.5)
        d.snapshot(args.out/"native-discover-online.png")
        d.call('options-set', 'lang', 'en_us')
        assert (d.home/"servers.dat").read_bytes()==d.original
        d.click(fixture.label("settings"))
        d.click(fixture.label("online_directory")+": "+fixture.label("on"))
        d.click(fixture.label("apply"))
        state=json.loads((d.home/"horizon-server-browser.json").read_text(encoding="utf8"))
        assert state["settings"]["online_directory"] is False
        report.update(result="PASS",checks=["Fresh settings enable Discover; real HTTPS pages from minecraftserve.rs and Modrinth are parsed/cached/displayed; no external ping metadata claimed as a direct ping; Load directory and Save selected absent; explicit Off persists; servers.dat unchanged"],public_servers_in_first_page=len(page["servers"]),modrinth_servers_in_first_page=len(extra["servers"]))
    finally:
        d.close()
        (args.out/"report.json").write_text(json.dumps(report,indent=2),encoding="utf8")
        print(json.dumps(report,indent=2),flush=True)


if __name__=="__main__":
    main()
