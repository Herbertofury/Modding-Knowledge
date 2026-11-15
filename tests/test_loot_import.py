import ast,importlib.util,json,tempfile,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('loot',Path(__file__).resolve().parents[1]/'tools/import_loot.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
ROOT=Path(__file__).resolve().parents[1]
class LootTests(unittest.TestCase):
 def test_pipeline_text_io_has_explicit_utf8(self):
  for filename in ['build_reviewed_bundle.py','validate_snapshot.py']:
   tree=ast.parse((ROOT/'tools'/filename).read_text(encoding='utf-8'))
   for node in ast.walk(tree):
    if isinstance(node,ast.Call) and isinstance(node.func,ast.Attribute) and node.func.attr in ['read_text','write_text']:
     encodings=[k.value for k in node.keywords if k.arg=='encoding']
     self.assertTrue(any(isinstance(x,ast.Constant) and x.value=='utf-8' for x in encodings),filename+' lacks explicit UTF-8')
 def test_unreviewed_bytes_rejected_without_output(self):
  with tempfile.TemporaryDirectory() as t:
   out=Path(t)/'out'
   with self.assertRaises(ValueError):m.write_new(b'plugins: []',1,out)
   self.assertFalse(out.exists())
 def test_oversize_rejected(self):
  with self.assertRaises(ValueError):m.project(b'x'*(4*1024*1024+1),1)
 def test_actual_pinned_coverage_and_attribution(self):
  data=json.loads((ROOT/'data/loot-oblivion-seed.json').read_text(encoding='utf-8'))
  receipt=json.loads((ROOT/'data/loot-import-receipt.json').read_text(encoding='utf-8'))
  self.assertEqual(len(data['entities']),4745);self.assertEqual(len(data['observations']),4745)
  self.assertEqual(receipt['upstream_plugin_rows'],receipt['imported_plugin_rows'])
  seen=set()
  for row in data['observations']:
   value=json.loads(row['value']);self.assertEqual(value['source_blob_sha'],m.BLOB);self.assertEqual(value['source_commit_sha'],m.COMMIT)
   self.assertEqual(value['condition_evaluation'],'not_evaluated');self.assertEqual(value['license'],'CC0-1.0');self.assertEqual(row['source_url'],m.SOURCE)
   self.assertNotIn(value['source_index'],seen);seen.add(value['source_index'])
  self.assertEqual(seen,set(range(4745)))
if __name__=='__main__':unittest.main()
