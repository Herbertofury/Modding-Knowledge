import json
from pathlib import Path
import sqlite3
import unittest
from urllib.parse import urlsplit

ROOT=Path(__file__).resolve().parents[1]
PACK=ROOT/'data/scumbumbo-sims4-reviewed.json'

class ScumbumboSeed(unittest.TestCase):
    def setUp(self):
        self.seed=json.loads(PACK.read_text(encoding='utf-8'))
        self.entities={e['id']:e for e in self.seed['entities']}
        self.rows=self.seed['observations']
    def test_exact_scope_and_public_project_routes(self):
        self.assertEqual(len(self.entities),52)
        self.assertEqual(len({r['id'] for r in self.rows}),198)
        projects=[r for r in self.rows if r['predicate']=='creator' and r['entity_id'].startswith('scumbumbo:')]
        self.assertEqual(len(projects),49)
        for r in projects:
            v=json.loads(r['value'])
            self.assertEqual(r['source_url'],v['project_url'])
            self.assertEqual(urlsplit(v['project_url']).netloc,'scumbumbomods.com')
            self.assertEqual(r['applicable_build'],'unverified')
        self.assertNotIn('data/scumbumbo-sims4-reviewed.json',(ROOT/'tools/build_reviewed_bundle.py').read_text(encoding='utf-8'))
    def test_status_is_not_exact_health_and_relationships_stay_conditional(self):
        by_id={r['entity_id']:json.loads(r['value']) for r in self.rows if r['predicate']=='maintainer_patch_status_v1'}
        self.assertEqual(by_id['scumbumbo:project:xml-injector']['status'],'status_not_confirmed')
        self.assertEqual(by_id['scumbumbo:project:dont-wash-dishes-where-you-angry-poop']['status'],'obsolete')
        self.assertEqual(sum(v['status']=='broken_historical_notice' for v in by_id.values()),4)
        self.assertFalse(any(r['predicate']=='health_claim_v1' for r in self.rows))
        includes=[r for r in self.rows if r['predicate']=='maintainer_reported_inclusion_v1']
        self.assertEqual(len(includes),9)
        self.assertTrue(all(json.loads(r['value'])['target_id']=='deaderpool:project:mc-command-center' for r in includes))
        rel=[json.loads(r['value']) for r in self.rows if r['predicate']=='mod.relationships.v1']
        self.assertEqual(len(rel),49)
        self.assertTrue(all(x['coverage']['required']=='unknown' and not x['none_evidence'] for x in rel))
        entries=[item for envelope in rel for item in envelope['relationships']]
        self.assertEqual({r['relation'] for r in entries},{'works_with','incompatible'})
        self.assertTrue(all(x['condition'] and x['target_id'] in self.entities for x in entries))
    def test_native_materialization_exact_when_ci_has_created_it(self):
        path=ROOT/'scumbumbo-sims4.sqlite'
        if not path.is_file(): self.skipTest('Run native sidecar builder first')
        with sqlite3.connect(path.as_uri()+'?mode=ro',uri=True) as db:
            self.assertEqual(db.execute('pragma integrity_check').fetchone(),('ok',))
            field_order=['id','entity_id','source_id','source_url','predicate','value','observed_unix','applicable_build','mod_version','completeness','policy_revision']
            actual=db.execute('select '+','.join(field_order)+' from observations order by id').fetchall()
            expected=[tuple(r[k] for k in field_order) for r in self.rows]
            self.assertEqual(actual,expected)
            self.assertEqual(db.execute('select count(*) from entities').fetchone(),(52,))
