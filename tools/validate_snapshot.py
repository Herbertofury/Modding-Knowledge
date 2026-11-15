"""Read-only independent verification of the built public snapshot."""
import hashlib
import json
import sqlite3
import sys
from contextlib import closing
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_REVISION = 'reviewed-public-facts-2026-11-14'
OBSERVATION_KEYS = ['id','entity_id','source_id','source_url','predicate','value',
                    'observed_unix','applicable_build','mod_version','completeness',
                    'policy_revision']


def require(condition, message):
    # Unlike assert, validation must remain active under python -O.
    if not condition:
        raise ValueError(message)


def validate_contents(conn, bundle):
    require(conn.execute('pragma integrity_check').fetchall() == [('ok',)], 'Integrity check failed')
    require(conn.execute('pragma foreign_key_check').fetchall() == [], 'Foreign key check failed')
    require(conn.execute('pragma user_version').fetchone() == (1,), 'Unsupported schema version')
    require(conn.execute('select count(*) from refresh_attempts').fetchone() == (0,),
            'Unreviewed refresh attempts in public snapshot')
    games = dict(conn.execute('select id,name from games'))
    require(games == bundle['games'], 'Game manifest mismatch')
    entity_keys = ['id', 'game_id', 'kind', 'name']
    entities = [dict(zip(entity_keys, row)) for row in conn.execute(
        'select id,game_id,kind,name from entities order by id')]
    # Manifest entities may carry provenance metadata not persisted by the native seeder.
    expected_entities = [{key: row[key] for key in entity_keys} for row in bundle['entities']]
    require(entities == sorted(expected_entities, key=lambda r: r['id']), 'Entity manifest mismatch')
    revisions = bundle.get('source_policy_revisions', {})
    sources = conn.execute(
        'select id,policy_url,can_retain,can_publish,policy_revision from sources order by id'
    ).fetchall()
    expected_sources = sorted((ident, url, 1, 1, revisions.get(ident, DEFAULT_REVISION))
                              for ident, url in bundle['sources'].items())
    require(sources == expected_sources, 'Source policy manifest mismatch')
    rows = [dict(zip(OBSERVATION_KEYS, row)) for row in conn.execute(
        'select ' + ','.join(OBSERVATION_KEYS) + ' from observations order by id')]
    require(rows == sorted(bundle['observations'], key=lambda r: r['id']), 'Observation manifest mismatch')
    for row in rows:
        if row['predicate'] == 'conditional_plugin_metadata':
            value = json.loads(row['value'])
            require(value.get('condition_evaluation') == 'not_evaluated'
                    and 'plugin_metadata' in value, 'Invalid unevaluated LOOT metadata')
    require(any(row['predicate'] == 'unevaluated_source_context' for row in rows),
            'Missing unevaluated source context')
    return {'entities': len(entities), 'observations': len(rows),
            'source_scopes': len(sources), 'game_namespaces': len(games)}


def validate_snapshot(path, bundle_path):
    path = Path(path).resolve()
    bundle_path = Path(bundle_path)
    bundle_bytes = bundle_path.read_bytes()
    bundle = json.loads(bundle_bytes)
    with closing(sqlite3.connect(path.as_uri() + '?mode=ro', uri=True)) as conn:
        counts = validate_contents(conn, bundle)
    return {'sqlite_version': sqlite3.sqlite_version,
            'snapshot_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
            'source_manifest_sha256': hashlib.sha256(bundle_bytes).hexdigest(),
            **counts, 'exact_manifest_equality': True, 'integrity': 'ok',
            'foreign_key_violations': 0,
            'scope': 'Snapshot contents matched reviewed manifest; no native game/provider condition evaluation'}


if __name__ == '__main__':
    print(json.dumps(validate_snapshot(sys.argv[1], ROOT/'data/reviewed-bundle.json'), indent=2))
