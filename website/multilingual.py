"""Render complete language-specific pages without network services or dependencies."""
from pathlib import Path
import html, json, os, hashlib, shutil
from urllib.parse import urlsplit
ROOT=Path(__file__).resolve().parent
DIST=ROOT/'dist'
BASE=os.environ.get('SITE_BASE_PATH','/Horizon-Server-Browser').rstrip('/')
HOST=os.environ.get('SITE_ORIGIN','https://musti3214.github.io').rstrip('/')
CANONICAL=os.environ.get('SITE_CANONICAL_ORIGIN',HOST+BASE).rstrip('/')
assert urlsplit(CANONICAL).scheme in ('http','https') and urlsplit(CANONICAL).netloc
assert not BASE or (BASE.startswith('/') and not BASE.startswith('//') and '..' not in BASE and not any(c in BASE for c in '\\?"#'))
VERSION,DAY='2.1.0','2026-11-03'
SHA='bb6bb8e5e89299a0d853ce85db520205564da8b77576b61fbf6054eb200bfd58'
REPO='https://github.com/Musti3214/Horizon-Server-Browser'
SUPPORT='https://ko-fi.com/foryou41'
SUPPORT_TEXT=dict(en='Support this project',de='Projekt unterstützen',fr='Soutenir le projet',es='Apoyar el proyecto')
L={code:json.loads((ROOT/'locales'/f'{code}.json').read_text(encoding='utf8')) for code in ('en','de','fr','es')}
for code,t in L.items():
    assert t.keys()==L['en'].keys() and len(t['gallery'])==13 and len(t['routes'])==6,code
G=json.loads((ROOT/'gallery.json').read_text(encoding='utf8'))
S=json.loads((ROOT/'data/servers.json').read_text(encoding='utf8'))['servers']
assert len(S)==len({s['address'].lower() for s in S})==2835
def e(s): return html.escape(str(s),quote=True)
def br(s): return '<br>'.join(map(e,s))
def route(lang,i=0):
    parts=([lang] if lang!='en' else [])+([L[lang]['routes'][i]] if i else [])
    return '/'+('/'.join(parts)+'/' if parts else '')
def u(lang,i=0): return BASE+route(lang,i)
def absolute(lang,i=0): return CANONICAL+route(lang,i)
def asset(file): return BASE+'/assets/'+file+'?v='+VERSION
def icon(kind='arrow'):
    d={'arrow':'M4 10h12m-5-5 5 5-5 5','download':'M10 3v10m-4-4 4 4 4-4M4 14v3h12v-3','copy':'M7 6h9v11H7zM12 6V3H3v11h4','check':'m4 10 4 4 8-8','search':'M13 13l4 4M14 8.5a5.5 5.5 0 1 1-11 0 5.5 5.5 0 0 1 11 0'}[kind]
    return f'<svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="{d}"/></svg>'
def brand(): return '<svg class="brand-icon" viewBox="0 0 32 36" aria-hidden="true"><path d="m16 1 15 8.5v17L16 35 1 26.5v-17Z" fill="#9fbd8e"/><path d="m1 9.5 15 8.5 15-8.5L16 1Z" fill="#bed2b0"/><path d="M16 18v17l15-8.5v-17Z" fill="#628752"/></svg>'
def cube(): return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.3" aria-hidden="true"><path d="m12 2 9 5v10l-9 5-9-5V7ZM3 7l9 5 9-5M12 12v10"/></svg>'
def btn(label,href,style='primary',symbol='arrow',download=False): return f'<a class="button {style}" href="{e(href)}"'+(' download' if download else '')+f'>{e(label)}{icon(symbol)}</a>'
def link(label,href): return f'<a class="text-link" href="{e(href)}">{e(label)} {icon()}</a>'
def wordmark(lang): return f'<a class="wordmark" href="{u(lang)}">{brand()}<span>Server Browser</span></a>'
def languages(lang,i): return ''.join(f'<a href="{u(code,i)}" lang="{code}" hreflang="{code}"'+(' aria-current="true"' if code==lang else '')+f'>{e(L[code]["name"])}</a>' for code in L)
def nav(lang,i):
    t=L[lang]
    items=''.join(f'<a href="{u(lang,j)}"'+(' aria-current="page"' if j==i else '')+f'>{e(t["pages"][j])}</a>' for j in (0,1,2,4))
    options=''.join(f'<option value="{u(code,i)}"'+(' selected' if code==lang else '')+f'>{e(L[code]["name"])}</option>' for code in L)
    return f'''<a class="skip" href="#main">{e(t['skip'])}</a><header class="site-nav" data-open="false"><div class="container nav-inner">{wordmark(lang)}<button class="mobile-toggle" type="button" aria-expanded="false" aria-controls="navigation" aria-label="{e(t['open_menu'])}"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 8h16M4 16h16"/></svg></button><nav class="nav-links" id="navigation" aria-label="{e(t['navigation'])}">{items}<a class="nav-download" href="{u(lang,3)}">{e(t['download_button'])}</a><label class="language-picker"><span class="sr-only">{e(t['language'])}</span><select data-language aria-label="{e(t['language'])}">{options}</select></label></nav></div></header><noscript><nav class="container language-fallback">{languages(lang,i)}</nav></noscript>'''
def footer(lang,i):
    t=L[lang]; links=''.join(f'<a href="{u(lang,j)}">{e(t["pages"][j])}</a>' for j in (2,3,4,5))
    return f'''<footer class="site-footer"><div class="container"><div class="footer-top">{wordmark(lang)}<nav class="footer-links" aria-label="{e(t['more_links'])}">{links}<a href="{REPO}">GitHub ↗</a><a class="support-link" href="{SUPPORT}" rel="noopener">{e(SUPPORT_TEXT[lang])} ↗</a></nav></div><nav class="footer-languages" aria-label="{e(t['language'])}">{languages(lang,i)}</nav><div class="footer-small"><p>{e(t['legal'])}</p><span>{e(t['footer_note'])}</span></div></div></footer><div class="toast" id="toast" role="status" aria-live="polite" aria-atomic="true"></div>'''
def hero(lang,i,key):
    t=L[lang]
    return f'<header class="page-hero"><nav class="breadcrumbs"><a href="{u(lang)}">Server Browser</a><span aria-hidden="true">/</span><span aria-current="page">{e(t["pages"][i])}</span></nav><div class="eyebrow">{e(t[key+"_eyebrow"])}</div><h1>{br(t[key+"_h1"])}</h1><p class="lead">{e(t[key+"_intro"])}</p></header>'
def head(eyebrow,title,intro): return f'<div class="section-head"><div><div class="eyebrow">{e(eyebrow)}</div><h2>{br(title) if isinstance(title,list) else e(title)}</h2></div><p>{e(intro)}</p></div>'
def photo(lang,i,large=False):
    title,label,desc=L[lang]['gallery'][i]; src=asset('gallery/'+G[i]['file'])
    return f'<a class="{"hero-photo" if large else "gallery-photo"}" href="{src}" data-gallery data-title="{e(title)}" data-caption="{e(desc)}" aria-label="{e(label)} — {e(L[lang]["enlarge"])}"><img src="{src}" width="1920" height="1080" '+('fetchpriority="high"' if large else 'loading="lazy"')+f' alt="Server Browser 2.1 · {e(label)} · Horizon"><span class="{"hero-enlarge" if large else "photo-expand"}" aria-hidden="true">{e(L[lang]["enlarge"]) if large else ""} ↗</span></a>'
def card(lang,i):
    title,label,desc=L[lang]['gallery'][i]
    return f'<article class="gallery-card" data-category="{G[i]["category"]}">{photo(lang,i)}<div class="gallery-caption"><span class="eyebrow">{i+1:02d} / {e(label)}</span><h2>{e(title)}</h2><p>{e(desc)}</p></div></article>'
def dialog(lang):
    t=L[lang]
    return f'''<dialog class="lightbox" aria-labelledby="lightbox-title"><div class="lightbox-top"><span>{e(t['lightbox_title'])}</span><button type="button" class="lightbox-close" aria-label="{e(t['lightbox_close'])}">{e(t['lightbox_close'])} <span aria-hidden="true">×</span></button></div><div class="lightbox-stage"><img alt=""></div><div class="lightbox-bottom"><div><h2 id="lightbox-title"></h2><p id="lightbox-caption"></p><a class="lightbox-original" href="#" target="_blank" rel="noopener">{e(t['lightbox_original'])} ↗</a></div><div class="lightbox-controls"><button type="button" data-prev aria-label="{e(t['lightbox_previous'])}">←</button><span data-position aria-live="polite"></span><button type="button" data-next aria-label="{e(t['lightbox_next'])}">→</button></div></div></dialog>'''
def servercard(lang,s):
    t=L[lang]['client']; name=e(s['name']); address=e(s['address'])
    return f'<article class="server-card"><span class="server-cube">{cube()}</span><div class="server-copy"><h3 title="{name}">{name}</h3><div class="server-ip">{address}</div><div class="server-tags">{e(" · ".join(s["tags"][:3]) or "Minecraft Java")}</div></div><button class="copy-ip" type="button" data-copy="{address}" aria-label="{e(t["copy_label"].replace("{name}",s["name"]))}" title="{e(t["copy_title"])}">{icon("copy")}</button></article>'
def catalog(lang,full=False):
    t=L[lang]; count=24 if full else 6
    filters=''.join(f'<button type="button" class="filter" data-tag="{key}" aria-pressed="{str(i==0).lower()}">{e(t["filter_labels"][i])}</button>' for i,key in enumerate(('all','survival','skyblock','pvp','minigames')))
    order=f'<label class="sr-only" for="sort">{e(t["sort_label"])}</label><select id="sort" data-sort><option value="curated">{e(t["sort_options"][0])}</option><option value="name">{e(t["sort_options"][1])}</option></select>' if full else e(t['preview'])
    more=f'<button type="button" class="button secondary load-more" data-more hidden>{e(t["client"]["load_more"].replace("{count}","2811"))}</button>' if full else ''
    return f'''<div class="catalog-shell" data-catalog="{'full' if full else 'preview'}" data-limit="{count}"><div class="browser-bar"><span class="window-dots"><i></i><i></i><i></i></span><span>{e(t['catalog_edition'])}</span></div><div class="catalog-controls"><label class="search-field" for="server-search">{icon('search')}<span class="sr-only">{e(t['search_label'])}</span><input type="search" id="server-search" placeholder="{e(t['search_placeholder'])}" maxlength="120" autocomplete="off"></label><div class="filter-row" role="group" aria-label="{e(t['search_label'])}">{filters}</div></div><div class="catalog-meta"><span data-count aria-live="polite">{e(t['catalog_edition'])}</span>{order}</div><div class="server-list">{''.join(servercard(lang,s) for s in S[:count])}</div><div class="catalog-bottom"><p data-catalog-note>{e(t['directory_note'])}</p>{link(t['in_game'] if full else t['all_servers'],u(lang,3 if full else 2))}</div></div>{more}'''
def faq(lang):
    t=L[lang]
    return f'<section class="section container faq-layout"><div><div class="eyebrow">{e(t["faq_eyebrow"])}</div><h2>{e(t["faq_title"])}</h2></div><div class="faq-list">'+''.join(f'<details><summary>{e(q)}</summary><p>{e(a)}</p></details>' for q,a in t['faq'])+'</div></section>'
def closing(lang):
    t=L[lang]
    return f'<div class="container"><section class="closing"><div class="eyebrow">{e(t["closing_eyebrow"])}</div><h2>{br(t["closing_h2"])}</h2><p>{e(t["closing_text"])}</p><div class="actions">{btn(t["horizon_download"],u(lang,3),"light","download")}{btn(t["explore"],u(lang,2),"dark-outline")}</div><div class="small-note">{e(t["closing_note"])}</div></section></div>'
def platforms(lang):
    t=L[lang]
    cards=''.join(f'<article class="platform-card"><span class="platform-symbol" aria-hidden="true">{symbol}</span><h3>{name}</h3><p>{e(t["platform_text"][i])}</p><span class="badge'+(' planned' if i else '')+f'">{e(t["planned"] if i else t["available"])}</span>{link(t["development"] if i else t["download_details"],u(lang,5)+"#development" if i else u(lang,3))}</article>' for i,(name,symbol) in enumerate((('Horizon','H'),('Fabric','F'),('Forge','◈'))))
    return f'<section class="section container">{head(t["platform_eyebrow"],t["platform_h2"],t["platform_intro"])}<div class="roadmap">{cards}</div></section>'
def home(lang):
    t=L[lang]; facts=''.join(f'<div><strong>{e(a)}</strong><span>{e(b)}</span></div>' for a,b in t['facts']); points=''.join(f'<li>{icon("check")}{e(p)}</li>' for p in t['story1_points'])
    return f'''<section class="product-hero container"><div class="hero-intro"><div><div class="release-label"><span class="dot"></span>SERVER BROWSER <span>2.1 / HORIZON</span></div><h1>{e(t['hero_h1'][0])}<br><span>{e(t['hero_h1'][1])}</span></h1></div><div class="hero-summary"><p>{e(t['hero_lead'])}</p><div class="actions">{btn(t['horizon_download'],u(lang,3),'primary','download')}{btn(t['view_mod'],'#in-game','secondary')}</div><span class="availability">{e(t['availability'])}</span></div></div><figure class="hero-product"><div class="product-chrome"><span><i></i> {e(t['recorded'])}</span><span>SERVER BROWSER / 2.1</span></div>{photo(lang,1,True)}<figcaption><span>{e(t['product_caption'])}</span>{link(t['all_images'],u(lang,1))}</figcaption></figure><div class="product-facts">{facts}</div></section>
<section class="section container" id="in-game">{head(t['game_eyebrow'],t['game_h2'],t['game_intro'])}<div class="feature-story"><div class="feature-story-photo gallery-card">{photo(lang,0)}</div><div class="feature-story-copy"><span class="story-number">{e(t['story1_label'])}</span><h3>{br(t['story1_h3'])}</h3><p>{e(t['story1_text'])}</p><ul class="story-points">{points}</ul></div></div><div class="feature-story reverse"><div class="feature-story-photo gallery-card">{photo(lang,5)}</div><div class="feature-story-copy"><span class="story-number">{e(t['story2_label'])}</span><h3>{br(t['story2_h3'])}</h3><p>{e(t['story2_text'])}</p>{link(t['functions_link'],u(lang,1))}</div></div></section>
<section class="section gallery-band"><div class="container">{head(t['gallery_eyebrow'],t['gallery_h2'],t['gallery_intro'])}<div class="gallery-grid gallery-preview">{''.join(card(lang,i) for i in (3,8,11))}</div><p class="gallery-all">{link(t['gallery_link'],u(lang,1))}</p></div></section>
<section class="section container">{head(t['directory_eyebrow'],t['directory_h1'],t['directory_home_intro'])}{catalog(lang)}</section>{platforms(lang)}{faq(lang)}{closing(lang)}{dialog(lang)}'''
def screenshots(lang):
    t=L[lang]; filters=''.join(f'<button type="button" class="filter" data-gallery-filter="{key}" aria-pressed="{str(i==0).lower()}">{e(t["gallery_filters"][i])}</button>' for i,key in enumerate(('all','entdecken','verwalten','anpassen')))
    return f'''<div class="container"><header class="page-hero gallery-hero"><div class="eyebrow">{e(t['gallery_eyebrow'])}</div><h1>{br(t['insights_h1'])}</h1><p class="lead">{e(t['insights_lead'])}</p><div class="capture-note"><span class="dot"></span>{e(t['capture_note'])}</div></header><section aria-label="{e(t['pages'][1])}"><div class="gallery-toolbar"><div class="filter-row" role="group" aria-label="{e(t['pages'][1])}">{filters}</div><span data-gallery-count role="status">{e(t['client']['gallery_count_many'].replace('{count}','13'))}</span></div><div class="gallery-grid">{''.join(card(lang,i) for i in range(13))}</div></section><aside class="capture-context"><div><h2>{br(t['capture_context_title'])}</h2></div><div>{''.join('<p>'+e(p)+'</p>' for p in t['capture_context'])}</div></aside></div>{dialog(lang)}<section class="section">{closing(lang)}</section>'''
def directory(lang):
    t=L[lang]; details=''.join(f'<section><h2>{e(a)}</h2><p>{e(b)}</p></section>' for a,b in t['directory_details'])
    return f'<div class="container">{hero(lang,2,"directory")}{catalog(lang,True)}<div class="directory-description">{details}</div><div class="callout">{e(t["directory_note"])} {link(t["sources_link"],u(lang,5)+"#sources")}</div></div><section class="section">{closing(lang)}</section>'
def download(lang):
    t=L[lang]; meta=''.join(f'<div><span>{e(a)}</span><strong>{e(b)}</strong></div>' for a,b in zip(t['release_labels'],(VERSION,'26.3',t['release_size'],'Windows x64')))
    upcoming=''.join(f'<article><span class="badge planned">{e(t["planned"])}</span><h3>{name}</h3><p>{e(t["platform_text"][i])}</p></article>' for i,name in ((1,'Fabric'),(2,'Forge')))
    return f'''<div class="container">{hero(lang,3,'download')}<div class="download-grid"><section class="release-card"><span class="badge"><span class="dot"></span>{e(t['available'])}</span><h2>{e(t['release_title'])}</h2><p>{e(t['release_text'])}</p><div class="release-meta">{meta}</div><div class="actions">{btn(t['horizon_download'],BASE+'/downloads/server-browser-2.1.0.hnmod','primary','download',True)}</div><p class="small-note">{e(t['release_build'])}</p><p class="release-notes"><a href="{BASE}/downloads/SHA256SUMS.txt">{e(t['checksum'])}</a> · <a href="{u(lang,4)}">{e(t['install_link'])}</a></p><p class="release-notes">{e(t['mail_limit'])}</p></section><div class="upcoming">{upcoming}</div></div><section class="install-banner"><div><h2>{e(t['install_banner_title'])}</h2><p>{e(t['install_banner_text'])}</p></div>{btn(t['install_link'],u(lang,4),'secondary')}</section></div>{faq(lang)}'''
def guide(lang):
    t=L[lang]; steps=''.join(f'<section class="step"><div><h2>{e(a)}</h2><p>{e(b)}</p>'+('<pre>horizon-native-mods</pre>' if i==1 else '')+'</div></section>' for i,(a,b) in enumerate(t['guide_steps'])); helptext=''.join(f'<h2>{e(a)}</h2><p>{e(b)}</p>' for a,b in t['guide_help'])
    return f'<div class="narrow">{hero(lang,4,"guide")}<div class="steps">{steps}</div><aside class="callout">{e(t["native_notice"])}</aside><section class="prose">{helptext}</section></div><section class="section">{closing(lang)}</section>'
def project(lang):
    t=L[lang]; ids=('project','development','sources','discovery','privacy','images')
    sections=''.join(f'<section id="{ids[i]}"><h2>{e(a)}</h2><p>{e(b)}</p></section>' for i,(a,b) in enumerate(t['project_sections']))
    refs=f'<p class="project-references"><a href="{REPO}">GitHub ↗</a> · <a href="{REPO}/blob/main/docs/VALIDATION.md">Validation ↗</a> · <a href="https://minecraftserve.rs/">minecraftserve.rs ↗</a> · <a href="https://modrinth.com/servers">Modrinth ↗</a> · <a href="https://serverify.louchat.site/">Serverify ↗</a> · <a href="https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement">GitHub privacy ↗</a> · <a href="{BASE}/downloads/LICENSE.txt">MIT</a></p>'
    support=f'<aside class="support-card"><span class="eyebrow">KO-FI</span><h2>{e(SUPPORT_TEXT[lang])}</h2><a class="button secondary" href="{SUPPORT}" rel="noopener">Ko-fi {icon()}</a></aside>'
    return f'<div class="narrow">{hero(lang,5,"project")}<article class="prose">{sections}{refs}</article>{support}</div><section class="section">{closing(lang)}</section>'
def render(lang,i,body,missing=False):
    t=L[lang]; title=t['titles'][i] if not missing else t['not_found']+' — Server Browser'; description=t['descriptions'][i] if not missing else t['not_found_text']; canonical=absolute(lang,i)
    alternates=''.join(f'<link rel="alternate" hreflang="{code}" href="{absolute(code,i)}">' for code in L)+f'<link rel="alternate" hreflang="x-default" href="{absolute("en",i)}">'
    schema=[{'@context':'https://schema.org','@type':'WebPage','name':title,'description':description,'url':canonical,'inLanguage':lang,'isPartOf':{'@type':'WebSite','name':'Server Browser','url':CANONICAL+'/'}}]
    if i==0 and not missing:
        schema.append({'@context':'https://schema.org','@type':'SoftwareApplication','name':'Server Browser','applicationCategory':'GameApplication','operatingSystem':'Windows x64; Horizon for Minecraft Java 26.3','softwareVersion':VERSION,'description':description,'license':'https://opensource.org/license/mit','downloadUrl':CANONICAL+'/downloads/server-browser-2.1.0.hnmod','offers':{'@type':'Offer','price':'0','priceCurrency':'EUR'}})
    data=json.dumps(schema,ensure_ascii=False).replace('</','<\\/'); client=json.dumps(t['client'],ensure_ascii=False).replace('</','<\\/')
    return f'''<!doctype html><html lang="{lang}" data-base-path="{BASE}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><meta name="theme-color" content="#f6f8f5"><title>{e(title)}</title><meta name="description" content="{e(description)}"><meta name="robots" content="{'noindex,follow' if missing else 'index,follow,max-image-preview:large'}"><link rel="canonical" href="{canonical}">{alternates}<link rel="icon" href="{asset('favicon.svg')}" type="image/svg+xml"><link rel="stylesheet" href="{asset('style.css')}"><meta property="og:type" content="website"><meta property="og:locale" content="{t['og']}"><meta property="og:site_name" content="Server Browser"><meta property="og:title" content="{e(title)}"><meta property="og:description" content="{e(description)}"><meta property="og:url" content="{canonical}"><meta property="og:image" content="{CANONICAL}/assets/gallery/entdecken.png?v={VERSION}"><meta property="og:image:alt" content="{e(t['gallery'][1][1])} · Horizon"><meta name="twitter:card" content="summary_large_image"><meta name="twitter:title" content="{e(title)}"><meta name="twitter:description" content="{e(description)}"><meta name="twitter:image" content="{CANONICAL}/assets/gallery/entdecken.png?v={VERSION}"><script type="application/ld+json">{data}</script><script type="application/json" id="site-i18n">{client}</script><script src="{asset('app.js')}" defer></script></head><body>{nav(lang,i)}<main id="main">{body}</main>{footer(lang,i)}</body></html>'''
def build():
    DIST.mkdir(exist_ok=True); (DIST/'downloads').mkdir(exist_ok=True)
    mod=ROOT.parent/'downloads/server-browser-2.1.0.hnmod'
    assert hashlib.sha256(mod.read_bytes()).hexdigest()==SHA
    shutil.copy2(mod,DIST/'downloads/server-browser-2.1.0.hnmod')
    (DIST/'downloads/SHA256SUMS.txt').write_text(f'{SHA}  server-browser-2.1.0.hnmod\n',encoding='utf8')
    shutil.copy2(ROOT.parent/'mod/LICENSE',DIST/'downloads/LICENSE.txt')
    (DIST/'assets/servers.json').write_text(json.dumps({'updated':DAY,'source':'minecraftserve.rs, Modrinth and curated public listings','servers':S},ensure_ascii=False,separators=(',',':')),encoding='utf8')
    # Only remove generated HTML files, leaving source assets intact.
    for old in DIST.rglob('*.html'): old.unlink()
    for lang in L:
        for i,fn in enumerate((home,screenshots,directory,download,guide,project)):
            dest=DIST/route(lang,i).lstrip('/')/'index.html'; dest.parent.mkdir(parents=True,exist_ok=True)
            dest.write_text(render(lang,i,fn(lang)),encoding='utf8')
    t=L['en']
    (DIST/'404.html').write_text(render('en',0,f'<section class="hero-404"><p class="eyebrow">404</p><h1>{e(t["not_found"])}</h1><p>{e(t["not_found_text"])}</p>{btn(t["back_home"],u("en"))}</section>',True),encoding='utf8')
    urls=[]
    for lang in L:
        for i in range(6):
            alternates=''.join(f'<xhtml:link rel="alternate" hreflang="{code}" href="{absolute(code,i)}"/>' for code in L)+f'<xhtml:link rel="alternate" hreflang="x-default" href="{absolute("en",i)}"/>'
            urls.append(f'<url><loc>{absolute(lang,i)}</loc><lastmod>{DAY}</lastmod>{alternates}</url>')
    (DIST/'sitemap.xml').write_text('<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">'+''.join(urls)+'</urlset>\n',encoding='utf8')
    (DIST/'robots.txt').write_text(f'User-agent: *\nAllow: /\n\nSitemap: {CANONICAL}/sitemap.xml\n',encoding='utf8')
    (DIST/'.nojekyll').write_text('',encoding='utf8')
    print(json.dumps({'pages':24,'languages':list(L),'server_addresses':2835,'download_sha256':SHA,'origin':HOST+BASE,'canonical_origin':CANONICAL}))
if __name__=='__main__': build()
