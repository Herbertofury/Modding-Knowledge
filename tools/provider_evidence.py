"""Conservative provider projections; identity and issue-count evidence are distinct."""
from copy import deepcopy
from urllib.parse import urlsplit
import re


def _public_url(value):
    if not isinstance(value, str):
        raise ValueError('source URL required')
    u = urlsplit(value)
    if u.scheme != 'https' or not u.hostname or u.username or u.password or u.query or u.fragment:
        raise ValueError('canonical public HTTPS source required')


def md5_candidates(md5, records):
    """Preserve every reviewed candidate. Hash lookup never selects/merges identities."""
    if not isinstance(md5, str) or re.fullmatch('[0-9a-fA-F]{32}', md5) is None:
        raise ValueError('MD5 hex required')
    out = []
    for r in records:
        if set(r) != {'project_id', 'creator_id', 'file_id', 'size_bytes', 'evidence_url', 'observed_unix'}:
            raise ValueError('invalid normalized candidate')
        for key in ('project_id', 'creator_id', 'file_id'):
            if not isinstance(r[key], str) or not r[key].strip():
                raise ValueError('provider-qualified candidate identity required')
        if type(r['size_bytes']) is not int or r['size_bytes'] < 0 or type(r['observed_unix']) is not int or r['observed_unix'] < 0:
            raise ValueError('invalid size or observation time')
        _public_url(r['evidence_url'])
        out.append(deepcopy(r))
    return {'md5': md5.lower(), 'candidates': out, 'selected_file_id': None,
            'identity_merge_allowed': False, 'identity_status': 'unresolved',
            'coverage': 'provided lookup results only'}


def github_issue_page(records, *, repository, observed_unix, bug_labels=('bug',)):
    """Counts only supplied page records, never repository-wide totals or defects.

    Caller enforces public-source policy. Label mapping is repository-specific.
    PR detection uses key presence, including null or empty pull_request payloads.
    """
    if not isinstance(repository, str) or re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repository) is None:
        raise ValueError('repository identity required')
    if type(observed_unix) is not int or observed_unix < 0 or not isinstance(bug_labels, (tuple, list)) or any(not isinstance(x, str) or not x for x in bug_labels):
        raise ValueError('invalid observation or label mapping')
    seen = set(); sources = []; counts = {'open': 0, 'closed': 0}; bugs = {'open': 0, 'closed': 0}; excluded = 0
    for r in records:
        if not isinstance(r, dict):
            raise ValueError('issue object required')
        if 'pull_request' in r:
            excluded += 1
            continue
        number = r.get('number'); state = r.get('state'); labels = r.get('labels')
        if type(number) is not int or number < 1 or state not in counts or not isinstance(labels, list):
            raise ValueError('invalid issue observation')
        url = f'https://github.com/{repository}/issues/{number}'
        if r.get('html_url') != url:
            raise ValueError('mixed repository or unsupported issue URL')
        if number in seen:
            raise ValueError('duplicate issue; reconcile pages before counting')
        seen.add(number)
        names = []
        for label in labels:
            name = label.get('name') if isinstance(label, dict) else label
            if not isinstance(name, str):
                raise ValueError('invalid label')
            names.append(name)
        counts[state] += 1
        if set(names) & set(bug_labels):
            bugs[state] += 1
        sources.append(url)
    return {'repository': repository, 'observed_unix': observed_unix,
            'source_url': f'https://api.github.com/repos/{repository}/issues',
            'coverage': 'supplied page only', 'repository_total': None,
            'observed_issue_counts': counts, 'observed_bug_labeled_report_counts': bugs,
            'bug_label_mapping': list(bug_labels), 'excluded_pull_request_rows': excluded,
            'issue_sources': sources, 'confirmed_defect_count': None}
