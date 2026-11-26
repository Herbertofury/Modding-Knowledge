import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

TOOLS = Path(__file__).resolve().parent.parent / "tools"
sys.path.insert(0, str(TOOLS))
import review_skyrim_audio_layout as audio


class AudioLayoutTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)

    def put(self, name, value=""):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(value.encode() if isinstance(value, str) else value)
        return path

    def run_cli(self, *args):
        return subprocess.run([sys.executable, "-B", str(TOOLS / "review_skyrim_audio_layout.py"),
                               str(self.root), *args], capture_output=True, text=True, check=True)

    def test_exact_audio_scans_and_no_writes(self):
        self.put("Sounds_SRD.yaml", b"not parsed\xff")
        self.put("Sounds_SRD.json")
        self.put("Ambient.esp_SRD.jsonc")
        self.put("Wrong_SRD.yml")
        self.put("Wrong_srd.json")
        self.put("Wrong_MUS.INI")
        self.put("nested/Inner_MUS.ini", "[General]\nAny = 0x01~Hidden.esp")
        self.put("Track_MUS.ini", "[General]\nMUSVillage = 0x01~Needs.esp, 0x02~FOUND.esp")
        self.put("found.esp")
        before = {p.relative_to(self.root): p.read_bytes() for p in self.root.rglob("*") if p.is_file()}
        data = json.loads(self.run_cli().stdout)
        self.assertEqual(data["counts"], {
            "mtd-config-candidate": 1, "srd-general-name-candidate": 2,
            "srd-plugin-name-candidate": 1, "unsupported-suffix-or-case": 3,
            "reviewed-assignments": 1, "referenced-plugin-names-present-at-root": 1,
            "referenced-plugin-names-not-at-root": 1,
        })
        self.assertEqual(data["plugin_name_candidates"], [
            {"name": "FOUND.esp", "at-staging-root": True},
            {"name": "Needs.esp", "at-staging-root": False},
        ])
        self.assertEqual({p.relative_to(self.root): p.read_bytes() for p in self.root.rglob("*")
                          if p.is_file()}, before)

    def test_mixed_clear_and_mapping_competition_not_a_winner(self):
        self.put("01_MUS.ini", "[General]\nMUSFoo = 0x1~Music.esp\nMUSOther = 0x12~Music.esp\n"
                 "[Location]\nCity = Day\n[Region]\nCity = Night\nLand = Night")
        self.put("02_MUS.ini", "\ufeff[General]\nMUSFoo! = 0x21~Other.esl\nMUSOther = 0x22~Music.esp\n"
                 "[Location]\nCity = Night\n[Region]\nLand = Night")
        report = audio.inspect(self.root)
        self.assertEqual([(x["kind"], x["section"], x["target"]) for x in report["overlaps"]], [
            ("mixed-clear-and-append", "general", "MUSFoo"),
            ("competing-music-type-assignments", "location", "City"),
        ])
        self.assertEqual(report["overlaps"][0]["sites"],
                         [{"file": "01_MUS.ini", "line": 2}, {"file": "02_MUS.ini", "line": 2}])
        redacted = self.run_cli("--counts-only").stdout
        for detail in ("01_MUS.ini", "MUSFoo", "Music.esp", "City", str(self.root)):
            self.assertNotIn(detail, redacted)
        self.assertEqual(set(json.loads(redacted)), {"schema", "counts", "sources"})

    def test_partial_syntax_does_not_create_phantom_references(self):
        self.put("One_MUS.ini", "[General]\nA = 0x1~Real.esp\nB = 0x2 ~DoNotGuess.esp\nC = <<<DATA\n"
                 "[General]\nIgnored = 0x1~Hidden.esp\nDATA")
        self.put("Two_MUS.ini", b"\xff")
        report = audio.inspect(self.root)
        self.assertEqual([x["name"] for x in report["plugin_name_candidates"]], ["Real.esp"])
        for key in ("unsupported-syntax", "unsupported-track-assignment", "unreviewed-encoding"):
            self.assertEqual(report["counts"][key], 1)

    def test_nonmatching_links_not_followed_matching_entry_refused(self):
        outside = self.put("source.txt", "[General]\nA = 0x1~NeverRead.esp")
        alias = self.root / "alias.txt"
        try:
            alias.symlink_to(outside)
        except (OSError, NotImplementedError):
            self.skipTest("Windows runner does not allow creating symlinks.")
        self.assertFalse(audio.inspect(self.root)["candidates"])
        alias.rename(self.root / "alias_MUS.ini")
        with self.assertRaisesRegex(ValueError, "ordinary file"):
            audio.inspect(self.root)

    def test_directory_candidate_and_oversize_refuse(self):
        folder = self.root / "secret_MUS.ini"
        folder.mkdir()
        with self.assertRaisesRegex(ValueError, "ordinary file"):
            audio.inspect(self.root)
        folder.rmdir()
        self.put(folder.name, b"x" * (audio.MAX_FILE_BYTES + 1))
        with self.assertRaisesRegex(ValueError, "read budget"):
            audio.inspect(self.root)

    def test_total_rule_budget_applies_across_files(self):
        original = audio.MAX_RULES
        self.addCleanup(setattr, audio, "MAX_RULES", original)
        audio.MAX_RULES = 3
        for name in ("First", "Second"):
            self.put(name + "_MUS.ini", "[General]\nMUS1 = 0x01~One.esp, 0x02~Two.esp")
        with self.assertRaisesRegex(ValueError, "reference budget"):
            audio.inspect(self.root)


if __name__ == "__main__":
    unittest.main()
