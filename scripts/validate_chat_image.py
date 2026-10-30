"""Synthetic vision-model chat over loopback HTTP. No real provider, no real screenshots."""
import base64,json,os,sqlite3,struct,subprocess,sys,tempfile,threading,time,zlib
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
from pathlib import Path
requests=[];server_errors=[]
ATTACH_ID='ab'*16

def marker_png(width=64,height=48):
 row=b'\x00'+bytes([80,120,90,255])*width
 raw=row*height
 def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
 return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',width,height,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(raw))+chunk(b'IEND',b'')

def frame(text=None,finish=None):
 delta={} if text is None else {'content':text}
 return ('data: '+json.dumps({'choices':[{'index':0,'delta':delta,'finish_reason':finish}]},ensure_ascii=False)+'\r\n\r\n').encode()
class Service(BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_POST(self):
  try:
   assert self.path=='/v1/chat/completions'
   assert self.headers.get('Authorization')=='Bearer fixture-token'
   length=int(self.headers['Content-Length']);data=json.loads(self.rfile.read(length))
   last=data['messages'][-1];model=data['model'];assert data['stream']
   if not requests: # vision turn: content must be the official parts array
    assert model=='fixture-vision',model
    assert len(data['messages'])==1
    content=last['content'];assert isinstance(content,list) and len(content)==2,content
    assert content[0]['type']=='image_url' and content[1]['type']=='text',content
    payload=base64.b64decode(content[0]['image_url']['url'],validate=True)
    assert payload[:8]==b'\x89PNG\r\n\x1a\n'
    assert struct.unpack('>IIBBBBB',payload[16:29])[0:2]==(64,48),struct.unpack('>IIBBBBB',payload[16:29])
    ihdr_end=33;pos=payload.index(b'IDAT');idat_length=struct.unpack('>I',payload[pos-4:pos])[0]
    pixels=zlib.decompress(payload[pos+4:pos+4+idat_length])
    assert pixels[1:5]==bytes([80,120,90,255]),'marker pixels did not travel'
    assert content[1]['text']=='这张合成截图里有什么？'
    requests.append({'model':model,'text':content[1]['text'],'imageBytes':len(payload)})
   else: # text follow-up in the same conversation: plain string content
    assert model=='fixture-model',model
    assert len(data['messages'])==3,data['messages']
    assert last['content']=='complete-fixture'
    assert isinstance(last['content'],str)
    requests.append({'model':model,'text':last['content']})
   self.send_response(200);self.send_header('Content-Type','text/event-stream; charset=utf-8');self.end_headers()
   answer='合成图片已收到：这是合成视觉服务的回答。' if model=='fixture-vision' else '纯文本回答'
   self.wfile.write(frame(answer))
   self.wfile.write(frame(finish='stop')+b'data: [DONE]\n\n');self.wfile.flush()
  except (BrokenPipeError,ConnectionResetError):pass
  except Exception as exc:server_errors.append(f'{type(exc).__name__}: {exc}')
server=ThreadingHTTPServer(('127.0.0.1',0),Service);threading.Thread(target=server.serve_forever,daemon=True).start()
binary=Path(sys.argv[1]).resolve();out=Path('artifacts/validation');out.mkdir(parents=True,exist_ok=True)
try:
 with tempfile.TemporaryDirectory(prefix='zwen-chat-image-') as folder:
  root=Path(folder);png=marker_png()
  attachments=root/'attachments';attachments.mkdir()
  (attachments/f'{ATTACH_ID}.png').write_bytes(png)
  db=sqlite3.connect(root/'attachment.sqlite3')
  db.executescript("CREATE TABLE attachments(id TEXT PRIMARY KEY,width INTEGER NOT NULL,height INTEGER NOT NULL,bytes INTEGER NOT NULL,created_at INTEGER NOT NULL);PRAGMA user_version=1;")
  db.execute('INSERT INTO attachments VALUES(?,?,?,?,?)',(ATTACH_ID,64,48,len(png),123));db.commit();db.close()
  db=sqlite3.connect(root/'state.sqlite3')
  db.executescript("CREATE TABLE drafts(id INTEGER PRIMARY KEY CHECK(id=1),text TEXT NOT NULL,revision INTEGER NOT NULL CHECK(revision>=0));INSERT INTO drafts VALUES(1,'合成旧草稿',0);PRAGMA user_version=1;");db.close()
  env=os.environ.copy()
  env.update(ZWEN_DATA_DIR=folder,ZWEN_PROVIDER_TEST_KEY='fixture-token',ZWEN_PROVIDER_TEST_BASE=f'http://127.0.0.1:{server.server_port}/v1',ZWEN_SMOKE='chat-image',ZWEN_SMOKE_TIMEOUT_SECS='90')
  p=subprocess.Popen([str(binary)],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
  try:stdout,stderr=p.communicate(timeout=60)
  except subprocess.TimeoutExpired:p.kill();stdout,stderr=p.communicate();raise
  (out/'chat-image-stdout.txt').write_text(stdout);(out/'chat-image-stderr.txt').write_text(stderr)
  rows=[json.loads(line.split(':',1)[1]) for line in stdout.splitlines() if line.startswith('FOUNDATION_SMOKE:')]
  assert p.returncode==0 and rows and rows[-1].get('ok'),(p.returncode,rows,stderr[-1500:],requests,server_errors)
  assert len(requests)==2 and not server_errors,(requests,server_errors)
  assert requests[0]['model']=='fixture-vision' and requests[1]['model']=='fixture-model',requests
  with sqlite3.connect(root/'chat.sqlite3') as c:
   turns=c.execute('SELECT request_id,model,state,answer FROM turns ORDER BY id').fetchall()
  assert len(turns)==2 and turns[0][1]=='fixture-vision' and turns[1][1]=='fixture-model',turns
  assert all(state=='complete' for _,_,state,_ in turns),turns
  with sqlite3.connect(root/'attachment.sqlite3') as c:
   assert c.execute('SELECT count(*) FROM attachments').fetchone()[0]==0,'attachments were not consumed'
  assert not list(attachments.glob('*.png')),'attachment files were not consumed'
  with sqlite3.connect(root/'state.sqlite3') as c:
   assert c.execute('SELECT text FROM drafts').fetchone()[0]=='','draft was not cleared after accepted sends'
  for path in Path(folder).glob('*.sqlite3*'):assert b'fixture-token' not in path.read_bytes()
  summary={'passed':True,'exit':p.returncode,'requests':requests,'turns':[t[1] for t in turns]}
  (out/'chat-image.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2))
  print(json.dumps(summary,ensure_ascii=False))
  print('Vision routing, bounded payload, attachment consumption, text follow-up PASS; no key persisted')
finally:server.shutdown();server.server_close()
