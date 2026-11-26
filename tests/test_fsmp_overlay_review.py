import unittest
from tools.review_fsmp_overlay import review


class FsmpOverlayReview(unittest.TestCase):
    def test_stale_requirement_does_not_authorize_old_base(self):
        self.assertEqual(review('3.0.4', ['nonavx'], 'no')['reasons'],
                         ['base_version_differs_from_v4.2_notes'])
        self.assertIn('base_version_unknown', review('unknown', ['nonavx'], 'yes')['reasons'])

    def test_mutually_exclusive_cpu_choices_and_unknown_capability(self):
        for choices in ([], ['nonavx', 'avx2'], ['avx2', 'avx2']):
            self.assertIn('choose_exactly_one_variant', review('4.1.1', choices, 'yes')['reasons'])
        self.assertEqual(review('4.1.1', ['avx2'], 'unknown')['reasons'], ['avx2_capability_unknown'])
        self.assertEqual(review('4.1.1', ['avx2'], 'no')['reasons'], ['avx2_not_supported'])
        self.assertEqual(review('4.1.1', ['avx2'], 'yes')['reasons'], [])

    def test_even_matching_metadata_is_not_an_install_verdict(self):
        result = review('4.1.1', ['nonavx'], 'unknown')
        self.assertEqual(result['status'], 'review_only_matches_published_rules')
        self.assertFalse(result['selection_or_install_authorized'])
        self.assertFalse(result['game_compatibility_verified'])
        self.assertEqual(result['known_archive_ids'], [])
        self.assertEqual(result['replaced_file'], 'SKSE/Plugins/hdtsmp64.dll')
        self.assertEqual(result['legacy_mcm_effect'], 'unknown_separate_component')
