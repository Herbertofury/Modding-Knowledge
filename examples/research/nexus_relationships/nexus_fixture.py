"""Research-only decoder oracle. No network, credentials, disk writes or catalog imports."""


def field(value, key, expected):
    if type(value) is not dict or key not in value or type(value[key]) is not expected:
        raise ValueError(f"Unsupported shape for {key}")
    return value[key]


def opaque(value, key):
    text = field(value, key, str)
    if not text or text.strip() != text:
        raise ValueError(f"Invalid identifier {key}")
    return text


def scoped_range(raw):
    target = field(raw, "target_mod_file", dict)
    mod = field(target, "mod", dict)
    game = field(mod, "game", dict)
    group = opaque(target, "id")
    minimum = field(raw, "min_version", dict)
    if "max_version" not in raw:
        raise ValueError("Missing nullable max_version")
    maximum = raw["max_version"]
    for version in (minimum, maximum):
        if version is not None and opaque(field(version, "file", dict), "id") != group:
            raise ValueError("Version IDs cross file/update groups")
    return {
        "range_id": opaque(raw, "id"),
        "target_group_id": group,
        "target_mod_id": opaque(mod, "id"),
        "target_game_id": opaque(game, "id"),
        "target_mod_game_scoped_id": opaque(mod, "game_scoped_id"),
        "min_version_id": opaque(minimum, "id"),
        "max_version_id": opaque(maximum, "id") if maximum is not None else None,
    }


def scoped_dlc(raw):
    return {"entry_id": opaque(raw, "id"), "dlc_id": opaque(raw, "dlc_id")}


def groups(body, list_key, branch_key, project, branch_id):
    out = {}
    for group in field(body, list_key, list):
        group_id = opaque(group, "id")
        if group_id in out:
            raise ValueError("Duplicate definition")
        branches = [project(raw) for raw in field(group, branch_key, list)]
        if len({b[branch_id] for b in branches}) != len(branches):
            raise ValueError("Duplicate branch")
        if not branches:
            raise ValueError("Definition has no alternatives")
        out[group_id] = branches
    return out


def raw_review(status, request_version, body):
    """request_version is the already-associated exact v3 version ID, not a URL ID."""
    identity = opaque({"source": request_version}, "source")
    if status != 200:
        return {"source_version_id": identity, "coverage": "unavailable",
                "version_definitions": None, "dlc_definitions": None}
    version = groups(body, "dependency_definitions", "ranges", scoped_range, "range_id")
    dlc = groups(body, "dlc_dependency_definitions", "dlc_targets", scoped_dlc, "entry_id")
    return {"source_version_id": identity,
            "coverage": "definitions_observed" if version or dlc else "empty_response_scope_only",
            "version_definitions": version, "dlc_definitions": dlc}


def candidate_grouping(status, requested, body):
    """One page; candidates are not dependency declarations or availability proof."""
    if status != 200:
        return {"coverage": "unavailable", "by_version_definition": {}, "reported_total": None}
    meta = field(body, "meta", dict)
    page = field(meta, "page", int)
    size = field(meta, "page_size", int)
    total = field(meta, "total_count", int)
    if page < 1 or not 1 <= size <= 5000 or total < 0:
        raise ValueError("Invalid pagination")
    candidates = field(field(body, "data", dict), "candidates", list)
    if len(candidates) > size or (candidates and total < len(candidates)):
        raise ValueError("Invalid page count")
    result = {}
    seen = set()
    for row in candidates:
        v = opaque(row, "source_version_id")
        d = opaque(row, "definition_id")
        g = opaque(row, "mod_file_id")
        c = opaque(row, "version_id")
        mod = opaque(row, "mod_id")
        if v not in requested:
            raise ValueError("Unexpected source version in this response")
        if (v, d, c) in seen:
            raise ValueError("Duplicate candidate")
        seen.add((v, d, c))
        result.setdefault((v, d), []).append({
            "target_group_id": g, "candidate_version_id": c, "target_mod_id": mod,
            # The supplied decimal string is not normalized or used as ordering evidence.
            "position": field(row, "position", str), "category": field(row, "category", str),
            "mod_status": field(row, "mod_status", str),
        })
    return {"coverage": "candidate_page_only" if candidates else "empty_does_not_establish_no_dependency",
            "by_version_definition": result, "reported_total": total if candidates else None}
