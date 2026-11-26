"""Check the optional, read-only Sims 4 MCCC evidence package."""
import json
from pathlib import Path
import sqlite3
import sys
from urllib.parse import parse_qs, urlsplit

ROOT = Path(__file__).resolve().parents[1]
CORE = 'deaderpool:project:mc-command-center'
COMPANION = 'deaderpool:project:mc-woohoo'
VERSIONS = {
    '2026.6.0': ('1.129.39.1030', '1.129.39.1230'),
    '2026.5.0': ('1.128.90.1030', '1.128.90.1230'),
    '2026.4.0': ('1.127.41.1030', '1.127.41.1230'),
    '6.6.0': ('1.56.52.1020', '1.56.52.1220'),
}


def require(ok, message):
    if not ok:
        raise ValueError(message)


def check(expected, observations):
    require(observations == sorted(expected['observations'], key=lambda row: row['id']), 'Exact persisted facts')
    require(len(observations) == 37 and len({row['id'] for row in observations}) == 37, 'Stable observations')
    releases = {}
    support = {}
    conflicts = {}
    required = {}
    for row in observations:
        fact = json.loads(row['value'])
        e, v = row['entity_id'], row['mod_version']
        if row['predicate'] == 'public_release':
            releases[e, v] = fact
        if row['predicate'] == 'compatibility':
            support[e, v, fact['platform']] = row['applicable_build']
            require(fact['game_build'] == row['applicable_build'], 'Exact supported patch')
            require(not fact['independently_tested'] and not fact['later_builds_verified'], 'No independent or later-build claim')
        if row['predicate'] == 'mod.relationships.v1':
            require(fact['none_evidence'] == {}, 'Unobserved does not mean none')
            if 'required' in fact['coverage']:
                require(e == COMPANION and len(fact['relationships']) == 1, 'Companion only requires core')
                required[v] = fact['relationships'][0]
            else:
                conflicts[e] = fact['relationships']
    require(len(releases) == 8 and len(support) == 16 and set(required) == set(VERSIONS), 'Exact pair and platform coverage')
    files = set()
    for v, (windows, macos) in VERSIONS.items():
        lane = 'legacy_edition' if v == '6.6.0' else 'standard'
        for entity in (CORE, COMPANION):
            row = releases[entity, v]
            require(row['version'] == v and row['lane'] == lane and row['release_date'] is None, 'Correct release track and no fabricated mod date')
            require((support[entity, v, 'windows'], support[entity, v, 'macos']) == (windows, macos), 'Distinct exact OS builds')
            artifact = row['artifact']
            parts = urlsplit(artifact['author_link'])
            q = parse_qs(parts.query)
            file_id = artifact['provider_file_id']
            require(parts.scheme == 'https' and parts.netloc == 'drive.google.com' and q == {'export': ['download'], 'id': [file_id]}, 'Exact Google Drive source link')
            require(parts.path == ('/u/1/uc' if lane == 'legacy_edition' else '/uc'), 'Keep source path, not a guessed mirror')
            require(file_id not in files and not artifact['download_verified'], 'Distinct publisher-linked file IDs; no admission')
            require(all(artifact[k] is None for k in ('filename', 'size_bytes', 'sha256')), 'Unknown file characteristics')
            require(artifact['acquisition'] == 'manual_author_link_only', 'No automated fetch contract')
            files.add(file_id)
        edge = required[v]
        require(edge['target_id'] == CORE and edge['relation'] == 'required' and edge['version_constraint'] == '=' + v, 'Exact companion dependency')
        require(edge['target_scope']['mod_version'] == v, 'No dependency promotion across release lanes')
    require(set(conflicts) == {CORE, COMPANION} and len(conflicts[CORE]) == 5 and len(conflicts[COMPANION]) == 1, 'Conflict scope')
    require({e['target_id'] for e in conflicts[CORE]} == {None, 'modthesims:project:562569', 'twistedmexi:project:always-testingcheats'}, 'No guessed IDs')
    require(sum(x['target_id'] is None for x in conflicts[CORE]) == 3, 'Three unresolved core targets')
    require(conflicts[COMPANION][0]['target_id'] == 'modthesims:project:591183', 'Companion-specific conflict')
    require(all(e['relation'] == 'incompatible' and e['version_constraint'] is None for edges in conflicts.values() for e in edges), 'No invented conflict ranges')
    print('Verified eight exact release references, Windows/macOS build pairs, four same-version dependencies and six scoped conflicts.')


def main():
    expected = json.loads((ROOT / 'data/sims4-mccc-reviewed.json').read_text(encoding='utf-8'))
    if sys.argv[1:] == ['--data']:
        check(expected, sorted(expected['observations'], key=lambda row: row['id']))
        return
    if len(sys.argv) != 2:
        raise SystemExit('Usage: verify_mccc_snapshot.py <new snapshot.sqlite>|--data')
    uri = Path(sys.argv[1]).resolve(strict=True).as_uri() + '?mode=ro&immutable=1'
    with sqlite3.connect(uri, uri=True) as db:
        db.row_factory = sqlite3.Row
        require(db.execute('PRAGMA quick_check').fetchone()[0] == 'ok', 'SQLite integrity')
        require(not list(db.execute('PRAGMA foreign_key_check')), 'SQLite foreign keys')
        entities = [dict(row) for row in db.execute('SELECT id,game_id,kind,name FROM entities ORDER BY id')]
        require(entities == sorted(expected['entities'], key=lambda row: row['id']), 'Exact persisted entities')
        check(expected, [dict(row) for row in db.execute('SELECT * FROM observations ORDER BY id')])


if __name__ == '__main__':
    main()
