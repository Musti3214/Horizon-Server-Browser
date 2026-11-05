"""Developer-only public Java server snapshot; no accounts, scraping, scans or pings."""
import argparse
import datetime
import json
from pathlib import Path
import re
import time
import urllib.parse
import urllib.request

from update_directory_snapshot import address, clean

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    project = Path(__file__).resolve().parents[1]
    source = (project/'src/catalog.rs').read_text(encoding='utf8')
    baseline = set()
    for raw in re.findall(r'\(\s*"(?:\\.|[^"\\])*"\s*,\s*("(?:\\.|[^"\\])*")', source):
        parsed = urllib.parse.urlsplit('minecraft://' + json.loads(raw))
        baseline.add(address(parsed.hostname, parsed.port or 25565))
    seen = set(baseline)
    entries, pages = [], []
    args.out.mkdir(parents=True, exist_ok=True)
    offset, total = 0, 1
    while offset < total:
        query = urllib.parse.urlencode({'facets':json.dumps([['project_types:minecraft_java_server']],separators=(',',':')), 'limit':50, 'offset':offset, 'index':'relevance'})
        url = 'https://api.modrinth.com/v3/search?' + query
        request = urllib.request.Request(url,headers={'User-Agent':'HorizonServerBrowser/2.1.0 (+https://server-browser-mod.indiebini.chatgpt.site)','Accept':'application/json'})
        with urllib.request.urlopen(request,timeout=25) as response:
            raw = response.read(8388609)
        if len(raw)>8388608: raise ValueError('Oversized API response')
        data = json.loads(raw)
        hits = data['hits']
        assert len(hits)<=50
        total = min(data['total_hits'],10000)
        for hit in hits:
            if 'minecraft_java_server' not in hit.get('project_types',[]): continue
            java = hit.get('minecraft_java_server') or {}
            try:
                parsed = urllib.parse.urlsplit('minecraft://'+java['address'].strip())
                if parsed.username or parsed.password or parsed.path or parsed.query or parsed.fragment: continue
                key = address(parsed.hostname,parsed.port or 25565)
            except (KeyError,ValueError,UnicodeError,AttributeError,TypeError): continue
            if key in seen: continue
            seen.add(key)
            tags = [clean(t,64) for t in hit.get('categories',[])[:31] if isinstance(t,str)]
            if any(tag.strip().casefold() in {"cracked", "offline-mode"} for tag in tags):
                continue
            if java.get('content',{}).get('kind')=='modpack' and 'modded' not in tags: tags.append('modded')
            entries.append({'name':clean(hit.get('name') or parsed.hostname,256),'address':key,'tags':tags,'source':'https://modrinth.com/server/'+hit['slug']})
        pages.append({'offset':offset,'entries':len(hits),'total':data['total_hits']})
        offset += len(hits)
        if not hits and offset<total: raise ValueError('Unexpected empty page')
        report = {'source':'https://api.modrinth.com/v3/search','discovered_via':'https://serverify.louchat.site/docs/horizon','checked_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'baseline':len(baseline),'new_unique':len(entries),'requests':pages,'complete':offset>=total,'servers':entries}
        (args.out/'snapshot.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
        print(f'Modrinth: {offset}/{total} public entries, {len(entries)} new unique Java addresses',flush=True)
        if offset<total: time.sleep(1.0)
    literal=lambda s:json.dumps(s,ensure_ascii=False)
    rows=['// Additional public Modrinth addresses, checked '+report['checked_utc'][:10]+'.','pub const MODRINTH_SNAPSHOT: &[(&str, &str, &[&str])] = &[']
    rows += ['    ('+literal(s['name'])+','+literal(s['address'])+',&['+','.join(map(literal,s['tags']))+']),' for s in entries]
    (args.out/'modrinth_snapshot.rs').write_text('\n'.join(rows+["];\n"]),encoding='utf8')

if __name__=='__main__': main()
