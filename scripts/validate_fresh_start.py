"""Cold starts use a new conversation, without deleting history or an unsent draft."""
import json,os,sqlite3,subprocess,sys,tempfile,threading
from pathlib import Path
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
assert subprocess.run(['pgrep','-x','zwen'],capture_output=True).returncode!=0,'Close existing Zwen first'
requests=[];errors=[]
class Service(BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_POST(self):
  try:
   d=json.loads(self.rfile.read(int(self.headers['Content-Length'])));assert self.path=='/v1/chat/completions' and self.headers.get('Authorization')=='Bearer fixture-token'
   assert d['messages']==[{'role':'user','content':'fresh-fixture'}] and d['model']=='fixture-model','previous context leaked';requests.append(d)
   self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers();self.wfile.write(b'data: {"choices":[{"delta":{"content":"new synthetic answer"},"finish_reason":null}]}\n\ndata: {"choices":[{"delta":{},"finish_reason":"stop"}]}\n\ndata: [DONE]\n\n');self.wfile.flush()
  except Exception as e:errors.append(repr(e))
server=ThreadingHTTPServer(('127.0.0.1',0),Service);threading.Thread(target=server.serve_forever,daemon=True).start()
try:
 with tempfile.TemporaryDirectory(prefix='zwen-fresh-start-') as folder:
  root=Path(folder);db=sqlite3.connect(root/'chat.sqlite3');db.executescript("CREATE TABLE turns(id INTEGER PRIMARY KEY,request_id TEXT NOT NULL UNIQUE,conversation_id TEXT NOT NULL,provider_profile TEXT NOT NULL,context_turns INTEGER NOT NULL,prompt TEXT NOT NULL,model TEXT NOT NULL,answer TEXT NOT NULL DEFAULT '',revision INTEGER NOT NULL DEFAULT 0 CHECK(revision>=0),state TEXT NOT NULL DEFAULT 'streaming',error TEXT,created_at INTEGER NOT NULL DEFAULT (unixepoch()));PRAGMA user_version=2;")
  db.execute("INSERT INTO turns(request_id,conversation_id,provider_profile,context_turns,prompt,model,answer,revision,state) VALUES('prior','prior','p',0,'以前的问题','fixture-model','以前的回答',1,'complete')");db.commit();db.close()
  env=os.environ.copy();env.update(ZWEN_DATA_DIR=folder,ZWEN_SMOKE='fresh-start',ZWEN_PROVIDER_TEST_KEY='fixture-token',ZWEN_PROVIDER_TEST_BASE=f'http://127.0.0.1:{server.server_port}/v1')
  events=[]
  for phase in [1,2]:
   p=subprocess.run([str(Path(sys.argv[1]).resolve())],env=env,capture_output=True,text=True,timeout=35)
   rows=[json.loads(s.split(':',1)[1]) for s in p.stdout.splitlines() if s.startswith('FOUNDATION_SMOKE:')]
   assert p.returncode==0 and all(r.get('ok') for r in rows),(phase,p.returncode,rows,p.stderr[-1500:])
   if phase==2:assert rows and rows[-1].get('freshOnRestart')
   events+=rows
  assert len(requests)==1 and not errors,(requests,errors)
  with sqlite3.connect(root/'chat.sqlite3') as db:
   rows=db.execute('SELECT request_id,conversation_id,context_turns,prompt,answer FROM turns ORDER BY id').fetchall();assert len(rows)==2 and rows[0][3:]==('以前的问题','以前的回答') and rows[1][0]==rows[1][1] and rows[1][2]==0,rows
  result={'passed':True,'requests':1,'coldProcesses':2,'events':events};out=Path('artifacts/validation');out.mkdir(parents=True,exist_ok=True);(out/'fresh-start.json').write_text(json.dumps(result,ensure_ascii=False,indent=2));print(json.dumps(result,ensure_ascii=False))
finally:server.shutdown();server.server_close()
