"""Reviewed real-source assertions remain bounded and never become health verdicts."""
import json
from pathlib import Path
import unittest
ROOT = Path(__file__).resolve().parents[1]
class HealthSeed(unittest.TestCase):
    def test_actual_source_scope_and_non_inference(self):
        rows = json.loads((ROOT/'data/github-health-reviewed.json').read_text(encoding='utf-8'))['observations']
        claims = [r for r in rows if r['predicate']=='health_claim_v1']
        self.assertEqual(len(claims), 4)
        self.assertEqual(len({r['source_url'] for r in rows}), 6)
        for r in claims:
            v=json.loads(r['value'])
            self.assertEqual(v['authority'],'community_report')
            self.assertEqual(v['assertion'],'warning')
            self.assertLessEqual(v['reported_unix'],r['observed_unix'])
            self.assertEqual(v['lineage'],r['source_url'])
            self.assertEqual(r['completeness'],'partial')
        self.assertEqual(next(r for r in claims if r['source_url'].endswith('/804'))['applicable_build'],'unverified')
        self.assertEqual(sum(r['predicate']=='release_fix_report' for r in rows),2)
