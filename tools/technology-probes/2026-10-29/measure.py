"""Measure only the launched sample and its descendant processes."""
import os,sys,subprocess,json,time,selectors
from pathlib import Path
root=Path(__file__).resolve().parent
label=sys.argv[1];cmd=sys.argv[2:];out=root/'results'/label;out.mkdir(exist_ok=True)
env=os.environ.copy();env['STUDY_SPIKE_ROOT']=str(root);env['GSK_RENDERER']='cairo'
err=(out/'stderr.txt').open('w');p=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=err,text=True,bufsize=1,env=env);start=time.monotonic();sel=selectors.DefaultSelector();sel.register(p.stdout,selectors.EVENT_READ);events=[];samples=[];stage='starting';last_sample=0

def sample():
 allstats={}
 for d in Path('/proc').iterdir():
  if not d.name.isdigit():continue
  try:
   raw=(d/'stat').read_text();v=raw[raw.rfind(')')+2:].split();allstats[int(d.name)]=(int(v[1]),int(v[11])+int(v[12]))
  except (OSError,ValueError):pass
 ids={p.pid};changed=True
 while changed:
  new={pid for pid,(parent,_) in allstats.items() if parent in ids};changed=not new.issubset(ids);ids|=new
 rss=pss=cpu=0;seen=[]
 for pid in ids:
  try:
   values={ln.split(':')[0]:int(ln.split()[1]) for ln in Path(f'/proc/{pid}/smaps_rollup').read_text().splitlines() if ln.startswith(('Rss:','Pss:'))}
   rss+=values['Rss'];pss+=values['Pss'];cpu+=allstats[pid][1];seen.append(pid)
  except (OSError,KeyError):pass
 return dict(seconds=round(time.monotonic()-start,3),stage=stage,pss_kib=pss,rss_kib=rss,cpu_ticks=cpu,pids=seen)
try:
 while time.monotonic()-start<75:
  for key,_ in sel.select(.2):
   line=key.fileobj.readline()
   if line:
    try:
     event=json.loads(line);event['observed_seconds']=round(time.monotonic()-start,3);events.append(event);print(label,event,flush=True)
     if event.get('stage') in ['started','hidden','restored','cycles','done']:stage=event['stage']
     elif event.get('stage')=='page' and event.get('title','').startswith('SPIKE_READY:'):stage='ready'
    except json.JSONDecodeError:print('non-json',line.strip(),flush=True)
  if time.monotonic()-last_sample>=.75:
   s=sample();samples.append(s);last_sample=time.monotonic()
  if p.poll() is not None:break
 else:p.terminate()
 p.wait(timeout=5)
finally:
 err.close();(out/'events.json').write_text(json.dumps(events,ensure_ascii=False,indent=2));(out/'samples.json').write_text(json.dumps(samples,indent=2))
summary=dict(label=label,exit=p.returncode,duration=round(time.monotonic()-start,3),peak_pss_mib=round(max((s['pss_kib'] for s in samples),default=0)/1024,1),peak_rss_mib=round(max((s['rss_kib'] for s in samples),default=0)/1024,1),done=any(e.get('stage')=='done' for e in events),ready=any(e.get('title','').startswith('SPIKE_READY:') for e in events),cycles=any(e.get('stage')=='cycles' for e in events))
for st in ['ready','hidden','restored']:
 group=[s for s in samples if s['stage']==st]
 if group:summary[st+'_pss_mib']=round(group[-1]['pss_kib']/1024,1);summary[st+'_processes']=len(group[-1]['pids'])
(out/'summary.json').write_text(json.dumps(summary,indent=2));print(json.dumps(summary,indent=2),flush=True)

sys.exit(0 if summary['done'] and summary['ready'] and summary['cycles'] and summary['exit']==0 else 1)
