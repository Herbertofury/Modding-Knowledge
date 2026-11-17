import copy
import json
import unittest
from tools.query_game_progression import DEFAULT,query,validate

class ProgressionEvidenceTests(unittest.TestCase):
    def setUp(self): self.data=json.loads(DEFAULT.read_text())
    def test_real_data(self): self.assertEqual(len(validate(self.data)),8)
    def test_genres_are_separate(self):
        for genre,n in [('survivors_like',3),('rts',1),('action_rpg',4)]:
            self.assertEqual(len(query(self.data,genre=genre)),n)
    def test_historical_not_upcoming(self):
        self.assertEqual(len(query(self.data,status='historical_reference')),3)
        self.assertEqual(query(self.data,status='announced'),[])
    def test_due_inclusive(self):
        self.assertEqual(query(self.data,as_of='2026-11-30',due_only=True),[])
        self.assertEqual(len(query(self.data,as_of='2026-12-01',due_only=True)),8)
    def test_stale_is_retained(self):
        rows=query(self.data,as_of='2027-01-01')
        self.assertEqual(len(rows),8)
        self.assertTrue(all(r['freshness']=='due' for r in rows))
    def test_future_not_fresh(self):
        self.assertTrue(all(r['freshness']=='future_observation' for r in query(self.data,as_of='2026-01-01')))
    def test_search(self): self.assertEqual(len(query(self.data,search='DIABLO')),2)
    def test_reject_corruptions(self):
        for field,value in [('review_due','2026-01-01'),('source_url','https://user:secret@example.com'),('genre','anything'),('playtest','passed'),('observations',[])]:
            data=copy.deepcopy(self.data);data['records'][0][field]=value
            with self.assertRaises(ValueError): validate(data)
        self.data['records'].append(self.data['records'][0])
        with self.assertRaises(ValueError): validate(self.data)
    def test_unknown_filter(self):
        with self.assertRaises(ValueError): query(self.data,genre='FPS')
