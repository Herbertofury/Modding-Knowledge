"""Review a selected KID staging root. Prints source candidates, never changes files."""
from collections import Counter
from pathlib import Path
import argparse
import json
import os
import re
import stat
from skyrim_staged_io import read_checked, reparse, safe_dir

KID = "https://github.com/powerof3/Keyword-Item-Distributor/blob/99adaaa1d8cefe320524802b57e91723f66b1c7c/"
CLIB = "https://github.com/powerof3/CLibUtil/blob/aca80148853a8c0cb3e3cab3eaa20f1886cc99e7/"
MAX_CHILDREN, MAX_FILES, MAX_LINES, MAX_REFS = 4096, 256, 32_000, 20_000
MAX_FILE, MAX_BYTES = 128 * 1024, 2 * 1024 * 1024
PLUG = re.compile(r"[\w .()'\-]{1,200}\.es[mpl]\Z", re.IGNORECASE)
QUALIFIED = re.compile(r"(?:0x)?[0-9a-f]{1,8}\s*~\s*(.+)\Z", re.IGNORECASE)


def candidate(root):
    if not safe_dir(root):
        raise ValueError("Choose an ordinary staging directory, not a link or junction.")
    results = []
    plugins = set()
    other = Counter()
    with os.scandir(root) as it:
        for index, entry in enumerate(it, 1):
            if index > MAX_CHILDREN:
                raise ValueError("Staging directory exceeds the entry cap.")
            name = entry.name
            if PLUG.fullmatch(name):
                s = Path(entry.path).lstat()
                if stat.S_ISREG(s.st_mode) and not reparse(s):
                    plugins.add(name.casefold())
                continue
            if "_KID" not in name or Path(name).suffix != ".ini":
                if "_kid" in name.casefold() and name.casefold().endswith(".ini"):
                    other["case-mismatch"] += 1
                continue
            if len(results) >= MAX_FILES:
                raise ValueError("Too many matching KID entry names.")
            path = Path(entry.path)
            s = path.lstat()
            results.append((name, path, s))
    return sorted(results, key=lambda row: (row[0].casefold(), row[0])), plugins, other


def plugin_from(token, allow_bare):
    token = token.strip()
    match = QUALIFIED.fullmatch(token)
    if match and PLUG.fullmatch(match.group(1)):
        return match.group(1), "plugin-qualified-form"
    if allow_bare and PLUG.fullmatch(token):
        return token, "bare-filter-name"
    return None


def scan_text(text, plugins, cap, ref_cap):
    stats = Counter()
    refs = []
    def add(value):
        if len(refs) >= ref_cap:
            raise ValueError("Staging reference cap exceeded.")
        refs.append(value)

    named = False
    n = 0
    for n, raw in enumerate(text.splitlines(), 1):
        if n > cap:
            raise ValueError("Staging rule-line cap exceeded.")
        line = raw.strip()
        if not line or line.startswith(("#", ";")):
            continue
        if line.startswith("["):
            # This scanner does not emulate exotic SimpleINI section syntax.
            if line.startswith("[]"):
                stats["unsupported-region"] += 1
                break
            named = True
            continue
        key, sep, value = line.partition("=")
        if not sep:
            continue
        if named:
            stats["named-section-entry"] += 1
            continue
        if value.lstrip().startswith("<<<"):
            stats["unsupported-region"] += 1
            break
        kind = "exclusive-group" if key.strip() == "ExclusiveGroup" else "ordinary-entry"
        stats[kind] += 1
        fields = [piece.strip() for piece in value.split("|", 6)]
        if kind == "exclusive-group":
            if len(fields) < 2 or not all(fields[:2]):
                stats["incomplete-group"] += 1
                continue
            filter_field = fields[1]
        else:
            if len(fields) < 2 or not all(fields[:2]):
                stats["incomplete-rule"] += 1
                continue
            attached = plugin_from(fields[0], False)
            if attached:
                add((attached, "keyword-form", "positive", None, n))
            filter_field = fields[2] if len(fields) > 2 else ""
        for gi, group in enumerate(filter_field.split(","), 1):
            conjunction = "+" in group and kind != "exclusive-group"
            for term in group.split("+") if conjunction else [group]:
                term = term.strip()
                polarity = "excluded" if term.startswith("-") else "positive"
                term = term.removeprefix("-").strip()
                if term.startswith("*"):
                    continue  # Partial string is not an explicit plugin reference.
                attached = plugin_from(term, kind != "exclusive-group")
                if attached:
                    role = kind if kind == "exclusive-group" else ("all-filter" if conjunction else "any-filter")
                    add((attached, role, polarity, gi if conjunction else None, n))
    return stats, [
        {"plugin": name, "form": form, "role": role, "polarity": polarity, "and_group": group, "line": n,
         "staging_presence": "name-seen" if name.casefold() in plugins else "not-seen-in-selected-root"}
        for (name, form), role, polarity, group, n in refs
    ], n


def review(root, counts_only=False):
    entries, plugins, rejected = candidate(root)
    file_report = []
    categories, counts, references = Counter(), Counter(), Counter()
    used, line_total, ref_total = 0, 0, 0
    for name, path, s in entries:
        row = {"file": name, "marker": "at-end" if name.endswith("_KID.ini") else "within-name"}
        if not stat.S_ISREG(s.st_mode) or reparse(s):
            row["status"] = "not-a-plain-file"
        elif s.st_size > min(MAX_FILE, MAX_BYTES-used):
            row["status"] = "byte-budget"
        else:
            try:
                raw = read_checked(path, s, min(MAX_FILE, MAX_BYTES-used))
                used += len(raw)
                if b"\0" in raw:
                    raise UnicodeError("NUL")
                text = raw.decode("utf-8-sig")
            except (OSError, ValueError, UnicodeError):
                row["status"] = "unreadable-unsupported-or-changed"
            else:
                row["status"] = "sampled-utf8"
                stats, refs, n = scan_text(text, plugins, MAX_LINES-line_total, MAX_REFS-ref_total)
                line_total += n
                ref_total += len(refs)
                row["counts"] = dict(stats)
                row["references"] = refs
                counts.update(stats)
                references.update(f"{ref['role']}:{ref['polarity']}" for ref in refs)
        categories[row["status"]] += 1
        if row["marker"] == "within-name":
            counts["nonterminal-marker-filenames"] += 1
        file_report.append(row)
    return {
        "schema": "athena-skyrim-kid-layout-v1",
        "source": {"discovery": KID+"src/LookupConfigs.cpp", "helper": CLIB+"include/CLIBUtil/distribution.hpp",
                   "filters": KID+"src/Data/BaseFilters.cpp", "groups": KID+"src/ExclusiveGroups.cpp"},
        "scope": "Top-level staging candidates and basic UTF-8 references only. No runtime, resolved plugin dependency, ordering winner, game or load order asserted.",
        "file_states": dict(categories),
        "not_candidates": dict(rejected),
        "sample": dict(counts),
        "reference_roles": dict(references),
        "files": None if counts_only else file_report,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("--counts-only", action="store_true", help="omit filenames, plugin names and line numbers")
    args = parser.parse_args()
    try:
        print(json.dumps(review(args.root, args.counts_only), indent=2, ensure_ascii=True))
    except (OSError, ValueError) as error:
        parser.error(str(error) if isinstance(error, ValueError) else "Could not read the selected staging root.")


if __name__ == "__main__":
    main()
