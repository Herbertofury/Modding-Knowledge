"""Validate first-party CAS observation boundaries and exact reopened SQLite values."""
import json
from pathlib import Path
import sqlite3
import sys
import unittest
from urllib.parse import urlsplit

DATA = Path(__file__).resolve().parents[1] / 'data/sims4-cas-cc-reviewed.json'
DB = None


class CASEvidence(unittest.TestCase):
    def test_exact_posts_labels_and_relationship_scope(self):
        pack = json.loads(DATA.read_text(encoding='utf-8'))
        by_id = {e['id']: e for e in pack['entities']}
        rows = pack['observations']
        self.assertEqual(len(by_id), 9)
        self.assertEqual(len({r['id'] for r in rows}), 37)
        self.assertEqual(len({r['entity_id'] for r in rows if r['predicate'] == 'issue_count_coverage_v1'}), 7)
        for r in rows:
            self.assertIn(r['entity_id'], by_id)
            self.assertEqual(urlsplit(r['source_url']).netloc, 'www.patreon.com')
            self.assertEqual((r['applicable_build'], r['mod_version']), ('unverified', 'unverified'))
        files = [(row['entity_id'], json.loads(row['value'])) for row in rows if row['predicate'] == 'cas_attachment_label_v1']
        self.assertEqual(len(files), 13)
        self.assertEqual({pid: sum(row == pid for row, _ in files) for pid in {pid for pid, _ in files}},
                         {'patreon:post:93816776': 3, 'patreon:post:99261674': 5, 'patreon:post:94841540': 4, 'patreon:post:53944028': 1})
        self.assertEqual(sum(f['role'] == 'skin-overlay' for _, f in files), 4)
        self.assertEqual(sum(f['role'] == 'tray-bundle' for _, f in files), 3)
        self.assertTrue(all(f['target_folder'] == (None if f['role'] == 'tray-bundle' else 'Mods') for _, f in files))
        self.assertTrue(all(not any(f[key] for key in ('bytes_verified', 'provider_file_id', 'sha256', 'download_url')) for _, f in files))
        relations = [json.loads(r['value']) for r in rows if r['predicate'] == 'mod.relationships.v1']
        self.assertEqual(len(relations), 1)
        self.assertEqual(relations[0]['review'], 'candidate')
        self.assertNotIn('required', relations[0]['coverage'])
        edges = relations[0]['relationships']
        self.assertEqual({r['target_id'] for r in edges if r['relation'] == 'optional'},
                         {'patreon:post:53944028', 'patreon:post:55675247', 'patreon:post:56990147', 'patreon:post:54872952'})
        self.assertEqual([r['target_id'] for r in edges if r['relation'] == 'incompatible'], [None])

    def test_reopened_store_matches(self):
        if DB is None:
            self.skipTest('Supply a freshly seeded database to check native replay')
        pack = json.loads(DATA.read_text(encoding='utf-8'))
        with sqlite3.connect(Path(DB).resolve().as_uri() + '?mode=ro', uri=True) as conn:
            got = conn.execute('SELECT id, entity_id, source_url, predicate, value FROM observations ORDER BY id').fetchall()
            fields = ('id', 'entity_id', 'source_url', 'predicate', 'value')
            self.assertEqual(got, sorted(tuple(r[k] for k in fields) for r in pack['observations']))
            self.assertEqual(conn.execute('PRAGMA integrity_check').fetchone(), ('ok',))


if __name__ == '__main__':
    DB = sys.argv.pop(1)
    unittest.main(verbosity=2)
