import json
from pathlib import Path
import sqlite3
import unittest
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "data/skyrim-se-model-swapper-reviewed.json"
ORIGINAL = "github:Thiago099/Model-Swapper"
PORT = "github:DeadOnKeyboard1/ModelSwapper_1.3_Skyrim_1.7.104.0-update"


class ModelSwapperEvidenceTest(unittest.TestCase):
    def setUp(self):
        self.pack = json.loads(PACK.read_text(encoding="utf-8"))
        self.rows = self.pack["observations"]

    def read(self, entity, predicate):
        rows = [r for r in self.rows if r["entity_id"] == entity and r["predicate"] == predicate]
        self.assertEqual(len(rows), 1)
        return json.loads(rows[0]["value"])

    def test_release_and_port_are_not_installable_source_archives(self):
        a = self.read(ORIGINAL, "reviewed.github_release_source")
        b = self.read(PORT, "reviewed.github_release_source")
        self.assertEqual(a["tag_commit"], "82ba10532ea8c588edea427a5df5ae81d6d94d6b")
        self.assertEqual((a["release_id"], a["tag"], a["attached_asset_count"]), ("191963781", "1.3", 0))
        self.assertFalse(a["distribution"]["source_archive_is_installable"])
        self.assertTrue(a["distribution"]["url"].endswith("/mods/136461"))
        self.assertEqual(b["github_releases_observed"], 0)
        self.assertIsNone(b["release_id"])
        self.assertIsNone(b["attached_asset_count"])
        self.assertTrue(all(not r["distribution"]["automatic_binary_download"] for r in (a, b)))
        self.assertTrue(all(r["mod_version"] == "unverified" for r in self.rows))

    def test_port_needs_original_assets_not_both_dlls_and_unknowns_remain_unknown(self):
        a = self.read(ORIGINAL, "mod.relationships.v1")
        b = self.read(PORT, "mod.relationships.v1")
        self.assertEqual(a["coverage"], {"required": "partial"})
        self.assertEqual(b["coverage"], {"required": "partial"})
        self.assertEqual(b["none_evidence"], {})
        edges = b["relationships"]
        self.assertEqual(len(edges), 3)
        self.assertEqual(edges[0]["target_id"], ORIGINAL)
        self.assertEqual(edges[0]["raw_relation"], "requires-original-assets")
        self.assertIn("not a requirement to install both DLLs", edges[0]["condition"])
        self.assertIn("2.3.1", edges[1]["condition"])
        self.assertTrue(all(edge["target_scope"]["file_id"] is None for edge in edges))

    def test_pack_round_trips_existing_sqlite_schema_without_publication(self):
        with sqlite3.connect(":memory:") as db:
            db.executescript((ROOT / "src/schema.sql").read_text(encoding="utf-8"))
            db.executemany("INSERT INTO games VALUES (?,?)", self.pack["games"].items())
            for sid, url in self.pack["sources"].items():
                # Test-only source registration; publish is explicitly false.
                db.execute("INSERT INTO sources VALUES (?,?,1,0,?)",
                           (sid, url, self.pack["source_policy_revisions"][sid]))
            columns = ("id", "game_id", "kind", "name")
            db.executemany("INSERT INTO entities VALUES (?,?,?,?)",
                           [tuple(e[k] for k in columns) for e in self.pack["entities"]])
            cols = ("id", "entity_id", "source_id", "source_url", "predicate", "value",
                    "observed_unix", "applicable_build", "mod_version", "completeness", "policy_revision")
            for row in self.rows:
                url = urlsplit(row["source_url"])
                self.assertEqual((url.scheme, url.netloc, url.query, url.fragment), ("https", "github.com", "", ""))
                self.assertEqual(row["policy_revision"], self.pack["source_policy_revisions"][row["source_id"]])
            values = sorted(tuple(row[k] for k in cols) for row in self.rows)
            db.executemany("INSERT INTO observations VALUES (?,?,?,?,?,?,?,?,?,?,?)", values)
            self.assertEqual(db.execute("SELECT * FROM observations ORDER BY id").fetchall(), values)
            self.assertEqual(db.execute("SELECT can_publish FROM sources").fetchone(), (0,))
            self.assertFalse(db.execute("PRAGMA foreign_key_check").fetchall())
        published = json.loads((ROOT / "data/reviewed-bundle.json").read_text(encoding="utf-8"))
        self.assertTrue(set(self.pack["sources"]).isdisjoint(published["sources"]))
        self.assertEqual(len(self.pack["entities"]), 4)
        self.assertEqual(len(self.rows), 11)
