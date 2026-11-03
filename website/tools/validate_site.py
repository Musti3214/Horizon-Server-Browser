"""Release checks: public pages, internal links, metadata, gallery and real download."""
from html.parser import HTMLParser
import hashlib
import json
import os
from pathlib import Path
from urllib.parse import urlsplit, unquote
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
DIST = ROOT / 'dist'
BASE_PATH=os.environ.get('SITE_BASE_PATH','/Horizon-Server-Browser').rstrip('/')
ORIGIN=os.environ.get('SITE_ORIGIN','https://musti3214.github.io').rstrip('/')+BASE_PATH
CANONICAL=os.environ.get('SITE_CANONICAL_ORIGIN',ORIGIN).rstrip('/')

class Page(HTMLParser):
    def __init__(self, path):
        super().__init__()
        self.path = path
        self.links = []
        self.ids = set()
        self.h1 = 0
        self.title = ''
        self.in_title = False
        self.in_json = False
        self.raw_json = ''
        self.schemas = []
        self.description = None
        self.canonical = None
        self.lang = None
        self.alternates = {}
        self.support = False
    def handle_starttag(self, tag, pairs):
        a = dict(pairs)
        if a.get('id'):
            assert a['id'] not in self.ids, (self.path, 'duplicate id', a['id'])
            self.ids.add(a['id'])
        if tag == 'html': self.lang = a.get('lang')
        if tag == 'h1': self.h1 += 1
        if tag == 'title': self.in_title = True
        if tag == 'meta' and a.get('name') == 'description': self.description = a.get('content')
        if tag == 'link' and a.get('rel') == 'canonical': self.canonical = a.get('href')
        if tag == 'link' and a.get('rel') == 'alternate': self.alternates[a.get('hreflang')]=a.get('href')
        if tag == 'a' and a.get('href') == 'https://ko-fi.com/foryou41': self.support = True
        if tag == 'script' and a.get('type') == 'application/ld+json': self.in_json = True
        for attr in ('src', 'href'):
            if a.get(attr): self.links.append(a[attr])
        if tag == 'img': assert 'alt' in a and (a['alt'] or 'src' not in a), (self.path, 'missing image alt')
    def handle_endtag(self, tag):
        if tag == 'title': self.in_title = False
        if tag == 'script' and self.in_json:
            self.schemas += json.loads(self.raw_json)
            self.raw_json = ''
            self.in_json = False
    def handle_data(self, data):
        if self.in_title: self.title += data
        if self.in_json: self.raw_json += data

pages = {}
for path in DIST.rglob('*.html'):
    parser = Page(path.relative_to(DIST).as_posix())
    parser.feed(path.read_text(encoding='utf8'))
    assert parser.lang in ('en','de','fr','es') and parser.h1 == 1, parser.path
    assert 20 <= len(parser.title) <= 85, (parser.path, parser.title)
    assert parser.description and len(parser.description) >= (40 if parser.path == '404.html' else 60), parser.path
    assert parser.canonical.startswith(CANONICAL), parser.path
    assert parser.support, (parser.path,'missing creator support')
    assert set(parser.alternates)=={'en','de','fr','es','x-default'}, parser.path
    assert parser.schemas, parser.path
    pages[parser.path] = parser
assert len(pages) == 25
assert all('world-hero' not in (DIST/p).read_text(encoding='utf8') for p in pages), 'AI image is no longer part of the site'
assert len({p.title for p in pages.values()}) == len(pages)
checked = 0
for page in pages.values():
    for link in page.links:
        url = urlsplit(link)
        if url.scheme or url.netloc or link == '#': continue
        assert link.startswith('/') or link.startswith('#'), (page.path, 'relative link', link)
        if url.path:
            assert url.path.startswith(BASE_PATH+'/'),(page.path,'outside project path',link)
        local_path=url.path[len(BASE_PATH):] if url.path else ''
        target = DIST / unquote(local_path).lstrip('/') if local_path else DIST / page.path
        if target.is_dir(): target /= 'index.html'
        assert target.is_file(), (page.path, 'broken link', link)
        if url.fragment:
            p = pages.get(target.relative_to(DIST).as_posix())
            assert p and unquote(url.fragment) in p.ids, (page.path, 'broken anchor', link)
        checked += 1

gallery = json.loads((ROOT/'gallery.json').read_text(encoding='utf8'))
assert len(gallery) == 13
for item in gallery:
    png = (DIST/'assets/gallery'/item['file']).read_bytes()
    assert png[:8] == b'\x89PNG\r\n\x1a\n' and len(png) > 100_000, item['file']
    assert int.from_bytes(png[16:20], 'big') == 1920 and int.from_bytes(png[20:24], 'big') == 1080, item['file']
servers = json.loads((DIST/'assets/servers.json').read_text(encoding='utf8'))['servers']
assert len(servers) == len({s['address'].lower() for s in servers}) == 2835
sha = hashlib.sha256((DIST/'downloads/server-browser-2.1.0.hnmod').read_bytes()).hexdigest()
assert sha == (DIST/'downloads/SHA256SUMS.txt').read_text().split()[0]
sitemap = ET.parse(DIST/'sitemap.xml').getroot()
assert len(sitemap) == 24
by_url={p.canonical:p for p in pages.values() if p.path!='404.html'}
assert len(by_url)==24
assert {p.lang for p in by_url.values()}=={'en','de','fr','es'}
for page in by_url.values():
    for lang,href in page.alternates.items():
        target=by_url[href]
        assert target.lang == ('en' if lang=='x-default' else lang)
        assert target.alternates == page.alternates, (page.path,'nonreciprocal hreflang')
    assert page.alternates[page.lang]==page.canonical
assert {item.find('{http://www.sitemaps.org/schemas/sitemap/0.9}loc').text for item in sitemap} == set(by_url)
for item in sitemap:
    alternate={a.attrib['hreflang']:a.attrib['href'] for a in item.findall('{http://www.w3.org/1999/xhtml}link')}
    assert alternate == by_url[item.find('{http://www.sitemaps.org/schemas/sitemap/0.9}loc').text].alternates
print(json.dumps({'result':'PASS','html_pages':len(pages),'internal_references':checked,'gallery_screenshots':len(gallery),'server_addresses':len(servers),'download_sha256':sha,'sitemap_urls':len(sitemap)}))
