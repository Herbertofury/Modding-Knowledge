"""Exact evidence and independent output checks for the old-MCM migration review."""
import base64
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import sys
import unittest
import xml.etree.ElementTree as ET

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
from audit_fsmp_legacy_mcm import audit
from build_skyrim_fsmp_mcm_history import build, classify, lineage, SCRIPT
DATA=json.loads((ROOT/'data/skyrim-fsmp-mcm-history-reviewed.json').read_text(encoding='utf-8'))
BASE=json.loads((ROOT/'data/skyrim-fsmp-reviewed.json').read_text(encoding='utf-8'))
ROWS=lineage(DATA)

class FsmpMcmHistory(unittest.TestCase):
    def test_exact_lineage_retains_old_pack_without_assuming_installed_transition(self):
        self.assertEqual(DATA['observations'][:len(BASE['observations'])],BASE['observations'])
        self.assertEqual([r['mod_version'] for r,_ in ROWS],['v3.5.0','v4.0.0','v4.0.1','v4.1.1'])
        self.assertEqual([classify(v)[0] for _,v in ROWS],['named-copy','no-rule','no-rule','conditional'])
        self.assertEqual(ROWS[1][1]['git_blob'],ROWS[2][1]['git_blob'])
        self.assertNotEqual(ROWS[1][0]['source_url'],ROWS[2][0]['source_url'])
        self.assertEqual(ROWS[3][1]['legacy_step_visibility']['states'],['Active','Inactive'])
        self.assertEqual(ROWS[3][1]['named_file_copy_rules'][0]['flags'],[['LEGACYMCM','On']])
        for row,value in ROWS:
            self.assertIn(value['source_commit'],row['source_url'])
            self.assertFalse(value['archive_contents_tested'])
            self.assertFalse(value['savegame_compatibility_tested'])
            self.assertFalse(value['automatic_upgrade_authorized'])

    def test_scoped_tag_audit_handles_named_file_and_rejects_unsupported_input(self):
        sample=b'<config><requiredInstallFiles><file destination="./FSMPM - The FSMP MCM.esp"/></requiredInstallFiles></config>'
        self.assertEqual(audit(sample)['named_file_copy_rules'],[{'scope':'required-file'}])
        misleading=sample.replace(b'./FSMPM',b'./different/FSMPM')
        self.assertEqual(audit(misleading)['named_file_copy_rules'],[])
        for invalid in [b'<!DOCTYPE config><config/>',
                        '<!DOCTYPE config><config/>'.encode('utf-16'),
                        b'<not_a_config/>', b'<x>'*262144]:
            with self.assertRaises((ValueError,ET.ParseError)):
                audit(invalid)

    def test_viewer_all_four_exact_links_and_no_mutation_controls(self):
        out=build(DATA)
        hash=base64.b64encode(hashlib.sha256(SCRIPT.encode()).digest()).decode()
        self.assertIn("script-src 'sha256-"+hash+"'",out)
        for row,_ in ROWS:
            self.assertIn(row['source_url'],out)
        self.assertIn('option value="unknown"',out)
        self.assertIn('id="plugin-state"',out)
        self.assertNotIn('<form',out)
        self.assertNotIn('localStorage',out)
        self.assertNotIn('fetch(',out)

    @unittest.skipUnless(os.environ.get('SKYRIM_FSMP_MCM_HISTORY_SQLITE'), 'requires native output')
    def test_exact_readonly_native_rows(self):
        path=Path(os.environ['SKYRIM_FSMP_MCM_HISTORY_SQLITE']).resolve()
        with sqlite3.connect(path.as_uri()+'?mode=ro',uri=True) as conn:
            self.assertEqual(conn.execute('PRAGMA integrity_check').fetchone(),('ok',))
            self.assertEqual(conn.execute('PRAGMA foreign_key_check').fetchall(),[])
            self.assertEqual(dict(conn.execute('SELECT id,value FROM observations')),
                             {r['id']:r['value'] for r in DATA['observations']})
