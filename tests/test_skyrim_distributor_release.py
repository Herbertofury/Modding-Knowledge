import json
import os
from pathlib import Path
import sqlite3
import unittest

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = json.loads((ROOT / "data/skyrim-distributor-release-reviewed.json").read_text(encoding="utf-8"))
ROWS = [(row, json.loads(row["value"])) for row in BUNDLE["observations"]]


class SkyrimDistributorEvidence(unittest.TestCase):
    def test_archives_have_unique_identity_not_just_names(self):
        evidence = [v for o, v in ROWS if o["predicate"] == "project_release_evidence_v1"]
        files = [f for r in evidence for f in r["files"]]
        self.assertEqual(len(evidence), 16)
        self.assertEqual(len({r["provider_release_id"] for r in evidence}), 16)
        self.assertEqual(len(files), 16)
        self.assertEqual(len({f["provider_asset_id"] for f in files}), 16)
        self.assertLess(len({f["name"] for f in files}), 16)
        self.assertEqual(sum(f["provider_sha256"] is None for f in files), 3)
        for record in evidence:
            self.assertFalse(record["install_authorized"])
            self.assertFalse(record["runtime_tested_by_catalog"])
        for f in files:
            self.assertTrue(f["official_download_url"].startswith("https://github.com/powerof3/"))
            self.assertFalse(f["archive_bytes_verified"])

    def test_desktop_and_vr_dependencies_are_conditional(self):
        relations = [v for o, v in ROWS if o["predicate"] == "mod.relationships.v1"]
        self.assertEqual(len(relations), 4)
        for r in relations:
            edges = r["relationships"]
            self.assertEqual([e["target_scope"]["game_id"] for e in edges], ["skyrim-se", "skyrim-vr"])
            self.assertIn("not a requirement for the desktop", edges[1]["condition"])
            self.assertEqual(r["coverage"]["required"], "partial")
            self.assertEqual({e["relation"] for e in edges}, {"required"})
        runtime = [v for o, v in ROWS if o["predicate"] == "distributor_source_runtime_scope_v1"]
        self.assertEqual(sum(r["old_binary_guard"] is not None for r in runtime), 3)
        self.assertTrue(all(r["exact_release_binary_runtime"] is None for r in runtime))
        self.assertTrue(all(r["ae_source_constant_value"] == "1.7.99.0" for r in runtime))
        self.assertFalse(any(o["predicate"] == "runtime_match_v1" for o, _ in ROWS))

    @unittest.skipUnless(os.environ.get("SKYRIM_DISTRIBUTOR_SQLITE"), "requires native output")
    def test_native_readonly_checksum_lineage_and_exact_observations(self):
        file = Path(os.environ["SKYRIM_DISTRIBUTOR_SQLITE"]).resolve()
        exact = dict((row["id"], row["value"]) for row, _ in ROWS)
        with sqlite3.connect(file.as_uri() + "?mode=ro", uri=True) as conn:
            self.assertEqual(conn.execute("PRAGMA integrity_check").fetchone(), ("ok",))
            self.assertEqual(conn.execute("PRAGMA foreign_key_check").fetchall(), [])
            self.assertEqual(conn.execute("SELECT count(*) FROM entities").fetchone(), (7,))
            self.assertEqual(dict(conn.execute("SELECT id,value FROM observations")), exact)
