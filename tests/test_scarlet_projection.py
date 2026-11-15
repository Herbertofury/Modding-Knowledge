import unittest,json,sys,copy
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'tools'))
from project_scarlet import project
class ProjectionTests(unittest.TestCase):
 def setUp(self): self.d=json.loads((ROOT/'data/scarlet-public-sample.json').read_text())
 def test_actual_snapshot(self):
  p=project(self.d);self.assertEqual(len(p['entities']),10);self.assertTrue(all(x['completeness']=='partial' for x in p['observations']))
 def test_duplicate(self):
  self.d['rows'].append(self.d['rows'][0]);self.assertRaises(ValueError,project,self.d)
 def test_unsafe(self):
  self.d['rows'][0][-1]='https://user:secret@example.com/';self.assertRaises(ValueError,project,self.d)
 def test_shared_url(self):
  self.d['rows'][1][-1]=self.d['rows'][0][-1];self.assertEqual(len({x['id'] for x in project(self.d)['entities']}),10)
 def test_no_1000_truncation(self):
  self.d['rows']=[[str(i),'creator','Unknown','','','https://example.com/'] for i in range(1001)];self.assertEqual(len(project(self.d)['entities']),1001)
 def test_fail_closed(self):
  self.d['rows'][0][2]='SAFE';self.assertRaises(ValueError,project,self.d)
if __name__=='__main__':unittest.main()
