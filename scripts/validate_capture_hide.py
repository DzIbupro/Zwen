"""Hidden-window failure/stale intent injection; no screenshot or model service."""
import json,os,sqlite3,subprocess,sys,tempfile
from pathlib import Path
binary=Path(sys.argv[1]).resolve()
assert subprocess.run(['pgrep','-x','zwen'],capture_output=True).returncode!=0,'Save and quit existing Zwen first'
results=[]
for mode in ['failure','stale']:
 with tempfile.TemporaryDirectory(prefix='zwen-hide-guard-') as folder:
  root=Path(folder);helpers=root/'bin';helpers.mkdir();helper=helpers/'niri'
  helper.write_text('''#!/usr/bin/python3
import pathlib,sys,time
root=pathlib.Path(__file__).resolve().parent.parent
args=sys.argv[1:]
if args[:2]==['msg','event-stream']:
 print('{"ConfigLoaded":{}}',flush=True);time.sleep(10);sys.exit(0)
if args[:3]==['msg','action','screenshot']:
 with (root/'capture-calls').open('a') as f:f.write('capture\\n')
 sys.exit(2)
sys.exit(2)
''');helper.chmod(0o700)
  for name in ['slurp','grim']:
   guarded=helpers/name;guarded.write_text("#!/usr/bin/python3\nfrom pathlib import Path\np=Path(__file__).resolve().parent.parent/'capture-calls'\nwith p.open('a') as f:f.write('capture\\n')\nraise SystemExit(2)\n");guarded.chmod(0o700)
  with sqlite3.connect(root/'state.sqlite3') as db:
   db.executescript("CREATE TABLE drafts(id INTEGER PRIMARY KEY CHECK(id=1),text TEXT NOT NULL,revision INTEGER NOT NULL CHECK(revision>=0));INSERT INTO drafts VALUES(1,'合成隐藏草稿',0);PRAGMA user_version=1;")
  env=os.environ.copy();env.update({'ZWEN_DATA_DIR':folder,'ZWEN_SMOKE':'capture-hide','ZWEN_CAPTURE_HIDE_FIXTURE_MODE':mode,'PATH':str(helpers)+':'+env['PATH'],'NIRI_SOCKET':str(root/'unused.sock')})
  p=subprocess.Popen([str(binary)],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
  try:stdout,stderr=p.communicate(timeout=15)
  finally:
   if p.poll() is None:p.terminate();p.wait(timeout=5)
  events=[json.loads(line.split(':',1)[1]) for line in stdout.splitlines() if line.startswith('FOUNDATION_SMOKE:')]
  count=len((root/'capture-calls').read_text().splitlines()) if (root/'capture-calls').exists() else 0
  result={'case':mode,'exit':p.returncode,'helperCalls':count,'events':events}
  results.append(result);print(json.dumps(result,ensure_ascii=False),flush=True)
  assert p.returncode==0 and any(e.get('ok') for e in events),(stdout,stderr[-1000:])
assert all(r['helperCalls']==0 for r in results),('Screenshot helper must not start after failed/stale hide',results)
Path('artifacts/validation').mkdir(parents=True,exist_ok=True)
Path('artifacts/validation/capture-hide.json').write_text(json.dumps({'passed':True,'cases':results},ensure_ascii=False,indent=2))
