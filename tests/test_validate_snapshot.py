"""Synthetic validator regressions; these are not native Rust/game acceptance."""
import copy
import importlib.util
from pathlib import Path
import sqlite3
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('validate_snapshot', ROOT/'tools/validate_snapshot.py')
validator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(validator)


class SnapshotValidation(unittest.TestCase):
    def setUp(self):
        self.conn = sqlite3.connect(':memory:')
        self.addCleanup(self.conn.close)
        self.conn.executescript((ROOT/'src/schema.sql').read_text(encoding='utf-8'))
        self.bundle = {
            'games': {'game': 'Game'},
            'sources': {'source': 'https://example.org/policy'},
            'source_policy_revisions': {'source': 'reviewed-test'},
            'entities': [{'id': 'guide', 'game_id': 'game', 'kind': 'guide', 'name': 'Guide'}],
            'observations': [dict(zip(validator.OBSERVATION_KEYS, [
                'observation', 'guide', 'source', 'https://example.org/source',
                'unevaluated_source_context', '{}', 1, 'unverified', 'unverified',
                'partial', 'reviewed-test']))]}
        self.conn.execute('insert into games values (?,?)', ('game', 'Game'))
        self.conn.execute('insert into sources values (?,?,?,?,?)',
                          ('source', 'https://example.org/policy', 1, 1, 'reviewed-test'))
        self.conn.execute('insert into entities values (?,?,?,?)', ('guide', 'game', 'guide', 'Guide'))
        self.conn.execute('insert into observations values (?,?,?,?,?,?,?,?,?,?,?)',
                          list(self.bundle['observations'][0].values()))

    def test_valid_fixture(self):
        self.assertEqual(validator.validate_contents(self.conn, self.bundle),
                         {'entities': 1, 'observations': 1, 'source_scopes': 1, 'game_namespaces': 1})

    def test_tampering_rejected(self):
        for statement in [
            "update games set name='Changed'",
            "update entities set name='Changed'",
            "update sources set policy_url='https://example.org/other'",
            "update sources set can_publish=0",
            "update sources set can_retain=0",
            "update sources set policy_revision='other'",
            "update observations set value='changed'",
            "insert into games values ('extra','Extra')",
            "insert into entities values ('extra','game','guide','Extra')",
            "insert into sources values ('extra','https://example.org',1,1,'reviewed-test')",
            "pragma user_version=2",
        ]:
            with self.subTest(statement=statement):
                self.conn.execute('savepoint mutation')
                self.conn.execute(statement)
                with self.assertRaises(ValueError):
                    validator.validate_contents(self.conn, self.bundle)
                self.conn.execute('rollback to mutation')
                self.conn.execute('release mutation')

    def test_default_source_revision(self):
        bundle = copy.deepcopy(self.bundle)
        bundle.pop('source_policy_revisions')
        self.conn.execute('update sources set policy_revision=?', (validator.DEFAULT_REVISION,))
        validator.validate_contents(self.conn, bundle)

    def test_missing_context_rejected_even_if_manifest_matches(self):
        self.bundle['observations'][0]['predicate'] = 'other'
        self.conn.execute("update observations set predicate='other'")
        with self.assertRaisesRegex(ValueError, 'Missing unevaluated'):
            validator.validate_contents(self.conn, self.bundle)
