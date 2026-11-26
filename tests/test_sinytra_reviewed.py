"""Compare the checked-in bundle and its reopened SQLite evidence."""
import json
from pathlib import Path
import sqlite3
import sys
import unittest

DATA = Path(__file__).resolve().parents[1] / "data/minecraft-sinytra-reviewed.json"
CONNECTOR = "github:Sinytra/Connector"


class SinytraEvidence(unittest.TestCase):
    def test_exact_lanes_and_validation_scope(self):
        data = json.loads(DATA.read_text(encoding="utf-8"))
        rows = data["observations"]
        files = [json.loads(row["value"]) for row in rows if row["predicate"] == "project_release_evidence_v1"]
        self.assertEqual(len({x["file_id"] for x in files}), 11)
        expected = {"26.2": 4, "26.1.2": 4, "1.21.1": 3}
        self.assertEqual({lane: sum(x["target_game_version"] == lane for x in files) for lane in expected}, expected)
        self.assertEqual(sum(x["sha256"] is None for x in files), 1)
        for x in files:
            self.assertFalse(x["bytes_verified"] or x["runtime_tested_by_catalog"] or x["download_or_install_authorized"])
            self.assertEqual(x["host_loader"], "NeoForge")
        edges = {row["applicable_build"]: json.loads(row["value"])["relationships"]
                 for row in rows if row["entity_id"] == CONNECTOR and row["predicate"] == "mod.relationships.v1"}
        for lane, entries in edges.items():
            mapping = {x["target_id"]: x["relation"] for x in entries}
            self.assertEqual(mapping["github:Sinytra/ForgifiedFabricAPI"], "required")
            self.assertEqual(mapping["github:Sinytra/ConnectorExtras"], "optional")
            if lane == "1.21.1":
                self.assertNotIn("github:Sinytra/Launchpad", mapping)
                self.assertEqual(mapping["github:FabricMC/fabric"], "incompatible")
            else:
                self.assertEqual(mapping["github:Sinytra/Launchpad"], "required")
        self.assertEqual(set(edges), set(expected))

    def test_seeded_database_matches_original_observations(self):
        if DB is None:
            self.skipTest("No explicitly seeded database supplied")
        db = sqlite3.connect(Path(DB).resolve().as_uri() + "?mode=ro", uri=True)
        try:
            got = db.execute("SELECT id, value FROM observations ORDER BY id").fetchall()
            expected = json.loads(DATA.read_text(encoding="utf-8"))["observations"]
            self.assertEqual(got, sorted((r["id"], r["value"]) for r in expected))
            self.assertEqual(db.execute("PRAGMA integrity_check").fetchone(), ("ok",))
        finally:
            db.close()


if __name__ == "__main__":
    DB = sys.argv.pop(1)
    unittest.main(verbosity=2)
else:
    DB = None
