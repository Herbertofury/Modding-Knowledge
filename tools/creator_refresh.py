"""Bounded discovery-refresh suggestions; no network, identity inference or installs."""


def plan_refresh(trigger_project, event, memberships, related_projects, *, max_projects=50):
    """Only reviewed memberships connect a creator to projects.

    Partial graphs produce partial plans. Consumers persist overflow as follow-up work.
    Related projects mean reviewed dependency/compatibility edges, not search similarity.
    """
    if not isinstance(trigger_project, str) or not trigger_project.strip():
        raise ValueError('provider-qualified project required')
    if event not in ('added', 'updated'):
        raise ValueError('unsupported event')
    if type(max_projects) is not int or not 1 <= max_projects <= 500:
        raise ValueError('bounded project budget required')
    validated = []
    for row in memberships:
        if set(row) != {'creator_id', 'project_id', 'verified'} or type(row['verified']) is not bool:
            raise ValueError('invalid membership')
        if any(not isinstance(row[k], str) or not row[k].strip() for k in ('creator_id', 'project_id')):
            raise ValueError('stable provider-qualified identity required')
        validated.append(row)
    edges = []
    for row in related_projects:
        if set(row) != {'source_project', 'target_project', 'relation', 'verified'} or type(row['verified']) is not bool:
            raise ValueError('invalid relation')
        if row['relation'] not in ('dependency', 'compatibility'):
            raise ValueError('unsupported relation')
        if any(not isinstance(row[k], str) or not row[k].strip() for k in ('source_project', 'target_project')):
            raise ValueError('project identity required')
        edges.append(row)
    creators = sorted({r['creator_id'] for r in validated if r['verified'] and r['project_id'] == trigger_project})
    projects = {trigger_project}
    projects.update(r['project_id'] for r in validated if r['verified'] and r['creator_id'] in creators)
    # One reviewed outgoing hop only. No recursively expanding crawl.
    related = {r['target_project'] for r in edges if r['verified'] and r['source_project'] in projects}
    ordered = [trigger_project] + sorted((projects | related) - {trigger_project})
    return {'event': event, 'creator_profiles_to_refresh': creators,
            'project_metadata_to_check': ordered[:max_projects],
            'deferred_project_ids': ordered[max_projects:],
            'truncated': len(ordered) > max_projects,
            'coverage': 'reviewed graph only; discovery may be incomplete',
            'actions': ['refresh_public_metadata', 'check_release_metadata'],
            'install_or_update': False}
