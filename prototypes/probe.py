#!/usr/bin/env python3
"""Linux research probe: native kill plus offline single-file persistence replay.

NOT a general filesystem model. Successful pwrite64/ftruncate plus sync barriers;
pre-existing single file, no namespace changes, no writable mmap or O_DIRECT.
"""
import argparse, hashlib, json, os, re, selectors, shutil, signal, subprocess, time
from pathlib import Path

HERE = Path(__file__).resolve().parent

def run(cmd, **kw):
    return subprocess.run(list(map(str, cmd)), capture_output=True, text=True, timeout=30, **kw)

def read(worker, image):
    try: r = run([worker, 'read', image])
    except subprocess.TimeoutExpired: return {'inconclusive':'recovery timeout'}
    if r.returncode: return {'error': r.stderr[-1800:], 'exit': r.returncode}
    try: return json.loads(r.stdout.strip().splitlines()[-1])
    except Exception: return {'error': r.stdout[-1000:]}

def trace(worker, path, mode, trace_path=None):
    cmd = [str(worker), 'write', str(path), mode]
    if trace_path:
        cmd = ['strace','-f','-yy','-xx','-s','16777216','-o',str(trace_path),'-e',
               'trace=pwrite64,pwritev,pwritev2,write,writev,fsync,fdatasync,ftruncate,mmap,msync', *cmd]
    p = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                         stderr=subprocess.PIPE, text=True, start_new_session=True)
    with selectors.DefaultSelector() as ready:
        ready.register(p.stdout,selectors.EVENT_READ)
        if not ready.select(timeout=30):
            os.killpg(p.pid,signal.SIGKILL); p.communicate(timeout=10)
            raise TimeoutError('worker did not reach acknowledgment within 30 seconds')
        ack = p.stdout.readline().strip()
    if not ack:
        p.kill(); out, err=p.communicate(timeout=10)
        raise RuntimeError(f'No ack: {out} {err}')
    os.killpg(p.pid, signal.SIGKILL)
    p.communicate(timeout=10)
    assert json.loads(ack)['status']=='ok', ack
    return ack

def events_from_trace(text, path):
    # strace can split syscalls while another traced thread runs.
    pending, lines = {}, []
    for raw in text.splitlines():
        m=re.match(r'\s*(\d+)\s+(.*)',raw)
        if not m: continue
        tid,s=m.groups()
        if '<unfinished ...>' in s:
            pending[tid]=s.split('<unfinished ...>')[0]; continue
        m=re.match(r'<\.\.\. \w+ resumed>(.*)',s)
        if m:
            if tid not in pending: raise ValueError('unmatched resumed syscall')
            s=pending.pop(tid)+m.group(1)
        lines.append(s)
    events=[]; rejected=[]
    encoded_path=''.join('\\x%02x'%b for b in str(path).encode())
    for s in lines:
        if str(path) not in s and encoded_path not in s: continue
        if s.startswith('mmap('):
            if 'PROT_WRITE' in s and 'MAP_SHARED' in s: rejected.append(s)
            continue
        if s.startswith('pwrite64('):
            m=re.search(r'pwrite64\([^,]+, "([^"]*)", (\d+), (\d+)\)\s+= (\d+)',s)
            if not m: rejected.append(s[:300]); continue
            data,n,off,ret=m.groups(); data=bytes.fromhex(data.replace('\\x',''))
            if int(ret)!=int(n) or len(data)!=int(n): rejected.append('partial/malformed write'); continue
            events.append({'kind':'write','offset':int(off),'data':data.hex()})
        elif s.startswith(('fsync(', 'fdatasync(')):
            if re.search(r'= 0$',s): events.append({'kind':'sync'})
            else: rejected.append(s)
        elif s.startswith('ftruncate('):
            m=re.search(r', (\d+)\)\s+= 0$',s)
            if m: events.append({'kind':'resize','size':int(m.group(1))})
            else: rejected.append(s)
        elif re.match(r'(write|writev|pwritev|pwritev2|msync)\(',s): rejected.append(s[:300])
    if rejected: raise ValueError('unsupported trace effects: '+repr(rejected))
    if pending: raise ValueError('incomplete storage syscall trace')
    if not events: raise ValueError('empty storage trace: cannot establish observation completeness')
    return events

def apply(image, e):
    if e['kind']=='write':
        data=bytes.fromhex(e['data']); off=e['offset']; end=off+len(data)
        if end>len(image): image.extend(b'\0'*(end-len(image)))
        image[off:end]=data
    elif e['kind']=='resize':
        size=e['size']
        if size<len(image): del image[size:]
        else: image.extend(b'\0'*(size-len(image)))

def images(base, events):
    # Only sample storage outcomes; no exhaustive-coverage claim.
    durable=bytearray(base); volatile=bytearray(base); epoch=[]
    for i,e in enumerate(events):
        if e['kind']=='sync':
            durable=volatile.copy(); epoch=[]
        else:
            apply(volatile,e); epoch.append(e)
        candidates=[('drop-unsynced',durable),('retain-all',volatile)]
        for j,w in enumerate(epoch):
            candidate=durable.copy(); apply(candidate,w)
            candidates.append((f'retain-effect-{j}',candidate))
        for j,w in enumerate(epoch):
            if w['kind']=='write' and len(bytes.fromhex(w['data']))>512:
                data=bytes.fromhex(w['data'])
                for name,off,part in [('first-sector',w['offset'],data[:512]),('last-sector',w['offset']+len(data)-512,data[-512:])]:
                    candidate=durable.copy(); apply(candidate,{'kind':'write','offset':off,'data':part.hex()})
                    candidates.append((f'{name}-{j}',candidate))
        for name,image in candidates:
            yield i,name,bytes(image)

def check(values, acknowledged):
    if 'inconclusive' in values: return 'inconclusive'
    if 'error' in values: return 'violation'
    pair=(values.get('a'),values.get('b'))
    return 'no_violation_observed' if pair in ([('1','1')] if acknowledged else [('0','0'),('1','1')]) else 'violation'

def campaign(worker, language, dest, repetitions):
    start=time.monotonic(); dest.mkdir(parents=True,exist_ok=True)
    baseline=dest/'baseline.db'
    if baseline.exists(): raise ValueError('Use a fresh output directory for each campaign')
    r=run([worker,'init',baseline]); assert r.returncode==0,r.stderr
    base=baseline.read_bytes(); results=[]
    for mode in ['safe','unsafe']:
        native=[]
        for k in range(repetitions):
            p=dest/f'native-{mode}.db'; shutil.copyfile(baseline,p)
            trace(worker,p,mode)
            v=read(worker,p); native.append({'values':v,'verdict':check(v,True)})
        p=dest/f'traced-{mode}.db'; shutil.copyfile(baseline,p)
        tr=dest/f'{mode}.strace'; trace(worker,p,mode,tr)
        events=events_from_trace(tr.read_text(),p)
        (dest/f'{mode}.effects.json').write_text(json.dumps(events,indent=2))
        trials=[]; seen={}; witness=None
        all_images=list(images(base,events))
        # Include post-ACK durable-only even if the unsafe mode emits no writes.
        durable=bytearray(base); volatile=bytearray(base)
        for e in events:
            if e['kind']=='sync': durable=volatile.copy()
            else: apply(volatile,e)
        all_images.extend([(len(events),'post-ack-drop-unsynced',bytes(durable)),(len(events),'post-ack-retain-all',bytes(volatile))])
        full_image=dest/'full-replay.db'; full_image.write_bytes(volatile)
        observed_native=read(worker,p); replayed_native=read(worker,full_image)
        if observed_native!=replayed_native:
            raise ValueError(f'full-trace replay disagrees with native: {observed_native} != {replayed_native}')
        for cut,policy,image in all_images:
            acknowledged=cut==len(events)
            digest=hashlib.sha256(image).hexdigest(); key=(digest,acknowledged)
            if key in seen:
                trials.append({'cut':cut,'policy':policy,'sha256':digest,'acknowledged':acknowledged,'duplicate_of':seen[key]}); continue
            image_path=dest/'candidate.db'; image_path.write_bytes(image)
            v=read(worker,image_path); verdict=check(v,acknowledged)
            row={'cut':cut,'policy':policy,'sha256':digest,'acknowledged':acknowledged,'values':v,'verdict':verdict}
            seen[key]=len(trials); trials.append(row)
            if verdict=='violation' and acknowledged and witness is None:
                witness=dest/f'{mode}-witness.db'; witness.write_bytes(image)
                replay=[]
                for k in range(5):
                    image_path.write_bytes(image); replay.append(read(worker,image_path))
                row['replay_values']=replay
        result={'language':language,'mode':mode,'native_kill':native,'effect_count':len(events),
                'writes':sum(e['kind']=='write' for e in events),'syncs':sum(e['kind']=='sync' for e in events),
                'image_candidates':len(trials),'unique_checks':len(seen),'full_replay_matches_native':True,'trials':trials}
        results.append(result)
    return {'language':language,'seconds':round(time.monotonic()-start,3),'baseline_bytes':len(base),'results':results}

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--rust',required=True); ap.add_argument('--go',required=True)
    ap.add_argument('--out',type=Path,default=HERE/'results'); ap.add_argument('--repetitions',type=int,default=10)
    args=ap.parse_args(); args.out.mkdir(parents=True,exist_ok=True)
    results=[campaign(Path(worker),lang,(args.out/lang).resolve(),args.repetitions) for lang,worker in [('rust',args.rust),('go',args.go)]]
    (args.out/'results.json').write_text(json.dumps({'schema':'adversary.research.v1','fault_profile':'single-file-pwrite-barrier-sample-v1','campaigns':results},indent=2))
    for c in results:
        for r in c['results']:
            unique=[t for t in r['trials'] if 'verdict' in t]
            print(json.dumps({'language':c['language'],'mode':r['mode'],'seconds':c['seconds'],'native_violations':sum(t['verdict']=='violation' for t in r['native_kill']),'native_runs':len(r['native_kill']),'effects':r['effect_count'],'writes':r['writes'],'syncs':r['syncs'],'unique_images':len(unique),'pre_ack_violations':sum(t['verdict']=='violation' and not t['acknowledged'] for t in unique),'post_ack_violations':sum(t['verdict']=='violation' and t['acknowledged'] for t in unique)}))
if __name__=='__main__': main()
