"""Local synthetic HTTP service + isolated native windows/databases. No real provider."""
import json,os,sqlite3,subprocess,sys,tempfile,threading,time
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
from pathlib import Path
requests=[];server_errors=[]
def frame(text=None,finish=None):
 delta={} if text is None else {'content':text}
 return ('data: '+json.dumps({'choices':[{'index':0,'delta':delta,'finish_reason':finish}]},ensure_ascii=False)+'\r\n\r\n').encode()
class Service(BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_POST(self):
  try:
   assert self.path=='/v1/chat/completions'
   assert self.headers.get('Authorization')=='Bearer fixture-token'
   length=int(self.headers['Content-Length']);assert length<300000
   data=json.loads(self.rfile.read(length));prompt=data['messages'][-1]['content'];assert data['stream'] and data['model']=='fixture-model';requests.append(prompt)
   if prompt=='stop-fixture':assert len(data['messages'])==3 and data['messages'][0]['content']=='complete-fixture'
   if prompt=='partial-restart':assert len(data['messages'])==5 and data['messages'][3]['content']=='stoppable-partial'
   self.send_response(200);self.send_header('Content-Type','text/event-stream; charset=utf-8');self.end_headers()
   if prompt.startswith('stress-'):
    number=int(prompt.split('-')[-1]);text='## Long answer fixture\n\n'+('正文用于验证资源占用和正常公式。 $E=mc^2$\n\n| value |\n|---|\n| fixture |\n\n'*STRESS_REPEAT)+f'stress-finished-{number}'
    self.wfile.write(frame(text)+frame(finish='stop')+b'data: [DONE]\n\n');self.wfile.flush()
   elif prompt=='complete-fixture':
    text='## Fixture\n\n| value |\n|---|\n| fixture |\n\n$E=mc^2$\n\n中文 🙂'
    payload=frame(text)+frame(finish='stop')+b'data: [DONE]\n\n'
    for n in range(0,len(payload),3):self.wfile.write(payload[n:n+3])
    self.wfile.flush()
   elif prompt in ('stop-fixture','partial-restart'):
    self.wfile.write(frame('stoppable-partial' if prompt=='stop-fixture' else 'durable-partial'));self.wfile.flush();time.sleep(5)
    self.wfile.write(frame('late-must-not-appear')+frame(finish='stop')+b'data: [DONE]\n\n');self.wfile.flush()
   else:raise AssertionError('Unexpected synthetic prompt')
  except (BrokenPipeError,ConnectionResetError):pass
  except Exception as exc:server_errors.append(type(exc).__name__)
server=ThreadingHTTPServer(('127.0.0.1',0),Service);thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
binary=Path(sys.argv[1]).resolve();out=Path('artifacts/validation');out.mkdir(parents=True,exist_ok=True)
def memory(pid,stop,samples):
 known={pid}
 while not stop.is_set():
  pending=[pid];alive=set()
  while pending:
   current=pending.pop()
   if current in alive:continue
   alive.add(current)
   try:pending.extend(int(v) for v in Path(f'/proc/{current}/task/{current}/children').read_text().split())
   except OSError:pass
  known.update(alive);pss=rss=0
  for current in known:
   try:
    values={line.split(':',1)[0]:int(line.split()[1]) for line in Path(f'/proc/{current}/smaps_rollup').read_text().splitlines() if line.startswith(('Pss:','Rss:'))}
    pss+=values.get('Pss',0);rss+=values.get('Rss',0)
   except OSError:pass
  samples.append({'pssMiB':round(pss/1024,2),'rssMiB':round(rss/1024,2)})
  stop.wait(.05)
stress=len(sys.argv)>2 and sys.argv[2]=='--stress'
STRESS_REPEAT=int(sys.argv[3]) if stress and len(sys.argv)>3 else 90
assert 1<=STRESS_REPEAT<=200
try:
 with tempfile.TemporaryDirectory(prefix='zwen-chat-native-') as folder:
  env=os.environ.copy();env.update(ZWEN_DATA_DIR=folder,ZWEN_PROVIDER_TEST_KEY='fixture-token',ZWEN_PROVIDER_TEST_BASE=f'http://127.0.0.1:{server.server_port}/v1')
  for phase in (('stress',) if stress else ('chat','chat-recover')):
   env['ZWEN_SMOKE']=phase
   p=subprocess.Popen([str(binary),*(['capsule'] if phase in ('chat-recover','stress') else [])],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
   stopped=threading.Event();samples=[];monitor=threading.Thread(target=memory,args=(p.pid,stopped,samples));monitor.start()
   try:stdout,stderr=p.communicate(timeout=40)
   except subprocess.TimeoutExpired:p.kill();stdout,stderr=p.communicate();raise
   finally:stopped.set();monitor.join()
   if samples:
    stats={'kind':'synthetic-smoke-50ms-sampling','peakPssMiB':max(s['pssMiB'] for s in samples),'peakRssMiB':max(s['rssMiB'] for s in samples),'samples':samples}
    (out/f'{phase}-memory.json').write_text(json.dumps(stats));print(phase,'memory',stats['peakPssMiB'],stats['peakRssMiB'])
   (out/f'{phase}-stdout.txt').write_text(stdout);(out/f'{phase}-stderr.txt').write_text(stderr)
   rows=[json.loads(line.split(':',1)[1]) for line in stdout.splitlines() if line.startswith('FOUNDATION_SMOKE:')]
   assert p.returncode==0 and rows and all(r.get('ok') for r in rows),(phase,p.returncode,stdout,stderr)
   if phase=='chat':
    assert any(r.get('mode')=='chat-store-phase1' for r in rows),rows
    with sqlite3.connect(Path(folder,'chat.sqlite3')) as c:
     state,answer=c.execute('SELECT state,answer FROM turns ORDER BY id DESC LIMIT 1').fetchone();assert (state,answer)==('streaming','durable-partial'),(state,answer)
   print(phase,'PASS',rows)
  with sqlite3.connect(Path(folder,'chat.sqlite3')) as c:
   rows=c.execute('SELECT prompt,state,answer FROM turns ORDER BY id').fetchall();assert (len(rows)==20 and all(r[1]=='complete' for r in rows)) if stress else (len(rows)==3 and rows[-1]==('partial-restart','interrupted','durable-partial')),rows
  assert requests==([f'stress-{i}' for i in range(20)] if stress else ['complete-fixture','stop-fixture','partial-restart']),requests
  assert not server_errors,server_errors
  for path in Path(folder).glob('*.sqlite3*'):assert b'fixture-token' not in path.read_bytes()
  print('Chat request/stop/renderer reload/process recovery PASS; no key persisted; no automatic retry')
finally:server.shutdown();server.server_close()
