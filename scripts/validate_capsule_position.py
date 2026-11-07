"""Native Niri capsule positioning with synthetic stream, no mouse or personal data."""
import json,os,selectors,subprocess,sys,tempfile,time
from pathlib import Path
from capsule_fixture import CapsuleFixture
binary=Path(sys.argv[1]).resolve()
assert subprocess.run(['pgrep','-x','zwen'],capture_output=True).returncode!=0,'Save and quit existing Zwen first'
def niri(*args):
 return subprocess.check_output(['niri','msg',*args],text=True,timeout=2)
with CapsuleFixture() as fixture,tempfile.TemporaryDirectory(prefix='zwen-position-') as folder:
 env=os.environ.copy();env.update(fixture.env);env['ZWEN_DATA_DIR']=folder;env['ZWEN_SMOKE']='capsule';ack=Path(folder)/'ack';env['ZWEN_CAPSULE_FIXTURE_ACK']=str(ack)
 with tempfile.TemporaryFile() as errors:
  p=subprocess.Popen([str(binary),'capsule'],env=env,stdout=subprocess.PIPE,stderr=errors,bufsize=0)
  sel=selectors.DefaultSelector();sel.register(p.stdout,selectors.EVENT_READ);buffer=b'';events=[];geometry=[];violations=[];empty=None;filled=None;offset=None
  def own(floating=None):
   for _ in range(50):
    windows=json.loads(niri('-j','windows'));w=next((w for w in reversed(windows) if w.get('pid')==p.pid and (floating is None or w['is_floating']==floating) and (floating is not True or w['layout']['tile_pos_in_workspace_view'] is not None)),None)
    if w:return w
    time.sleep(.03)
   raise AssertionError('Own native window missing')
  def point():
   return own(True)['layout']['tile_pos_in_workspace_view']
  def move(x,y):
   w=own();niri('action','move-floating-window','--id',str(w['id']),'--x',str(x),'--y',str(y));return point()
  def near(a,b):return all(abs(x-y)<=2 for x,y in zip(a,b))
  start=time.monotonic()
  try:
   while time.monotonic()-start<45:
    for key,_ in sel.select(.1):
     chunk=os.read(key.fd,65536);buffer+=chunk
     while b'\n' in buffer:
      line,buffer=buffer.split(b'\n',1);text=line.decode(errors='replace')
      if text.startswith('CAPSULE_PROGRESS:'):
       e=json.loads(text.split(':',1)[1]);stage=e['stage'];print('stage',stage,flush=True);w=own(stage!='full');layout=w['layout'];xy=layout['tile_pos_in_workspace_view'];geometry.append(dict(stage=stage,floating=w['is_floating'],layout=layout));events.append(e)
       if stage=='move-empty':
        zero=move(0,0);offset=xy[1]-zero[1]
        if abs(offset-8)>2:violations.append(f'default top inset is {offset}, expected8 work-area pixels')
        empty=move(112,84);ack.write_text(stage)
       if stage in ['growing','capped'] and not near(xy,empty):violations.append(f'{stage} reset moved empty position: {xy} != {empty}')
       if stage=='move-filled':filled=move(572,168);ack.write_text(stage)
       if stage=='move-reactivated':ack.write_text(stage)
       if stage=='filled-resized' and not near(xy,filled):violations.append(f'resize reset moved filled position: {xy} != {filled}')
       if stage=='reactivated' and abs(xy[1]-(empty[1]-84+8))>2:violations.append('reactivation did not return to default top inset')
      if text.startswith('FOUNDATION_SMOKE:'):events.append(json.loads(text.split(':',1)[1]))
    if p.poll() is not None:break
   p.wait(timeout=5)
   passed=p.returncode==0 and any(e.get('ok') and e.get('mode')=='capsule' for e in events) and not violations
   for stage in ['initial','move-empty','growing','capped','move-filled','filled-resized','reactivated','full']:
    assert any(e.get('stage')==stage for e in events),('Missing stage',stage)
   result=dict(passed=passed,exit=p.returncode,defaultInset=offset,emptyMoved=empty,filledMoved=filled,violations=violations,geometry=geometry,evidence='Temporary app and loopback synthetic model; actual Niri move/resize, no physical mouse drag')
   out=Path('artifacts/validation/capsule-position.json');out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(result,ensure_ascii=False,indent=2))
   print(json.dumps(result,ensure_ascii=False));assert passed,'Capsule positioning regression'
  finally:
   diagnostic=Path('/tmp/zwen-position-partial.json');diagnostic.write_text(json.dumps({'events':events,'geometry':geometry,'violations':violations},ensure_ascii=False,indent=2))
   if p.poll() is None:p.terminate();p.wait(timeout=5)
