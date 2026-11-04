"""Apply the reviewed additive catalog, translations and release version."""
import json
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
extra=ROOT.parent/'horizon-server-browser/.local/native-browser/modrinth-20261103'
report=json.loads((extra/'snapshot.json').read_text(encoding='utf8'))
assert report['complete'] and report['baseline']==488 and report['new_unique']==2347
catalog=ROOT/'src/catalog.rs'
text=catalog.read_text(encoding='utf8')
assert 'MODRINTH_SNAPSHOT' not in text
catalog.write_text(text+'\n'+(extra/'modrinth_snapshot.rs').read_text(encoding='utf8'),encoding='utf8')
builder=ROOT/'build_bundle.py'
text=builder.read_text(encoding='utf8').replace('2.0.0','2.1.0')
text=text.replace('catalog::CURATED_SERVERS.iter().map(', 'catalog::CURATED_SERVERS.iter().chain(catalog::MODRINTH_SNAPSHOT).map(')
builder.write_text(text,encoding='utf8')
for file in ['native.rs','src/ui/feedback.rs','verify_native.py']:
    path=ROOT/file
    text=path.read_text(encoding='utf8').replace('2.0.0','2.1.0')
    if file=='native.rs': text=text.replace('.chain(super::super::catalog::DIRECTORY_SNAPSHOT)', '.chain(super::super::catalog::DIRECTORY_SNAPSHOT).chain(super::super::catalog::MODRINTH_SNAPSHOT)')
    path.write_text(text,encoding='utf8')
for locale in ['de_de','en_us']:
    path=ROOT/f'src/lang/{locale}.json'
    data=json.loads(path.read_text(encoding='utf8'))
    data['horizon.browser.modern_style']='Minecraft'
    data['horizon.browser.directory_source']='Online · minecraftserve.rs + Modrinth'
    data['horizon.browser.directory_partial']='Eine Serverquelle ist gerade nicht verfügbar. Die übrigen Einträge bleiben nutzbar.' if locale=='de_de' else 'One server source is currently unavailable. Other entries remain usable.'
    data['horizon.browser.privacy_note']=(
        'Entdecken verwendet minecraftserve.rs und die öffentliche Modrinth-API. Die Dienste sehen deine IP-Adresse, die angefragte Seite und öffentliche Tagfilter; minecraftserve.rs erhält auch öffentliche Versionsfilter. Gespeicherte Server, Notizen, Spielernamen und Suchtext bleiben lokal. Online-Entdecken lässt sich in den Einstellungen ausschalten. Jede Quelle wird 5 Minuten gecacht; höchstens eine Anfrage je Quelle und Minute. Geladene Server werden direkt für Status-Pings kontaktiert. Modrinth-Bilder und Modpacks werden nicht nachgeladen. Einträge werden erst durch Hinzufügen gespeichert.' if locale=='de_de' else
        'Discover uses minecraftserve.rs and the public Modrinth API. The services see your IP address, requested page and public tag filters; minecraftserve.rs also receives public version filters. Saved servers, notes, player names and search text stay local. You can disable online discovery in Settings. Each source is cached for 5 minutes, with at most one request per source per minute. Loaded servers are contacted directly for status checks. Modrinth images and modpacks are not downloaded. Entries are saved only when you choose Add server.')
    path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Prepared Server Browser 2.1.0: 2,835 unique bundled addresses; Minecraft menu style; two live sources.')
