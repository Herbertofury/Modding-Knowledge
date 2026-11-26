"""Offline relationship preflight. No hardware probing or file-system mod access."""
from argparse import ArgumentParser
import json
from pathlib import Path

SOURCE = Path(__file__).resolve().parents[1] / 'data/skyrim-fsmp-third-party-overlay-reviewed.json'


def review(base, choices, avx2):
    d = json.loads(SOURCE.read_text(encoding='utf-8'))
    decisions = []
    if base is None or base == 'unknown':
        decisions.append('base_version_unknown')
    elif base != d['upstream']['version']:
        decisions.append('base_version_differs_from_v4.2_notes')
    group = d['choices']
    selected = list(choices)
    supported = {entry['id']: entry for entry in group['variants']}
    if not group['minimum'] <= len(selected) <= group['maximum']:
        decisions.append('choose_exactly_one_variant')
    elif selected[0] not in supported:
        decisions.append('unknown_variant')
    elif supported[selected[0]]['requires'] == 'avx2' and avx2 != 'yes':
        decisions.append('avx2_capability_unknown' if avx2 == 'unknown' else 'avx2_not_supported')
    return {
        'status': 'review_only_matches_published_rules' if not decisions else 'review_only_incomplete_or_mismatch',
        'reasons': decisions,
        'source': d['project']['description_url'],
        'reported_base_version': base,
        'required_base': d['upstream'],
        'replaced_file': d['relations'][1]['target_path'],
        'selected_variants': selected,
        'known_archive_ids': [],
        'selection_or_install_authorized': False,
        'game_compatibility_verified': d['qualifications']['runtime_verified'],
        'legacy_mcm_effect': d['qualifications']['legacy_mcm_effect'],
    }


def main():
    p = ArgumentParser(description=__doc__)
    p.add_argument('--base', default='unknown', help='Version explicitly observed by the user')
    p.add_argument('--variant', action='append', choices=['nonavx', 'avx2'], default=[])
    p.add_argument('--avx2', choices=['yes', 'no', 'unknown'], default='unknown')
    a = p.parse_args()
    print(json.dumps(review(a.base, a.variant, a.avx2), indent=2))


if __name__ == '__main__':
    main()
