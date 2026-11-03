from pathlib import Path
import subprocess,os,selectors,time,shutil,json,gzip,hashlib
import argparse,tempfile,urllib.request
ap=argparse.ArgumentParser();ap.add_argument('--assets',type=Path,required=True);ap.add_argument('--output',type=Path);ap.add_argument('--repetitions',type=int,default=3);ap.add_argument('--replay-image',type=Path);ap.add_argument('--replay-impl',choices=['file','bbolt'],default='bbolt');ap.add_argument('--replay-case',default='unsafe');args=ap.parse_args()
repo=Path(__file__).resolve().parent
if not 1<=args.repetitions<=100:raise ValueError('repetitions must be 1..100')
if args.output and args.output.exists():raise ValueError('Use a fresh experiment output directory')
root=args.output or Path(tempfile.mkdtemp(prefix='adversary-guest-loss-'))
root.mkdir(exist_ok=True)
original=root/'original';original.mkdir(exist_ok=True)
for item in json.loads((repo/'assets.json').read_text()):
 name=item['url'].rsplit('/',1)[1];p=args.assets/name
 if not p.exists():
  p.parent.mkdir(parents=True,exist_ok=True);urllib.request.urlretrieve(item['url'],p)
 if hashlib.sha256(p.read_bytes()).hexdigest()!=item['sha256']:raise ValueError('Asset differs from tested profile: '+name)
 shutil.copyfile(p,root/name)
with gzip.open(root/'initramfs-virt','rb')as f:subprocess.run(['cpio','-idm','--quiet'],input=f.read(),cwd=original,capture_output=True,check=True)
subprocess.run(['go','build','-o',str(root/'guest-init'),str(repo/'guest.go')],env=dict(os.environ,CGO_ENABLED='0'),check=True)
shutil.copyfile(root/'guest-init',original/'init')
subprocess.run(['go','build','-mod=readonly','-o',str(root/'bbolt-worker'),'.'],cwd=repo/'bbolt-worker',env=dict(os.environ,CGO_ENABLED='0'),check=True)
shutil.copyfile(root/'bbolt-worker',original/'bbolt-worker')
(original/'bbolt-worker').chmod(0o755)
files=subprocess.check_output(['find','.','-print0'],cwd=original)
cpio=subprocess.run(['cpio','--null','-o','--format=newc','--quiet'],input=files,cwd=original,capture_output=True,check=True).stdout
with gzip.open(root/'adversary-initrd.gz','wb',compresslevel=1)as f:f.write(cpio)

def boot(image,mode,case,impl,label):
 cmd=['qemu-system-x86_64','-machine','q35,accel=kvm','-cpu','host','-m','256','-smp','1','-display','none','-monitor','none','-serial','stdio','-no-reboot','-net','none','-kernel',str(root/'vmlinuz-virt'),'-initrd',str(root/'adversary-initrd.gz'),'-append',f'console=ttyS0 quiet panic=-1 adv.mode={mode} adv.case={case} adv.impl={impl}', '-drive',f'file={image},format=raw,if=virtio,cache=none,aio=threads','-drive',f'file={root/"modloop-virt"},format=raw,if=virtio,readonly=on']
 start=time.monotonic();p=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
 lines=[];buf=b'';result=None
 with selectors.DefaultSelector()as sel:
  sel.register(p.stdout,selectors.EVENT_READ)
  while time.monotonic()-start<30:
   if not sel.select(.1):
    if p.poll()is not None:break
    continue
   data=os.read(p.stdout.fileno(),65536)
   if not data:break
   buf+=data
   while b'\n' in buf:
    line,buf=buf.split(b'\n',1);lines.append(line.decode(errors='replace'))
    if b'ADV:' in line:
     result=json.loads(line.split(b'ADV:',1)[1]);break
   if result is not None:break
 if p.poll()is None:p.kill()
 p.wait(timeout=10)
 (root/(label+'.log')).write_text('\n'.join(lines))
 if result is None:raise RuntimeError(f'No result {label}: '+ '\n'.join(lines)[-6000:])
 if 'error'in result:raise RuntimeError(result)
 result['seconds']=round(time.monotonic()-start,3);result['qemu_exit']=p.returncode
 return result

results=[]
if args.replay_image:
 before=hashlib.sha256(args.replay_image.read_bytes()).hexdigest()
 for repetition in range(args.repetitions):
  image=root/f'replay-{repetition}.img';shutil.copyfile(args.replay_image,image)
  results.append(boot(image,'recover',args.replay_case,args.replay_impl,f'replay-{repetition}'))
 after=hashlib.sha256(args.replay_image.read_bytes()).hexdigest()
 if before!=after:raise ValueError('Original witness changed during replay')
 (root/'replay-results.json').write_text(json.dumps({'witness_sha256':before,'original_unchanged':True,'results':results},indent=2))
 print(json.dumps(results,indent=2));print('Evidence:',root);raise SystemExit(0)
for impl,cases in [('file',['unsynced','synced','rename-synced']),('bbolt',['unsafe','safe'])]:
 baseline=root/(impl+'-baseline.img')
 with baseline.open('wb')as f:f.truncate(64*1024*1024)
 subprocess.run(['mkfs.ext4','-q','-F',str(baseline)],check=True)
 print('BASELINE',impl,boot(baseline,'baseline','safe',impl,impl+'-baseline'),flush=True)
 for case in cases:
  for repetition in range(args.repetitions):
   image=root/f'{impl}-{case}-{repetition}.img';shutil.copyfile(baseline,image)
   ack=boot(image,'write',case,impl,f'{impl}-{case}-{repetition}-capture')
   witness=root/f'{impl}-{case}-{repetition}.witness.img';shutil.copyfile(image,witness)
   observed=boot(image,'recover',case,impl,f'{impl}-{case}-{repetition}-recovery')
   row={'impl':impl,'case':case,'repetition':repetition,'ack':ack,'recovered':observed};results.append(row)
   print(json.dumps(row),flush=True)
(root/'results.json').write_text(json.dumps({'profile':'linux-guest-kill-completed-host-writes-retained-v0','kernel':'6.18.58-0-virt','filesystem':'ext4-default','qemu':'11.0.1','cache':'none','fault':'host SIGKILL of QEMU after guest acknowledgment; no graceful quit','asset_hashes':json.loads((repo/'assets.json').read_text()),'device_volatile_cache':'not modeled','deterministic_execution':False,'results':results},indent=2))

print('Evidence:',root)
