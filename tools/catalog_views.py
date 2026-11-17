"""Validated saved-view presentation contract; never mutates installation/load order."""
from copy import deepcopy


def validate_view(view, allowed_fields):
    """allowed_fields is the consumer's privacy-filtered stable field-ID allowlist.

    This validates storage, not query execution. No SQL or arbitrary expressions accepted.
    """
    expected = {'schema_version', 'scope', 'columns', 'sort', 'group_by', 'density',
                'theme', 'search_scope', 'filters', 'collection_id'}
    if set(view) != expected or type(view['schema_version']) is not int or view['schema_version'] != 1:
        raise ValueError('unsupported saved view schema')
    if view['scope'] not in ('catalog', 'collection'):
        raise ValueError('explicit view scope required')
    if view['scope'] == 'collection':
        if not isinstance(view['collection_id'], str) or not view['collection_id'].strip():
            raise ValueError('collection identity required')
    elif view['collection_id'] is not None:
        raise ValueError('collection must match scope')
    if view['density'] not in ('compact', 'comfortable') or view['theme'] not in ('deep_black', 'high_contrast', 'custom'):
        raise ValueError('unsupported presentation setting')
    if view['search_scope'] not in ('all_allowed_fields', 'visible_columns'):
        raise ValueError('explicit search scope required')
    for key in ('columns', 'group_by'):
        values = view[key]
        if not isinstance(values, list) or any(not isinstance(f, str) or f not in allowed_fields for f in values) or len(values) != len(set(values)):
            raise ValueError('unknown, unavailable or duplicate field')
    if not view['columns'] or len(view['group_by']) > 5:
        raise ValueError('columns required; grouping bounded')
    if not isinstance(view['sort'], list) or not 1 <= len(view['sort']) <= 8:
        raise ValueError('bounded explicit sort required')
    seen = set()
    for spec in view['sort']:
        if not isinstance(spec, dict) or set(spec) != {'field', 'direction', 'nulls'}:
            raise ValueError('invalid sort')
        f = spec['field']
        if not isinstance(f, str) or f not in allowed_fields or f in seen or spec['direction'] not in ('asc', 'desc') or spec['nulls'] not in ('first', 'last'):
            raise ValueError('invalid sort field/order/null placement')
        seen.add(f)
    if view['sort'][-1]['field'] != 'id':
        raise ValueError('stable ID tie-breaker required')
    remaining = [200]
    def query(node, depth=0):
        remaining[0] -= 1
        if remaining[0] < 0:
            raise ValueError('filter node budget exceeded')
        if not isinstance(node, dict) or depth > 8:
            raise ValueError('invalid or excessively nested filter')
        op = node.get('op')
        if op in ('all', 'any'):
            if set(node) != {'op', 'children'} or not isinstance(node['children'], list) or not 1 <= len(node['children']) <= 20:
                raise ValueError('bounded explicit Boolean groups required')
            for child in node['children']:
                query(child, depth + 1)
        else:
            if set(node) != {'op', 'field', 'value'} or not isinstance(node['field'], str) or node['field'] not in allowed_fields:
                raise ValueError('invalid filter field')
            if op == 'is_unknown':
                if node['value'] is not None:
                    raise ValueError('unknown is not zero or false')
            elif op == 'eq':
                if type(node['value']) not in (str, int, bool):
                    raise ValueError('scalar exact filter required')
            else:
                raise ValueError('unsupported operator')
    if view['filters'] is not None:
        query(view['filters'])
    return deepcopy(view)


def default_view():
    return {'schema_version': 1, 'scope': 'catalog', 'collection_id': None,
            'columns': ['image', 'name', 'content', 'category'],
            'sort': [{'field': 'id', 'direction': 'asc', 'nulls': 'last'}],
            'group_by': [], 'density': 'comfortable', 'theme': 'deep_black',
            'search_scope': 'all_allowed_fields', 'filters': None}
