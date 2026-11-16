import copy
import unittest
from tools.creator_refresh import plan_refresh


class CreatorRefresh(unittest.TestCase):
    def rows(self):
        return [{'creator_id': 'site:a', 'project_id': p, 'verified': True} for p in ('site:one', 'site:two')]

    def test_added_and_updated_check_creator_catalog_without_install(self):
        rows = self.rows(); before = copy.deepcopy(rows)
        for event in ('added', 'updated'):
            plan = plan_refresh('site:one', event, rows, [])
            self.assertEqual(plan['creator_profiles_to_refresh'], ['site:a'])
            self.assertEqual(plan['project_metadata_to_check'], ['site:one', 'site:two'])
            self.assertFalse(plan['install_or_update'])
        self.assertEqual(rows, before)

    def test_unverified_name_match_does_not_expand(self):
        rows = self.rows(); rows[0]['verified'] = False
        self.assertEqual(plan_refresh('site:one', 'added', rows, [])['project_metadata_to_check'], ['site:one'])

    def test_bounded_with_deterministic_overflow(self):
        plan = plan_refresh('site:one', 'updated', self.rows(), [], max_projects=1)
        self.assertTrue(plan['truncated'])
        self.assertEqual(plan['deferred_project_ids'], ['site:two'])

    def test_one_hop_only_and_reviewed_edges(self):
        edges = [{'source_project': a, 'target_project': b, 'relation': 'dependency', 'verified': v} for a, b, v in [('site:two', 'site:dep', True), ('site:dep', 'site:recursive', True), ('site:two', 'site:guess', False)]]
        plan = plan_refresh('site:one', 'added', self.rows(), edges)
        self.assertEqual(plan['project_metadata_to_check'], ['site:one', 'site:dep', 'site:two'])

    def test_invalid_budget_and_records(self):
        for budget in (0, 501, True):
            with self.assertRaises(ValueError): plan_refresh('site:one', 'added', [], [], max_projects=budget)
        with self.assertRaises(ValueError): plan_refresh('site:one', 'installed', [], [])
        rows = self.rows(); rows[0]['verified'] = 'yes'
        with self.assertRaises(ValueError): plan_refresh('site:one', 'added', rows, [])
