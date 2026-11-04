"""Publish only checksum-verified mod assets through the repository's Actions token."""
from pathlib import Path
import hashlib
import json
import os
from urllib.error import HTTPError
from urllib.parse import quote
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[1]
REPO = os.environ['GITHUB_REPOSITORY']
BASE = 'https://api.github.com/repos/' + REPO
TOKEN = os.environ['GITHUB_TOKEN']


def request(url, method='GET', body=None, binary=None):
    headers = {'Authorization': 'Bearer ' + TOKEN, 'Accept': 'application/vnd.github+json',
               'X-GitHub-Api-Version': '2022-11-28', 'User-Agent': 'ServerAtlas-release'}
    data = None
    if body is not None:
        data = json.dumps(body).encode(); headers['Content-Type'] = 'application/json'
    if binary is not None:
        data = binary; headers['Content-Type'] = 'application/octet-stream'
    with urlopen(Request(url, data=data, headers=headers, method=method), timeout=120) as response:
        return json.load(response)


def publish(tag, title, body, files, beta):
    for path, expected in files:
        assert hashlib.sha256(path.read_bytes()).hexdigest() == expected, path.name
    try:
        release = request(BASE + '/releases/tags/' + quote(tag))
    except HTTPError as error:
        if error.code != 404: raise
        release = request(BASE + '/releases', 'POST', {'tag_name': tag,
            'target_commitish': os.environ['GITHUB_SHA'], 'name': title, 'body': body,
            'draft': False, 'prerelease': beta, 'make_latest': 'false' if beta else 'true'})
    existing = {asset['name']: asset for asset in request(BASE + f"/releases/{release['id']}/assets?per_page=100")}
    for path, expected in files:
        if path.name in existing:
            asset = existing[path.name]
            assert asset['size'] == path.stat().st_size
            if asset.get('digest'): assert asset['digest'] == 'sha256:' + expected
            else:
                with urlopen(asset['browser_download_url'], timeout=120) as response:
                    assert hashlib.sha256(response.read()).hexdigest() == expected
            continue
        request(release['upload_url'].split('{')[0] + '?name=' + quote(path.name), 'POST', binary=path.read_bytes())
    print(release['html_url'])


metadata = json.loads((ROOT / 'website/releases.json').read_text())
native = metadata['horizon']
publish('v2.1.0', 'Server Atlas — Horizon 2.1.0',
    'Native Rust edition, formerly Horizon Server Browser. Minecraft Java 26.3. '
    'Install the .hnmod using the Horizon launcher and build the matching instance. '
    'Source and exact tested revision pins are in mod/. Native code runs with your user permissions. '
    'Automatic email delivery is unconfigured.\n\n'
    'Website: https://musti3214.github.io/Horizon-Server-Browser/\n'
    'Validation: https://github.com/' + REPO + '/blob/main/docs/VALIDATION.md',
    [(ROOT / 'downloads' / native['name'], native['sha256'])], False)
java = metadata['java']
publish('v2.2.0-beta.1', 'Server Atlas — Fabric & Forge 2.2.0 beta',
    'Client-only Java editions for Minecraft 26.3 / Java 25, matching the Horizon browser interface. '
    'Choose one JAR for your loader: Fabric Loader 0.20.1+ or Minecraft Forge 66.1.5+. '
    'Required Fabric API modules are bundled. Put the JAR in your instance mods folder and open Multiplayer. '
    'No native Horizon build is needed.\n\n'
    'Public beta: 96 integration checks passed per loader. See java/VALIDATION.md for runtime coverage and limits. '
    'Feedback remains a local draft; email sending is unconfigured.\n\n'
    'Website: https://musti3214.github.io/Horizon-Server-Browser/download/\n'
    'License: MIT; sources and build instructions in java/.',
    [(ROOT / 'downloads' / row['name'], row['sha256']) for row in java['files']], True)
