"""Verify the latest snapshot and its reviewed manifest, without writing source files."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import re
import tempfile

from validate_snapshot import validate_snapshot

MAX_RECEIPT = 64 * 1024
MAX_COMPRESSED = 64 * 1024 * 1024
MAX_MANIFEST = 64 * 1024 * 1024
MAX_SNAPSHOT = 512 * 1024 * 1024
CHUNK = 1024 * 1024
DIGESTS = ('compressed_sha256', 'snapshot_sha256', 'source_manifest_sha256')
COUNTS = ('entities', 'observations', 'source_scopes', 'game_namespaces', 'foreign_key_violations')


def unique_fields(items):
    result = {}
    for key, value in items:
        if key in result:
            raise ValueError('Duplicate field in validation receipt')
        result[key] = value
    return result


def read_receipt(path):
    with Path(path).open('rb') as stream:
        raw = stream.read(MAX_RECEIPT + 1)
    if len(raw) > MAX_RECEIPT:
        raise ValueError('Validation receipt exceeds size limit')
    result = json.loads(raw, object_pairs_hook=unique_fields)
    if not isinstance(result, dict):
        raise ValueError('Expected a latest-snapshot validation receipt')
    if not all(isinstance(result.get(k), str) and re.fullmatch('[a-f0-9]{64}', result[k]) for k in DIGESTS):
        raise ValueError('Missing or malformed snapshot digest')
    if not all(type(result.get(k)) is int and result[k] >= 0 for k in COUNTS):
        raise ValueError('Missing or malformed snapshot counts')
    if result.get('integrity') != 'ok' or result.get('exact_manifest_equality') is not True:
        raise ValueError('Receipt does not report successful validation')
    return result


def capture(stream, output, limit, label):
    sha = hashlib.sha256()
    size = 0
    with output.open('xb') as dest:
        while True:
            piece = stream.read(min(CHUNK, limit - size + 1))
            if not piece:
                break
            size += len(piece)
            if size > limit:
                raise ValueError(f'{label} exceeds size limit')
            sha.update(piece)
            dest.write(piece)
    return sha.hexdigest()


def verify_packaged(archive, receipt, manifest):
    expected = read_receipt(receipt)
    with tempfile.TemporaryDirectory(prefix='knowledge-verify-') as scratch:
        temp = Path(scratch)
        frozen_archive, frozen_manifest, snapshot = (
            temp / 'input.sqlite.gz', temp / 'manifest.json', temp / 'snapshot.sqlite'
        )
        # Hash and verify the exact captured inputs; never reopen original names
        # after a digest check. No receipt-provided path is interpreted.
        with Path(archive).open('rb') as stream:
            compressed = capture(stream, frozen_archive, MAX_COMPRESSED, 'Archive')
        if compressed != expected['compressed_sha256']:
            raise ValueError('Compressed archive does not match the receipt')
        with Path(manifest).open('rb') as stream:
            manifest_hash = capture(stream, frozen_manifest, MAX_MANIFEST, 'Manifest')
        if manifest_hash != expected['source_manifest_sha256']:
            raise ValueError('Reviewed manifest does not match the receipt')
        with gzip.open(frozen_archive, 'rb') as stream:
            raw = capture(stream, snapshot, MAX_SNAPSHOT, 'Decompressed snapshot')
        if raw != expected['snapshot_sha256']:
            raise ValueError('Decompressed snapshot does not match the receipt')
        with snapshot.open('rb') as stream:
            prefix = stream.read(20)
        if prefix[:16] != b'SQLite format 3\0' or prefix[18:20] != b'\x01\x01':
            raise ValueError('Expected a standalone SQLite snapshot')

        actual = validate_snapshot(snapshot, frozen_manifest)
        for key in ('snapshot_sha256', 'source_manifest_sha256', *COUNTS,
                    'integrity', 'exact_manifest_equality'):
            if actual.get(key) != expected[key]:
                raise ValueError(f'Recomputed validation does not match receipt field: {key}')
    return {
        **actual,
        'compressed_sha256': compressed,
        'read_only_inputs': True,
        'verified': 'archive hashes, schema, integrity, counts and reviewed manifest',
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive', type=Path)
    parser.add_argument('receipt', type=Path)
    parser.add_argument('manifest', type=Path)
    args = parser.parse_args()
    print(json.dumps(verify_packaged(args.archive, args.receipt, args.manifest), indent=2))


if __name__ == '__main__':
    main()
