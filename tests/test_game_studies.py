import copy
import json
import unittest
from tools.query_game_studies import DEFAULT, query, validate


class GameStudiesTests(unittest.TestCase):
    def setUp(self):
        self.data = json.loads(DEFAULT.read_text(encoding='utf-8'))

    def test_real_manifest_and_filter(self):
        self.assertEqual(len(validate(self.data)), 13)
        self.assertEqual(len(query(self.data, 'inspiration')), 10)
        self.assertEqual(len(query(self.data, 'foundation_candidate')), 3)

    def test_missing_reference_not_guessed(self):
        catalog = {'catalogId': self.data['catalog']['catalog_id'], 'items': []}
        rows = query(self.data, 'inspiration', catalog)
        self.assertTrue(all(row['resolution'] == 'missing' for row in rows))
        catalog['items'] = [{'id': rows[0]['catalog_ref']}]
        self.assertEqual(query(self.data, 'inspiration', catalog)[0]['resolution'], 'resolved')

    def test_reject_duplicate_and_unmeasured_claim(self):
        data = copy.deepcopy(self.data)
        data['records'].append(data['records'][0])
        with self.assertRaises(ValueError): validate(data)
        data = copy.deepcopy(self.data)
        data['records'][0]['evaluation'] = 'best'
        with self.assertRaises(ValueError): validate(data)

    def test_wrong_catalog(self):
        with self.assertRaises(ValueError):
            query(self.data, catalog={'catalogId': 'unrelated', 'items': []})

    def test_bool_schema_and_duplicate_catalog(self):
        self.data['schema_version'] = True
        with self.assertRaises(ValueError): validate(self.data)
        self.data['schema_version'] = 1
        catalog = {'catalogId': self.data['catalog']['catalog_id'], 'items': [{'id': 'same'}, {'id': 'same'}]}
        with self.assertRaises(ValueError): query(self.data, catalog=catalog)
