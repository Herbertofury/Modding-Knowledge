"""Package the exact validated bytes of a standalone native snapshot."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile

from validate_snapshot import validate_snapshot

ROOT = Path(__file__).resolve().parents[1]
ARTIFACT = 'modding-knowledge-latest.sqlite.gz'
SIDECARS = ('-wal', '-shm', '-journal')


def reject_sidecars(source):
    if any(os.path.lexists(str(source) + suffix) for suffix in SIDECARS):
        raise ValueError('Close the database and provide a standalone SQLite snapshot')


def publish(path, payload):
    # Unique sibling staging: an interrupted write never truncates the prior file.
    fd, pending = tempfile.mkstemp(prefix='.knowledge-snapshot-', dir=path.parent)
    try:
        with os.fdopen(fd, 'wb') as stream:
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(pending, path)
    finally:
        if os.path.lexists(pending):
            os.unlink(pending)


def package_snapshot(source, manifest, output_dir):
    source = Path(source).resolve(strict=True)
    if not source.is_file():
        raise ValueError('Expected a standalone SQLite file')
    reject_sidecars(source)
    raw = source.read_bytes()
    reject_sidecars(source)
    if raw[:16] != b'SQLite format 3\0' or raw[18:20] != b'\x01\x01':
        raise ValueError('Expected a standalone rollback-journal SQLite snapshot')

    # SQLite never opens the original. Compress only these exact captured bytes,
    # even if a later writer replaces the original name during validation.
    with tempfile.TemporaryDirectory(prefix='knowledge-capture-') as temporary:
        captured = Path(temporary) / 'captured.sqlite'
        captured.write_bytes(raw)
        report = validate_snapshot(captured, manifest)
    if report['snapshot_sha256'] != hashlib.sha256(raw).hexdigest():
        raise ValueError('Captured snapshot does not match validation receipt')

    compressed = gzip.compress(raw, mtime=0)
    report['compressed_sha256'] = hashlib.sha256(compressed).hexdigest()
    report['artifact'] = ARTIFACT
    report['source_refresh_state'] = '../data/live/modrinth-refresh.json'
    receipt = (json.dumps(report, indent=2) + '\n').encode('utf-8')
    dest = Path(output_dir)
    publish(dest / ARTIFACT, compressed)
    # The receipt is published last. A local interruption between files can
    # leave unequal hashes; the hosted job commits the pair only after success.
    publish(dest / 'latest-validation.json', receipt)
    return report


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('Usage: package_snapshot.py STANDALONE_SNAPSHOT.sqlite')
    package_snapshot(sys.argv[1], ROOT / 'data/reviewed-bundle.json', ROOT / 'artifacts')
