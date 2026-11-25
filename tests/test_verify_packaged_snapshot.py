from contextlib import closing
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
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

    def verify(self, output=None):
        return verifier.verify_packaged(self.archive, self.receipt, self.manifest, output)

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

    def test_extract_exact_validated_bytes_to_a_fresh_queryable_database(self):
        output = self.archive.parent / 'offline catalogue.sqlite'
        result = self.verify(output)
        self.assertTrue(result['extracted_verified_copy'])
        self.assertEqual(output.read_bytes(), gzip.decompress(self.archive.read_bytes()))
        with closing(sqlite3.connect(output.as_uri() + '?mode=ro', uri=True)) as store:
            self.assertEqual(store.execute('SELECT count(*) FROM entities').fetchone()[0], result['entities'])
        self.assertFalse(self.verify()['extracted_verified_copy'])
        with self.assertRaises(FileExistsError):
            self.verify(output)

    def test_existing_sidecar_or_invalid_archive_leaves_destination_untouched(self):
        output = self.archive.parent / 'new.sqlite'
        sidecar = Path(str(output) + '-shm')
        sidecar.write_bytes(b'keep existing bytes')
        with patch.object(verifier, 'validate_snapshot') as validate:
            with self.assertRaises(FileExistsError):
                self.verify(output)
            validate.assert_not_called()
        self.assertEqual(sidecar.read_bytes(), b'keep existing bytes')
        sidecar.unlink()
        self.rewrite_receipt(compressed_sha256='0' * 64)
        with self.assertRaisesRegex(ValueError, 'Compressed archive'):
            self.verify(output)
        self.assertFalse(output.exists())
        self.assertEqual(list(output.parent.glob('knowledge-verify-*')), [])

    def test_parent_components_and_links_are_refused_before_extraction(self):
        root = self.archive.parent
        with self.assertRaisesRegex(ValueError, 'parent components'):
            self.verify(root / '..' / 'not-created.sqlite')
        linked = root / 'linked'
        try:
            linked.symlink_to(root, target_is_directory=True)
        except OSError:
            return  # This checkout cannot create a disposable directory symlink.
        with self.assertRaisesRegex(ValueError, 'non-linked'):
            self.verify(linked / 'not-created.sqlite')
        self.assertFalse((root / 'not-created.sqlite').exists())

    def test_a_concurrent_destination_cannot_be_replaced(self):
        output = self.archive.parent / 'race.sqlite'
        operation = 'rename' if os.name == 'nt' else 'link'
        original = getattr(verifier.os, operation)

        def racing_create(src, dst):
            output.write_bytes(b'existing writer')
            return original(src, dst)

        with patch.object(verifier.os, operation, side_effect=racing_create):
            with self.assertRaises(FileExistsError):
                self.verify(output)
        self.assertEqual(output.read_bytes(), b'existing writer')
        self.assertEqual(list(output.parent.glob('knowledge-verify-*')), [])


if __name__ == '__main__':
    unittest.main()
