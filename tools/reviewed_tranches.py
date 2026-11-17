"""Explicit checked-in tranche admission; never glob research or trust Pro output."""
import json
from pathlib import Path

# Reviewed data and its separate admission policy must be committed together.
TRANCHES = (
    ('bethesda-creators-reviewed.json', 'bethesda-creators-reviewed-source-policy.json'),
    ('minecraft-creators-reviewed.json', 'minecraft-creators-reviewed-source-policy.json'),
    ('sims4-creators-reviewed.json', 'sims4-reviewed-source-policy.json'),
    ('cyberpunk-hytale-creators-reviewed.json', 'cyberpunk-hytale-creators-reviewed-source-policy.json'),
)

def add_tranche(bundle, tranche, policy):
    if tranche.get('schema') != 'athena-reviewed-observations-v1':
        raise ValueError('Unsupported reviewed tranche schema')
    if not all(policy.get(k) is True for k in ('enabled', 'allow_retain', 'allow_publish')):
        raise ValueError('Reviewed tranche policy does not permit publication')
    rev = policy.get('revision')
    if not isinstance(rev, str) or not rev:
        raise ValueError('Missing policy revision')
    if 'source_ids' in policy and set(policy['source_ids']) != set(tranche['sources']):
        raise ValueError('Policy source scope mismatch')
    revisions = tranche.get('source_policy_revisions', {})
    for source in tranche['sources']:
        if revisions.get(source) != rev:
            raise ValueError('Source policy revision mismatch')
    # Validate into a copy so a rejected tranche cannot partially mutate the bundle.
    merged = json.loads(json.dumps(bundle))
    tranche = json.loads(json.dumps(tranche))
    for section in ('games', 'sources', 'source_policy_revisions'):
        for key, value in tranche.get(section, {}).items():
            if key in merged[section] and merged[section][key] != value:
                raise ValueError('Conflicting '+section+' '+key)
            merged[section][key] = value
    for section in ('entities', 'observations'):
        known = {x['id']: x for x in merged[section]}
        seen = set()
        for row in tranche[section]:
            key = row['id']
            if key in seen:
                raise ValueError('Duplicate tranche identity '+key)
            seen.add(key)
            if key in known and known[key] != row:
                raise ValueError('Conflicting '+section+' '+key)
            if key not in known:
                merged[section].append(row)
                known[key] = row
    entities = {x['id'] for x in merged['entities']}
    for e in tranche['entities']:
        if e['game_id'] not in merged['games']:
            raise ValueError('Unknown game')
    for o in tranche['observations']:
        if o['entity_id'] not in entities or o['source_id'] not in tranche['sources']:
            raise ValueError('Unknown entity/source')
        if o['policy_revision'] != rev:
            raise ValueError('Observation policy revision mismatch')
        if not o['source_url'].startswith('https://') or o['completeness'] != 'partial':
            raise ValueError('Evidence URL or partial coverage missing')
        json.loads(o['value'])
    bundle.clear()
    bundle.update(merged)

def load_reviewed_tranches(bundle, root: Path):
    for filename, policyname in TRANCHES:
        tranche = json.loads((root/'data'/filename).read_text(encoding='utf-8'))
        policy = json.loads((root/'config'/policyname).read_text(encoding='utf-8'))
        add_tranche(bundle, tranche, policy)
