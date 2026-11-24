"""Admit two independently pinned creator/relation packs. No external reads."""
import hashlib
import json
from urllib.parse import urlsplit

PINS = (
    'skyrim-se-distributors-reviewed.json',
    'sims4-utility-relationships-reviewed.json',
)
SCHEMA = 'athena-reviewed-observations-v1'


def admit_distributor_packs(bundle, root):
    policy = json.loads((root/'config/reviewed-distributor-packs.json').read_text(encoding='utf-8'))
    if set(policy['packs']) != set(PINS):
        raise ValueError('Reviewed distributor pack inventory changed')
    games = dict(bundle['games'])
    sources = dict(bundle['sources'])
    revisions = dict(bundle['source_policy_revisions'])
    entities = {row['id'].casefold():row for row in bundle['entities']}
    observation_ids = {row['id'].casefold() for row in bundle['observations']}
    new_entities, new_observations = [], []
    for name in PINS:
        grant = policy['packs'][name]
        raw = (root/'data'/name).read_bytes().replace(b'\r\n', b'\n')
        if hashlib.sha256(raw).hexdigest() != grant['sha256_lf']:
            raise ValueError('Reviewed distributor bytes changed: '+name)
        pack = json.loads(raw)
        if pack['schema'] != SCHEMA or bundle['schema'] != SCHEMA:
            raise ValueError('Wrong distributor schema')
        for gid, label in pack['games'].items():
            if gid in games and games[gid] != label:
                raise ValueError('Conflicting game identity: '+gid)
            games[gid] = label
        allowed = grant['sources']
        if set(allowed) != set(pack['sources']) or set(allowed) != set(pack['source_policy_revisions']):
            raise ValueError('Unreviewed distributor source inventory')
        for sid, approval in allowed.items():
            if (sid in sources or sid in revisions
                    or approval['allow_retain'] is not True or approval['allow_publish'] is not True
                    or pack['sources'][sid] != approval['policy_url']
                    or pack['source_policy_revisions'][sid] != approval['revision']):
                raise ValueError('Conflicting or withdrawn distributor source policy: '+sid)
            sources[sid] = approval['policy_url']
            revisions[sid] = approval['revision']
        for row in pack['entities']:
            key = row['id'].casefold()
            if row['game_id'] not in pack['games']:
                raise ValueError('Entity escaped reviewed game namespace')
            if key in entities:
                # Preserve an existing exact normalized entity (e.g. powerof3).
                # Distinct provider IDs and near-matches are never merged.
                fields = ('id', 'game_id', 'kind', 'name')
                if any(row[k] != entities[key][k] for k in fields):
                    raise ValueError('Conflicting distributor entity: '+row['id'])
                continue
            entities[key] = row
            new_entities.append(row)
        for row in pack['observations']:
            key = row['id'].casefold()
            url = urlsplit(row['source_url'])
            sid = row['source_id']
            entity_id = row['entity_id']
            if (key in observation_ids or entity_id.casefold() not in entities
                    or entities[entity_id.casefold()]['id'] != entity_id
                    or entities[entity_id.casefold()]['game_id'] not in pack['games']
                    or sid not in allowed or row['policy_revision'] != revisions[sid]
                    or url.scheme != 'https' or url.netloc != allowed[sid]['host']
                    or url.query or url.fragment):
                raise ValueError('Invalid distributor observation: '+row['id'])
            observation_ids.add(key)
            new_observations.append(row)
    bundle['games'], bundle['sources'], bundle['source_policy_revisions'] = games, sources, revisions
    bundle['entities'].extend(new_entities)
    bundle['observations'].extend(new_observations)
