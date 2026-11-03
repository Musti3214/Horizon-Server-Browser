"""Real rendered UI checks and unedited screenshots, in a disposable test profile."""
import argparse
import json
from pathlib import Path
import struct
import sys
import threading
import time

ROOT=Path(__file__).resolve().parent
sys.path.insert(0,str(ROOT/'tools'))
import native_fixture as f

def main():
    p=argparse.ArgumentParser()
    for name in ('game','ctl','resources','out'): p.add_argument('--'+name,type=Path,required=True)
    args=p.parse_args()
    for name in ('game','ctl','resources','out'): setattr(args,name,getattr(args,name).resolve())
    args.out.mkdir(parents=True,exist_ok=True)
    args.mod=None;args.large=0
    original=f.seed
    def seed(home,port,large):
        original(home,port,0)
        entries=[('Dorfwelt | Survival',f'127.0.0.1:{port}'),('Wolkeninsel | Skyblock','127.0.0.1:65010'),('Bauwelt | Kreativ','127.0.0.1:65011')]
        compounds=[f.tag(8,'name',f.nbt_string(name))+f.tag(8,'ip',f.nbt_string(ip))+b'\0' for name,ip in entries]
        raw=f.tag(10,'',f.tag(9,'servers',b'\x0a'+struct.pack('>i',len(compounds))+b''.join(compounds))+b'\0')
        (home/'servers.dat').write_bytes(raw)
        state=json.loads((home/'horizon-server-browser.json').read_text())
        state['settings']['refresh_seconds']=0
        state['metadata']={ip:{'favorite':i<2,'tags':[['survival','freunde'],['skyblock','insel'],['creative','bauen']][i], 'notes':['Gemeinsam bauen. Treffpunkt am Dorf.','Unsere Insel in den Wolken.','Platz fuer grosse Ideen.'][i],'auth':'offline','region':'EU','language':'de'} for i,(_,ip) in enumerate(entries)}
        (home/'horizon-server-browser.json').write_text(json.dumps(state),encoding='utf8')
        return raw
    f.seed=seed
    de=json.loads((ROOT/'src/lang/de_de.json').read_text(encoding='utf8'))
    label=lambda key:de['horizon.browser.'+key]
    report={'result':'FAIL','version':'2.1.0','screenshots':[],'personal_data_used':False}
    with f.StatusServer(('127.0.0.1',0),f.StatusHandler) as status:
        worker=threading.Thread(target=status.serve_forever,daemon=True);worker.start()
        d=f.Driver(args,args.out,1920,1080,status.server_address[1]);f.seed=original
        try:
            d.wait(lambda:d.node('Multiplayer'),'title',seconds=180)
            d.click('Multiplayer');d.call('options-set','lang','de_de')
            d.click('Dorfwelt | Survival','row')
            for scale in (2,3,4,1):
                d.call('options-set','guiScale',scale)
                d.wait(lambda:d.node(label('search'),'field'),'scaled Minecraft browser')
                d.call('click-at',-10,-10)
                d.ui();time.sleep(.2)
                name=f'minecraft-browser-de-scale{scale}.png'
                d.snapshot(args.out/name);report['screenshots'].append(name)
            d.call('options-set','guiScale',2)
            d.click(label('filters'))
            d.click(label('value_filters'))
            d.field(label('max_ping'),'150')
            d.call('click-at',-10,-10);d.ui()
            d.snapshot(args.out/'minecraft-filter-de.png');report['screenshots'].append('minecraft-filter-de.png')
            d.click(label('back'));d.click(label('settings'))
            assert d.node(label('menu_style')+': Minecraft')
            d.call('click-at',-10,-10);d.ui()
            d.snapshot(args.out/'minecraft-settings-de.png');report['screenshots'].append('minecraft-settings-de.png')
            d.click(label('back'))
            assert not (d.home/'modrinth').exists(), 'Explicit discovery Off must remain Off'
            assert (d.home/'servers.dat').read_bytes()==d.original
            report['result']='PASS'
        finally:
            d.close();status.shutdown();worker.join(timeout=2)
            (args.out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
    print(json.dumps(report,ensure_ascii=False),flush=True)

if __name__=='__main__':main()
