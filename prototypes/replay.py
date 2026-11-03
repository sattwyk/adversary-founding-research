"""Replay a saved storage image without claiming instruction-level replay."""
import argparse,json,shutil,subprocess,tempfile
from pathlib import Path
ap=argparse.ArgumentParser();ap.add_argument('--worker',required=True);ap.add_argument('--image',type=Path,required=True);ap.add_argument('--repeat',type=int,default=5);a=ap.parse_args()
with tempfile.TemporaryDirectory(prefix='adversary-replay-') as d:
 for i in range(a.repeat):
  p=Path(d)/'db';shutil.copyfile(a.image,p)
  r=subprocess.run([a.worker,'read',str(p)],capture_output=True,text=True,timeout=30)
  print(json.dumps({'attempt':i,'exit':r.returncode,'stdout':r.stdout.strip(),'stderr':r.stderr}))
