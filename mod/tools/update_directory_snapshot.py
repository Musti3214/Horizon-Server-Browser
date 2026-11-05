"""Developer-only public API snapshot. One request/minute; never scans or pings servers."""
import argparse
import datetime
import ipaddress
import json
from pathlib import Path
import re
import time
import urllib.parse
import urllib.request


def address(host, port=25565):
    host=host.strip().rstrip('.').lower()
    try:
        ip=ipaddress.ip_address(host)
        if not ip.is_global:
            raise ValueError('not a public address')
        host=str(ip)
    except ValueError as error:
        if 'not a public' in str(error):
            raise
        host=host.encode('idna').decode('ascii')
        if len(host)>253 or '.' not in host or host.endswith(('.local','.localhost','.internal')) or not all(
            1<=len(label)<=63 and re.fullmatch(r'[a-z0-9](?:[a-z0-9-]*[a-z0-9])?',label)
            for label in host.split('.')
        ):
            raise ValueError('invalid host')
    port=int(port)
    if not 1<=port<=65535:
        raise ValueError('invalid port')
    return f'[{host}]:{port}' if ':' in host else f'{host}:{port}'


def existing_keys(source):
    source=source.split('// Public listing fallback, refreshed ',1)[0]
    keys=set()
    for _,raw in re.findall(r'\(\s*"([^"\n]+)",\s*"([^"\n]+)"',source):
        parsed=urllib.parse.urlsplit('minecraft://'+raw)
        keys.add(address(parsed.hostname,parsed.port or 25565))
    return keys


def clean(text, length):
    return ''.join(c for c in str(text) if ord(c)>=32 and ord(c)!=127)[:length]


def main():
    p=argparse.ArgumentParser()
    p.add_argument('--out',required=True,type=Path)
    args=p.parse_args()
    project=Path(__file__).resolve().parents[1]
    baseline=existing_keys((project/'src/catalog.rs').read_text(encoding='utf8'))
    seen=set(baseline)
    added=[]
    pages=[]
    previous=0.0
    page=1
    total_pages=1
    args.out.mkdir(parents=True,exist_ok=True)
    while page<=total_pages:
        delay=60.5-(time.monotonic()-previous)
        while delay>0:
            time.sleep(min(delay,10))
            delay=60.5-(time.monotonic()-previous)
        url=f'https://minecraftserve.rs/api/servers?edition=java&per=50&page={page}'
        previous=time.monotonic()
        request=urllib.request.Request(url,headers={'User-Agent':'Horizon-Server-Browser/2.0 catalog maintenance','Accept':'application/json'})
        with urllib.request.urlopen(request,timeout=20) as response:
            data=response.read(2097153)
        if len(data)>2097152:
            raise ValueError('oversized API page')
        result=json.loads(data)
        assert result['page']==page and len(result['servers'])<=50
        total_pages=min(int(result['pages']),200)
        pages.append({'page':page,'pages':result['pages'],'total':result['total'],'entries':len(result['servers'])})
        for server in result['servers']:
            if server.get('edition')!='java':
                continue
            try:
                key=address(server['host'],server.get('port',25565))
            except (ValueError,KeyError,UnicodeError):
                continue
            if key in seen:
                continue
            seen.add(key)
            tags = [clean(t,64) for t in server.get('tags',[])[:32] if isinstance(t,str)]
            if any(tag.strip().casefold() in {"cracked", "offline-mode"} for tag in tags):
                continue
            added.append({'name':clean(server.get('name',server['host']),256),'address':key,'tags':tags})
        report={'source':'https://minecraftserve.rs/api-docs','checked_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'requests':pages,'baseline':len(baseline),'new_unique':len(added),'complete':page==total_pages,'servers':added}
        (args.out/'snapshot.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
        print(f'Public API page {page}/{total_pages}: {len(added)} additional unique public Java addresses',flush=True)
        page+=1
    literal=lambda s:json.dumps(s,ensure_ascii=False)
    rows=['// Public listing fallback, refreshed '+report['checked_utc'][:10]+'. Live discovery remains authoritative.','pub const DIRECTORY_SNAPSHOT: &[(&str, &str, &[&str])] = &[']
    for s in added:
        rows.append('    ('+literal(s['name'])+','+literal(s['address'])+',&['+','.join(map(literal,s['tags']))+']),')
    rows.append('];')
    (args.out/'directory_snapshot.rs').write_text('\n'.join(rows)+'\n',encoding='utf8')


if __name__=='__main__':
    main()
