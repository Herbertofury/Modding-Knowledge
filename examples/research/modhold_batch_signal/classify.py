"""Compare batched public ModHold version signals without declaring a full update."""
import json
import sys

MAX_BYTES = 2 * 1024 * 1024
MAX_PROJECTS = 100
MAX_ID = 2**53 - 1

def exact_id(value):
    return type(value) is int and 1 <= value <= MAX_ID

def text(value):
    return (type(value) is str and 0 < len(value) <= 128 and value == value.strip()
            and all(ord(c) >= 32 and ord(c) != 127 for c in value))

def projection(value):
    if type(value) is not dict:
        raise ValueError('invalid project shape')
    v, stamp, files = (value.get(key) for key in ('version', 'updated', 'files'))
    if not text(v) or not text(stamp) or type(files) is not list or len(files) > 10000:
        raise ValueError('invalid signal fields')
    identities = {}
    for f in files:
        if (type(f) is not dict or not exact_id(f.get('id')) or not text(f.get('kind'))
                or not text(f.get('version')) or f['id'] in identities):
            raise ValueError('invalid or duplicate file identity')
        identities[f['id']] = (f['kind'], f['version'])
    return v, stamp, identities

def compare(previous, response):
    if type(previous) is not list or not (1 <= len(previous) <= MAX_PROJECTS):
        raise ValueError('expected one to one hundred exact project baselines')
    selected = {}
    for row in previous:
        if type(row) is not dict or not exact_id(row.get('id')) or row['id'] in selected:
            raise ValueError('invalid or duplicate project baseline identity')
        selected[row['id']] = projection(row)
    if type(response) is not dict or type(response.get('mods')) is not dict:
        raise ValueError('invalid batch envelope')
    mods = response['mods']
    if not mods.keys() <= {str(k) for k in selected}:
        raise ValueError('batch returned an unrequested project identifier')
    result = []
    for ident, old in selected.items():
        if str(ident) not in mods:
            status = 'not_returned'
        else:
            try:
                new = projection(mods[str(ident)])
            except ValueError:
                status = 'invalid_row'
            else:
                status = 'same_version_signal' if old == new else 'version_signal_changed'
        # Detail-only fields still need the normal full-metadata path. The
        # caller must not use this result to remove or replace saved records.
        result.append({'id': ident, 'status': status, 'requires_full_detail_for_update_verdict': True})
    return result

def unique(pairs):
    output = {}
    for key, value in pairs:
        if key in output:
            raise ValueError('duplicate JSON key')
        output[key] = value
    return output

def main():
    try:
        raw = sys.stdin.buffer.read(MAX_BYTES + 1)
        if len(raw) > MAX_BYTES:
            raise ValueError('input too large')
        value = json.loads(raw, object_pairs_hook=unique)
        if type(value) is not dict or set(value) != {'previous', 'stats'}:
            raise ValueError('expected previous and stats')
        print(json.dumps(compare(value['previous'], value['stats']), separators=(',', ':')))
    except (ValueError, RecursionError) as error:
        print(f'ModHold batch review refused: {error}', file=sys.stderr)
        return 2
    return 0

if __name__ == '__main__':
    sys.exit(main())
