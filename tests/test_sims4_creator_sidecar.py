import json
import os
from pathlib import Path
import sqlite3
import unittest

BUNDLE = json.loads((Path(__file__).resolve().parents[1] /
                     "data/sims4-frankk-simsonian-reviewed.json").read_text(encoding="utf-8"))


class CreatorSidecar(unittest.TestCase):
    def value(self, key, predicate):
        rows = [r for r in BUNDLE["observations"]
                if r["entity_id"] == key and r["predicate"] == predicate]
        self.assertEqual(len(rows), 1)
        return json.loads(rows[0]["value"])

    def test_source_coverage_and_canonical_links(self):
        e = BUNDLE["entities"]
        self.assertEqual(len(e), 48)
        self.assertEqual(len({x["id"] for x in e}), 48)
        self.assertEqual(sum(x["id"].startswith("frankk:project:") for x in e), 14)
        self.assertEqual(sum(x["id"].startswith("simsonian:project:") for x in e), 21)
        self.assertEqual(sum(x["kind"] == "tool" for x in e), 7)
        self.assertNotIn("frankk:project:simway", {x["id"] for x in e})
        for x in e:
            if x["kind"] == "mod":
                value = self.value(x["id"], "mod.relationships.v1")
                self.assertEqual(value["review"], "reviewed")
                self.assertNotIn("complete", value["coverage"].values())
                loc = self.value(x["id"], "publishing_locations")
                self.assertTrue(loc["project_url"].startswith("https://"))
                for route in loc.get("routes", []):
                    self.assertFalse(route["automatic_download"])
                    if route["provider"] == "Google Drive":
                        self.assertIsNone(route["landing_page"])
                        self.assertIsNone(route["legacy_http"])
                    self.assertEqual(route["file_bytes_identity"], "unverified")
                    self.assertTrue(route["landing_page"] is None or
                                    route["landing_page"].startswith("https://"))
        self.assertTrue(all(r["applicable_build"] == "unverified" for r in BUNDLE["observations"]))

    def test_requirements_keep_real_conditions_and_relationship_types(self):
        def edges(slug):
            return self.value("simsonian:project:"+slug, "mod.relationships.v1")["relationships"]

        kit = edges("archaeology-kit")
        spell = next(e for e in kit if e["target_id"] == "simsonian:project:spellbook-injector")
        self.assertEqual(spell["relation"], "required")
        self.assertIn("Only for", spell["condition"])
        whole = next(e for e in edges("classic-rpg-spells") if e["target_id"] == spell["target_id"])
        self.assertIsNone(whole["condition"])
        conflict = next(e for e in kit if e["target_id"] == "curseforge:project:670837")
        self.assertEqual(conflict["relation"], "incompatible")
        candy = next(e for e in edges("candy-factory") if e["target_id"])
        self.assertEqual(candy["relation"], "works_with")
        pirate = edges("pirate-treasure")
        self.assertEqual({e["label"] for e in pirate}, {"XML Injector", "Island Living (EA pack)"})
        self.assertTrue(all(e["relation"] == "required" for e in pirate))
        self.assertEqual(self.value("simsonian:project:archaeology-kit",
                                    "maintainer_release_evidence_v1")["version_label"], "2.0.13")
        status = self.value("frankk:project:festive-milestones", "maintainer_patch_status_v1")
        self.assertEqual(status["date"], "2026-02-19")
        self.assertFalse(status["current_build_verified"])
        self.assertEqual(status["status"], "minor_issues_reported")

    @unittest.skipUnless(os.environ.get("SIMS4_CREATOR_SQLITE"), "requires native seed")
    def test_native_output_integrity_and_exact_values(self):
        db = Path(os.environ["SIMS4_CREATOR_SQLITE"]).resolve()
        with sqlite3.connect(db.as_uri()+"?mode=ro", uri=True) as c:
            self.assertEqual(c.execute("PRAGMA integrity_check").fetchone(), ("ok",))
            self.assertEqual(c.execute("PRAGMA foreign_key_check").fetchall(), [])
            self.assertEqual(c.execute("SELECT count(*) FROM entities").fetchone(), (48,))
            truth = {r["id"]: r["value"] for r in BUNDLE["observations"]}
            self.assertEqual(dict(c.execute("SELECT id,value FROM observations")), truth)
