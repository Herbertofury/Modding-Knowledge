"""Read-only genre-specific evidence queries; never treats stale research as fresh."""
import argparse
from datetime import date
import json
from pathlib import Path
from urllib.parse import urlsplit

DEFAULT = Path(__file__).resolve().parents[1] / 'data/game-progression-evidence.json'
GENRES = ('survivors_like', 'rts', 'action_rpg')
STATUSES = ('released', 'early_access', 'announced', 'historical_reference', 'unknown')


def validate(data):
    if type(data.get('schema_version')) is not int or data['schema_version'] != 1:
        raise ValueError('unsupported evidence schema')
    seen = set()
    for row in data['records']:
        if not isinstance(row['id'], str) or not row['id'] or row['id'] in seen:
            raise ValueError('missing or duplicate identity')
        seen.add(row['id'])
        if row['genre'] not in GENRES or row['release_status'] not in STATUSES:
            raise ValueError('unknown genre or release status')
        for key in ('name', 'evidence_scope', 'design_interpretation', 'reuse'):
            if not isinstance(row[key], str) or not row[key].strip():
                raise ValueError('missing ' + key)
        url = urlsplit(row['source_url'])
        if url.scheme != 'https' or not url.hostname or url.username or url.password:
            raise ValueError('invalid source URL')
        checked = date.fromisoformat(row['checked_on'])
        if date.fromisoformat(row['review_due']) <= checked:
            raise ValueError('review date must follow check')
        for key in ('observations', 'acceptance'):
            if not isinstance(row[key], list) or not row[key] or not all(isinstance(v,str) and v.strip() for v in row[key]):
                raise ValueError('missing ' + key)
        if row['playtest'] != 'not_performed':
            raise ValueError('playtest claims need a dedicated evidence contract')
    return data['records']


def query(data, genre=None, status=None, as_of=None, due_only=False, search=''):
    if genre is not None and genre not in GENRES:
        raise ValueError('unknown genre filter')
    if status is not None and status not in STATUSES:
        raise ValueError('unknown status filter')
    today = date.fromisoformat(as_of) if as_of else date.today()
    results = []
    for row in validate(data):
        if genre and row['genre'] != genre or status and row['release_status'] != status:
            continue
        if search.casefold() not in json.dumps(row,ensure_ascii=False).casefold():
            continue
        checked, due = date.fromisoformat(row['checked_on']), date.fromisoformat(row['review_due'])
        state = 'future_observation' if checked > today else 'due' if due <= today else 'within_review_window'
        if due_only and state != 'due':
            continue
        # Never silently discard stale records or reinterpret historical facts as current.
        results.append(dict(row, freshness=state))
    return sorted(results,key=lambda r:r['name'].casefold())


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--genre', choices=GENRES)
    parser.add_argument('--status', choices=STATUSES)
    parser.add_argument('--as-of')
    parser.add_argument('--due-only',action='store_true')
    parser.add_argument('--search',default='')
    args=parser.parse_args()
    print(json.dumps(query(json.loads(DEFAULT.read_text()),args.genre,args.status,args.as_of,args.due_only,args.search),indent=2))
