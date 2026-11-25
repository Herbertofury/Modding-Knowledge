from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
from review_skyrim_flm_layout import report, MAX_FILE


class FormListLayout(unittest.TestCase):
    def test_source_folder_differences_and_overlap_without_precedence_claim(self):
        with tempfile.TemporaryDirectory() as tmp:
            d = Path(tmp)
            (d/'FLM').mkdir()
            (d/'Source_FLM.ini').write_bytes(b'Alias=Market | Targets\n')
            (d/'Unrelated.ini').write_bytes(b'Alias=NotRead|Another\n')
            (d/'FLM'/'Optional.ini').write_bytes(b';ignored\nalias=market | Different\n[Other]\nAlias=OutOfScope|Other\n')
            (d/'FLM'/'Upper.INI').write_bytes(b'Alias=Ignored|Other\n')
            (d/'FLM'/'FormListManipulator_DEBUG.ini').write_bytes(b'')
            (d/'FLM'/'Malformed.ini').write_bytes(b'\xff\xfe\x00')
            summary = report(d)
            self.assertEqual(summary['file_count'], 5)
            self.assertEqual(summary['file_states']['read-utf8'], 2, summary['file_states'])
            self.assertEqual(summary['possible_overlap_count'], 1)
            self.assertEqual(summary['possible_overlaps'][0]['locations'], [
                {'path':'Source_FLM.ini','line':1}, {'path':'FLM/Optional.ini','line':2}])
            self.assertEqual(summary['file_states']['extension-case-needs-review'], 1)
            self.assertEqual(summary['file_states']['unreadable-unsupported-or-changed'], 1)
            self.assertIn('not runtime order', summary['scope'])
            self.assertIn('without an explicit sort', summary['readme_source_difference'])
            safe = report(d, counts_only=True)
            self.assertIsNone(safe['possible_overlaps'])
            self.assertIsNone(safe['files'])
            self.assertEqual(safe['file_states'], summary['file_states'])

    def test_large_or_non_plain_candidate_refused_and_files_unchanged(self):
        with tempfile.TemporaryDirectory() as tmp:
            d = Path(tmp)
            source = d/'oversize_FLM.ini'
            old = b'x' * (MAX_FILE + 1)
            source.write_bytes(old)
            (d/'directory_FLM.ini').mkdir()
            result = report(d)
            self.assertEqual(result['file_states'], {'byte-budget':1,'not-a-plain-file':1})
            self.assertEqual(source.read_bytes(), old)
            self.assertEqual(result['possible_overlap_count'], 0)

    def test_symbolic_link_subfolder_is_never_opened(self):
        with tempfile.TemporaryDirectory() as tmp:
            d = Path(tmp)
            (d/'out').mkdir()
            try:
                (d/'FLM').symlink_to(d/'out', target_is_directory=True)
            except OSError:
                self.skipTest('fixture privilege to create symlink unavailable')
            with self.assertRaisesRegex(ValueError, 'real directory'):
                report(d)
