"""Contract and observation-completeness checks, independent of library probes."""
import unittest
from pathlib import Path
from probe import check, events_from_trace, images
class Fidelity(unittest.TestCase):
 def test_synced_data_never_erased(self):
  events=[{'kind':'write','offset':0,'data':b'B'.hex()},{'kind':'sync'},{'kind':'write','offset':1,'data':b'C'.hex()}]
  after_sync=[im for cut,_,im in images(b'AA',events) if cut>=1]
  self.assertTrue(after_sync);self.assertTrue(all(im[0]==ord('B') for im in after_sync))
 def test_unsynced_durability_detected_but_pending_allowed(self):
  self.assertEqual(check({'a':'0','b':'0'},True),'violation')
  self.assertEqual(check({'a':'0','b':'0'},False),'no_violation_observed')
  self.assertEqual(check({'a':'1','b':'0'},False),'violation')
  self.assertEqual(check({'inconclusive':'timeout'},True),'inconclusive')
 def test_empty_or_truncated_trace_rejected(self):
  with self.assertRaises(ValueError): events_from_trace('',Path('/db'))
  with self.assertRaises(ValueError): events_from_trace('1 pwrite64(3</db>, ""..., 2, 0) = 2',Path('/db'))
 def test_writable_shared_mapping_rejected(self):
  with self.assertRaises(ValueError):events_from_trace('1 mmap(NULL, 4096, PROT_WRITE, MAP_SHARED, 3</db>, 0) = 0xa',Path('/db'))
if __name__=='__main__':unittest.main()
