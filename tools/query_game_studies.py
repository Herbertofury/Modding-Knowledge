"""Read-only shared design-study query; never downloads or infers reuse permission."""
import argparse
import json
from pathlib import Path

DEFAULT = Path(__file__).resolve().parents[1] / 'data/game-design-studies.json'


def validate(data):
    if type(data.get('schema_version')) is not int or data['schema_version'] != 1:
        raise ValueError('unsupported study schema')
    records = data['records']
    ids = [row['id'] for row in records]
    if len(ids) != len(set(ids)):
        raise ValueError('duplicate study identity')
    if records != sorted(records, key=lambda row: row['name'].casefold()):
        raise ValueError('studies must be name sorted')
    for row in records:
        if row['kind'] not in ('inspiration', 'foundation_candidate'):
            raise ValueError('unknown study kind')
        if row['evaluation'] != 'not_tested':
            raise ValueError('measured claims require a new evidence contract')
        for field in ('name', 'source_url', 'review_basis', 'study_question', 'reuse', 'windows'):
            if not isinstance(row.get(field), str) or not row[field].strip():
                raise ValueError('missing field: ' + field)
        if not row['source_url'].startswith('https://'):
            raise ValueError('HTTPS source required')
        if row['kind'] == 'inspiration' and row['catalog_ref'] != row['id']:
            raise ValueError('inspiration must reference its canonical identity')
    return records


def query(data, kind=None, catalog=None):
    records = validate(data)
    index = None
    if catalog is not None:
        if catalog['catalogId'] != data['catalog']['catalog_id']:
            raise ValueError('wrong source catalog')
        index = {row['id']: row for row in catalog['items']}
        if len(index) != len(catalog['items']):
            raise ValueError('duplicate source catalog identity')
    result = []
    for row in records:
        if kind and row['kind'] != kind:
            continue
        out = dict(row)
        ref = row['catalog_ref']
        out['resolution'] = 'not_requested' if index is None else ('not_applicable' if ref is None else ('resolved' if ref in index else 'missing'))
        # Missing references remain explicit, never silently match by display name.
        result.append(out)
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--kind', choices=['inspiration', 'foundation_candidate'])
    parser.add_argument('--catalog', type=Path, help='Local Games data/catalog.json; no network request')
    args = parser.parse_args()
    catalog = json.loads(args.catalog.read_text(encoding='utf-8')) if args.catalog else None
    print(json.dumps(query(json.loads(DEFAULT.read_text(encoding='utf-8')), args.kind, catalog), indent=2))
