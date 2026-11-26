import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from review_skyrim_kid_layout import MAX_FILE, review, scan_text


class KeywordItemReview(unittest.TestCase):
    def test_nonterminal_marker_root_only_case_and_section_behavior(self):
        with tempfile.TemporaryDirectory(prefix="KID staging ") as d:
            p = Path(d)
            (p / "P_KID-optional.ini").write_text(
                "Keyword = 0x111~Present.esp|Armor|Watched.esl+0x2~-bad.esm, -Absent.esm, *Image.esp\n"
                "Keyword = 0x90~Present.esp|Weapon|0x22~Quest.esm\n"
                "ExclusiveGroup = Spells|0x7~Group.esl,-0x8~Hidden.esm\n"
                "[named]\nKeyword = 0x112~NamedOnly.esm|Armor|0x7~Hidden.esm\n", encoding="utf-8",
            )
            (p / "Present.esp").write_bytes(b"synthetic only")
            (p / "UpperExtension_KID.INI").write_text("Keyword=Anything|Armor", encoding="utf-8")
            (p / "LowerMarker_kid.ini").write_text("Keyword=Anything|Armor", encoding="utf-8")
            (p / "Child").mkdir()
            (p / "Child/Ignore_KID.ini").write_text("Keyword=Hidden|Armor", encoding="utf-8")
            data = review(p)
            self.assertEqual(data["file_states"], {"sampled-utf8": 1})
            self.assertEqual(data["not_candidates"], {"case-mismatch": 2})
            self.assertEqual(data["sample"]["nonterminal-marker-filenames"], 1)
            self.assertEqual(data["sample"]["named-section-entry"], 1)
            refs = data["files"][0]["references"]
            self.assertEqual([r["plugin"] for r in refs], [
                "Present.esp", "Watched.esl", "-bad.esm", "Absent.esm",
                "Present.esp", "Quest.esm", "Group.esl", "Hidden.esm",
            ])
            self.assertEqual([r["role"] for r in refs[:4]], ["keyword-form", "all-filter", "all-filter", "any-filter"])
            self.assertEqual([r["and_group"] for r in refs[:4]], [None, 1, 1, None])
            self.assertEqual(refs[3]["polarity"], "excluded")
            self.assertEqual(refs[-1]["polarity"], "excluded")
            self.assertEqual(refs[0]["staging_presence"], "name-seen")
            self.assertEqual(refs[1]["staging_presence"], "not-seen-in-selected-root")
            scope = data["scope"]
            self.assertIn("No runtime", scope)
            self.assertIn("ordering winner", scope)

    def test_limit_and_redaction_and_no_changes(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)
            (p / "Heavy_KID.ini").write_bytes(b" " * (MAX_FILE+1))
            (p / "Folder_KID.ini").mkdir()
            hidden = p / "PrivateAlias_KID.ini"
            hidden.write_bytes(b"Keyword = 0xF~PrivatePlugin.esp|Armor\n")
            before = hidden.read_bytes()
            result = subprocess.run([sys.executable, "-B", str(ROOT / "tools/review_skyrim_kid_layout.py"), str(p), "--counts-only"],
                                    capture_output=True, text=True, check=True, timeout=10)
            values = json.loads(result.stdout)
            self.assertEqual(values["file_states"], {"not-a-plain-file": 1, "byte-budget": 1, "sampled-utf8": 1})
            self.assertIsNone(values["files"])
            self.assertNotIn("Private", result.stdout)
            self.assertNotIn(str(p), result.stdout)
            self.assertEqual(hidden.read_bytes(), before)

    def test_unsupported_syntax_and_caps(self):
        counts, rows, _ = scan_text("Keyword=<<<END\n0xFF~Noise.esp|Armor\nEND\n", set(), 20, 20)
        self.assertEqual(counts["unsupported-region"], 1)
        self.assertEqual(rows, [])
        with self.assertRaisesRegex(ValueError, "line cap"):
            scan_text("\n\n\n", set(), 1, 0)
        with self.assertRaisesRegex(ValueError, "reference cap"):
            scan_text("Keyword=0xFF~Seen.esp|Armor", set(), 20, 0)


if __name__ == "__main__":
    unittest.main()
