"""Isolated own-app action UI driver; fake PRIMARY/source/anchor, no real clipboard/provider."""
import json,os,sqlite3,subprocess,sys,tempfile,threading,shutil,time
from pathlib import Path
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
binary=Path(sys.argv[1]).resolve()
assert subprocess.run(['pgrep','-x','zwen'],capture_output=True).returncode!=0,'Close existing Zwen before isolated native validation'
requests=[];errors=[]
class Service(BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_POST(self):
  try:
   data=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
   assert self.path=='/v1/chat/completions' and self.headers['Authorization']=='Bearer fixture-token'
   assert len(data['messages'])==1,'old conversation leaked'
   prompt=data['messages'][0]['content'];assert isinstance(prompt,str),'old image leaked'
   assert any(x in prompt for x in ['translate fixture','explain fixture','document fixture','failure fixture','stop fixture','partial fixture']),prompt
   requests.append(prompt)
   if 'failure fixture' in prompt:
    self.send_response(429);self.send_header('Content-Type','application/json');self.end_headers();self.wfile.write(b'{"error":{"code":"1113","message":"synthetic permanent limit"}}');return
   transient=any(x in prompt for x in ['stop fixture','translate fixture','document fixture'])
   if transient and ('stop fixture' in prompt or requests.count(prompt)==1):
    self.send_response(429);self.send_header('Content-Type','application/json');self.send_header('Retry-After','1');self.end_headers();self.wfile.write(b'{"error":{"code":"1302"}}');return
   if 'partial fixture' in prompt:
    self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers();self.wfile.write(b'data: {"choices":[{"delta":{"content":"Partial fixture answer"},"finish_reason":null}]}\n\n');self.wfile.flush();return
   answer='Translated fixture' if 'translate fixture' in prompt else 'Explained fixture' if 'explain fixture' in prompt else '# Document fixture\n\n|key|value|\n|---|---|\n|fixture|saved|'
   self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
   frame=lambda delta,finish:('data: '+json.dumps({'choices':[{'delta':delta,'finish_reason':finish}]})+'\n\n').encode()
   if 'translate fixture' in prompt or 'explain fixture' in prompt:
    self.wfile.flush();time.sleep(1.5)
    self.wfile.write(frame({'content':answer[:4]},None));self.wfile.flush();time.sleep(.3)
    answer=answer[4:]
   self.wfile.write(frame({'content':answer},None)+frame({},'stop')+b'data: [DONE]\n\n');self.wfile.flush()
  except Exception as e:errors.append(repr(e))
server=ThreadingHTTPServer(('127.0.0.1',0),Service);threading.Thread(target=server.serve_forever,daemon=True).start()
try:
 with tempfile.TemporaryDirectory(prefix='zwen-actions-native-') as folder:
  root=Path(folder);fake=root/'bin';fake.mkdir();selection=root/'selection';selection.write_text('');source=root/'source-window';source.write_text('100')
  helper=fake/'wl-paste';helper.write_text('''#!/usr/bin/python3
import os,sys,time,subprocess
from pathlib import Path
p=Path(os.environ['ZWEN_ACTIONS_FIXTURE_SELECTION_FILE'])
if '--watch' in sys.argv:
 last=None
 while True:
  current=p.read_text()
  if current!=last:
   last=current;subprocess.run(sys.argv[sys.argv.index('--watch')+1:],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
  time.sleep(.02)
else:
 current=p.read_text()
 if current.startswith('focus-race'):time.sleep(.5)
 sys.stdout.write(current)
''');helper.chmod(0o755)
  real_niri=shutil.which('niri');assert real_niri
  niri=fake/'niri';niri.write_text('#!/usr/bin/python3\nimport os,sys,json\nfrom pathlib import Path\nif sys.argv[1:]==["msg","-j","focused-window"]:print(json.dumps({"pid":999999,"id":int(Path(os.environ["ZWEN_ACTIONS_FIXTURE_SOURCE_FILE"]).read_text())}))\nelse:os.execv('+repr(real_niri)+',["niri"]+sys.argv[1:])\n');niri.chmod(0o755)
  # Seed prior chat; assertions at the local server forbid leaking it to actions.
  db=sqlite3.connect(root/'chat.sqlite3');db.executescript("CREATE TABLE turns(id INTEGER PRIMARY KEY,request_id TEXT NOT NULL UNIQUE,conversation_id TEXT NOT NULL,provider_profile TEXT NOT NULL,context_turns INTEGER NOT NULL,prompt TEXT NOT NULL,model TEXT NOT NULL,answer TEXT NOT NULL DEFAULT '',revision INTEGER NOT NULL DEFAULT 0 CHECK(revision>=0),state TEXT NOT NULL DEFAULT 'streaming',error TEXT,created_at INTEGER NOT NULL DEFAULT (unixepoch()));PRAGMA user_version=2;")
  db.execute("INSERT INTO turns(request_id,conversation_id,provider_profile,context_turns,prompt,model,answer,revision,state) VALUES('prior','prior','p',0,'must-not-send-old-context','fixture-model','old-answer',1,'complete')");db.commit();db.close()
  env=os.environ.copy();env.update(ZWEN_DATA_DIR=folder,ZWEN_SMOKE='actions',ZWEN_ACTIONS_FIXED_ANCHOR='1',ZWEN_SMOKE_TIMEOUT_SECS='70',ZWEN_PROVIDER_TEST_KEY='fixture-token',ZWEN_PROVIDER_TEST_BASE=f'http://127.0.0.1:{server.server_port}/v1',ZWEN_ACTIONS_FIXTURE_SOURCE_FILE=str(source),ZWEN_ACTIONS_FIXTURE_SELECTION_FILE=str(selection),ZWEN_ACTIONS_FIXTURE_OUTPUT_DIR=str(root/'output'),PATH=str(fake)+':'+env['PATH'])
  p=subprocess.Popen([str(binary)],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,start_new_session=True)
  try:stdout,stderr=p.communicate(timeout=80)
  except subprocess.TimeoutExpired:
   import signal
   os.killpg(p.pid,signal.SIGTERM);stdout,stderr=p.communicate(timeout=5);raise AssertionError(('timeout',stdout,stderr[-2000:]))
  out=Path('artifacts/validation');out.mkdir(parents=True,exist_ok=True);(out/'actions-stdout.txt').write_text(stdout);(out/'actions-stderr.txt').write_text(stderr)
  rows=[json.loads(s.split(':',1)[1]) for s in stdout.splitlines() if s.startswith('FOUNDATION_SMOKE:')]
  assert p.returncode==0 and rows and all(r.get('ok') for r in rows),(p.returncode,rows,stderr[-2000:])
  assert len(requests)==8 and not errors,(requests,errors)
  md=list((root/'output').glob('*.md'));assert len(md)==1 and 'Document fixture' in md[0].read_text()
  assert not list((root/'output').glob('*.tmp'))
  assert sum('stop fixture' in p for p in requests)==1
  assert sum('partial fixture' in p for p in requests)==1
  with sqlite3.connect(root/'chat.sqlite3') as db:
   assert db.execute("SELECT answer FROM turns WHERE request_id='prior'").fetchone()==('old-answer',),'saved history changed'
   turns=db.execute('SELECT request_id,conversation_id,context_turns,state FROM turns WHERE request_id!="prior"').fetchall()
   assert len(turns)==6 and all(a==b and c==0 for a,b,c,d in turns),turns
   assert sorted(d for a,b,c,d in turns)==['complete','complete','complete','failed','failed','stopped'],turns
  (out/'actions.json').write_text(json.dumps({'passed':True,'requests':len(requests),'anchorMode':'synthetic-fixed','events':rows},ensure_ascii=False,indent=2))
  print(json.dumps({'passed':True,'requests':len(requests),'anchorMode':'synthetic-fixed','events':rows},ensure_ascii=False))
finally:server.shutdown();server.server_close()
