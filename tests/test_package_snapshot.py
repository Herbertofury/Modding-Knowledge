import gzip
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
import package_snapshot as pack


class PackagedSnapshotTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.bytes = gzip.decompress((ROOT / 'artifacts' / pack.ARTIFACT).read_bytes())
        cls.manifest = ROOT / 'data/reviewed-bundle.json'

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        self.source = self.root / 'source.sqlite'
        self.source.write_bytes(self.bytes)
        self.artifact = self.root / pack.ARTIFACT
        self.receipt = self.root / 'latest-validation.json'

    def package(self):
        return pack.package_snapshot(self.source, self.manifest, self.root)

    def test_original_replacement_cannot_change_validated_publication(self):
        validator = pack.validate_snapshot
        def replace_original(captured, manifest):
            self.assertNotEqual(captured, self.source)
            self.source.write_bytes(b'new unreviewed source bytes')
            return validator(captured, manifest)
        with patch.object(pack, 'validate_snapshot', side_effect=replace_original):
            report = self.package()
        self.assertEqual(self.source.read_bytes(), b'new unreviewed source bytes')
        self.assertEqual(self.artifact.read_bytes(), gzip.compress(self.bytes, mtime=0))
        self.assertEqual(report['snapshot_sha256'], hashlib.sha256(self.bytes).hexdigest())
        self.assertEqual(report['compressed_sha256'], hashlib.sha256(self.artifact.read_bytes()).hexdigest())
        self.assertEqual(json.loads(self.receipt.read_text()), report)

    def test_sidecars_and_wal_header_refuse_without_replacing_previous_pair(self):
        self.artifact.write_bytes(b'last artifact')
        self.receipt.write_bytes(b'last receipt')
        for suffix in pack.SIDECARS:
            with self.subTest(suffix=suffix):
                sidecar = Path(str(self.source) + suffix)
                sidecar.write_bytes(b'kept')
                with self.assertRaisesRegex(ValueError, 'standalone'):
                    self.package()
                self.assertEqual(sidecar.read_bytes(), b'kept')
                sidecar.unlink()
        self.source.write_bytes(self.bytes[:18] + b'\x02\x02' + self.bytes[20:])
        with self.assertRaisesRegex(ValueError, 'standalone'):
            self.package()
        self.assertEqual(self.artifact.read_bytes(), b'last artifact')
        self.assertEqual(self.receipt.read_bytes(), b'last receipt')

    def test_bad_validation_digest_or_initial_publication_error_preserves_pair(self):
        self.artifact.write_bytes(b'last artifact')
        self.receipt.write_bytes(b'last receipt')
        with patch.object(pack, 'validate_snapshot', return_value={'snapshot_sha256':'wrong'}):
            with self.assertRaisesRegex(ValueError, 'does not match'):
                self.package()
        with patch.object(pack.os, 'replace', side_effect=OSError('unavailable')):
            with self.assertRaisesRegex(OSError, 'unavailable'):
                self.package()
        self.assertEqual(self.artifact.read_bytes(), b'last artifact')
        self.assertEqual(self.receipt.read_bytes(), b'last receipt')
        self.assertEqual(list(self.root.glob('.knowledge-snapshot-*')), [])


if __name__ == '__main__':
    unittest.main()
