import base64
import hashlib
from html.parser import HTMLParser
import json
import os
from pathlib import Path
import sqlite3
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = json.loads((ROOT / 'data/skyrim-fsmp-reviewed.json').read_text(encoding='utf-8'))
DATA = [(o, json.loads(o['value'])) for o in BUNDLE['observations']]
sys.path.insert(0, str(ROOT / "tools"))
from build_skyrim_fsmp_catalog import build, SCRIPT, link

def select(predicate):
    return [value for row, value in DATA if row['predicate'] == predicate]

class SkyrimFsmpEvidence(unittest.TestCase):
    def test_publisher_archive_identity_does_not_authorize_install(self):
        releases = select('project_release_evidence_v1')
        self.assertEqual(len(releases), 5)
        self.assertEqual(sum(r['provider_prerelease_flag'] for r in releases), 1)
        newest, = (r for r in releases if r['publisher_latest_endpoint_match'])
        self.assertEqual((newest['provider_release_id'], newest['tag']), (377386950, 'v4.1.1'))
        self.assertEqual(newest['files'][0]['provider_asset_id'], 531271125)
        self.assertEqual(len({f['provider_asset_id'] for r in releases for f in r['files']}), 5)
        for row in releases:
            self.assertFalse(row['runtime_tested_by_catalog'])
            self.assertFalse(row['install_authorized'])
            for file in row['files']:
                self.assertFalse(file['archive_bytes_verified'])
                self.assertRegex(file['provider_sha256'], r'^[a-f0-9]{64}$')
                self.assertTrue(file['official_download_url'].startswith(
                    'https://github.com/DaymareOn/hdtSMP64/releases/download/'+row['tag']+'/'))

    def test_source_only_flags_and_conditional_legacy_plugin(self):
        source, = select('fomod_source_selection_evidence_v1')
        variants, legacy = source['groups']
        self.assertEqual([r['flag'] for r in variants['options']], ['NOAVX','AVX','AVX2','AVX512'])
        self.assertIsNone(variants['performance_order'])
        self.assertEqual(legacy['visibility']['state'], ['Active','Inactive'])
        self.assertFalse(legacy['apply_without_user_review'])
        self.assertIsNone(source['exact_binary_member'])
        self.assertEqual([source['game_runtime'][k] for k in ('se','ae','gog','vr')], [None]*4)
        advice, = select('mod_maintenance_advisory_evidence_v1')
        self.assertTrue(advice['review_required'])
        self.assertEqual(advice['applies_when']['previous_track'], '3.x')
        rel, = select('mod.relationships.v1')
        self.assertEqual(rel['coverage']['required'], 'unknown')
        self.assertEqual([r['relation'] for r in rel['relationships']], ['optional','works_with'])
        self.assertFalse(any(r['predicate'] == 'runtime_match_v1' for r, _ in DATA))

    def test_offline_cards_keep_exact_links_and_verified_script(self):
        html = build(BUNDLE)
        cards = []
        anchors = set()

        class Read(HTMLParser):
            def handle_starttag(self, tag, attrs):
                values = dict(attrs)
                if tag == 'article':
                    cards.append(values['data-channel'])
                if tag == 'a':
                    anchors.add(values['href'])

        Read().feed(html)
        self.assertEqual(cards.count('stable'), 4)
        self.assertEqual(cards.count('development'), 1)
        for r in select('project_release_evidence_v1'):
            for file in r['files']:
                self.assertIn(file['official_download_url'], anchors)
        hashed = base64.b64encode(hashlib.sha256(SCRIPT.encode()).digest()).decode()
        self.assertIn("script-src 'sha256-"+hashed+"'", html)
        self.assertIn("default-src 'none'", html)
        self.assertNotIn('<script src=', html)
        self.assertNotIn('<img', html)
        self.assertIn('before removing that plugin', html)
        with self.assertRaises(ValueError):
            link('unsafe', 'https://github.com@other.invalid/thing')

    @unittest.skipUnless(os.environ.get('SKYRIM_FSMP_SQLITE'), 'requires native output')
    def test_exact_native_readonly_rows(self):
        db = Path(os.environ['SKYRIM_FSMP_SQLITE']).resolve()
        with sqlite3.connect(db.as_uri()+'?mode=ro', uri=True) as conn:
            self.assertEqual(conn.execute('PRAGMA integrity_check').fetchone(), ('ok',))
            self.assertEqual(conn.execute('PRAGMA foreign_key_check').fetchall(), [])
            self.assertEqual(conn.execute('SELECT count(*) FROM entities').fetchone(), (6,))
            self.assertEqual(dict(conn.execute('SELECT id,value FROM observations')),
                             {r['id']:r['value'] for r, _ in DATA})
