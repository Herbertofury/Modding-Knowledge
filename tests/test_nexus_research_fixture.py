import copy
import importlib.util
import json
from pathlib import Path
import unittest

EXAMPLE = Path(__file__).resolve().parents[1] / "examples/research/nexus_relationships"
SPEC = importlib.util.spec_from_file_location("nexus_research_examples", EXAMPLE / "nexus_fixture.py")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
candidate_grouping, raw_review = MODULE.candidate_grouping, MODULE.raw_review

F = json.loads((EXAMPLE / "synthetic_nexus_v3_relations.json").read_bytes())
RAW, BATCH = F["raw"], F["batch"]
A, B, UNRESOLVED = BATCH["requested"]


class NexusFixture(unittest.TestCase):
    def test_raw_groups_and_opaque_bound_order(self):
        raw = raw_review(200, A, RAW["body"])
        self.assertEqual(raw["source_version_id"], A)
        self.assertNotEqual(raw["source_version_id"], RAW["source_game_scoped_file_id"])
        self.assertEqual(raw["coverage"], "definitions_observed")
        defs = raw["version_definitions"]
        self.assertEqual(set(defs), {"def-A", "def-B"})
        self.assertEqual([a["target_group_id"] for a in defs["def-A"]], ["group-a", "group-b"])
        first, alt = defs["def-A"]
        self.assertEqual((first["min_version_id"], first["max_version_id"]), ("9900", "1001"))
        self.assertNotEqual(first["target_mod_id"], first["target_mod_game_scoped_id"])
        self.assertIsNone(alt["max_version_id"])
        self.assertEqual([k["dlc_id"] for k in raw["dlc_definitions"]["dlc-A"]],
                         ["game-expansion-A", "game-expansion-B"])
        self.assertEqual(len(raw["dlc_definitions"]["dlc-B"]), 1)

    def test_missing_alternatives_or_cross_group_bounds_fail_closed(self):
        for op in (
            lambda r: r.pop("dlc_dependency_definitions"),
            lambda r: r["dependency_definitions"][0]["ranges"].clear(),
            lambda r: r["dependency_definitions"][0]["ranges"][0].pop("max_version"),
            lambda r: r["dependency_definitions"][0]["ranges"][0]["max_version"]["file"].update(id="wrong"),
            lambda r: r["dependency_definitions"].append(r["dependency_definitions"][0]),
            lambda r: r["dependency_definitions"][0]["ranges"][0]["min_version"].update(id=9990),
        ):
            raw = copy.deepcopy(RAW["body"])
            op(raw)
            with self.assertRaises(ValueError):
                raw_review(200, A, raw)

    def test_recorded_empty_is_distinct_from_refusal_and_not_installer_safety(self):
        empty = {"dependency_definitions": [], "dlc_dependency_definitions": []}
        self.assertEqual(raw_review(200, A, empty)["coverage"], "empty_response_scope_only")
        for status in (401, 403, 404, 429, 451, 503):
            raw = raw_review(status, A, empty)
            self.assertEqual(raw["coverage"], "unavailable")
            self.assertIsNone(raw["version_definitions"])
            self.assertEqual(candidate_grouping(status, {A}, {})["coverage"], "unavailable")

    def test_materialized_rows_not_new_requirements_or_optional_role(self):
        page = candidate_grouping(200, set(BATCH["requested"]), BATCH["body"])
        self.assertEqual(page["coverage"], "candidate_page_only")
        self.assertEqual(page["reported_total"], 4)
        candidate_sets = page["by_version_definition"]
        self.assertEqual(set(candidate_sets), {(A, "def-A"), (B, "def-A")})
        self.assertNotIn((A, "def-B"), candidate_sets)
        self.assertFalse(any(source == UNRESOLVED for source, _ in candidate_sets))
        c0, c1 = candidate_sets[(A, "def-A")]
        self.assertEqual((c0["candidate_version_id"], c1["candidate_version_id"]), ("9900", "1001"))
        self.assertEqual(c1["category"], "optional")  # candidate file classification, not dependency role
        self.assertEqual(c1["mod_status"], "published")  # not an eligibility decision
        self.assertNotIn("role", c1)
        self.assertNotIn("eligible", c1)
        self.assertEqual(raw_review(200, A, RAW["body"])["coverage"], "definitions_observed")

    def test_position_preserved_verbatim_and_no_float_coercion(self):
        page = candidate_grouping(200, {A, B}, BATCH["body"])
        self.assertEqual(page["by_version_definition"][(A, "def-A")][1]["position"], "3.0200000000000000001")
        data = copy.deepcopy(BATCH["body"])
        data["data"]["candidates"][0]["position"] = 1.5
        with self.assertRaises(ValueError):
            candidate_grouping(200, {A, B}, data)

    def test_empty_page_not_truth_about_requirements_or_total(self):
        page = candidate_grouping(200, {A, B, UNRESOLVED}, F["empty_batch_page"])
        self.assertEqual(page["coverage"], "empty_does_not_establish_no_dependency")
        self.assertIsNone(page["reported_total"])

    def test_invalid_page_or_unknown_source_version_not_accepted(self):
        page = BATCH["body"]
        with self.assertRaises(ValueError):
            candidate_grouping(200, {B}, page)
        bad = copy.deepcopy(page)
        bad["meta"]["page_size"] = 2
        with self.assertRaises(ValueError):
            candidate_grouping(200, {A, B}, bad)
        bad = copy.deepcopy(page)
        bad["data"]["candidates"][1] = bad["data"]["candidates"][0]
        with self.assertRaises(ValueError):
            candidate_grouping(200, {A, B}, bad)


if __name__ == "__main__":
    unittest.main()
