from pathlib import Path
from base64 import b64encode
from hashlib import sha256
import re
import json
import os
import sqlite3
import unittest

ROOT = Path(__file__).resolve().parents[1]


class ReviewedFormList(unittest.TestCase):
    def setUp(self):
        self.bundle = json.loads((ROOT / "data/skyrim-flm-reviewed.json").read_text(encoding="utf-8"))

    def test_distinct_versions_targets_and_no_binary_claim(self):
        by = {}
        for row in self.bundle["observations"]:
            value = json.loads(row["value"])
            self.assertEqual(row["completeness"], "partial")
            for key in ("runtime_tested_by_catalog", "exact_nexus_file_verified", "automatic_install_authorized"):
                self.assertIs(value[key], False)
            by[(row["entity_id"], row["predicate"])] = value
        original = by["nexus:skyrim-se:project:74037", "publisher_project_identity_v1"]
        self.assertEqual(original["project_display_version"], "1.8.1")
        self.assertEqual(by["github:MaskedRPGFan/FormList-Manipulator", "source_build_identity_v1"]["source_version"], "1.7.0.0")
        self.assertEqual(by["nexus:skyrim-se:project:74037", "publisher_optional_integration_v1"]["relation"], "works_with")
        for project, creator in [(189947, "drLemis"), (193349, "abcd99")]:
            alternative = by[f"nexus:skyrim-se:project:{project}", "publisher_alternative_v1"]
            self.assertEqual(alternative["creator"], creator)
            self.assertIsNone(alternative["exact_file_id"])
            self.assertEqual(alternative["original_project"], "nexus:skyrim-se:project:74037")

    def test_rendered_offline_rows_and_runtime_claims(self):
        path = Path(os.environ.get("SKYRIM_FLM_HTML", ROOT / "Open FLM Evidence.html"))
        html = path.read_bytes().decode("utf-8")
        raw = re.search('<script type="application/json" id="data">(.*?)</script>', html, re.S)
        script = re.search('<script>(.*?)</script>', html, re.S)
        self.assertIsNotNone(raw)
        self.assertIsNotNone(script)
        payload = json.loads(raw.group(1))
        self.assertEqual(len(payload["rows"]), 9)
        self.assertEqual(len([r for r in payload["rows"] if r["source"] == "flm-creator-source"]), 4)
        self.assertTrue(all(r["fact"]["exact_nexus_file_verified"] is False for r in payload["rows"]))
        self.assertIn("sha256-" + b64encode(sha256(script.group(1).encode()).digest()).decode(), html)
        self.assertIn("default-src 'none'", html)

    def test_native_reopen_matches_all_provenance_and_fact_bytes(self):
        db = Path(os.environ.get("SKYRIM_FLM_SQLITE", ROOT / "skyrim-flm.sqlite"))
        self.assertTrue(db.is_file())
        with sqlite3.connect(db.resolve().as_uri() + "?mode=ro", uri=True) as conn:
            self.assertEqual(conn.execute("pragma integrity_check").fetchone(), ("ok",))
            self.assertEqual(conn.execute("select count(*) from entities").fetchone(), (9,))
            columns = ("id", "entity_id", "source_id", "source_url", "predicate", "value", "observed_unix", "applicable_build", "mod_version", "completeness", "policy_revision")
            actual = sorted(conn.execute("select " + ",".join(columns) + " from observations"))
            expected = sorted(tuple(row[col] for col in columns) for row in self.bundle["observations"])
            self.assertEqual(actual, expected)


if __name__ == "__main__":
    unittest.main()
