import argparse, json, shutil, subprocess, tempfile, time
from pathlib import Path
ap=argparse.ArgumentParser()
ap.add_argument('--faultbox',default='/tmp/adversary-faultbox')
ap.add_argument('--rust',default='/tmp/adversary-build-rust/release/adversary-redb-probe')
ap.add_argument('--go',default='/tmp/adversary-bbolt-worker')
ap.add_argument('--campaign-root',type=Path,default=Path('/tmp/adversary-probe-final'))
ap.add_argument('--out',type=Path)
args=ap.parse_args()
root=args.out or Path(tempfile.mkdtemp(prefix='adversary-baseline-'));root.mkdir(exist_ok=True)
results=[]
for lang,worker in [('rust',args.rust),('go',args.go)]:
 db=root/(lang+'.db')
 shutil.copyfile(args.campaign_root/lang/'baseline.db',db)
 start=time.monotonic()
 r=subprocess.run([args.faultbox,'run','--fault','fdatasync=EIO:100%','--',worker,'write',str(db),'safe'],input='',capture_output=True,text=True,timeout=30)
 (root/(lang+'.stdout.log')).write_text(r.stdout);(root/(lang+'.stderr.log')).write_text(r.stderr)
 results.append({'language':lang,'command':'faultbox run --fault fdatasync=EIO:100% -- worker write db safe','exit':r.returncode,'seconds':round(time.monotonic()-start,3),'ack_emitted':'"status":"ok"' in r.stdout,'stdout_tail':r.stdout[-3000:],'stderr_tail':r.stderr[-3000:]})
(root/'results.json').write_text(json.dumps(results,indent=2));print(json.dumps(results,indent=2))
