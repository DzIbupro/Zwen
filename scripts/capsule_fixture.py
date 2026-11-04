"""Loopback-only synthetic stream for native capsule geometry checks."""
import json,threading,time
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
class CapsuleFixture:
 def __enter__(self):
  self.requests=[];self.errors=[];owner=self
  class Service(BaseHTTPRequestHandler):
   def log_message(self,*args):pass
   def do_POST(self):
    try:
     data=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
     assert self.path=='/v1/chat/completions' and self.headers.get('Authorization')=='Bearer fixture-token'
     assert data['model']=='fixture-model' and data['messages']==[{'role':'user','content':'capsule-fixture'}],data
     owner.requests.append(data)
     self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
     text='## 合成服务回复\n\n| 项目 | 值 |\n|---|---|\n| fixture | geometry |\n\n公式 $E=mc^2$\n\n'+''.join(f'### {i}. 合成段落\n\n'+('仅用于验证胶囊增长和滚动。'*8)+'\n\n' for i in range(18))
     for pos in range(0,len(text),100):
      self.wfile.write(('data: '+json.dumps({'choices':[{'delta':{'content':text[pos:pos+100]},'finish_reason':None}]},ensure_ascii=False)+'\n\n').encode());self.wfile.flush();time.sleep(.06)
     self.wfile.write(b'data: {"choices":[{"delta":{},"finish_reason":"stop"}]}\n\ndata: [DONE]\n\n');self.wfile.flush()
    except (BrokenPipeError,ConnectionResetError):pass
    except Exception as e:owner.errors.append(repr(e))
  self.server=ThreadingHTTPServer(('127.0.0.1',0),Service);threading.Thread(target=self.server.serve_forever,daemon=True).start()
  self.env={'ZWEN_PROVIDER_TEST_KEY':'fixture-token','ZWEN_PROVIDER_TEST_BASE':f'http://127.0.0.1:{self.server.server_port}/v1'};return self
 def __exit__(self,kind,value,tb):
  self.server.shutdown();self.server.server_close()
  if kind is None:assert len(self.requests)==1 and not self.errors,(len(self.requests),self.errors)
