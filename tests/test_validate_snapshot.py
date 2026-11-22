"""Synthetic validator regressions; these are not native Rust/game acceptance."""
import copy
from contextlib import closing
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
            "insert into refresh_attempts values (1,\'source\',1,\'failed\',\'unreviewed response body\')",
        ]:
            with self.subTest(statement=statement):
                self.conn.execute('savepoint mutation')
                self.conn.execute(statement)
                with self.assertRaises(ValueError):
                    validator.validate_contents(self.conn, self.bundle)
                self.conn.execute('rollback to mutation')
                self.conn.execute('release mutation')

    def test_entity_manifest_provenance_is_not_a_storage_column(self):
        self.bundle['entities'][0]['source_id'] = 'source'
        validator.validate_contents(self.conn, self.bundle)

    def test_rejects_unreviewed_schema_objects_and_columns(self):
        mutations = [
            ("create table extra_payload (value text)",
             "insert into extra_payload values ('synthetic unreviewed row')"),
            ("alter table games add column extra_payload text",
             "update games set extra_payload='synthetic unreviewed field'"),
            ("create view extra_view as select id from games",),
            ("create trigger extra_trigger after insert on games begin select 1; end",),
            ("create index extra_index on games(name)",),
            ("drop index observations_entity_time",),
        ]
        for statements in mutations:
            with self.subTest(statements=statements):
                self.conn.execute('savepoint schema_mutation')
                try:
                    for statement in statements:
                        self.conn.execute(statement)
                    with self.assertRaisesRegex(ValueError, 'Snapshot storage schema mismatch'):
                        validator.validate_contents(self.conn, self.bundle)
                finally:
                    self.conn.execute('rollback to schema_mutation')
                    self.conn.execute('release schema_mutation')

    def test_portable_lf_and_crlf_schema_signatures_in_both_directions(self):
        lf = (ROOT/'src/schema.sql').read_bytes().decode('utf-8').replace('\r\n', '\n')
        crlf = lf.replace('\n', '\r\n')
        signatures = []
        for sqlite_ddl in (lf, crlf):
            with closing(sqlite3.connect(':memory:')) as conn:
                conn.executescript(sqlite_ddl)
                signatures.append(validator.schema_signature(conn))
                for trusted_ddl in (lf, crlf):
                    with self.subTest(trusted_crlf='\r' in trusted_ddl, sqlite_crlf='\r' in sqlite_ddl):
                        self.assertTrue(validator.matches_storage_schema(conn, trusted_ddl))
        self.assertNotEqual(*signatures)

    def test_never_rewrites_line_endings_in_quoted_names_or_literals(self):
        ddl = ("CREATE TABLE \"line\nname\" (\n"
               "  [other\ncolumn] TEXT DEFAULT 'first''piece\nsecond',\n"
               "  `third\ncolumn` TEXT\n"
               ");\n")
        exact_crlf_format = ("CREATE TABLE \"line\nname\" (\r\n"
                             "  [other\ncolumn] TEXT DEFAULT 'first''piece\nsecond',\r\n"
                             "  `third\ncolumn` TEXT\r\n"
                             ");\r\n")
        self.assertEqual(list(validator.trusted_schema_variants(ddl)), [ddl, exact_crlf_format])
        for target in (exact_crlf_format, ddl.replace('\n', '\r\n')):
            with closing(sqlite3.connect(':memory:')) as conn:
                conn.executescript(target)
                self.assertEqual(validator.matches_storage_schema(conn, ddl), target == exact_crlf_format)

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
