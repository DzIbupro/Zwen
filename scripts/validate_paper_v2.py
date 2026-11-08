"""Real Poppler/native UI with owned synthetic PDF, isolated data and loopback model."""
import json,os,subprocess,sys,tempfile,threading,time
from pathlib import Path
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer

def pdf(path):
    objects=[b'<< /Type /Catalog /Pages 2 0 R >>',b'<< /Type /Pages /Kids [3 0 R 5 0 R 7 0 R] /Count 3 >>']
    for idx,text in enumerate(['FIRST_PAGE_EVIDENCE Method improves representation.','LAST_PAGE_RESULT Experiment EER 0.65 percent.','']):
        content=('BT /F1 12 Tf 50 720 Td ('+text+') Tj ET').encode()
        objects.extend([f'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 9 0 R >> >> /Contents {4+idx*2} 0 R >>'.encode(),b'<< /Length '+str(len(content)).encode()+b' >>\nstream\n'+content+b'\nendstream'])
    objects.append(b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>')
    data=b'%PDF-1.4\n';offsets=[0]
    for i,obj in enumerate(objects,1):offsets.append(len(data));data+=str(i).encode()+b' 0 obj\n'+obj+b'\nendobj\n'
    xref=len(data);data+=f'xref\n0 {len(offsets)}\n0000000000 65535 f \n'.encode()
    data+=b''.join(f'{n:010d} 00000 n \n'.encode() for n in offsets[1:]);data+=f'trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode();path.write_bytes(data)

binary=Path(sys.argv[1]).resolve();requests=[];errors=[]
class Service(BaseHTTPRequestHandler):
    def log_message(self,*args):pass
    def do_POST(self):
        try:
            body=json.loads(self.rfile.read(int(self.headers['Content-Length'])));requests.append(body)
            last=body['messages'][-1]['content']
            assert 'FIRST_PAGE_EVIDENCE' in last and 'LAST_PAGE_RESULT' in last
            assert '第3页无可提取文字' in last and 'PDF物理页 2' in last
            assert body.get('response_format') is None and all(isinstance(m['content'],str) for m in body['messages'])
            self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
            text='# fixture report\n\n| Claim | Evidence |\n|---|---|\n| improvement | page 2 |\n\nUnverified figure. trailing } is ordinary text.'
            for i in range(0,len(text),18):
                payload={'choices':[{'delta':{'content':text[i:i+18]},'finish_reason':None}]}
                self.wfile.write(('data: '+json.dumps(payload)+'\n\n').encode());self.wfile.flush();time.sleep(.3 if 'slow request' in last else .02)
            self.wfile.write(b'data: {"choices":[{"delta":{},"finish_reason":"stop"}]}\n\ndata: [DONE]\n\n');self.wfile.flush()
        except (BrokenPipeError,ConnectionResetError):pass
        except Exception as e:errors.append(repr(e))
server=ThreadingHTTPServer(('127.0.0.1',0),Service);threading.Thread(target=server.serve_forever,daemon=True).start()
out=Path('artifacts/validation/paper-v2');out.mkdir(parents=True,exist_ok=True)
try:
    with tempfile.TemporaryDirectory(prefix='zwen-paper-v2-native-') as folder:
        path=Path(folder)/'fixture.pdf';pdf(path)
        env=os.environ.copy();env.update(ZWEN_DATA_DIR=folder,ZWEN_SMOKE='paper-v2',ZWEN_PROVIDER_TEST_KEY='synthetic-paper-key',ZWEN_PROVIDER_TEST_BASE=f'http://127.0.0.1:{server.server_port}/v1',ZWEN_PAPER_FIXTURE_PDF=str(path),ZWEN_SMOKE_TIMEOUT_SECS='45')
        p=subprocess.run([str(binary),'show'],env=env,capture_output=True,text=True,timeout=55)
        (out/'stdout.txt').write_text(p.stdout);(out/'stderr.txt').write_text(p.stderr)
        rows=[json.loads(s.split(':',1)[1]) for s in p.stdout.splitlines() if s.startswith('FOUNDATION_SMOKE:')]
        assert p.returncode==0 and any(r.get('ok') for r in rows),(p.returncode,rows,p.stderr[-1800:])
        assert len(requests)==3 and not errors,(len(requests),errors)
        assert len(requests[0]['messages'])==1 and len(requests[1]['messages'])==3
        (out/'result.json').write_text(json.dumps({'native':rows,'requests':len(requests),'realModel':False},indent=2))
        print('Native paper v2 PASS:',rows)
finally:server.shutdown();server.server_close()
