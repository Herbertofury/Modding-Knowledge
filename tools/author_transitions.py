"""Shared evidence validation and read-only tracking decisions, never account enforcement."""
from urllib.parse import urlsplit


def _url(value):
    if not isinstance(value, str):
        raise ValueError('URL required')
    u = urlsplit(value)
    if u.scheme != 'https' or not u.hostname or u.username or u.password or u.query or u.fragment:
        raise ValueError('public canonical HTTPS URL required')


def validate(record):
    allowed = {'schema_version', 'id', 'creator_id', 'source_account', 'status',
               'status_evidence', 'destination_account', 'identity_evidence',
               'project_mappings', 'observed_unix', 'review_after_unix', 'lifecycle'}
    if set(record) != allowed or type(record['schema_version']) is not int or record['schema_version'] != 1:
        raise ValueError('unsupported transition fields/schema')
    for key in ('id', 'creator_id'):
        if not isinstance(record[key], str) or not record[key].strip():
            raise ValueError('stable identity required')
    for key in ('observed_unix', 'review_after_unix'):
        if type(record[key]) is not int or record[key] < 0:
            raise ValueError('invalid timestamp')
    if record['review_after_unix'] <= record['observed_unix']:
        raise ValueError('review deadline must follow observation')
    if record['lifecycle'] not in ('active', 'retracted', 'superseded'):
        raise ValueError('invalid lifecycle')
    if record['status'] not in ('unknown', 'unavailable', 'removed', 'suspended', 'banned', 'active'):
        raise ValueError('invalid status')
    _url(record['source_account'])
    if record['destination_account'] is not None:
        _url(record['destination_account'])
    for field, kinds in [('status_evidence', ('provider_notice', 'creator_statement', 'retrieval_result')),
                         ('identity_evidence', ('creator_crosslink', 'provider_verified_link'))]:
        if not isinstance(record[field], list):
            raise ValueError('evidence list required')
        for evidence in record[field]:
            if set(evidence) != {'url', 'kind', 'observed_unix'} or evidence['kind'] not in kinds:
                raise ValueError('invalid evidence')
            _url(evidence['url'])
            if type(evidence['observed_unix']) is not int or not 0 <= evidence['observed_unix'] <= record['observed_unix']:
                raise ValueError('invalid evidence time')
    if record['status'] in ('banned', 'suspended', 'removed') and not any(
            e['kind'] in ('provider_notice', 'creator_statement') for e in record['status_evidence']):
        raise ValueError('missing page/HTTP error cannot establish enforcement')
    if record['identity_evidence'] and record['destination_account'] is None:
        raise ValueError('identity evidence needs destination')
    seen = set()
    if not isinstance(record['project_mappings'], list):
        raise ValueError('mapping list required')
    for m in record['project_mappings']:
        if set(m) != {'source_project', 'destination_project', 'game', 'evidence_url', 'verified'} or type(m['verified']) is not bool:
            raise ValueError('invalid project mapping')
        for key in ('source_project', 'destination_project', 'evidence_url'):
            _url(m[key])
        if not isinstance(m['game'], str) or not m['game'].strip() or m['source_project'] in seen:
            raise ValueError('ambiguous project mapping')
        seen.add(m['source_project'])
    return record


def decision(record, now_unix, tracking_consent=False, adapter_available=False):
    """An adapter validates evidence authorship/content; this function validates its contract.

    Flags are explicit consumer inputs, not authorization conferred by shared public data.
    """
    r = validate(record)
    if type(now_unix) is not int or now_unix < r['observed_unix']:
        raise ValueError('invalid evaluation time')
    if type(tracking_consent) is not bool or type(adapter_available) is not bool:
        raise ValueError('explicit boolean consumer gates required')
    fresh = now_unix < r['review_after_unix'] and r['lifecycle'] == 'active'
    identity = bool(r['identity_evidence']) and r['destination_account'] is not None
    mappings = [m for m in r['project_mappings'] if m['verified']]
    ready = fresh and identity and bool(mappings) and tracking_consent and adapter_available
    return {'record_id': r['id'], 'status': r['status'], 'fresh': fresh,
            'status_authorities': sorted({e['kind'] for e in r['status_evidence']}),
            'identity_link_supported': identity,
            'action': 'track_additional_source' if ready else 'review_required',
            'projects_to_track': mappings if ready else [],
            'preserve_original_source': True,
            'coverage': 'explicit mappings only; never entire author catalog',
            'automatic_downloads': False}
