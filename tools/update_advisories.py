"""Read-only pre-update advisories. Never a permission or installation decision."""
from copy import deepcopy
from urllib.parse import urlsplit


def validate(r):
    fields = {'schema_version', 'id', 'project_id', 'affected_versions', 'game_build',
              'runtime_context', 'symptom', 'evidence_level', 'source_url', 'lineage',
              'reported_unix', 'observed_unix', 'review_after_unix', 'lifecycle'}
    if set(r) != fields or type(r['schema_version']) is not int or r['schema_version'] != 1:
        raise ValueError('unsupported advisory schema')
    for key in ('id', 'project_id', 'lineage'):
        if not isinstance(r[key], str) or not r[key].strip():
            raise ValueError('stable identifiers required')
    for key in ('game_build', 'runtime_context'):
        if r[key] is not None and (not isinstance(r[key], str) or not r[key].strip()):
            raise ValueError('context must be exact or null (unknown)')
    versions = r['affected_versions']
    if not isinstance(versions, list) or any(not isinstance(v, str) or not v.strip() for v in versions) or len(set(versions)) != len(versions):
        raise ValueError('exact versions required; empty means unknown')
    if r['symptom'] not in ('crash', 'freeze', 'bug'):
        raise ValueError('invalid symptom')
    if r['evidence_level'] not in ('community_report', 'maintainer_report', 'reproduced', 'maintainer_confirmed_cause'):
        raise ValueError('invalid evidence level')
    if r['lifecycle'] not in ('active', 'retracted', 'superseded'):
        raise ValueError('invalid lifecycle')
    if not isinstance(r['source_url'], str):
        raise ValueError('public source URL required')
    u = urlsplit(r['source_url'])
    if u.scheme != 'https' or not u.hostname or u.username or u.password or u.query or u.fragment:
        raise ValueError('canonical public HTTPS source required')
    for key in ('reported_unix', 'observed_unix', 'review_after_unix'):
        if type(r[key]) is not int or r[key] < 0:
            raise ValueError('invalid timestamp')
    if not r['reported_unix'] <= r['observed_unix'] < r['review_after_unix']:
        raise ValueError('invalid chronology')
    return r


def pre_update_advisories(records, *, project_id, target_version, game_build,
                          runtime_context, now_unix, max_age_seconds):
    """Adapters establish evidence, release aliases and public-source policy first.

    No semantic-version ranges, cross-provider aliases, voting or causality inference.
    All matching observations remain visible, including stale/retracted history.
    """
    for value in (project_id, target_version, game_build, runtime_context):
        if not isinstance(value, str) or not value.strip():
            raise ValueError('exact candidate identity and context required')
    if type(now_unix) is not int or now_unix < 0 or type(max_age_seconds) is not int or max_age_seconds < 0:
        raise ValueError('invalid query time or freshness window')
    notices = []
    ids = set()
    for r in records:
        validate(r)
        if r['id'] in ids:
            raise ValueError('duplicate observation ID; resolve before projection')
        ids.add(r['id'])
        if r['observed_unix'] > now_unix:
            raise ValueError('future observation')
        if r['project_id'] != project_id:
            continue
        if r['affected_versions'] and target_version not in r['affected_versions']:
            continue
        if any(r[k] is not None and r[k] != v for k, v in [('game_build', game_build), ('runtime_context', runtime_context)]):
            continue
        scope = 'exact' if r['affected_versions'] and r['game_build'] is not None and r['runtime_context'] is not None else 'uncertain'
        fresh = now_unix < r['review_after_unix'] and now_unix - r['reported_unix'] <= max_age_seconds
        notices.append({'observation': deepcopy(r), 'scope': scope, 'fresh': fresh,
                        'display': 'current_report' if fresh and r['lifecycle'] == 'active' else 'historical_or_stale'})
    notices.sort(key=lambda n: n['observation']['id'])
    return {'update_allowed': True, 'blocks_update': False, 'automatic_action': None,
            'coverage': 'partial_evidence' if notices else 'unknown', 'notices': notices,
            'distinct_report_lineages': len({n['observation']['lineage'] for n in notices})}
