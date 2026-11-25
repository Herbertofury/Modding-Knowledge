import hashlib
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
import verify_packaged_snapshot as verifier


class VerifyPackagedSnapshot(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        root = Path(tmp.name)
        self.archive = root / 'catalog.sqlite.gz'
        self.receipt = root / 'receipt.json'
        self.manifest = root / 'manifest.json'
        shutil.copyfile(ROOT / 'artifacts/modding-knowledge-latest.sqlite.gz', self.archive)
        shutil.copyfile(ROOT / 'artifacts/latest-validation.json', self.receipt)
        shutil.copyfile(ROOT / 'data/reviewed-bundle.json', self.manifest)

    def verify(self):
        return verifier.verify_packaged(self.archive, self.receipt, self.manifest)

    def rewrite_receipt(self, **fields):
        record = json.loads(self.receipt.read_text(encoding='utf-8'))
        record.update(fields)
        self.receipt.write_text(json.dumps(record), encoding='utf-8')

    def test_verified_published_bytes_are_unchanged(self):
        before = {p: hashlib.sha256(p.read_bytes()).hexdigest() for p in
                  (self.archive, self.receipt, self.manifest)}
        result = self.verify()
        self.assertEqual(result['integrity'], 'ok')
        self.assertTrue(result['exact_manifest_equality'])
        self.assertEqual(result['compressed_sha256'], before[self.archive])
        self.assertEqual(before, {p: hashlib.sha256(p.read_bytes()).hexdigest() for p in before})

    def test_archive_and_manifest_hash_mismatch_reject_before_sqlite(self):
        old = self.archive.read_bytes()
        with patch.object(verifier, 'validate_snapshot') as validator:
            self.archive.write_bytes(old + b'extra')
            with self.assertRaisesRegex(ValueError, 'Compressed archive'):
                self.verify()
            self.archive.write_bytes(old)
            self.manifest.write_bytes(self.manifest.read_bytes() + b' ')
            with self.assertRaisesRegex(ValueError, 'Reviewed manifest'):
                self.verify()
            validator.assert_not_called()

    def test_inner_hash_and_decompression_limit_reject_before_sqlite(self):
        with patch.object(verifier, 'validate_snapshot') as validator:
            with patch.object(verifier, 'MAX_SNAPSHOT', 64):
                with self.assertRaisesRegex(ValueError, 'Decompressed snapshot exceeds'):
                    self.verify()
            self.rewrite_receipt(snapshot_sha256='0' * 64)
            with self.assertRaisesRegex(ValueError, 'Decompressed snapshot does not match'):
                self.verify()
            validator.assert_not_called()

    def test_inconsistent_counts_and_duplicate_receipt_are_rejected(self):
        self.rewrite_receipt(entities=0)
        with self.assertRaisesRegex(ValueError, 'receipt field: entities'):
            self.verify()
        dup = self.receipt.read_text(encoding='utf-8').replace('"entities": 0', '"entities": 1, "entities": 0')
        self.receipt.write_text(dup, encoding='utf-8')
        with self.assertRaisesRegex(ValueError, 'Duplicate field'):
            self.verify()


if __name__ == '__main__':
    unittest.main()
