import json
import os
from pathlib import Path
import sqlite3
import unittest

B = json.loads((Path(__file__).resolve().parents[1] /
                "data/cyberpunk2077-core-reviewed.json").read_text(encoding="utf-8"))


def observations(slug, predicate):
    return [json.loads(r["value"]) for r in B["observations"]
            if r["entity_id"] == "cyberpunk:project:" + slug and r["predicate"] == predicate]


class CyberpunkSidecar(unittest.TestCase):
    def test_release_and_asset_identity_remain_scoped(self):
        self.assertEqual(len(B["entities"]), 21)
        self.assertEqual(len(B["observations"]), 64)
        seen = set()
        for p in B["entities"]:
            for rec in observations(p["id"].rsplit(":", 1)[-1], "project_release_evidence_v1"):
                if p["kind"] == "creator":
                    continue
                self.assertFalse(rec["install_authorized"])
                self.assertFalse(rec["game_target_verified"])
                for f in rec["files"]:
                    self.assertNotIn(f["github_id"], seen)
                    seen.add(f["github_id"])
                    self.assertFalse(f["bytes_verified"])
                    self.assertRegex(f["sha256"], "^[0-9a-f]{64}$")
                    self.assertTrue(f["metadata_url"].endswith("/" + str(f["github_id"])))
                    if "pdb" in f["name"].lower():
                        self.assertEqual(f["role"], "debug-symbols")
        self.assertEqual(len(seen), 42)
        wk = {r["tag"]:r for r in observations("wolvenkit", "project_release_evidence_v1")}
        self.assertEqual(wk["9.1.1"]["channel"], "stable")
        self.assertEqual(wk["10.0.0-beta.1"]["channel"], "beta")
        nightly = wk["10.0.0-beta.2-nightly.2026-11-23"]
        self.assertEqual(nightly["channel"], "nightly")
        self.assertFalse(nightly["provider_prerelease_flag"])
        self.assertEqual(observations("wolvenkit", "maintainer_notice_v1")[0]["requires_wolvenkit_major"], 10)

    def test_mod_requirements_and_attributed_issue(self):
        flight = observations("let-there-be-flight", "mod.relationships.v1")[0]
        edges = {e["label"]: e for e in flight["relationships"]}
        self.assertEqual({key for key, e in edges.items() if e["relation"] == "required"},
                         {"RED4ext", "TweakXL", "redscript", "Input Loader"})
        self.assertEqual({key for key, e in edges.items() if e["relation"] == "optional"},
                         {"ArchiveXL", "Mod Settings"})
        self.assertIn("2.5", edges["ArchiveXL"]["condition"])
        self.assertEqual(edges["Input Loader"]["version_constraint"], ">=0.2.4")
        eq = observations("equipment-ex", "mod.relationships.v1")[0]
        archive = next(e for e in eq["relationships"] if e["label"] == "ArchiveXL")
        self.assertEqual(archive["relation"], "required")
        self.assertEqual(archive["version_constraint"], ">=1.28.7")
        notice = observations("let-there-be-flight", "maintainer_notice_v1")[0]
        self.assertEqual(notice["status"], "maintainer_relays_unreproduced_report")
        self.assertIsNone(notice["user_report_count"])
        self.assertFalse(observations("let-there-be-flight", "health_claim_v1"))
        self.assertEqual(observations("cet", "mod.relationships.v1")[0]["coverage"]["required"], "unknown")

    @unittest.skipUnless(os.environ.get("CYBERPUNK2077_SQLITE"), "requires native sidecar")
    def test_native_rows_match_and_do_not_merge(self):
        db = Path(os.environ["CYBERPUNK2077_SQLITE"]).resolve()
        with sqlite3.connect(db.as_uri() + "?mode=ro", uri=True) as c:
            self.assertEqual(c.execute("PRAGMA integrity_check").fetchone(), ("ok",))
            self.assertEqual(c.execute("PRAGMA foreign_key_check").fetchall(), [])
            self.assertEqual(c.execute("SELECT id FROM games").fetchall(), [("cyberpunk2077",)])
            self.assertEqual(c.execute("SELECT count(*) FROM entities").fetchone(), (21,))
            self.assertEqual(dict(c.execute("SELECT id,value FROM observations")),
                             {r["id"]:r["value"] for r in B["observations"]})
