"""Check the source-only native bundle and the untouched host build contracts."""
import hashlib
import json
from pathlib import Path
import zipfile

ROOT=Path(__file__).resolve().parents[1]
LAB=ROOT.parent/'horizon-server-browser/.local/native-browser'
package=ROOT/'server-browser-2.1.0.hnmod'
with zipfile.ZipFile(package) as archive:
    names=archive.namelist()
    assert len(names)==len(set(names))
    assert all(not n.startswith('/') and '..' not in Path(n).parts and Path(n).suffix in ('.rs','.toml','.json') for n in names)
    manifest=json.loads(archive.read('horizon-native.json'))
    assert manifest['id']=='server_browser' and manifest['version']=='2.1.0'
    assert manifest['horizon']=='142388e1c136e622652ec7505af15d8ad195e041'
    for name in names:
        assert archive.read(name)==(ROOT/name).read_bytes(),name
    runtime=archive.read('mixins/server_browser/runtime.rs')
    assert b'ModrinthDirectory' in runtime and b'MODRINTH_SNAPSHOT' in runtime
    assert b'm.kabo' not in runtime and b'outlook.de' not in runtime
    assert b'const FORM_KEY: &str = "";' in runtime

pristine,stage=LAB/'pristine',LAB/'stage'
contracts=sorted(set(pristine.rglob('Cargo.toml'))|set(pristine.rglob('Cargo.lock'))|set(pristine.rglob('*.wit')))
for source in contracts:
    relative=source.relative_to(pristine)
    assert source.read_bytes()==(stage/relative).read_bytes(),str(relative)
report={'package':package.name,'sha256':hashlib.sha256(package.read_bytes()).hexdigest(),'bytes':package.stat().st_size,'source_files':len(names),'host_build_and_wit_contracts_unchanged':len(contracts),'result':'PASS'}
(LAB/'minecraft-20261103/release-audit.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
(ROOT/'server-browser-2.1.0.sha256').write_text(report['sha256']+'  '+package.name+'\n',encoding='utf8')
print(json.dumps(report))
