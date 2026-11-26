"""Create and smoke-test a read-only portable knowledge-manager evaluation ZIP."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import re
import stat
import subprocess
import sys
import tempfile
import zipfile

from validate_snapshot import ROOT, validate_snapshot

MAX_INPUT = 64 * 1024 * 1024


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read_regular(path):
    info = path.lstat()
    if (not stat.S_ISREG(info.st_mode) or getattr(info, 'st_file_attributes', 0) & 0x400
            or not 0 < info.st_size <= MAX_INPUT):
        raise ValueError('Expected a bounded regular file (not a link)')
    with path.open('rb') as source:
        data = source.read(MAX_INPUT + 1)
    if len(data) != info.st_size:
        raise ValueError('Input size changed while being read')
    return data


def parse_native(result):
    return json.loads(result.stdout.decode('utf-8', errors='strict'))


def smoke(folder, executable, entity_count, observed_count, expected_database):
    db = folder / 'catalog.sqlite'
    binary = folder / executable

    def run(*args):
        return subprocess.run([str(binary), *map(str, args)], cwd=folder, capture_output=True,
                              timeout=30, check=True)
    if b'browse-in' not in run('--help').stdout:
        raise ValueError('Packaged binary does not support the scoped command')
    status = parse_native(run(db, 'status'))
    if (status['entities'], status['observations'], status['withheld_observations']) != (
            entity_count, observed_count, 0):
        raise ValueError('Packaged manager opened a different catalog')
    if parse_native(run(db, 'check'))['ok'] is not True:
        raise ValueError('Packaged manager rejected catalog integrity')
    scoped = parse_native(run(db, 'browse-in', 'minecraft-java', 'mod', 'Iris'))
    if [entry['id'] for entry in scoped['entities']] != ['modrinth:project:YL57xq9U']:
        raise ValueError('Expected exact Minecraft Iris project from the reviewed catalog')
    relations = parse_native(run(db, 'evidence', 'adeepindigo:project:dental-care', '--mod-relations-v2'))
    result = relations['mod_relationships']
    if result['schema'] != 2 or result['coverage'] != 'unknown':
        raise ValueError('Relationship report does not preserve the original uncertainty')
    required = [item['target_id'] for row in result['observations'] for col in row['columns']
                if col['role'] == 'required' for item in col['evidence']]
    if 'lumpinou:project:the-mood-pack-mod' not in required:
        raise ValueError('Source-specific Dental Care dependency was not retained')
    view = run(db, 'visual-in', 'minecraft-java', 'mod', 'Iris').stdout
    if b'Iris Shaders' not in view or b'Vampirism' in view:
        raise ValueError('Visual export did not retain scoped selection')
    if sha(read_regular(db)) != expected_database or any(
            (folder / (db.name + s)).exists() for s in ('-wal', '-shm', '-journal')):
        raise ValueError('Read-only commands altered the catalog or created sidecars')
    return (['help', 'status', 'integrity', 'scoped lookup', 'evidence and relationships', 'scoped visual', 'unchanged database'], view)


def package(binary, snapshot, output, commit):
    if not re.fullmatch(r'[0-9a-f]{40}', commit):
        raise ValueError('Pass the full source commit')
    if sys.platform not in ('win32', 'linux'):
        raise ValueError('Supported candidate hosts are Windows and Linux')
    if output.suffix != '.zip' or output.exists() or output.is_symlink():
        raise ValueError('Choose a fresh .zip output path')
    if any(snapshot.with_name(snapshot.name + s).exists() for s in ('-wal', '-shm', '-journal')):
        raise ValueError('Refusing a snapshot with SQLite sidecars')
    exe = 'knowledge-manager.exe' if sys.platform == 'win32' else 'knowledge-manager'
    binary_bytes = read_regular(binary)
    database_bytes = read_regular(snapshot)
    guide = f"""MODDING-KNOWLEDGE / PORTABLE EVALUATION

Extract the entire ZIP into a fresh folder. Open a terminal in that folder.
This is an unsigned test build, not an installer or a public release.

Read-only examples (works with the included reviewed database):
  ./{exe} catalog.sqlite status
  ./{exe} catalog.sqlite check
  ./{exe} catalog.sqlite browse-in minecraft-java mod Iris
  ./{exe} catalog.sqlite evidence adeepindigo:project:dental-care --mod-relations-v2

Open the included iris-catalog.html for a bounded, offline example. A source media
link only loads after you select it. The database and executable stay right
here. The commands do not update the catalog, install mods, or check runtime
compatibility. See candidate.json for source, file hashes and test results.
Keep your other snapshots unchanged.
""".encode('utf-8')
    with tempfile.TemporaryDirectory(prefix='Portable Manager smoke ', dir=output.parent) as folder:
        stage = Path(folder).resolve()
        path = stage / 'catalog.sqlite'
        path.write_bytes(database_bytes)
        trusted = validate_snapshot(path, ROOT / 'data/reviewed-bundle.json')
        if trusted['snapshot_sha256'] != sha(database_bytes):
            raise ValueError('Validation did not inspect the exact snapshot bytes')
        files = {exe: binary_bytes, 'catalog.sqlite': database_bytes, 'START-HERE.txt': guide}
        probe = io.BytesIO()
        with zipfile.ZipFile(probe, 'w', zipfile.ZIP_DEFLATED, compresslevel=9) as z:
            for name, data in files.items():
                z.writestr(name, data)
        # Exercise the actual packaged copy from a separate path with spaces.
        extracted = stage / 'extracted evaluation'
        with zipfile.ZipFile(probe) as z:
            if z.testzip() is not None or z.namelist() != list(files):
                raise ValueError('Candidate ZIP verification failed')
            z.extractall(extracted)
        (extracted / exe).chmod(0o700)
        for name, data in files.items():
            if sha(read_regular(extracted / name)) != sha(data):
                raise ValueError('Extracted candidate bytes differ')
        checked, files['iris-catalog.html'] = smoke(extracted, exe, trusted['entities'], trusted['observations'],
                                                   trusted['snapshot_sha256'])
        receipt = {'schema': 1, 'source_commit': commit, 'platform': sys.platform,
                   'unsigned': True, 'release': False, 'real_host_cli_smoke': checked,
                   'reviewed_data': trusted,
                   'files': {name: {'bytes': len(data), 'sha256': sha(data)}
                             for name, data in files.items()}}
    # Do not change any already accepted destination; publish only after tests.
    with zipfile.ZipFile(output, 'x', zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        for name, data in files.items():
            z.writestr(name, data)
        z.writestr('candidate.json', json.dumps(receipt, indent=2) + '\n')
    with zipfile.ZipFile(output) as z:
        if z.testzip() is not None or z.namelist() != [*files, 'candidate.json']:
            raise ValueError('Candidate ZIP verification failed after writing')
        if any(sha(z.read(n)) != info['sha256'] for n, info in receipt['files'].items()):
            raise ValueError('Shipped candidate bytes differ from tested copy')
    return {**receipt, 'archive_sha256': sha(output.read_bytes()), 'archive_bytes': output.stat().st_size}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--snapshot', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--commit', required=True)
    args = parser.parse_args()
    print(json.dumps(package(args.binary, args.snapshot, args.output, args.commit), indent=2))
