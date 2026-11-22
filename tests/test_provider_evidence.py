import unittest
from tools.provider_evidence import md5_candidates, github_issue_page


def candidate(project='site:project:1', creator='site:creator:1'):
    return dict(project_id=project, creator_id=creator, file_id=project + ':file:1', size_bytes=20,
                evidence_url='https://example.org/file/1', observed_unix=100)


def issue(number, labels=None):
    return dict(number=number, html_url=f'https://github.com/example/repo/issues/{number}', state='open', labels=labels or [])


class ProviderEvidence(unittest.TestCase):
    def test_same_hash_preserves_different_projects_and_creators(self):
        a = candidate(); b = candidate('site:project:2', 'site:creator:2')
        out = md5_candidates('a' * 32, [a, b])
        self.assertEqual(len(out['candidates']), 2)
        self.assertFalse(out['identity_merge_allowed']); self.assertIsNone(out['selected_file_id'])
        out['candidates'][0]['size_bytes'] = 0
        self.assertEqual(a['size_bytes'], 20)

    def test_single_or_empty_hash_result_is_not_identity_proof(self):
        for records in ([], [candidate()]):
            self.assertEqual(md5_candidates('a' * 32, records)['identity_status'], 'unresolved')

    def test_pr_marker_excluded_even_empty_null_and_bug_label(self):
        rows = [issue(1, ['bug'])]
        for n, marker in enumerate((None, {}, {'url': 'https://example.org/pr'}), 2):
            r = issue(n, ['bug']); r['pull_request'] = marker; rows.append(r)
        out = github_issue_page(rows, repository='example/repo', observed_unix=100)
        self.assertEqual(out['observed_issue_counts']['open'], 1)
        self.assertEqual(out['observed_bug_labeled_report_counts']['open'], 1)
        self.assertEqual(out['excluded_pull_request_rows'], 3)
        self.assertIsNone(out['repository_total']); self.assertIsNone(out['confirmed_defect_count'])

    def test_issue_not_automatically_bug_and_state_counts(self):
        a = issue(1, [{'name': 'enhancement'}]); b = issue(2, [{'name': 'bug'}]); b['state'] = 'closed'
        out = github_issue_page([a, b], repository='example/repo', observed_unix=100)
        self.assertEqual(out['observed_bug_labeled_report_counts'], {'open': 0, 'closed': 1})
        self.assertEqual(out['observed_issue_counts'], {'open': 1, 'closed': 1})

    def test_page_empty_is_not_repo_zero(self):
        out = github_issue_page([], repository='example/repo', observed_unix=100)
        self.assertIsNone(out['repository_total'])

    def test_mixed_or_duplicate_issues_rejected(self):
        with self.assertRaises(ValueError): github_issue_page([issue(1), issue(1)], repository='example/repo', observed_unix=100)
        with self.assertRaises(ValueError): github_issue_page([issue(1)], repository='other/repo', observed_unix=100)
        r = candidate(); r['evidence_url'] += '?token=secret'
        with self.assertRaises(ValueError): md5_candidates('a' * 32, [r])
