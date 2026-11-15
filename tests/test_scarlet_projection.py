import unittest,json,sys,copy,gzip,sqlite3
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
 def test_checked_projection(self):
  self.assertEqual(project(self.d),json.loads((ROOT/'data/scarlet-reviewed.json').read_text(encoding='utf-8')))
 def test_native_artifact_exact_records(self):
  raw=gzip.decompress((ROOT/'artifacts/scarlet-sfw-sample.sqlite.gz').read_bytes())
  db=sqlite3.connect(':memory:');db.deserialize(raw);db.row_factory=sqlite3.Row
  self.assertEqual(db.execute('pragma integrity_check').fetchone()[0],'ok')
  self.assertEqual(db.execute('pragma foreign_key_check').fetchall(),[])
  actual=[dict(r) for r in db.execute('select * from observations order by id')]
  self.assertEqual(actual,sorted(project(self.d)['observations'],key=lambda r:r['id']))
  db.close()
if __name__=='__main__':unittest.main()
