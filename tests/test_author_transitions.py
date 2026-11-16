import json
from pathlib import Path
import copy
import unittest
from tools.author_transitions import validate, decision


def fixture():
    return {'schema_version': 1, 'id': 'fixture:move:1', 'creator_id': 'fixture:creator:1',
            'source_account': 'https://old.example/author/1', 'status': 'banned',
            'status_evidence': [{'url': 'https://old.example/notices/1', 'kind': 'provider_notice', 'observed_unix': 90}],
            'destination_account': 'https://new.example/author/2',
            'identity_evidence': [{'url': 'https://old.example/author/1', 'kind': 'creator_crosslink', 'observed_unix': 95}],
            'project_mappings': [{'source_project': 'https://old.example/mod/1', 'destination_project': 'https://new.example/mod/2', 'game': 'skyrim-se', 'evidence_url': 'https://new.example/mod/2', 'verified': True}],
            'observed_unix': 100, 'review_after_unix': 200, 'lifecycle': 'active'}


class Transitions(unittest.TestCase):
    def test_reviewed_halffaces_status_does_not_invent_destination(self):
        r = json.loads((Path(__file__).resolve().parents[1] / 'data/author-status-halffaces-2026-11-16.json').read_text())
        d = decision(r, r['observed_unix'], True, True)
        self.assertEqual(d['status'], 'banned')
        self.assertEqual(d['status_authorities'], ['provider_notice'])
        self.assertEqual(d['action'], 'review_required')
        self.assertEqual(d['projects_to_track'], [])
        self.assertFalse(decision(r, r['review_after_unix'], True, True)['fresh'])

    def test_ready_adds_not_replaces(self):
        r = fixture(); before = copy.deepcopy(r)
        d = decision(r, 150, True, True)
        self.assertEqual(d['action'], 'track_additional_source')
        self.assertTrue(d['preserve_original_source']); self.assertFalse(d['automatic_downloads'])
        self.assertEqual(r, before)

    def test_missing_is_not_ban(self):
        r = fixture(); r['status_evidence'][0]['kind'] = 'retrieval_result'
        with self.assertRaises(ValueError): validate(r)
        r['status'] = 'unavailable'; validate(r)

    def test_all_gates(self):
        for field, value in [('identity_evidence', []), ('project_mappings', []), ('lifecycle', 'retracted'), ('lifecycle', 'superseded')]:
            r = fixture(); r[field] = value
            self.assertEqual(decision(r, 150, True, True)['projects_to_track'], [])
        self.assertEqual(decision(fixture(), 200, True, True)['action'], 'review_required')
        for consent, adapter in [(False, True), (True, False), (False, False)]:
            self.assertEqual(decision(fixture(), 150, consent, adapter)['action'], 'review_required')

    def test_no_name_only_catalog_migration(self):
        r = fixture(); r['project_mappings'][0]['verified'] = False
        self.assertEqual(decision(r, 150, True, True)['projects_to_track'], [])

    def test_reject_private_urls_and_unknown_claims(self):
        for url in ['http://old.example/a', 'https://u:p@old.example/a', 'https://old.example/a?token=secret']:
            r = fixture(); r['source_account'] = url
            with self.assertRaises(ValueError): validate(r)
        r = fixture(); r['ban_reason'] = 'unsupported allegation'
        with self.assertRaises(ValueError): validate(r)

    def test_ambiguity_and_future_evidence(self):
        r = fixture(); r['project_mappings'] *= 2
        with self.assertRaises(ValueError): validate(r)
        r = fixture(); r['identity_evidence'][0]['observed_unix'] = 101
        with self.assertRaises(ValueError): validate(r)

    def test_attribution_not_provider_verdict(self):
        r = fixture(); r['status_evidence'][0]['kind'] = 'creator_statement'
        self.assertEqual(decision(r, 150)['status_authorities'], ['creator_statement'])

if __name__ == '__main__': unittest.main()
