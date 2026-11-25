import json
import os
from pathlib import Path
import sqlite3
import unittest

B = json.loads((Path(__file__).resolve().parents[1] /
                "data/hytale-file-channels-reviewed.json").read_text(encoding="utf-8"))
ROWS = [(o, json.loads(o["value"])) for o in B["observations"]]


class HytaleFileChannels(unittest.TestCase):
    def test_provider_and_file_ids_are_unique(self):
        self.assertEqual(len(B["entities"]), 24)
        files = [v for o, v in ROWS if o["predicate"] == "project_release_evidence_v1"]
        self.assertEqual(len(files), 17)
        self.assertEqual(len({(f["provider"], f["file_id"]) for f in files}), 17)
        for f in files:
            self.assertFalse(f["runtime_tested_by_catalog"])
            self.assertFalse(f["bytes_verified"])
            self.assertFalse(f["download_or_install_authorized"])
            if f["provider"] == "CurseForge":
                self.assertIsNone(f["sha256"])

    def test_prerelease_and_same_archive_name_do_not_collapse(self):
        files = {v["file_id"]:v for o, v in ROWS if o["predicate"] == "project_release_evidence_v1"}
        for stable, pre in [(9430272,9433095),(9430278,9433130)]:
            self.assertEqual(files[stable]["provider_game_tag"], "0.7")
            self.assertIsNone(files[stable]["target_claim"])
            self.assertEqual(files[pre]["provider_game_tag"], "0.8")
            self.assertEqual(files[pre]["displayed_file_channel"], "R")
            self.assertEqual(files[pre]["game_channel"], "pre-release")
            self.assertEqual(files[pre]["target_claim"], "0.8.0-pre.3")
        old,new=files[9261993],files[9486348]
        self.assertEqual(old["version_label"],new["version_label"])
        self.assertEqual(old["file_name"],new["file_name"])
        self.assertEqual({old["target_claim"],new["target_claim"]},{"0.7.2","0.7.6"})
        self.assertEqual(files[9343033]["target_claim_type"],"maintainer_targets_server_version")
        self.assertEqual(files[456135583]["game_channel"], "pre-release")
        self.assertEqual(files[456135584]["game_channel"], "release")
        links = [v for o,v in ROWS if o["predicate"] == "file_dependency_listing_v1"]
        self.assertEqual(len(links), 4)
        self.assertTrue(all(not v["global_no_dependencies"] and
                            v["required_runtime_coverage"] == "unknown" for v in links))

    @unittest.skipUnless(os.environ.get("HYTALE_CHANNEL_SQLITE"), "requires native output")
    def test_native_reopen_and_exact_manifest(self):
        db = Path(os.environ["HYTALE_CHANNEL_SQLITE"]).resolve()
        with sqlite3.connect(db.as_uri()+"?mode=ro", uri=True) as c:
            self.assertEqual(c.execute("PRAGMA integrity_check").fetchone(), ("ok",))
            self.assertEqual(c.execute("PRAGMA foreign_key_check").fetchall(), [])
            self.assertEqual(c.execute("SELECT id FROM games").fetchall(), [("hytale",)])
            self.assertEqual(c.execute("SELECT count(*) FROM entities").fetchone(), (24,))
            self.assertEqual(dict(c.execute("SELECT id,value FROM observations")),
                             {v["id"]:v["value"] for v in B["observations"]})
