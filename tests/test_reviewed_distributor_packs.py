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
        self.assertEqual((len(expected['entities']),len(expected['observations'])), (30,62))
        self.assertEqual(set(expected['games']), {'skyrim-se','sims4'})
        self.assertEqual(len(expected['sources']),4)
        for name in PINS:
            file=self.root/'data'/name
            file.write_bytes(file.read_bytes().replace(b'\n', b'\r\n'))
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
        self.assertEqual(len(self.base['entities']),30)
        self.assertEqual(len(self.base['observations']),62)

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
