"""Bounded, read-only review of a user-selected staging Data directory."""
from collections import Counter, defaultdict
from pathlib import Path
import argparse
from hashlib import sha256
import json
import os
import re
import stat

SOURCE_PIN = '82a694e5d33194ae223f23846b1f180ee394cdee'
BASE = 'https://github.com/MaskedRPGFan/FormList-Manipulator/blob/' + SOURCE_PIN
ALIAS = re.compile(r'(?i:alias)[ \t]*=[ \t]*([^|]+)\|(.+)\Z')
MAX_CHILDREN, MAX_FILES, MAX_ALIASES = 4096, 256, 20_000
MAX_FILE, MAX_BYTES = 128 * 1024, 2 * 1024 * 1024


def reparse(s):
    return getattr(s, 'st_file_attributes', 0) & getattr(stat, 'FILE_ATTRIBUTE_REPARSE_POINT', 0)


def safe_dir(p):
    s = p.lstat()
    return not p.is_symlink() and not reparse(s) and stat.S_ISDIR(s.st_mode)


def staged_files(data):
    if not safe_dir(data):
        raise ValueError('Select a real Data staging directory, not a symbolic link or junction')
    sub = data / 'FLM'
    paths = [(data, 'Data-root')]
    try:
        if not safe_dir(sub):
            raise ValueError('FLM subfolder must be a real directory, not a symbolic link or junction')
    except FileNotFoundError:
        pass
    else:
        paths.append((sub, 'FLM-subfolder'))
    entries = []
    for folder, role in paths:
        with os.scandir(folder) as children:
            for i, info in enumerate(children, start=1):
                if i > MAX_CHILDREN:
                    raise ValueError('Directory exceeds review entry limit')
                lower = info.name.casefold()
                if role == 'Data-root' and not lower.endswith('_flm.ini') or role == 'FLM-subfolder' and not lower.endswith('.ini'):
                    continue
                if len(entries) >= MAX_FILES:
                    raise ValueError('Too many candidate configuration files')
                relative = info.name if role == 'Data-root' else 'FLM/' + info.name
                s = info.stat(follow_symlinks=False)
                state = 'candidate' if stat.S_ISREG(s.st_mode) and not reparse(s) else 'not-a-plain-file'
                if lower == 'formlistmanipulator_debug.ini':
                    state = 'debug-toggle-not-analyzed'
                if role == 'FLM-subfolder' and not info.name.endswith('.ini') and state == 'candidate':
                    state = 'extension-case-needs-review'
                entries.append((relative, role, state, Path(info.path), s))
    return sorted(entries, key=lambda e: (e[1] != 'Data-root', e[0].casefold(), e[0]))


def report(data, counts_only=False):
    files, aliases, statuses = [], defaultdict(list), Counter()
    used = declarations = 0
    for relative, role, status, file, initial in staged_files(data):
        row = {'path': relative, 'location': role, 'status': status}
        if status == 'candidate':
            if initial.st_size > MAX_FILE or used + initial.st_size > MAX_BYTES:
                row['status'] = 'byte-budget'
            else:
                try:
                    flags = os.O_RDONLY | getattr(os, 'O_BINARY', 0) | getattr(os, 'O_NOFOLLOW', 0) | getattr(os, 'O_NONBLOCK', 0)
                    with os.fdopen(os.open(file, flags), 'rb') as handle:
                        now = os.fstat(handle.fileno())
                        if (now.st_dev, now.st_ino) != (initial.st_dev, initial.st_ino) or not stat.S_ISREG(now.st_mode) or reparse(now):
                            raise ValueError('Changed while opening')
                        raw = handle.read(min(MAX_FILE, MAX_BYTES - used) + 1)
                        if len(raw) > MAX_FILE or used + len(raw) > MAX_BYTES:
                            raise ValueError('Changed beyond budget')
                    used += len(raw)
                    if b'\0' in raw:
                        raise UnicodeError('NUL is not supported')
                    text = raw.decode('utf-8-sig')
                except (OSError, ValueError, UnicodeError):
                    row['status'] = 'unreadable-unsupported-or-changed'
                else:
                    row['status'] = 'read-utf8'
                    row['sha256'] = sha256(raw).hexdigest()
                    current = 0
                    in_section = False
                    for line_number, source in enumerate(text.splitlines(), 1):
                        line = source.strip()
                        if line.startswith('['):
                            in_section = True
                        if in_section or line.startswith(('#', ';')):
                            continue
                        candidate = ALIAS.fullmatch(line)
                        if candidate is None:
                            continue
                        key = candidate.group(1).strip()
                        if not key or len(key) > 100:
                            continue
                        current += 1
                        declarations += 1
                        if declarations > MAX_ALIASES:
                            raise ValueError("Alias declarations exceed the review limit")
                        aliases[key.casefold()].append({'path':relative,'line':line_number})
                    row['sampled_alias_declarations'] = current
        files.append(row)
        statuses[row['status']] += 1
    overlap = [{'alias_key_casefolded': key, 'locations': loc} for key, loc in sorted(aliases.items()) if len(loc) > 1]
    return {
        'schema':'athena-skyrim-flm-layout-review-v1',
        'source_commit':SOURCE_PIN,
        'references':{'readme':BASE+'/README.md','discovery':BASE+'/src/Manipulator/Manipulator.hpp#L346-L374',
                      'alias':BASE+'/src/Manipulator/Manipulator.hpp#L826-L878'},
        'scope':'Read-only candidate inventory and possible duplicate top-level UTF-8 Alias declarations. Report sort is not runtime order. No winner, deployed DLL, virtual filesystem, game, load order or save verified.',
        'readme_source_difference':'Pinned README describes alphabetic processing and an _FLM.ini suffix; pinned subfolder code appends native directory iteration of .ini files without an explicit sort or suffix rule.',
        'file_count':len(files), 'file_states':dict(sorted(statuses.items())),
        'possible_overlap_count':len(overlap),
        'files':None if counts_only else files,
        'possible_overlaps':None if counts_only else overlap,
    }


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('data_root', type=Path, help='a staging Data directory chosen by you')
    p.add_argument('--counts-only', action='store_true', help='omit filenames, hashes, line numbers and alias strings')
    a = p.parse_args()
    try:
        result = report(a.data_root, a.counts_only)
    except (OSError, ValueError) as err:
        # Restrict error output to our own message; OS errors can contain an absolute path.
        p.error(str(err) if isinstance(err, ValueError) else 'Could not review the selected Data directory')
    print(json.dumps(result, indent=2, ensure_ascii=True))


if __name__ == '__main__':
    main()
