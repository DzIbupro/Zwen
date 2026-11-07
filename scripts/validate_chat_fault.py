"""Storage fault on an isolated DB; retain unsaved reply in native RAM across reload."""
import os,json,sqlite3,subprocess,sys,tempfile,threading,time,selectors
from pathlib import Path
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
release=threading.Event();unlocked=threading.Event();calls=[];folder=None
class Service(BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_POST(self):
  data=json.loads(self.rfile.read(int(self.headers['Content-Length'])));calls.append(data['messages'][-1]['content'])
  c=sqlite3.connect(Path(folder,'chat.sqlite3'));c.execute('BEGIN IMMEDIATE')
  self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
  self.wfile.write(('data: '+json.dumps({'choices':[{'index':0,'delta':{'content':'fault-partial'},'finish_reason':None}]})+'\n\n').encode());self.wfile.flush()
  release.wait(25);c.rollback();c.close();unlocked.set()
server=ThreadingHTTPServer(('127.0.0.1',0),Service);threading.Thread(target=server.serve_forever,daemon=True).start()
binary=Path(sys.argv[1]).resolve();out=Path('artifacts/validation');out.mkdir(parents=True,exist_ok=True)
try:
 with tempfile.TemporaryDirectory(prefix='zwen-chat-fault-') as folder:
  env=os.environ.copy();ack=Path(folder,'fault-ready');env.update(ZWEN_DATA_DIR=folder,ZWEN_PROVIDER_TEST_KEY='fixture-token',ZWEN_PROVIDER_TEST_BASE=f'http://127.0.0.1:{server.server_port}/v1',ZWEN_SMOKE='chat-fault',ZWEN_FIXTURE_ACK=str(ack))
  with (out/'chat-fault-stderr.txt').open('w') as err:
   p=subprocess.Popen([str(binary)],env=env,stdout=subprocess.PIPE,stderr=err,text=True,bufsize=1)
   selector=selectors.DefaultSelector();selector.register(p.stdout,selectors.EVENT_READ);lines=[];start=time.monotonic();staged=False
   while time.monotonic()-start<35:
    for key,_ in selector.select(.2):
     line=key.fileobj.readline()
     if line:lines.append(line)
     if line.startswith('FAULT_STAGE:') and not staged:
      release.set();assert unlocked.wait(3)
      db=Path(folder,'chat.sqlite3');db.rename(Path(folder,'original-chat-fixture.sqlite3'))
      for suffix in ('-wal','-shm'):
       f=Path(str(db)+suffix)
       if f.exists():f.rename(Path(str(db)+suffix+'.fixture-backup'))
      db.mkdir();ack.write_text('ready');staged=True
    if p.poll() is not None:break
   else:p.terminate()
   p.wait(timeout=5)
  raw=''.join(lines);(out/'chat-fault-stdout.txt').write_text(raw)
  rows=[json.loads(line.split(':',1)[1]) for line in raw.splitlines() if line.startswith('FOUNDATION_SMOKE:')]
  assert p.returncode==0 and any(r.get('ok') and r.get('ramRecovered') for r in rows),(p.returncode,raw)
  assert calls==['fault-fixture'],calls
  print('Unsaved RAM recovery / send guard / failed quit settlement PASS',rows)
finally:release.set();server.shutdown();server.server_close()
