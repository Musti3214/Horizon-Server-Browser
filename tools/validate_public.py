"""Verify the public export, exact native source bundle and documentation assets."""
from pathlib import Path
import hashlib
import json
import re
import zipfile

ROOT=Path(__file__).resolve().parents[1]
EXPECTED='bb6bb8e5e89299a0d853ce85db520205564da8b77576b61fbf6054eb200bfd58'
package=ROOT/'downloads/server-browser-2.1.0.hnmod'
assert hashlib.sha256(package.read_bytes()).hexdigest()==EXPECTED
assert (ROOT/'downloads/server-browser-2.1.0.sha256').read_text().split()[0]==EXPECTED
config=json.loads((ROOT/'mod/feedback-public.json').read_text(encoding='utf8'))
assert config=={'provider':'staticforms','form_public_key':''},'Public export must not contain a configured mail key'
with zipfile.ZipFile(package) as archive:
    names=archive.namelist()
    assert len(names)==len(set(names))==29
    for name in names:
        path=Path(name)
        assert not path.is_absolute() and '..' not in path.parts
        assert path.suffix in ('.rs','.json','.toml')
        assert archive.read(name)==(ROOT/'mod'/path).read_bytes(),name
    manifest=json.loads(archive.read('horizon-native.json'))
    assert manifest['id']=='server_browser' and manifest['version']=='2.1.0'
    assert manifest['horizon']=='142388e1c136e622652ec7505af15d8ad195e041'
    assert manifest['mixins']=='a2af5c0dd4117d51ecd70066ba7a00b4a2c10dab'
    assert manifest['game']=='26.3'
    assert b'const FORM_KEY: &str = "";' in archive.read('mixins/server_browser/runtime.rs')

count=0
skip={'.git','__pycache__','preview','qa','node_modules','target','.local'}
for file in ROOT.rglob('*'):
    relative=file.relative_to(ROOT)
    if any(part in skip for part in relative.parts):continue
    assert not file.is_symlink(),relative
    if not file.is_file():continue
    count+=1
    assert not any(part in ('.aws','.codex','.sites-runtime') for part in relative.parts),relative
    assert file.suffix.lower() not in ('.exe','.dll','.jar','.mrpack','.pem','.key','.pfx','.env'),relative
    assert file.name not in ('.env','credentials.json','accounts.json','servers.dat'),relative
    if file.suffix in ('.md','.py','.json','.toml','.rs','.js','.html','.yml','.yaml','.txt'):
        data=file.read_text(encoding='utf8')
        assert not re.search(r'gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,}|-----BEGIN [A-Z ]*PRIVATE KEY-----',data),relative
        assert not re.search(r'[A-Za-z]:[\\/]Users[\\/]',data),relative

for readme in ('README.md','README.de.md'):
    text=(ROOT/readme).read_text(encoding='utf8')
    for target in re.findall(r'!\[[^\]]*\]\(([^)]+)\)',text):
        assert (ROOT/target).is_file(),target
servers=json.loads((ROOT/'website/data/servers.json').read_text(encoding='utf8'))['servers']
assert len(servers)==len({s['address'].lower() for s in servers})==2835
print(json.dumps({'result':'PASS','public_files':count,'native_source_files':len(names),'server_addresses':len(servers),'package_sha256':EXPECTED}))
