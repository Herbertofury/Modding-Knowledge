import copy
import unittest
from tools.update_advisories import pre_update_advisories, validate


def fixture():
    return dict(schema_version=1, id='synthetic:1', project_id='provider:project:1',
                affected_versions=['2.0'], game_build='game:1', runtime_context='loader:1',
                symptom='crash', evidence_level='community_report',
                source_url='https://example.org/issues/1', lineage='original:1',
                reported_unix=90, observed_unix=100, review_after_unix=200, lifecycle='active')


def query(records, **kw):
    args = dict(project_id='provider:project:1', target_version='2.0', game_build='game:1',
                runtime_context='loader:1', now_unix=150, max_age_seconds=100)
    args.update(kw)
    return pre_update_advisories(records, **args)


class UpdateAdvisories(unittest.TestCase):
    def test_informative_never_blocking_and_no_mutation(self):
        r = fixture(); before = copy.deepcopy(r)
        for symptom in ('crash', 'freeze', 'bug'):
            r['symptom'] = symptom
            result = query([r])
            self.assertTrue(result['update_allowed']); self.assertFalse(result['blocks_update'])
            self.assertIsNone(result['automatic_action'])
            result['notices'][0]['observation']['affected_versions'].append('mutated')
            self.assertEqual(r['affected_versions'], before['affected_versions'])

    def test_exact_candidate_scope(self):
        for kw in [dict(target_version='1.0'), dict(project_id='another'), dict(game_build='game:2'), dict(runtime_context='loader:2')]:
            self.assertEqual(query([fixture()], **kw)['notices'], [])

    def test_unknown_is_not_safe_or_exact(self):
        self.assertEqual(query([])['coverage'], 'unknown')
        r = fixture(); r['affected_versions'] = []; r['runtime_context'] = None
        self.assertEqual(query([r])['notices'][0]['scope'], 'uncertain')

    def test_fresh_scrape_does_not_refresh_old_report(self):
        r = fixture(); r['observed_unix'] = 140
        self.assertFalse(query([r], max_age_seconds=10)['notices'][0]['fresh'])
        self.assertFalse(query([r], now_unix=200)['notices'][0]['fresh'])

    def test_retraction_and_supersession_retained_not_active(self):
        for state in ('retracted', 'superseded'):
            r = fixture(); r['lifecycle'] = state
            self.assertEqual(query([r])['notices'][0]['display'], 'historical_or_stale')

    def test_no_cause_promotion_or_mirror_votes(self):
        r = fixture(); other = copy.deepcopy(r); other['id'] = 'synthetic:2'
        other['evidence_level'] = 'maintainer_confirmed_cause'
        out = query([r, other])
        self.assertEqual(out['distinct_report_lineages'], 1)
        self.assertEqual(len(out['notices']), 2)
        self.assertEqual(out['notices'][0]['observation']['evidence_level'], 'community_report')

    def test_invalid_records(self):
        for key, value in [('source_url', 'https://user:secret@example.org/a'), ('source_url', 'https://example.org/a?token=x'), ('reported_unix', 101), ('schema_version', True), ('affected_versions', ['2.0', '2.0'])]:
            r = fixture(); r[key] = value
            with self.assertRaises(ValueError): validate(r)
        r = fixture(); r['private_comment'] = 'not permitted'
        with self.assertRaises(ValueError): validate(r)
        with self.assertRaises(ValueError): query([fixture(), fixture()])
        with self.assertRaises(ValueError): query([fixture()], now_unix=99)
