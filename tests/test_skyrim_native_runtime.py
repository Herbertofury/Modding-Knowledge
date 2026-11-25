import json
import os
from pathlib import Path
import sqlite3
import unittest

B = json.loads((Path(__file__).resolve().parents[1] /
                "data/skyrim-native-runtime-reviewed.json").read_text(encoding="utf-8"))
ROWS = [(v, json.loads(v["value"])) for v in B["observations"]]


def records(entity, predicate):
    return [v for o, v in ROWS if o["entity_id"] == entity and o["predicate"] == predicate]


class NativeRuntime(unittest.TestCase):
    def test_same_skse_label_is_not_runtime_identity(self):
        rows = records("silverlock:skse64", "runtime_match_v1")
        exact = {(v["storefront"],v["skse_version"],v["game_runtime"]) for v in rows}
        self.assertEqual(exact, {("Steam","2.3.1","1.7.104"),("GOG","2.2.6","1.6.1179"),
                                 ("Steam","2.2.6","1.6.1170"),("Legacy desktop","2.0.20","1.5.97")})
        self.assertEqual(len(records("silverlock:skse64", "storefront_support_v1")),2)
        self.assertEqual(records("silverlock:sksevr", "runtime_match_v1")[0]["game_runtime"], "1.4.15")
        self.assertIsNone(records("nexus:skyrim-se:project:32444", "release_scope_notice_v1")[0]["file_id"])

    def test_optional_scope_and_real_asset_identity(self):
        underwater="nexus:skyrim-se:project:187230"
        rel=records(underwater,"mod.relationships.v1")[0]["relationships"]
        menu=next(r for r in rel if r["label"]=="SKSE Menu Framework")
        self.assertEqual(menu["relation"],"optional")
        self.assertIn("not the core",menu["condition"])
        esl=next(r for r in rel if r["label"]=="Backported Extended ESL Support")
        self.assertIn("older than 1.6.1130",esl["condition"])
        scope=records(underwater,"maintainer_test_scope_v1")[0]
        self.assertEqual(scope["renderer"],"native")
        self.assertFalse(scope["independent_runtime_acceptance"])
        rows=[v for o,v in ROWS if o["predicate"]=="project_release_evidence_v1"]
        files=[f for x in rows for f in x["files"]]
        self.assertEqual(len(files),20)
        self.assertEqual(len({f["provider_asset_id"] for f in files}),20)
        self.assertTrue(all(not x["install_authorized"] and not x["runtime_tested_by_catalog"] for x in rows))
        self.assertTrue(all(not f["archive_bytes_verified"] for f in files))
        fixes=records("github:aers/EngineFixesSkyrim64","project_release_evidence_v1")[0]
        self.assertEqual(fixes["release_type"],"publisher beta label")
        self.assertFalse(fixes["provider_prerelease_flag"])
        cs=records("github:community-shaders/skyrim-community-shaders","project_release_evidence_v1")[0]
        self.assertEqual([f["role"] for f in cs["files"]].count("core"),1)
        self.assertEqual([f["role"] for f in cs["files"]].count("bundle-alternative"),1)

    @unittest.skipUnless(os.environ.get("SKYRIM_RUNTIME_SQLITE"),"requires native output")
    def test_exact_reopen_and_no_other_games(self):
        db=Path(os.environ["SKYRIM_RUNTIME_SQLITE"]).resolve()
        with sqlite3.connect(db.as_uri()+"?mode=ro",uri=True) as c:
            self.assertEqual(c.execute("PRAGMA integrity_check").fetchone(),("ok",))
            self.assertEqual(c.execute("PRAGMA foreign_key_check").fetchall(),[])
            self.assertEqual(c.execute("SELECT id FROM games ORDER BY id").fetchall(),
                             [("skyrim-se",),("skyrim-vr",)])
            self.assertEqual(c.execute("SELECT count(*) FROM entities").fetchone(),(11,))
            self.assertEqual(dict(c.execute("SELECT id,value FROM observations")),
                             {v["id"]:v["value"] for v in B["observations"]})
