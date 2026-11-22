import copy
import unittest
from tools.catalog_views import default_view, validate_view

FIELDS = {'id', 'image', 'name', 'content', 'category', 'author', 'issue_count'}


class CatalogViews(unittest.TestCase):
    def test_visual_default_and_independent_search(self):
        view = validate_view(default_view(), FIELDS)
        self.assertEqual(view['theme'], 'deep_black')
        self.assertEqual(view['search_scope'], 'all_allowed_fields')
        self.assertNotIn('author', view['columns'])
        view['filters'] = {'op': 'eq', 'field': 'author', 'value': 'provider:1'}
        validate_view(view, FIELDS)

    def test_input_not_mutated_or_shared(self):
        view = default_view(); before = copy.deepcopy(view)
        out = validate_view(view, FIELDS); out['columns'].clear()
        self.assertEqual(view, before)

    def test_no_load_order_or_unknown_fields(self):
        for key in ('load_order', 'install_priority', 'sql'):
            view = default_view(); view[key] = 'change'
            with self.assertRaises(ValueError): validate_view(view, FIELDS)
        view = default_view(); view['columns'].append('private_account')
        with self.assertRaises(ValueError): validate_view(view, FIELDS)

    def test_stable_multisort_and_nulls(self):
        view = default_view()
        view['sort'].insert(0, {'field': 'issue_count', 'direction': 'desc', 'nulls': 'last'})
        validate_view(view, FIELDS)
        view['sort'].pop()
        with self.assertRaises(ValueError): validate_view(view, FIELDS)

    def test_explicit_boolean_and_unknown(self):
        view = default_view()
        view['filters'] = {'op': 'any', 'children': [{'op': 'is_unknown', 'field': 'issue_count', 'value': None}, {'op': 'eq', 'field': 'issue_count', 'value': 0}]}
        validate_view(view, FIELDS)
        view['filters']['children'][0]['value'] = 0
        with self.assertRaises(ValueError): validate_view(view, FIELDS)

    def test_scope_and_bounds(self):
        view = default_view(); view['scope'] = 'collection'
        with self.assertRaises(ValueError): validate_view(view, FIELDS)
        view['collection_id'] = 'private:collection:1'; validate_view(view, FIELDS)
        node = {'op': 'eq', 'field': 'name', 'value': 'test'}
        for _ in range(10): node = {'op': 'all', 'children': [node]}
        view['filters'] = node
        with self.assertRaises(ValueError): validate_view(view, FIELDS)

    def test_wide_filter_budget_and_custom_theme(self):
        view = default_view(); view['theme'] = 'custom'; validate_view(view, FIELDS)
        leaf = {'op': 'eq', 'field': 'name', 'value': 'test'}
        view['filters'] = {'op': 'all', 'children': [{'op': 'any', 'children': [leaf] * 20}] * 20}
        with self.assertRaises(ValueError): validate_view(view, FIELDS)
