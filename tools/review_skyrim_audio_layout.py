"""Review top-level SRD/MTD staging candidates without loading Skyrim or changing files."""
import argparse
from collections import Counter, defaultdict
import json
import os
from pathlib import Path
import re
import stat
import sys

if __name__ == "__main__":
    sys.dont_write_bytecode = True

from skyrim_staged_io import read_checked, reparse, safe_dir

SRD_SOURCE = "https://github.com/doodlum/skyrim-srd/blob/7d674959b20bbaa1887e8a3a07f112452c1cfa63/src/DataStorage.cpp"
MTD_SOURCE = "https://github.com/DeadOnKeyboard1/MusicTypeDistributor_Skyrim_1.7.104.0-Update/blob/4e26d5496c605266afa0734f8cc96c5155552190/src/Parser.cpp"
MAX_ENTRIES, MAX_FILES, MAX_LINES, MAX_RULES = 4096, 256, 32_000, 8_000
MAX_FILE_BYTES, MAX_TOTAL_BYTES = 128 * 1024, 2 * 1024 * 1024
AUDIO_SUFFIX = ("_SRD.json", "_SRD.jsonc", "_SRD.yaml", "_MUS.ini")
CASE_SUFFIX = tuple(s.casefold() for s in (*AUDIO_SUFFIX, "_SRD.yml"))
EDITOR = re.compile(r"[A-Za-z_][A-Za-z0-9_]{0,127}\Z")
FORM = re.compile(r"(?:0x)?([0-9a-f]{1,8})~([\w .()'\-]{1,200}\.es[mpl])\Z", re.IGNORECASE)
SECTIONS = {"general", "location", "region"}


def linked_token(raw):
    # The source removes the exact single space after a comma, not arbitrary
    # spacing inside the FormID/plugin pair. Keep the accepted subset explicit.
    match = FORM.fullmatch(raw)
    return match.group(2) if match else None


def inspect_ini(body, counts, line_budget, rule_budget):
    section, rows = None, []
    used = 0
    for used, raw in enumerate(body.splitlines(), 1):
        if used > line_budget:
            raise ValueError("Staging line budget exceeded.")
        line = raw.strip()
        if not line or line.startswith((";", "#")):
            continue
        if line.startswith("["):
            if not line.endswith("]") or not line[1:-1].strip():
                counts["unsupported-syntax"] += 1
                return rows, used, rule_budget
            section = line[1:-1].strip().lower()
            continue
        key, sep, value = line.partition("=")
        key, value = key.strip(), value.strip()
        if value.startswith("<<<"):
            counts["unsupported-syntax"] += 1
            # A multiline value can contain fake sections. Do not guess where it ends.
            return rows, used, rule_budget
        if section not in SECTIONS:
            counts["entry-outside-reviewed-sections"] += 1
            continue
        clear = section == "general" and key.endswith("!")
        target = key[:-1] if clear else key
        if not sep or not EDITOR.fullmatch(target):
            counts["unsupported-syntax"] += 1
            continue
        payload = value.replace(", ", ",").split(",") if section == "general" else [value]
        if section == "general":
            if len(payload) > rule_budget:
                raise ValueError("Staging reference budget exceeded.")
            plugins = [linked_token(token) for token in payload]
            rule_budget -= len(payload)
            if not plugins or not all(plugins):
                counts["unsupported-track-assignment"] += 1
                continue
            detail = tuple(sorted(set(plugins), key=lambda x: (x.casefold(), x)))
        else:
            if rule_budget < 1:
                raise ValueError("Staging reference budget exceeded.")
            rule_budget -= 1
            if not EDITOR.fullmatch(value):
                counts["unsupported-mapping"] += 1
                continue
            detail = value
        rows.append((section, target, clear, detail, used))
    return rows, used, rule_budget


def inspect(root, counts_only=False):
    if not safe_dir(root):
        raise ValueError("Choose an ordinary staging directory, not a link or junction.")
    counts, candidates, targets, plugins = Counter(), [], defaultdict(list), set()
    with os.scandir(root) as stream:
        for index, entry in enumerate(stream, 1):
            if index > MAX_ENTRIES:
                raise ValueError("Staging directory exceeds the entry cap.")
            name = entry.name
            if name.casefold().endswith((".esm", ".esp", ".esl")):
                s = Path(entry.path).lstat()
                if stat.S_ISREG(s.st_mode) and not reparse(s):
                    plugins.add(name.casefold())
            suffix = next((s for s in AUDIO_SUFFIX if name.endswith(s)), None)
            if suffix is None:
                if name.casefold().endswith(CASE_SUFFIX):
                    counts["unsupported-suffix-or-case"] += 1
                continue
            if len(candidates) >= MAX_FILES:
                raise ValueError("Too many audio config candidates.")
            path = Path(entry.path)
            candidates.append((name, suffix, path, path.lstat()))
    files, refs = [], {}
    bytes_left, lines_left, rules_left = MAX_TOTAL_BYTES, MAX_LINES, MAX_RULES
    for name, suffix, path, info in sorted(candidates, key=lambda x: x[0]):
        if not stat.S_ISREG(info.st_mode) or reparse(info):
            raise ValueError("A matching staged entry is not an ordinary file.")
        if suffix != "_MUS.ini":
            # SRD source names ".es" anywhere in the stem as its routing heuristic.
            kind = "srd-plugin-name-candidate" if ".es" in path.stem else "srd-general-name-candidate"
            counts[kind] += 1
            files.append({"file": name, "kind": kind})
            continue
        counts["mtd-config-candidate"] += 1
        budget = min(MAX_FILE_BYTES, bytes_left)
        if info.st_size > budget:
            raise ValueError("MTD file exceeds the available read budget.")
        raw = read_checked(path, info, budget)
        bytes_left -= len(raw)
        try:
            body = raw.decode("utf-8-sig")
        except UnicodeDecodeError:
            counts["unreviewed-encoding"] += 1
            files.append({"file": name, "kind": "mtd", "utf8-decoded": False})
            continue
        rows, used, rules_left = inspect_ini(body, counts, lines_left, rules_left)
        lines_left -= used
        counts["reviewed-assignments"] += len(rows)
        files.append({"file": name, "kind": "mtd", "utf8-decoded": True})
        for section, target, clear, detail, line in rows:
            targets[section, target].append((name, line, clear, detail))
            if section == "general":
                for plugin in detail:
                    refs.setdefault(plugin.casefold(), plugin)
    overlaps = []
    for (section, target), declarations in sorted(targets.items()):
        if section == "general":
            layers = {clear for _, _, clear, _ in declarations}
            if len(layers) != 2:
                continue
            kind = "mixed-clear-and-append"
        elif len({detail for _, _, _, detail in declarations}) > 1:
            kind = "competing-music-type-assignments"
        else:
            continue
        counts[kind] += 1
        if not counts_only:
            overlaps.append({"kind": kind, "section": section, "target": target,
                             "sites": [{"file": name, "line": line} for name, line, _, _ in declarations]})
    counts["referenced-plugin-names-present-at-root"] = len(refs.keys() & plugins)
    counts["referenced-plugin-names-not-at-root"] = len(refs.keys() - plugins)
    result = {"schema": "skyrim-audio-staging-review-v1",
              "counts": dict(sorted(counts.items())), "sources": [MTD_SOURCE, SRD_SOURCE]}
    if not counts_only:
        result.update(candidates=files, overlaps=overlaps, plugin_name_candidates=[
            {"name": value, "at-staging-root": key in plugins} for key, value in sorted(refs.items())])
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("staging", type=Path)
    parser.add_argument("--counts-only", action="store_true")
    args = parser.parse_args()
    try:
        print(json.dumps(inspect(args.staging, args.counts_only), ensure_ascii=True, indent=2))
    except (OSError, ValueError):
        # An OSError string can contain the user's real path.
        parser.exit(2, "Audio staging review refused: verify the ordinary directory, file type and scan budgets.\n")


if __name__ == "__main__":
    main()
