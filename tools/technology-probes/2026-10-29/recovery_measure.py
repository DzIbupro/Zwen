"""Inject failure only into the isolated sample's own WebKit renderer."""
import os,subprocess,selectors,json,time,signal
from pathlib import Path
root=Path(__file__).resolve().parent
out=root/'results'/'tauri-renderer-recovery';out.mkdir(exist_ok=True)
env=os.environ.copy();env['SPIKE_RECOVERY_TEST']='1';env['STUDY_SPIKE_ROOT']=str(root)
p=subprocess.Popen([str(root/'build/release/study-spike-tauri')],stdout=subprocess.PIPE,stderr=(out/'stderr.txt').open('w'),bufsize=0,env=env)
sel=selectors.DefaultSelector();sel.register(p.stdout,selectors.EVENT_READ);events=[];killed=[];begin=time.monotonic();buffer=b'';samples=[];last_sample=0;stage="starting"
def renderer_pids():
 processes={}
 for d in Path('/proc').iterdir():
  if not d.name.isdigit():continue
  try:
   stat=(d/'stat').read_text();parent=int(stat[stat.rfind(')')+2:].split()[1]);cmd=(d/'cmdline').read_bytes().replace(b'\0',b' ').decode(errors='replace');processes[int(d.name)]=(parent,cmd)
  except (OSError,ValueError):pass
 ids={p.pid}
 while True:
  found={pid for pid,(parent,_) in processes.items() if parent in ids}
  if found.issubset(ids):break
  ids|=found
 return [pid for pid in ids if pid in processes and Path(processes[pid][1].split()[0]).name=='WebKitWebProcess'],ids
while time.monotonic()-begin<180:
 for key,_ in sel.select(.2):
  chunk=os.read(key.fd,65536)
  if not chunk:continue
  buffer+=chunk
  while b'\n' in buffer:
   line,buffer=buffer.split(b'\n',1)
   try:e=json.loads(line)
   except ValueError:continue
   e['seconds']=round(time.monotonic()-begin,3);events.append(e);print(e,flush=True)
   stage=e.get('stage',stage)
   if stage=='recovery_armed':
    targets,_=renderer_pids()
    for pid in targets:os.kill(pid,signal.SIGKILL);killed.append(pid)
    print('Injected renderer termination:',killed,flush=True)
 if stage=='long_hidden' and time.monotonic()-last_sample>=1:
  _,ids=renderer_pids();pss=rss=0
  for pid in ids:
   try:
    for line in Path(f'/proc/{pid}/smaps_rollup').read_text().splitlines():
     if line.startswith('Pss:'):pss+=int(line.split()[1])
     if line.startswith('Rss:'):rss+=int(line.split()[1])
   except OSError:pass
  samples.append({'seconds':round(time.monotonic()-begin,3),'pss_MiB':round(pss/1024,2),'rss_MiB':round(rss/1024,2),'processes':len(ids)})
  last_sample=time.monotonic()
 if p.poll() is not None:break
else:p.terminate()
(out/'hidden-samples.json').write_text(json.dumps(samples,indent=2))
p.wait(timeout=5);summary={'exit':p.returncode,'killed_renderer_count':len(killed),'ready_events':sum(e.get('title','').startswith('SPIKE_READY:') for e in events),'recovered':any(e.get('stage')=='recovery_complete' and e.get('ready_count',0)>=2 for e in events),'duration':round(time.monotonic()-begin,2)}
(out/'events.json').write_text(json.dumps(events,indent=2));(out/'summary.json').write_text(json.dumps(summary,indent=2));print(summary,flush=True)
assert summary['exit']==0 and summary['killed_renderer_count'] and summary['recovered']
