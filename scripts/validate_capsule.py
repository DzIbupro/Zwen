"""Synthetic capsule UI validation; never reads user draft data."""
import os,json,selectors,subprocess,tempfile,time,sys
from pathlib import Path
binary=Path(sys.argv[1]).resolve();out=Path('artifacts/validation');out.mkdir(parents=True,exist_ok=True)
with tempfile.TemporaryDirectory(prefix='study-capsule-') as folder:
 env=os.environ.copy();env['ZWEN_DATA_DIR']=folder;env['ZWEN_SMOKE']='capsule'
 with (out/'capsule-stderr.txt').open('wb') as err:
  p=subprocess.Popen([str(binary),'capsule'],env=env,stdout=subprocess.PIPE,stderr=err,bufsize=0)
  sel=selectors.DefaultSelector();sel.register(p.stdout,selectors.EVENT_READ);buffer=b'';events=[];windows=[];start=time.monotonic()
  while time.monotonic()-start<35:
   for key,_ in sel.select(.2):
    chunk=os.read(key.fd,65536);buffer+=chunk
    while b'\n' in buffer:
     line,buffer=buffer.split(b'\n',1);text=line.decode(errors='replace');print(text,flush=True)
     if text.startswith('CAPSULE_PROGRESS:'):
      e=json.loads(text.split(':',1)[1]);events.append(e)
      try:
       raw=subprocess.run(['niri','msg','-j','windows'],capture_output=True,timeout=1)
       for w in json.loads(raw.stdout):
        if w.get('pid')==p.pid:windows.append({'stage':e['stage'],'id':w['id'],'is_floating':w['is_floating'],'layout':w['layout']})
      except (OSError,ValueError,subprocess.TimeoutExpired):pass
     if text.startswith('FOUNDATION_SMOKE:'):events.append(json.loads(text.split(':',1)[1]))
   if p.poll() is not None:break
  else:p.terminate()
  p.wait(timeout=5)
 summary={'exit':p.returncode,'passed':p.returncode==0 and any(e.get('ok') and e.get('mode')=='capsule' for e in events),'events':events,'own_windows':windows}
 (out/'capsule.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2));print(summary,flush=True)
 assert summary['passed'],'Capsule native validation failed'
 for w in windows:
  if w['stage']!='full':assert w['is_floating'],'Capsule must actually float in Niri'

 if env.get('NIRI_SOCKET'):
  caps=[w for w in windows if w['stage'] in ['initial','growing','capped']]
  assert len(caps)>=2,'Niri geometry evidence missing'
  origin=caps[0]['layout']['tile_pos_in_workspace_view']
  for w in caps:
   assert w['layout']['window_size'][0]==round(events[0]['width'])
   assert all(abs(a-b)<=2 for a,b in zip(origin,w['layout']['tile_pos_in_workspace_view'])),'Capsule top anchor moved'
