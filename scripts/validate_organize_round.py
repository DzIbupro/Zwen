"""Temporary notes + loopback synthetic model only. No production export paths."""
import json,os,re,subprocess,sys,tempfile,threading,time
from pathlib import Path
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
assert subprocess.run(['pgrep','-x','zwen'],capture_output=True).returncode!=0,'Save and quit existing Zwen before isolated tests'
calls=[];errors=[]
class Service(BaseHTTPRequestHandler):
 def log_message(self,*a):pass
 def do_POST(self):
  try:
   data=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
   assert self.headers['Authorization']=='Bearer fixture-token'
   ids=re.findall(r'\[笔记 ([0-9a-f]{32})\]',data['messages'][-1]['content']);assert 1<=len(ids)<=8
   calls.append({'model':data['model'],'ids':ids});number=len(calls)
   if number==2:self.send_response(500);self.end_headers();return
   if number==5:time.sleep(2)
   answer=json.dumps({'topics':[{'name':'合成主题'}],'assign':[{'note':i,'topics':['合成主题']} for i in ids]},ensure_ascii=False)
   self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
   frame={'choices':[{'index':0,'delta':{'content':answer},'finish_reason':None}]}
   finish={'choices':[{'index':0,'delta':{},'finish_reason':'stop'}]}
   self.wfile.write(('data: '+json.dumps(frame)+'\n\ndata: '+json.dumps(finish)+'\n\ndata: [DONE]\n\n').encode());self.wfile.flush()
  except (BrokenPipeError,ConnectionResetError):pass
  except Exception as e:errors.append(str(e))
server=ThreadingHTTPServer(('127.0.0.1',0),Service);threading.Thread(target=server.serve_forever,daemon=True).start()
with tempfile.TemporaryDirectory(prefix='zwen-round-') as folder:
 env=os.environ.copy();env.update(ZWEN_DATA_DIR=folder,ZWEN_SMOKE='organize-round',ZWEN_SMOKE_TIMEOUT_SECS='90',ZWEN_PROVIDER_TEST_KEY='fixture-token',ZWEN_PROVIDER_TEST_BASE=f'http://127.0.0.1:{server.server_port}/v1')
 p=subprocess.Popen([str(Path(sys.argv[1]).resolve())],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
 try:stdout,stderr=p.communicate(timeout=100)
 finally:
  if p.poll() is None:p.terminate();p.wait(timeout=5)
 events=[json.loads(line.split(':',1)[1]) for line in stdout.splitlines() if line.startswith('FOUNDATION_SMOKE:')]
 print(json.dumps({'exit':p.returncode,'events':events,'calls':[{**c,'ids':len(c['ids'])} for c in calls],'errors':errors},ensure_ascii=False))
 assert p.returncode==0 and any(e.get('ok') for e in events),(stdout,stderr[-2000:])
 restarted=subprocess.run([str(Path(sys.argv[1]).resolve())],env=env,capture_output=True,text=True,timeout=30)
 restart_events=[json.loads(line.split(':',1)[1]) for line in restarted.stdout.splitlines() if line.startswith('FOUNDATION_SMOKE:')]
 print(json.dumps({'restartExit':restarted.returncode,'events':restart_events},ensure_ascii=False))
 assert restarted.returncode==0 and any(e.get('processRestart') for e in restart_events),(restarted.stdout,restarted.stderr[-1500:])
 assert not errors and len(calls)==5
 assert [len(c['ids']) for c in calls]==[8,8,8,1,8]
 assert all(c['model']=='fixed-model' for c in calls[:4]) and calls[4]['model']=='changed-model'
 server.shutdown()
