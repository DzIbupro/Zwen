"""Two native processes, temporary preferences/draft only; no model service."""
import json,os,subprocess,sys,tempfile
from pathlib import Path
assert subprocess.run(['pgrep','-x','zwen'],capture_output=True).returncode!=0,'Close existing Zwen first'
binary=Path(sys.argv[1]).resolve();rows=[]
with tempfile.TemporaryDirectory(prefix='zwen-ui-appearance-') as folder:
 for restored in [False,True]:
  env=os.environ.copy();env.update(ZWEN_DATA_DIR=folder,ZWEN_SMOKE='appearance')
  p=subprocess.run([str(binary)],env=env,capture_output=True,text=True,timeout=40)
  events=[json.loads(s.split(':',1)[1]) for s in p.stdout.splitlines() if s.startswith('FOUNDATION_SMOKE:')]
  assert p.returncode==0 and events and all(e.get('ok') and e.get('restored')==restored for e in events),(p.returncode,events,p.stderr[-2000:])
  rows+=events
out=Path('artifacts/validation');out.mkdir(parents=True,exist_ok=True)
(out/'appearance.json').write_text(json.dumps({'passed':True,'events':rows},ensure_ascii=False,indent=2))
print(json.dumps({'passed':True,'events':rows},ensure_ascii=False))
