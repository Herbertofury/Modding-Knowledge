import copy
import json
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from review_modhold_detail import LIMIT, project_detail


def fixture():
    return {
        "id": 6, "game": "skyrim-se",
        "page": "https://modholdmods.com/g/skyrim-se/mods/6/test-mod-",
        "version": "1.2.35", "updated": "2026-11-25 08:00:00",
        "description": "discarded raw description",
        "files": [
            {"id": 2937, "kind": "main", "version": "1.2.35", "size": 512, "sha256": "a"*64},
            {"id": 2938, "kind": "main", "version": "1.2.35", "size": 1024},
            {"id": 560, "kind": "optional", "version": "1.0.1", "size": 1, "sha256": "b"*64},
            {"id": 500, "kind": "future-label", "version": "1.2.35"},
        ],
        "requirement_list": [
            {"name": "Conditional support", "required": True, "note": "Only on certain runtimes",
             "url": "https://www.nexusmods.com/skyrimspecialedition/mods/106441", "nexus_mod_id": 106441},
            {"name": "Optional integration", "required": False, "modhold_id": 11,
             "url": "https://modholdmods.com/m/11"},
        ],
        "conflicts": [], "loadAfter": [],
    }


def report(obj):
    return project_detail(json.dumps(obj).encode(), 6, "skyrim-se")


class ModholdDetail(unittest.TestCase):
    def test_two_exact_main_files_survive_and_optional_never_becomes_main(self):
        r = report(fixture())
        self.assertEqual(r["matching_main_file_ids"], [2937, 2938])
        self.assertEqual([f["key"] for f in r["files"]], ["modhold:file:2937", "modhold:file:2938",
                                                         "modhold:file:560", "modhold:file:500"])
        self.assertEqual(r["files"][2]["version"], "1.0.1")
        self.assertFalse(r["files"][2]["bytes_verified"])
        self.assertFalse(r["files"][3]["role_known"])
        self.assertNotIn("description", r)
        self.assertEqual(r["file_version_evidence"], "corroborated")
        self.assertEqual(r["updated_provider_text"], "2026-11-25 08:00:00")

    def test_provider_required_never_resolves_runtime_or_absence(self):
        r = report(fixture())
        self.assertEqual([q["provider_required"] for q in r["recorded_requirements"]], [True, False])
        self.assertTrue(all(q["resolved_required"] is None for q in r["recorded_requirements"]))
        self.assertEqual(r["conflict_count_reported"], 0)
        obj = fixture()
        obj.pop("conflicts")
        obj.pop("requirement_list")
        obj["version"] = "without-matching-main"
        r = report(obj)
        self.assertEqual(r["file_version_evidence"], "unverified")
        self.assertIsNone(r["recorded_requirements"])
        self.assertIsNone(r["conflict_count_reported"])

    def test_mismatched_source_and_duplicate_files_refuse(self):
        for key, value in (("id", 13), ("game", "skyrim-vr"), ("id", True),
                           ("page", "https://example.com/g/skyrim-se/mods/6/test"),
                           ("page", "https://modholdmods.com/g/skyrim-se/mods/60/test")):
            with self.subTest(key=key, value=value):
                d = fixture()
                d[key] = value
                with self.assertRaises(ValueError):
                    report(d)
        d = fixture()
        d["files"].append(copy.deepcopy(d["files"][0]))
        with self.assertRaisesRegex(ValueError, "file identity"):
            report(d)

    def test_bad_json_oversize_and_wrong_required_flag_refuse(self):
        for data in (b'{"id":6,"id":13}', b'{"id":NaN}', b' '*(LIMIT+1), b'\xff'):
            with self.subTest(length=len(data)):
                with self.assertRaises(ValueError):
                    project_detail(data, 6, "skyrim-se")
        d = fixture()
        d["requirement_list"][0]["required"] = "false"
        with self.assertRaisesRegex(ValueError, "flag"):
            report(d)

    def test_cli_returns_bounded_result_and_never_echoes_invalid_input(self):
        cmd = [sys.executable, str(ROOT / "tools/review_modhold_detail.py"), "--id", "6", "--game", "skyrim-se"]
        ok = subprocess.run(cmd, input=json.dumps(fixture()).encode(), capture_output=True, check=True)
        self.assertEqual(json.loads(ok.stdout)["key"], "modhold:mod:6")
        no = subprocess.run(cmd, input=b"Do not echo secret fixture", capture_output=True, check=False)
        self.assertEqual(no.returncode, 2)
        self.assertFalse(no.stdout)
        self.assertNotIn(b"secret fixture", no.stderr)


if __name__ == "__main__":
    unittest.main()
