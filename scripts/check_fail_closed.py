#!/usr/bin/env python3
"""Integration checks for missing evidence, budgets and changing a contract."""
import argparse,json,subprocess,tempfile,os,time
from pathlib import Path
ap=argparse.ArgumentParser();ap.add_argument('--cli',type=Path,required=True);ap.add_argument('--demo',type=Path,required=True);ap.add_argument('--out',type=Path);a=ap.parse_args()
root=a.out or Path(tempfile.mkdtemp(prefix='adversary-negative-'));root.mkdir(exist_ok=True)
base=(a.demo/'rust-atomic-after-ack.toml').read_text();results=[]
p=subprocess.run([str(a.cli),'--not-a-real-option'],capture_output=True,text=True,timeout=10)
assert p.returncode==3;results.append({'name':'invalid-cli-usage','expected_exit':3,'actual_exit':p.returncode})
def run(name,config,expected,extra=None):
 path=root/(name+'.toml');path.write_text(config);out=root/name
 start=time.monotonic();cmd=[str(a.cli),'test','--manifest',str(path),'--output',str(out)]
 if extra:cmd=extra
 p=subprocess.run(cmd,capture_output=True,text=True,timeout=10)
 assert p.returncode==expected,(name,p.returncode,p.stdout,p.stderr)
 if expected==2:
  report=json.loads((out/'run.json').read_text());assert report['outcome']=='inconclusive';assert all(r['outcome']=='inconclusive'for r in report['results'])
 results.append({'name':name,'expected_exit':expected,'actual_exit':p.returncode,'seconds':round(time.monotonic()-start,3)})
run('missing-checkpoint',base.replace('kind = "after-ack"','kind = "checkpoint"\ncheckpoint = "never-happens"'),2)
run('unsupported-power-backend',base.replace('backend = "native-process"','backend = "guest-reset"'),3)
run('unknown-config-field',base+'\nunknown_field = 1\n',3)
run('zero-cases',base.replace('cases = 3','cases = 0'),3)
for name,body in [('malformed','print("not-json",flush=True)'),('wrong-id','import json,sys\nfor line in sys.stdin: print(json.dumps({"v":1,"id":"wrong","kind":"response","result":{}}),flush=True)'),('hung','import time\ntime.sleep(30)')]:
 adapter=root/(name+'.py');adapter.write_text('#!/usr/bin/env python3\n'+body+'\n');adapter.chmod(0o755)
 cfg=base.replace(next(x for x in base.splitlines()if x.startswith('command =')),'command = ['+json.dumps(str(adapter.resolve()))+']').replace('timeout_ms = 5000','timeout_ms = 100\nwall_time_ms = 200')
 run(name,cfg,2)
# A purported fix may not silently weaken the original requirement.
weakened=root/'weakened.toml';weakened.write_text(base.replace('a = "0"','a = "changed-initial-state"').replace('kind = "after-ack"','kind = "checkpoint"\ncheckpoint = "after-first-write"'))
p=subprocess.run([str(a.cli),'replay',str(a.demo/'rust-split-checkpoint'),'--output',str(root/'weakened'),'--allow-changed','--manifest',str(weakened)],capture_output=True,text=True,timeout=10)
assert p.returncode==3,(p.stdout,p.stderr);results.append({'name':'contract-weakening','expected_exit':3,'actual_exit':p.returncode})
# Existing artifact directories are immutable; repeat a command into one.
p=subprocess.run([str(a.cli),'test','--manifest',str(a.demo/'rust-atomic-after-ack.toml'),'--output',str(a.demo/'rust-atomic-after-ack')],capture_output=True,text=True,timeout=10)
assert p.returncode==3;results.append({'name':'immutable-run','expected_exit':3,'actual_exit':p.returncode})
(root/'summary.json').write_text(json.dumps(results,indent=2));print(json.dumps(results,indent=2))
