import copy
import hashlib
import json
from pathlib import Path
import shutil
import sys
from tempfile import TemporaryDirectory
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'tools'))
from reviewed_distributor_packs import PINS, admit_distributor_packs


class DistributorPacks(unittest.TestCase):
    def setUp(self):
        self.temp = TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for directory in ('data', 'config'):
            (self.root/directory).mkdir()
        for name in PINS:
            shutil.copyfile(ROOT/'data'/name, self.root/'data'/name)
        self.policy = self.root/'config/reviewed-distributor-packs.json'
        shutil.copyfile(ROOT/'config'/self.policy.name, self.policy)
        self.base = dict(schema='athena-reviewed-observations-v1',
                         games={'skyrim-se':'Skyrim Special Edition'}, sources={},
                         source_policy_revisions={}, entities=[], observations=[])

    def refuse(self, error):
        before = copy.deepcopy(self.base)
        with self.assertRaisesRegex(ValueError, error):
            admit_distributor_packs(self.base, self.root)
        self.assertEqual(self.base, before)

    def test_exact_two_packs_and_windows_line_endings(self):
        expected = copy.deepcopy(self.base)
        admit_distributor_packs(expected, self.root)
        self.assertEqual((len(expected['entities']),len(expected['observations'])), (50,127))
        self.assertEqual(set(expected['games']), {'skyrim-se','sims4'})
        self.assertEqual(len(expected['sources']),4)
        for name in PINS:
            file=self.root/'data'/name
            lf = file.read_bytes().replace(b'\r\n', b'\n')
            crlf = lf.replace(b'\n', b'\r\n')
            self.assertIn(b'\r\n', crlf)
            self.assertNotIn(b'\r\r\n', crlf)
            file.write_bytes(crlf)
        admit_distributor_packs(self.base, self.root)
        self.assertEqual(self.base, expected)

    def test_tampering_and_policy_authority_are_separate(self):
        file=self.root/'data'/PINS[1]
        original=file.read_bytes()
        file.write_bytes(original.replace(b'1.129.39',b'1.129.38',1))
        self.refuse('bytes changed')
        file.write_bytes(original)
        policy=json.loads(self.policy.read_text())
        policy['packs'][PINS[1]]['sources']['sims4-lot51-official']['allow_publish']=False
        self.policy.write_text(json.dumps(policy))
        self.refuse('withdrawn')

    def test_an_equivalent_preexisting_creator_keeps_its_original_record(self):
        prior={'id':'github:creator:powerof3','kind':'creator','game_id':'skyrim-se',
               'name':'powerof3','retained_fixture_context':'do not overwrite'}
        self.base['entities']=[prior.copy()]
        admit_distributor_packs(self.base,self.root)
        self.assertEqual([x for x in self.base['entities'] if x['id']==prior['id']],[prior])
        self.assertEqual(len(self.base['entities']),50)
        self.assertEqual(len(self.base['observations']),127)

    def test_casefold_collision_and_exact_observation_collision_fail_closed(self):
        old={'id':'github:creator:POWEROF3','kind':'creator','game_id':'skyrim-se','name':'unknown'}
        self.base['entities']=[old]
        self.refuse('Conflicting distributor entity')
        self.base['entities']=[]
        fact=json.loads((self.root/'data'/PINS[1]).read_text())['observations'][0]
        self.base['observations']=[fact]
        self.refuse('Invalid distributor observation')

    def test_digest_edit_does_not_grant_new_source_authority(self):
        f=self.root/'data'/PINS[1]
        doc=json.loads(f.read_bytes())
        row=doc['observations'][0]
        row['source_url']='https://example.org/other'
        f.write_text(json.dumps(doc))
        policy=json.loads(self.policy.read_text())
        policy['packs'][PINS[1]]['sha256_lf']=hashlib.sha256(f.read_bytes().replace(b'\r\n',b'\n')).hexdigest()
        self.policy.write_text(json.dumps(policy))
        self.refuse('Invalid distributor observation')

    def test_ambiguity_and_one_of_never_become_two_hard_edges(self):
        admit_distributor_packs(self.base,self.root)
        rows={r['id']:json.loads(r['value']) for r in self.base['observations']
              if r['predicate']=='mod.relationships.v1'}
        suite=rows['lot51:project:suite-life:20261124:mod.relationships.v1']['relationships']
        self.assertEqual([(r['target_id'],r['version_constraint']) for r in suite if r['condition'] is None],
                         [('lot51:project:core-library','>= 1.38')])
        self.assertEqual([r['raw_relation'] for r in suite if r['target_id'] is None], ['one-of'])
        edu=rows['adeepindigo:project:education-overhaul:20261124:mod.relationships.v1:packaging-review']
        self.assertEqual(edu['review'],'candidate')
        self.assertEqual(len(edu['relationships']),3)
        self.assertTrue(all(r['condition'] and 'ZIP' in r['condition'] for r in edu['relationships']))
        gtw=rows['adeepindigo:project:gtw-health-overhaul:20261124:mod.relationships.v1']['relationships']
        self.assertEqual(gtw[0]['relation'],'works_with')
        self.assertIn('core',gtw[0]['condition'])

    def test_skyrim_runtime_and_feature_requirements_remain_conditional(self):
        admit_distributor_packs(self.base, self.root)
        source = 'skyrim-se-distributors-reviewed-github-facts'
        rows = {r['entity_id']: r for r in self.base['observations']
                if r['predicate'] == 'mod.relationships.v1' and r['source_id'] == source
                and r['id'].endswith(':mod.relationships.v1')}
        boutique = 'github:aglowinthefield/boutique'
        studio = 'github:Jrollsons/skypatcher-studio'
        oif = 'github:arachnocid/ObjectImpactFramework'
        self.assertEqual(set(rows), {boutique, studio, oif})
        relations = {}
        for entity, r in rows.items():
            value = json.loads(r['value'])
            self.assertEqual((r['applicable_build'], r['mod_version'], r['completeness']),
                             ('unverified', 'unverified', 'partial'))
            self.assertEqual(value['coverage'], {'required': 'partial'})
            self.assertEqual(value['none_evidence'], {})
            relations[entity] = value['relationships']
            self.assertTrue(all(rel['condition'] and rel['basis'] == 'explicit'
                                and rel['relation'] == 'required'
                                and rel['evidence']['location']
                                for rel in relations[entity]))
        self.assertEqual([x['raw_relation'] for x in relations[boutique]], ['one-of'])
        self.assertIsNone(relations[boutique][0]['target_id'])
        self.assertEqual([(x['label'], x['target_id']) for x in relations[studio]],
                         [('SKSE', None), ('SkyPatcher', 'github:Zzyxz/SkyPatcher')])
        self.assertTrue(all('exported patch in-game' in x['condition'] for x in relations[studio]))
        self.assertEqual([(x['label'], x['target_id']) for x in relations[oif]], [('Grab And Throw', None)])
        self.assertIn('Throw event', relations[oif][0]['condition'])


    def test_skyrim_desktop_address_library_preserves_edition_limits(self):
        admit_distributor_packs(self.base, self.root)
        root = 'github:powerof3/'
        expected = {
            root + 'BaseObjectSwapper': 'Needed for SSE/AE',
            root + 'SeasonsOfSkyrim': 'Needed for SSE/AE',
            root + 'Keyword-Item-Distributor': 'Needed for SSE',
            root + 'Spell-Perk-Item-Distributor': 'Needed for SSE',
        }
        rows = {r['entity_id']: r for r in self.base['observations']
                if r['predicate'] == 'mod.relationships.v1'
                and r['id'].endswith(':address-library')}
        self.assertEqual(set(rows), set(expected))
        for entity, requirement in expected.items():
            r = rows[entity]
            self.assertEqual((r['applicable_build'], r['mod_version'], r['completeness']),
                             ('unverified', 'unverified', 'partial'))
            p = json.loads(r['value'])
            self.assertEqual(p['coverage'], {'required': 'partial'})
            self.assertEqual(p['none_evidence'], {})
            relation, = p['relationships']
            self.assertEqual(relation['raw_relation'], requirement)
            self.assertEqual(relation['evidence']['quote'], requirement)
            self.assertEqual(relation['label'], 'Address Library for SKSE')
            self.assertEqual((relation['relation'], relation['basis']), ('required', 'explicit'))
            self.assertIn('VR library is outside', relation['condition'])
            self.assertIsNone(relation['version_constraint'])
            self.assertIsNone(relation['target_id'])
            self.assertEqual(relation['target_scope'], {
                'game_id': 'skyrim-se', 'mod_version': None, 'file_id': None, 'runtime': None,
            })

    def test_lot51_projects_keep_exact_author_scope_and_pack_roles(self):
        admit_distributor_packs(self.base, self.root)
        prefix = 'lot51:project:'
        expected = {
            'simzlink': ('1.10.5', '2026-07-21', '>= 1.27'),
            'flicbuster': ('2.2', '2025-07-03', '>= 1.27'),
            'smarter-phone-use': ('2.2.1', '2026-02-27', '>= 1.27'),
            'telephone': ('1.0.4', '2026-03-23', '>= 1.40.1'),
            'free-range': ('1.8.2', '2026-03-20', '>= 1.27'),
            'bug-life': ('3.1', '2025-07-02', '>= 1.27'),
        }
        scope = {prefix + slug for slug in expected}
        self.assertEqual({x['id'] for x in self.base['entities']} & scope, scope)
        rows = {}
        for o in self.base['observations']:
            if o['entity_id'] in scope:
                rows.setdefault(o['entity_id'].removeprefix(prefix), {})[o['predicate']] = o
        edges = {}
        for slug, (version, release, minimum) in expected.items():
            found = rows[slug]
            self.assertEqual(set(found), {'creator', 'public_release', 'compatibility',
                                          'mod.relationships.v1'})
            self.assertTrue(all(o['source_url'] == 'https://lot51.cc/mods/' + slug
                                and o['source_id'] == 'sims4-lot51-official'
                                for o in found.values()))
            read = lambda predicate: json.loads(found[predicate]['value'])
            self.assertEqual(read('creator')['creator_id'], 'lot51:creator')
            self.assertEqual((read('public_release')['version'], read('public_release')['released_date']),
                             (version, release))
            self.assertIsNone(read('public_release')['download_endpoint'])
            self.assertIs(read('compatibility')['independently_tested'], False)
            self.assertEqual(read('compatibility')['game_build'], '1.129.39')
            payload = read('mod.relationships.v1')
            self.assertEqual(payload['none_evidence'], {})
            self.assertTrue(all(v == 'partial' for v in payload['coverage'].values()))
            core, *other = payload['relationships']
            self.assertEqual((core['target_id'], core['relation'], core['version_constraint']),
                             (prefix + 'core-library', 'required', minimum))
            self.assertTrue(all(rel['evidence']['location'] for rel in payload['relationships']))
            edges[slug] = other
        self.assertEqual([(r['label'], r['relation'], r['target_id']) for r in edges['flicbuster']],
                         [('Movie Hangout Stuff (EA pack)', 'required', None),
                          ('Simzlink', 'works_with', prefix + 'simzlink')])
        self.assertEqual([(r['label'], r['relation'], r['target_id']) for r in edges['free-range']],
                         [('Cottage Living (EA pack)', 'required', None)])
        self.assertEqual({(r['label'], r['relation'], r['target_id']) for r in edges['bug-life']},
                         {(x + ' (EA pack)', 'optional', None)
                          for x in ('Outdoor Retreat', 'Seasons', 'Eco Lifestyle')})
        self.assertIsNone(edges['smarter-phone-use'][1]['target_id'])
        self.assertIn('High School Years', edges['smarter-phone-use'][1]['condition'])
        self.assertTrue(all(r['relation'] == 'works_with' and r['condition']
                            for slug in ('telephone', 'smarter-phone-use') for r in edges[slug]))

    def test_lumpinou_links_scopes_and_one_setting_incompatibility(self):
        admit_distributor_packs(self.base, self.root)
        prefix = 'lumpinou:project:'
        expected = {'first-impressions-mod', 'road-to-romance', 'no-strings-attached',
                    'contextual-social-interactions', 'memory-panel-a-memory-system-mod',
                    'lumpinous-lgbtqia-mod'}
        self.assertEqual(expected, {e['id'].removeprefix(prefix) for e in self.base['entities']
                                   if e['id'].startswith(prefix)
                                   and e['id'] != prefix + 'the-mood-pack-mod'})
        rows = {}
        for r in self.base['observations']:
            slug = r['entity_id'].removeprefix(prefix)
            if slug in expected:
                self.assertEqual(r['source_id'], 'sims4-lumpinou-official')
                self.assertTrue(r['source_url'].endswith('/' + slug + '/'))
                self.assertEqual((r['applicable_build'], r['mod_version'], r['completeness']),
                                 ('unverified', 'unverified', 'partial'))
                rows.setdefault(slug, {})[r['predicate']] = json.loads(r['value'])
        for claims in rows.values():
            self.assertEqual(set(claims), {'creator', 'publishing_locations', 'mod.relationships.v1'})
            self.assertEqual(claims['creator']['creator_id'], 'lumpinou:creator')
            self.assertEqual(claims['publishing_locations']['author_identified_delivery_host'], 'Patreon')
            self.assertIsNone(claims['publishing_locations']['download_endpoint'])
            self.assertIsNone(claims['publishing_locations']['automated_download_authorized'])
            self.assertIs(claims['publishing_locations']['access_verified'], False)
            self.assertEqual(claims['mod.relationships.v1']['none_evidence'], {})
            self.assertTrue(all(x == 'partial' for x in claims['mod.relationships.v1']['coverage'].values()))
        relations = {slug: claim['mod.relationships.v1']['relationships'] for slug, claim in rows.items()}
        self.assertEqual([(r['relation'], r['target_id']) for r in relations['first-impressions-mod']],
                         [('required', prefix + 'the-mood-pack-mod'), ('optional', None), ('optional', None)])
        road = relations['road-to-romance']
        self.assertEqual([r['relation'] for r in road], ['required', 'alternative'])
        self.assertTrue(all(r['condition'] and r['target_id'] is None for r in road))
        self.assertEqual([r['target_id'] for r in relations['no-strings-attached']],
                         [prefix + 'road-to-romance', prefix + 'contextual-social-interactions'])
        self.assertTrue(all(r['condition'] for r in relations['no-strings-attached']))
        incompatible = [r for r in relations['lumpinous-lgbtqia-mod'] if r['relation'] == 'incompatible']
        conflict, = incompatible
        self.assertEqual(conflict['label'], 'McWooHoo')
        self.assertIn('No string woohoo setting enabled', conflict['condition'])
        self.assertIsNone(conflict['target_id'])
        self.assertIsNone(conflict['version_constraint'])

    def test_adeepindigo_health_roles_preserve_source_conditions(self):
        admit_distributor_packs(self.base, self.root)
        pre = 'adeepindigo:project:'
        expected = {
            'cosmetic-enhancements': 5, 'dental-care': 7, 'divergent-sims': 4,
            'eye-care': 2, 'family-therapy': 5, 'healthy-living': 2,
            'personal-care-services': 4, 'personal-sim-hygiene': 5,
        }
        entities = {x['id'] for x in self.base['entities']}
        self.assertTrue({pre + s for s in expected} <= entities)
        selected = {}
        for row in self.base['observations']:
            key = row['entity_id'].removeprefix(pre)
            if key not in expected:
                continue
            self.assertEqual(row['source_id'], 'sims4-adeepindigo-official')
            self.assertEqual(row['source_url'], 'https://adeepindigomods.itch.io/' + key)
            self.assertEqual((row['mod_version'], row['applicable_build'], row['completeness']),
                             ('unverified', 'unverified', 'partial'))
            selected.setdefault(key, {})[row['predicate']] = json.loads(row['value'])
        self.assertEqual(set(selected), set(expected))
        relationships = {}
        for key, facts in selected.items():
            self.assertEqual(set(facts), {'creator', 'mod.relationships.v1'})
            self.assertEqual(facts['creator']['creator_id'], 'adeepindigo:creator')
            v = facts['mod.relationships.v1']
            self.assertEqual(v['none_evidence'], {})
            self.assertTrue(all(x == 'partial' for x in v['coverage'].values()))
            self.assertEqual(len(v['relationships']), expected[key])
            self.assertTrue(all(r['evidence']['location'] and not r['version_constraint']
                                and (not r['target_id'] or r['target_id'] in entities)
                                for r in v['relationships']))
            relationships[key] = v['relationships']
        hygiene_conflict, = (r for r in relationships['personal-sim-hygiene'] if r['relation'] == 'incompatible')
        self.assertEqual(hygiene_conflict['target_id'], pre + 'dental-care')
        self.assertIn('Only applies when', hygiene_conflict['condition'])
        self.assertIn('dental hygiene package', hygiene_conflict['condition'])
        self.assertEqual({r['label'] for r in relationships['dental-care'] if r['relation'] == 'incompatible'},
                         {'Brush the Taste Away', 'Faster Teeth Brushing'})
        self.assertEqual({r['label'] for r in relationships['family-therapy'] if r['relation'] == 'required'},
                         {'The Mood Pack Mod', 'General Pie Menus (Shared Utilities component)'})
        self.assertTrue(all(r['condition'] for r in relationships['family-therapy'] if r['relation'] == 'works_with'))
        self.assertEqual({r['relation'] for r in relationships['eye-care']}, {'works_with'})
        self.assertEqual(sum(r['relation'] == 'optional' for r in relationships['personal-care-services']), 3)
