"""Exercise an extracted portable manager using only a disposable exact public-row fixture."""
import argparse
from contextlib import closing
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def main(package):
    case = json.loads((ROOT / "tests/fixtures/kid_release_files.json").read_text(encoding="utf-8"))
    with tempfile.TemporaryDirectory(prefix="mk-manager-files-") as work:
        cwd = Path(work)
        with zipfile.ZipFile(package) as archive:
            binary = [member for member in archive.namelist()
                      if Path(member).name in {"knowledge-manager", "knowledge-manager.exe"}]
            assert len(binary) == 1
            exe = cwd / Path(binary[0]).name
            exe.write_bytes(archive.read(binary[0]))
            exe.chmod(0o700)
        db = cwd / "Exact Skyrim file facts.sqlite"
        with closing(sqlite3.connect(db)) as con, con:
            con.executescript((ROOT / "src/schema.sql").read_text(encoding="utf-8"))
            e = case["entity"]
            row = case["observations"][0]
            con.execute("insert into games values(?,?)",(e["game_id"],"Skyrim Special Edition"))
            con.execute("insert into entities values(?,?,?,?)",(e["id"], e["game_id"],e["kind"],e["name"]))
            con.execute("insert into sources values(?,?,?,?,?)",(row["source_id"],"https://github.com/powerof3",1,1,row["policy_revision"]))
            for r in case["observations"]:
                con.execute("insert into observations values(?,?,?,?,?,?,?,?,?,?,?)",
                    tuple(r[k] for k in ("id","entity_id","source_id","source_url","predicate","value","observed_unix","applicable_build","mod_version","completeness","policy_revision")))
        before = hashlib.sha256(db.read_bytes()).hexdigest()
        cli=[str(exe),str(db),"release-files",e["id"]]
        result=json.loads(subprocess.check_output(cli,cwd=cwd,encoding="utf-8",timeout=20))
        assert result["file_count"]==2
        assets=[file for release in result["releases"] for file in release["files"]]
        assert {f["provider_asset_id"] for f in assets}=={528411221,462208143}
        assert len({f["name"] for f in assets})==1
        assert all(f["official_download_url"] and not f["archive_bytes_verified"] for f in assets)
        page=subprocess.check_output(cli+["--html"],cwd=cwd,encoding="utf-8",timeout=20)
        assert page.count("Publisher download")==2 and "<script" not in page
        comparison = [str(exe), str(db), "compare-release-files", e["id"]]
        diff = json.loads(subprocess.check_output(
            comparison + ["v4.0.6", "v4.1.0"], cwd=cwd, encoding="utf-8", timeout=20))
        assert diff["reference"]["tag"] == "v4.0.6" and diff["compared"]["tag"] == "v4.1.0"
        assert diff["metadata_pairs"] == [{
            "reference_asset_id": 462208143, "compared_asset_id": 528411221,
            "same_provider_asset_id": False, "same_exact_filename": True,
            "reported_digest_observation": "reported_sha256_different",
        }]
        assert not diff["reference_without_metadata_pair"] and not diff["compared_without_metadata_pair"]
        missing = subprocess.run(comparison + ["v4.0.6", "v0.0.0"], cwd=cwd,
                                 capture_output=True, encoding="utf-8", timeout=20)
        assert missing.returncode != 0 and missing.stdout == ""
        assert hashlib.sha256(db.read_bytes()).hexdigest()==before
        assert set(p.name for p in cwd.iterdir())=={db.name,exe.name}
        print("Extracted native manager retained distinct KID asset IDs, compared exact tags with different reported digests, and left the database untouched.")


if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--package",type=Path,required=True)
    main(p.parse_args().package)
