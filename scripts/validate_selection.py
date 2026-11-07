"""Own-app synthetic PRIMARY integration. Never touches the real clipboard."""
import json,os,selectors,signal,sqlite3,subprocess,sys,tempfile,time
from pathlib import Path
binary=Path(sys.argv[1]).resolve()
assert subprocess.run(['pgrep','-x','zwen'],capture_output=True).returncode!=0,'Save and quit existing Zwen before isolated native tests'
results=[]
for scenario in ['selection','selection-recovery','selection-lifecycle']:
 with tempfile.TemporaryDirectory(prefix='zwen-selection-') as folder:
  root=Path(folder);helpers=root/'bin';helpers.mkdir();mode=root/'mode';mode.write_text('cold');count=root/'calls'
  helper=helpers/'wl-paste';helper.write_text('''#!/usr/bin/python3
import os,sys,time
from pathlib import Path
root=Path(os.environ['ZWEN_SELECTION_FIXTURE'])
assert sys.argv[1:]==['--primary','--no-newline','--type','text']
mode=(root/'mode').read_text()
with (root/'calls').open('a') as f:f.write(mode+'\\n')
if mode=='burst':time.sleep(.45)
if mode=='failure':print('MUST NOT USE THIS PARTIAL OUTPUT');sys.exit(1)
print({'cold':'合成选区','warm':'第二段','burst':'并发选区'}[mode],end='')
''');helper.chmod(0o700)
  db=sqlite3.connect(root/'state.sqlite3');db.executescript("CREATE TABLE drafts(id INTEGER PRIMARY KEY CHECK(id=1),text TEXT NOT NULL,revision INTEGER NOT NULL CHECK(revision>=0));INSERT INTO drafts VALUES(1,'合成旧草稿',0);PRAGMA user_version=1;");db.close()
  env=os.environ.copy();env.update({'PATH':str(helpers)+':'+env['PATH'],'ZWEN_DATA_DIR':folder,'ZWEN_SMOKE':scenario,'ZWEN_SELECTION_FIXTURE':folder})
  p=subprocess.Popen([str(binary),'explain-selection'],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,bufsize=0,start_new_session=True)
  sel=selectors.DefaultSelector();sel.register(p.stdout,selectors.EVENT_READ);buf=b'';passed=False;focus_guard=False;events=[]
  def action(name):
   child_env=env.copy();child_env.pop('ZWEN_SMOKE',None)
   subprocess.run([str(binary),name],env=child_env,timeout=6,check=True,capture_output=True)
  try:
   began=time.monotonic()
   while time.monotonic()-began<30:
    for key,_ in sel.select(.1):
     chunk=os.read(key.fd,65536);buf+=chunk
     while b'\n' in buf:
      line,buf=buf.split(b'\n',1);line=line.decode(errors='replace')
      if line.startswith('SELECTION_PROGRESS:'):
       e=json.loads(line.split(':',1)[1]);events.append(e)
       if e['stage']=='recover':action('recover')
       elif e['stage'] in ['hide-during','expand-during']:mode.write_text('burst');action('explain-selection')
       elif e['stage']=='assert-hidden':
        if env.get('NIRI_SOCKET'):
         own=[w for w in json.loads(subprocess.check_output(['niri','msg','-j','windows'])) if w.get('pid')==p.pid]
         assert not own,'Late capture remapped the explicitly hidden window'
       elif e['stage']=='warm':mode.write_text('warm');action('translate-selection')
       elif e['stage']=='failure':mode.write_text('failure');action('select')
       elif e['stage']=='burst':
        mode.write_text('burst');action('explain-selection')
        until=time.monotonic()+2
        while time.monotonic()<until and 'burst' not in count.read_text().splitlines():time.sleep(.01)
        action('translate-selection');time.sleep(.08)
        if env.get('NIRI_SOCKET'):
         windows=json.loads(subprocess.check_output(['niri','msg','-j','windows']))
         own=[w for w in windows if w.get('pid')==p.pid]
         assert not any(w.get('is_focused') for w in own),'A duplicate trigger focused Zwen before PRIMARY capture completed'
         focus_guard=True
      elif line.startswith('FOUNDATION_SMOKE:'):
       e=json.loads(line.split(':',1)[1]);events.append(e)
       if not e.get('ok'):raise AssertionError(e)
       if e.get('mode') in ['selection','selection-recovery','selection-lifecycle']:passed=True
    if p.poll() is not None:break
   p.wait(timeout=5);assert p.returncode==0 and passed,(p.returncode,events,p.stderr.read().decode(errors='replace')[-1500:])
   calls=count.read_text().splitlines();assert calls==(['cold','warm','failure','burst'] if scenario=='selection' else ['cold','burst','burst'] if scenario=='selection-lifecycle' else ['cold']),calls
   db=sqlite3.connect(root/'chat.sqlite3');assert db.execute('SELECT count(*) FROM turns').fetchone()[0]==0;db.close()
   result={'scenario':scenario,'passed':True,'helperCalls':calls,'focusGuard':focus_guard,'events':events};results.append(result);print(json.dumps(result,ensure_ascii=False),flush=True)
  finally:
   if p.poll() is None:os.killpg(p.pid,signal.SIGTERM);p.wait(timeout=5)
   sel.close()
Path('artifacts/validation').mkdir(parents=True,exist_ok=True)
Path('artifacts/validation/selection.json').write_text(json.dumps(results,ensure_ascii=False,indent=2))
