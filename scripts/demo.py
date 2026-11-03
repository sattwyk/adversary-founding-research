#!/usr/bin/env python3
"""Run the same native contract, reduction and fix evaluation in Rust and Go."""
from pathlib import Path
import argparse,json,subprocess,tempfile
ap=argparse.ArgumentParser();ap.add_argument('--cli',type=Path,required=True);ap.add_argument('--rust',type=Path,required=True);ap.add_argument('--go',type=Path,required=True);ap.add_argument('--output',type=Path);a=ap.parse_args()
root=a.output or Path(tempfile.mkdtemp(prefix='adversary-sdk-demo-'));root.mkdir(exist_ok=True)
def invoke(args,expected):
 r=subprocess.run([str(a.cli),*map(str,args)],capture_output=True,text=True,timeout=45)
 if r.returncode!=expected:raise RuntimeError(f'{args}: expected {expected}, got {r.returncode}\n{r.stdout}\n{r.stderr}')
 return r.stdout
results=[]
for lang,binary in [('rust',a.rust),('go',a.go)]:
 manifests={}
 for mode,fault,expect in [('atomic','after-ack',0),('atomic','checkpoint',0),('split','checkpoint',1),('relaxed','after-ack',1 if lang=='rust'else 0)]:
  name=f'{lang}-{mode}-{fault}';manifest=root/(name+'.toml')
  manifest.write_text(f'''schema = 1
name = "{name}"
backend = "native-process"
[adapter]
command = [{json.dumps(str(binary.resolve()))}]
[adapter.env]
ADVERSARY_EXAMPLE_MODE = "{mode}"
[contract]
kind = "atomic-map"
[contract.initial]
a = "0"
b = "0"
[campaign]
seed = 11
cases = 3
operations = 3
timeout_ms = 5000
[fault]
kind = "{fault}"
'''+('checkpoint = "after-first-write"\n'if fault=='checkpoint'else''))
  run=root/name
  text=invoke(['test','--manifest',manifest,'--output',run],expect)
  report=json.loads((run/'run.json').read_text());manifests[(mode,fault)]=manifest
  results.append({'scenario':name,'expected_exit':expect,'outcome':report['outcome'],'cases':len(report['results']),'stdout':text})
  print(text.strip(),flush=True)
 bad=root/f'{lang}-split-checkpoint'
 invoke(['replay',bad,'--output',root/f'{lang}-replay'],1)
 invoke(['reduce',bad,'--output',root/f'{lang}-reduced'],1)
 fixed=root/f'{lang}-fix-evaluation'
 invoke(['replay',bad,'--output',fixed,'--allow-changed','--manifest',manifests[('atomic','checkpoint')]],0)
 reduced=json.loads((root/f'{lang}-reduced/minimal/run.json').read_text())
 results.append({'scenario':f'{lang}-regression-workflow','reduced_operations':len(reduced['results'][0]['case']['operations']),'original_operations':3,'replay_failed_as_expected':True,'corrected_atomic_implementation_passed':True})
(root/'summary.json').write_text(json.dumps(results,indent=2));print('Evidence:',root)
