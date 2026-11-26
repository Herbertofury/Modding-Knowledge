"""Review one local, public Modhold detail response. Never calls a provider."""
import argparse
import hashlib
import json
import re
import sys
from urllib.parse import urlsplit

LIMIT = 1024 * 1024
MAX_ID = 2**53 - 1
KNOWN_KINDS = {"main", "optional", "misc", "old", "patch"}


def unique_object(pairs):
    out = {}
    for key, value in pairs:
        if key in out:
            raise ValueError("duplicate response field")
        out[key] = value
    return out


def positive_id(value):
    return type(value) is int and 0 < value <= MAX_ID


def refused(_):
    raise ValueError("non-JSON constant in response")


def exact_text(value, limit=1000):
    return (
        isinstance(value, str) and 0 < len(value) <= limit
        and not any(ord(c) < 32 or 127 <= ord(c) < 160 for c in value)
    )


def public_url(value):
    if not exact_text(value, 2048):
        return None
    try:
        u = urlsplit(value)
    except ValueError:
        return None
    if u.scheme != "https" or not u.hostname or u.username or u.password or u.netloc != u.hostname:
        return None
    return value


def project_detail(data, requested_id, game):
    if len(data) > LIMIT:
        raise ValueError("response exceeds 1 MiB")
    if not positive_id(requested_id) or not re.fullmatch(r"[a-z0-9][a-z0-9-]{0,63}", game):
        raise ValueError("request identity is invalid")
    r = json.loads(data.decode("utf-8"), object_pairs_hook=unique_object, parse_constant=refused)
    if not isinstance(r, dict) or not positive_id(r.get("id")) or r["id"] != requested_id or r.get("game") != game:
        raise ValueError("response mod or game does not match request")
    page = r.get("page")
    if public_url(page) is None or not re.fullmatch(
        rf"https://modholdmods\.com/g/{game}/mods/{requested_id}/[a-z0-9][a-z0-9-]*", page
    ):
        raise ValueError("response page does not match request")
    version = r.get("version")
    if not exact_text(version, 160):
        raise ValueError("response version is invalid")

    files = r.get("files")
    if not isinstance(files, list) or len(files) > 1000:
        raise ValueError("response files are unavailable or malformed")
    seen = set()
    output_files = []
    for f in files:
        if not isinstance(f, dict) or not positive_id(f.get("id")) or f["id"] in seen:
            raise ValueError("file identity is missing or ambiguous")
        seen.add(f["id"])
        role, ver, size, digest = (f.get(key) for key in ("kind", "version", "size", "sha256"))
        if not exact_text(role, 80) or not exact_text(ver, 160):
            raise ValueError("file role or version is invalid")
        if size is not None and (type(size) is not int or not 0 <= size <= MAX_ID):
            raise ValueError("file size is invalid")
        if digest is not None and (not isinstance(digest, str) or not re.fullmatch(r"[0-9a-fA-F]{64}", digest)):
            raise ValueError("file SHA-256 is invalid")
        output_files.append({
            "id": f["id"], "key": f"modhold:file:{f['id']}", "version": ver,
            "provider_role": role, "role_known": role in KNOWN_KINDS, "bytes": size,
            "reported_sha256": digest, "bytes_verified": False,
        })
    main = [f["id"] for f in output_files if f["provider_role"] == "main" and f["version"] == version]
    # No automatic file selection even when exactly one main file matches.
    file_status = "corroborated" if main else "unverified"

    requirements = r.get("requirement_list")
    reqs = None
    if isinstance(requirements, list) and len(requirements) <= 1000:
        reqs = []
        for q in requirements:
            if not isinstance(q, dict) or not exact_text(q.get("name"), 300):
                raise ValueError("relationship name is invalid")
            h, n = q.get("modhold_id"), q.get("nexus_mod_id")
            for target in (h, n):
                if target is not None and not positive_id(target):
                    raise ValueError("relationship provider ID is invalid")
            flag, note = q.get("required"), q.get("note")
            if flag is not None and type(flag) is not bool:
                raise ValueError("relationship flag is invalid")
            if note is not None and (not isinstance(note, str) or len(note) > 2000):
                raise ValueError("relationship note is invalid")
            reqs.append({
                "name": q["name"], "modhold_id": h, "nexus_mod_id": n,
                "target_url": public_url(q.get("url")),
                "provider_required": flag, "provider_note": note,
                "resolved_required": None,
            })

    updated = r.get("updated")
    return {
        "schema": "modhold-local-detail-review-v1",
        "id": requested_id, "key": f"modhold:mod:{requested_id}", "game": game,
        "source_url": f"https://modholdmods.com/api/v1/mods/{requested_id}",
        "page": page, "version": version, "updated_provider_text": updated if exact_text(updated, 100) else None,
        "input_sha256": hashlib.sha256(data).hexdigest(), "files": output_files,
        "matching_main_file_ids": main, "file_version_evidence": file_status,
        "recorded_requirements": reqs,
        "conflict_count_reported": len(r["conflicts"]) if isinstance(r.get("conflicts"), list) else None,
        "load_after_count_reported": len(r["loadAfter"]) if isinstance(r.get("loadAfter"), list) else None,
        "scope": "Selected provider payload only. No automatic choice, relationship resolution, absence or compatibility claim, or verified archive bytes.",
    }


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--id", type=int, required=True)
    p.add_argument("--game", required=True)
    a = p.parse_args(argv)
    try:
        report = project_detail(sys.stdin.buffer.read(LIMIT + 1), a.id, a.game)
    except (OSError, ValueError, RecursionError, TypeError):
        print("Modhold detail refused: verify input size, source identity and metadata shape.", file=sys.stderr)
        return 2
    print(json.dumps(report, ensure_ascii=True, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
