import json
import os
from pathlib import Path
import sqlite3
import unittest

P = Path(__file__).resolve().parents[1] / "data/skyrim-nasuno-smp-reviewed.json"
B = json.loads(P.read_text(encoding="utf-8"))
rows = [(r, json.loads(r["value"])) for r in B["observations"]]


def value(project, predicate):
    return next(v for r, v in rows if r["entity_id"].endswith(":" + project) and r["predicate"] == predicate)


class NasunoIndex(unittest.TestCase):
    def test_exact_project_coverage_attribution_and_manual_access(self):
        self.assertEqual(sum(r["predicate"] == "publisher_variants_v1" for r, _ in rows), 8)
        m = value("mages-daily", "reviewed.project_attribution")
        self.assertEqual(m["credits"], "Suna203 and nasuno04")
        self.assertEqual(value("mages-daily", "publisher_variants_v1")["download_route"]["provider_project_url"],
                         "https://www.nexusmods.com/skyrimspecialedition/mods/155656")
        self.assertTrue(any(e["relation"] == "required" and "main file" in e["label"] for e in
                            value("mages-daily", "mod.relationships.v1")["relationships"]))
        for r, v in rows:
            if r["predicate"] != "publisher_variants_v1":
                continue
            self.assertFalse(v["runtime_tested_by_catalog"] or v["rights_verified"])
            self.assertTrue(v["download_route"]["automation"].startswith("manual"))
            self.assertTrue(all(not x["access_verified"] and x["exact_file_id"] is None for x in v["variants"]))
        self.assertNotIn("https://drive.google.com", P.read_text(encoding="utf-8"))

    def test_renderer_runtime_and_bhunp_versions_do_not_bleed_across_files(self):
        nocturnal = value("vanilla-nocturnal", "publisher_variants_v1")["variants"]
        bhunp = [v for v in nocturnal if "BHUNP" in v["body"]]
        self.assertEqual([(r["version"], r["channel"]) for r in bhunp], [("0-2", "legacy")])
        old = value("backsteppo-french-maid", "publisher_variants_v1")["variants"]
        self.assertEqual([x["body"] for x in old if x["channel"] == "legacy"], ["BHUNP", "COCOv6"])
        no_patch = next(e for e in value("vanilla-nocturnal", "mod.relationships.v1")["relationships"] if e["label"] == "PGPatcher")
        self.assertIn("CS/PBR variant", no_patch["condition"])
        self.assertIn("explicitly excludes LE and VR", value("salahairs", "publisher_variants_v1")["scope"])
        self.assertIn("tentatively", value("kanrax2-sleeveless", "publisher_variants_v1")["scope"])
        self.assertIn("works on SE/AE/VR", value("vanilla-warlock", "publisher_variants_v1")["scope"])
        options = value("dc-sorceress", "mod.relationships.v1")["relationships"]
        self.assertEqual(len([r for r in options if r["raw_relation"] == "one-of" and r["relation"] == "required"]), 1)

    @unittest.skipUnless(os.environ.get("SKYRIM_NASUNO_SQLITE"), "requires native output")
    def test_native_read_only_snapshot(self):
        p = Path(os.environ["SKYRIM_NASUNO_SQLITE"]).resolve()
        with sqlite3.connect(p.as_uri()+"?mode=ro", uri=True) as c:
            self.assertEqual(c.execute("PRAGMA integrity_check").fetchone(), ("ok",))
            self.assertEqual(c.execute("PRAGMA foreign_key_check").fetchall(), [])
            self.assertEqual(c.execute("SELECT count(*) FROM entities").fetchone(), (16,))
            self.assertEqual(dict(c.execute("SELECT id,value FROM observations")),
                             {r["id"]:r["value"] for r,_ in rows})
