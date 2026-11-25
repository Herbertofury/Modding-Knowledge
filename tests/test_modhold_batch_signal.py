import importlib.util
from pathlib import Path
import subprocess
import sys
import unittest

EXAMPLE = Path(__file__).resolve().parents[1] / "examples" / "research" / "modhold_batch_signal" / "classify.py"
SPEC = importlib.util.spec_from_file_location("modhold_batch_signal_example", EXAMPLE)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
compare = MODULE.compare

def row(ident=12, version='1.0', files=None, **extra):
    return {'id': ident, 'version': version, 'updated': '2026-11-20 12:15:06',
            'files': [{'id': 20, 'kind': 'main', 'version': version}] if files is None else files,
            **extra}

class Contract(unittest.TestCase):
    def test_exact_signal_ignores_metrics_and_reorder_without_claiming_full_equality(self):
        r = row(files=[{'id': 20, 'kind': 'main', 'version': '1'}, {'id': 10, 'kind': 'optional', 'version': '2'}])
        incoming = {**r, 'files': list(reversed(r['files'])), 'views': 500, 'author': 'Changed'}
        self.assertEqual(compare([r], {'mods': {'12': incoming}}), [{
            'id': 12, 'status': 'same_version_signal', 'requires_full_detail_for_update_verdict': True}])

    def test_file_id_change_at_same_label_is_a_signal(self):
        previous = row()
        incoming = row(files=[{'id': 21, 'kind': 'main', 'version': '1.0'}])
        self.assertEqual(compare([previous], {'mods': {'12': incoming}})[0]['status'], 'version_signal_changed')

    def test_absent_and_malformed_are_not_removals_or_unchanged(self):
        statuses = compare([row(), row(13), row(14)], {'mods': {'12': row(), '14': {'version': '1'}}})
        self.assertEqual([x['status'] for x in statuses], ['same_version_signal', 'not_returned', 'invalid_row'])
        self.assertTrue(all(x['requires_full_detail_for_update_verdict'] for x in statuses))

    def test_duplicate_files_and_legacy_missing_files_fail_separately(self):
        old = row()
        incoming = row(files=old['files'] + old['files'])
        self.assertEqual(compare([old], {'mods': {'12': incoming}})[0]['status'], 'invalid_row')
        incoming.pop('files')
        self.assertEqual(compare([old], {'mods': {'12': incoming}})[0]['status'], 'invalid_row')

    def test_rejects_bad_project_scope_and_limit(self):
        for p, r in [([row()], {'mods': {'13': row(13)}}), ([row(True)], {'mods': {}}),
                     ([row(), row()], {'mods': {}}), ([row(i) for i in range(1,102)], {'mods': {}})]:
            with self.subTest(), self.assertRaises(ValueError): compare(p, r)

    def test_cli_refuses_duplicate_json_keys(self):
        result = subprocess.run([sys.executable, str(EXAMPLE)],
                                text=True, input='{"previous":[],"stats":{"mods":{},"mods":{}}}', capture_output=True)
        self.assertEqual(result.returncode, 2)
        self.assertFalse(result.stdout)
        self.assertIn('duplicate JSON key', result.stderr)

if __name__ == '__main__':
    unittest.main()
