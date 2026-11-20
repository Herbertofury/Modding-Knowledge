import datetime as dt
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
spec=importlib.util.spec_from_file_location('refresh',Path(__file__).resolve().parents[1]/'tools/refresh_modrinth.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
NOW=dt.datetime(2026,11,15,tzinfo=dt.timezone.utc)
class RefreshTests(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.root=Path(self.tmp.name);(self.root/'config').mkdir()
  self.policy=json.loads((m.ROOT/'config/modrinth-source-policy.json').read_text());self.policy['project_ids']=['AANobbMI'];self.save()
 def tearDown(self):self.tmp.cleanup()
 def save(self):(self.root/'config/modrinth-source-policy.json').write_text(json.dumps(self.policy))
 def fetch(self,endpoint):
  if endpoint.startswith('project/'):
   return {'id':endpoint.split('/')[1],'project_type':'mod','team':'4reLOAKe','slug':'synthetic','title':'Synthetic','published':'2026-01-01T00:00:00Z','updated':'2026-01-02T00:00:00Z','license':{'id':'MIT','secret':'must-not-publish'},'loaders':['fabric'],'body':'private-test-sentinel'}
  return [{'team_id':'4reLOAKe','accepted':True,'role':'Maintainer','permissions':123,'user':{'id':'DzLrfrbK','username':'test_creator','email':'private-test-sentinel','name':'private-test-sentinel','bio':'private-test-sentinel'}}]
 def test_projection_redacts_unrequested_fields(self):
  self.assertTrue(m.refresh(self.root,self.fetch,NOW));p=self.root/'data/live/modrinth-current.json';raw=p.read_text();self.assertNotIn('private-test-sentinel',raw);self.assertNotIn('must-not-publish',raw);data=json.loads(raw);self.assertEqual(data['observed_creator_accounts'],1);self.assertEqual(data['completeness'],'partial')
 def test_nested_unexpected_metadata_is_rejected(self):
  raw=self.fetch('project/AANobbMI');raw['organization']={'email':'private-test-sentinel'}
  with self.assertRaises(ValueError):m.project_projection(raw,'AANobbMI',NOW)
 def test_failure_preserves_exact_last_good(self):
  m.refresh(self.root,self.fetch,NOW);p=self.root/'data/live/modrinth-current.json';before=p.read_bytes()
  def failing(_):raise ValueError('failure body must never be retained')
  self.assertFalse(m.refresh(self.root,failing,NOW));self.assertEqual(before,p.read_bytes());self.assertNotIn('failure body',(self.root/'data/live/modrinth-refresh.json').read_text())
 def test_rate_limit_backoff_and_no_request(self):
  calls=[]
  def limited(endpoint):calls.append(endpoint);raise m.RateLimited(3600)
  self.assertFalse(m.refresh(self.root,limited,NOW));self.assertFalse(m.refresh(self.root,limited,NOW+dt.timedelta(minutes=1)));self.assertEqual(len(calls),1)
 def test_denied_policy_no_fetch(self):
  self.policy['allow_publish']=False;self.save()
  def forbidden(_):self.fail('Fetch must not run')
  self.assertFalse(m.refresh(self.root,forbidden,NOW))
 def test_explicit_bound_and_overflow(self):
  self.policy['project_ids']=['0000000'+str(i) for i in range(8)];self.save()
  self.assertFalse(m.refresh(self.root,lambda _: self.fail('oversized scope fetched'),NOW))
  self.policy['project_ids']=['AANobbMI'];self.policy['max_projects']=100;self.save()
  self.assertFalse(m.refresh(self.root,lambda _: self.fail('unexpected bound fetched'),NOW))
 def test_identity_and_partial_batch_fail_closed(self):
  m.refresh(self.root,self.fetch,NOW);p=self.root/'data/live/modrinth-current.json';before=p.read_bytes();self.policy['project_ids']=['AANobbMI','gvQqBUqZ'];self.save()
  def bad(endpoint):
   x=self.fetch(endpoint)
   if endpoint=='project/gvQqBUqZ':x['id']='bad'
   return x
  self.assertFalse(m.refresh(self.root,bad,NOW));self.assertEqual(before,p.read_bytes())
 def test_duplicate_accepted_identity_preserves_last_good(self):
  self.assertTrue(m.refresh(self.root,self.fetch,NOW))
  output=self.root/'data/live/modrinth-current.json';before=output.read_bytes()
  for conflicting in [False,True]:
   with self.subTest(conflicting=conflicting):
    def duplicate(endpoint):
     rows=self.fetch(endpoint)
     if endpoint.startswith('team/'):
      second=json.loads(json.dumps(rows[0]))
      if conflicting:second['role']='Owner';second['user']['username']='different_name'
      return rows+[second]
     return rows
    self.assertFalse(m.refresh(self.root,duplicate,NOW))
    self.assertEqual(output.read_bytes(),before)
    state=json.loads((self.root/'data/live/modrinth-refresh.json').read_text())
    self.assertEqual(state['outcome'],'failed');self.assertEqual(state['error_type'],'ValueError')
    self.assertTrue(state['last_good_retained'])
 def test_unaccepted_duplicate_is_not_published(self):
  rows=self.fetch('team/4reLOAKe/members');pending=json.loads(json.dumps(rows[0]));pending['accepted']=False
  self.assertEqual(m.member_projection(rows+[pending],'4reLOAKe','AANobbMI',NOW),m.member_projection(rows,'4reLOAKe','AANobbMI',NOW))
 def test_shared_team_remains_project_scoped_and_fetched_once(self):
  self.policy['project_ids']=['AANobbMI','gvQqBUqZ'];self.save();calls=[]
  def tracked(endpoint):calls.append(endpoint);return self.fetch(endpoint)
  self.assertTrue(m.refresh(self.root,tracked,NOW))
  result=json.loads((self.root/'data/live/modrinth-current.json').read_text())
  self.assertEqual(len(result['creator_memberships']),2)
  self.assertEqual(result['observed_creator_accounts'],1)
  self.assertEqual({r['project_id'] for r in result['creator_memberships']},set(self.policy['project_ids']))
  self.assertEqual(calls.count('team/4reLOAKe/members'),1)
if __name__=='__main__':unittest.main()

