"""Prove advisory read/export leaves unrelated save, sidecar, and CC bytes alone."""
import argparse
from contextlib import closing
import hashlib
import json
from pathlib import Path
import shutil
import sqlite3
from tempfile import TemporaryDirectory

from verify_lot51_snapshot import verify, render


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check(database, output):
    with TemporaryDirectory(prefix="lot51-disposable-") as tmp:
        temp=Path(tmp)
        workspace=temp/"catalog"
        profile=temp/"Example profile"; workspace.mkdir()
        example = {
            "saves/example.save": b"synthetic-save-do-not-touch",
            "Mods/Hotels/lot51_hotels.dat": b"synthetic-menu-presets",
            "Mods/Uniforms/lot51_fashion_authority_data/Sample.txt": b"synthetic-save-slot-threads",
            "Mods/CC/sample.package": b"synthetic-referenced-cc",
        }
        for name, blob in example.items():
            path=profile/name; path.parent.mkdir(parents=True,exist_ok=True); path.write_bytes(blob)
        copy=workspace/"catalog.sqlite"
        shutil.copyfile(database,copy)
        expected_db=digest(copy)
        entities, source=verify(copy)
        render(output,entities,source)
        page=Path(output).read_text(encoding="utf-8")
        for fragment in ["Before changing mods","lot51_hotels.dat","lot51_fashion_authority_data",
                         "https://lot51.cc/node/534","https://lot51.cc/node/710","https://lot51.cc/node/571"]:
            assert fragment in page,fragment
        assert page.count("<details>")==3
        assert digest(copy)==expected_db
        assert {x.relative_to(profile).as_posix():x.read_bytes()
                for x in profile.rglob("*") if x.is_file()}==example
        with closing(sqlite3.connect(copy)) as db, db:
            predicate="maintenance_kitchen_presets_v1"
            value=json.loads(db.execute("SELECT value FROM observations WHERE predicate=?",(predicate,)).fetchone()[0])
            value["preserve"][0]["scope"]="game_save"
            db.execute("UPDATE observations SET value=? WHERE predicate=?",(json.dumps(value),predicate))
        try:
            verify(copy)
        except ValueError as exc:
            assert "Exact observation comparison" in str(exc)
        else:
            raise AssertionError("An advisory that conflates the file with the game save was accepted")
    print("Three isolated advisory scopes, read-only export, untouched mock player files, modified record rejected.")


if __name__ == "__main__":
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("database");p.add_argument("html")
    a=p.parse_args()
    check(Path(a.database).resolve(strict=True),Path(a.html))
