import copy
import importlib.util
import json
from pathlib import Path
import unittest
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('reviewed_tranches',ROOT/'tools/reviewed_tranches.py'); module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
class Tranches(unittest.TestCase):
 def setUp(self):
  self.bundle={'games':{},'sources':{},'source_policy_revisions':{},'entities':[],'observations':[]}
  self.data=json.loads((ROOT/'data/bethesda-creators-reviewed.json').read_text())
  self.policy=json.loads((ROOT/'config/bethesda-creators-reviewed-source-policy.json').read_text())
 def test_replay_deduplicates(self):
  module.add_tranche(self.bundle,self.data,self.policy); before=copy.deepcopy(self.bundle)
  module.add_tranche(self.bundle,self.data,self.policy); self.assertEqual(before,self.bundle)
 def test_policy_denied_atomic(self):
  self.policy['allow_publish']=False
  with self.assertRaises(ValueError): module.add_tranche(self.bundle,self.data,self.policy)
  self.assertFalse(self.bundle['entities'])
 def test_revision_drift(self):
  self.data['observations'][0]['policy_revision']='other'
  with self.assertRaises(ValueError): module.add_tranche(self.bundle,self.data,self.policy)
  self.assertFalse(self.bundle['entities'])
 def test_conflicting_entity(self):
  module.add_tranche(self.bundle,self.data,self.policy); before=copy.deepcopy(self.bundle)
  self.data['entities'][0]['name']='Different'
  with self.assertRaises(ValueError): module.add_tranche(self.bundle,self.data,self.policy)
  self.assertEqual(before,self.bundle)
 def test_unknown_reference(self):
  self.data['observations'][0]['entity_id']='missing'
  with self.assertRaises(ValueError): module.add_tranche(self.bundle,self.data,self.policy)
 def test_policy_source_scope(self):
  self.policy['source_ids']=['wrong']
  with self.assertRaises(ValueError): module.add_tranche(self.bundle,self.data,self.policy)
 def test_duplicate_in_tranche(self):
  self.data['entities'].append(copy.deepcopy(self.data['entities'][0]))
  with self.assertRaises(ValueError): module.add_tranche(self.bundle,self.data,self.policy)
 def test_exact_allowlist(self):
  self.assertEqual(len(module.TRANCHES),4)
  self.assertTrue(all('creators-reviewed.json' in pair[0] for pair in module.TRANCHES))
if __name__=='__main__':unittest.main()
