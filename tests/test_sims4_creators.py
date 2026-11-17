"""Source-scoped creator identity, date and claim boundary regressions."""
import json
from pathlib import Path
import unittest
ROOT=Path(__file__).resolve().parents[1]
class SimsCreatorFacts(unittest.TestCase):
    def setUp(self):
        self.batch=json.loads((ROOT/'data/sims4-creators-reviewed.json').read_text())
        self.values={r['id']:json.loads(r['value']) for r in self.batch['observations']}
    def test_bounded_distinct_scope(self):
        self.assertEqual(len(self.batch['entities']),7)
        self.assertEqual(len(self.batch['observations']),12)
        self.assertEqual(sum(e['kind']=='creator' for e in self.batch['entities']),4)
        for section in ('entities','observations'):
            self.assertEqual(len(self.batch[section]),len({r['id'] for r in self.batch[section]}))
        for row in self.batch['observations']:
            self.assertEqual(row['completeness'],'partial')
            self.assertTrue(row['predicate'].startswith('provider_claim.'))
            self.assertEqual(row['policy_revision'],self.batch['source_policy_revisions'][row['source_id']])
    def test_handoff_does_not_merge_creator_and_maintainer(self):
        v=self.values['scumbumbomods:project:teleport-any-sim:20261117:maintenance_handoff']
        self.assertNotEqual(v['original_author'],v['maintainer'])
        self.assertEqual(v['listed_update_date'],'2024-03-01')
        self.assertFalse(v['latest_update_claim'])
        self.assertIn('timeout',v['destination_check'])
    def test_dates_and_compatibility_do_not_exceed_evidence(self):
        be=self.values['twistedmexi:project:better-exceptions:20261117:release_listing']
        self.assertIsNone(be['release_date'])
        mccc=self.values['deaderpool:project:mccc:20261117:public_release']
        self.assertIsNone(mccc['release_date'])
        compat=self.values['deaderpool:project:mccc:20261117:compatibility']
        self.assertEqual(compat['authority'],'publisher_declared')
        self.assertFalse(compat['independently_tested'])
        self.assertEqual(compat['future_patch_compatibility'],'unverified')
    def test_media_are_references_not_replicated_assets(self):
        for row in self.batch['observations']:
            if row['predicate'].endswith('media_location'):
                value=json.loads(row['value'])
                self.assertIsNone(value['asset_url'])
                self.assertIn('unknown',value['reuse_permission'])
if __name__=='__main__': unittest.main()
