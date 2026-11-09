"""Synthetic 60-page Poppler import + bounded function calls in isolated native UI."""
import json,os,subprocess,sys,tempfile,threading,time
from pathlib import Path
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer

def pdf(path):
    count=60;font=3+count*2
    objects=[b'<< /Type /Catalog /Pages 2 0 R >>',f'<< /Type /Pages /Kids [{" ".join(f"{3+2*i} 0 R" for i in range(count))}] /Count {count} >>'.encode()]
    for i in range(count):
        line=f'PAGE {i+1} SYNTHETIC EVIDENCE '+('representation improves under fixed conditions. '*2)
        content=('BT /F1 6 Tf 30 800 Td '+' '.join('('+line+') Tj 0 -9 Td' for _ in range(60))+' ET').encode()
        objects += [f'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 {font} 0 R >> >> /Contents {4+2*i} 0 R >>'.encode(),b'<< /Length '+str(len(content)).encode()+b' >>\nstream\n'+content+b'\nendstream']
    objects.append(b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>');data=b'%PDF-1.4\n';offsets=[0]
    for n,obj in enumerate(objects,1):offsets.append(len(data));data+=f'{n} 0 obj\n'.encode()+obj+b'\nendobj\n'
    xref=len(data);data+=f'xref\n0 {len(offsets)}\n0000000000 65535 f \n'.encode()+b''.join(f'{x:010d} 00000 n \n'.encode() for x in offsets[1:])+f'trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode();path.write_bytes(data)

requests=[];errors=[]
class Service(BaseHTTPRequestHandler):
    def log_message(self,*args):pass
    def do_POST(self):
        try:
            raw=self.rfile.read(int(self.headers['Content-Length']));assert len(raw)<=256*1024
            b=json.loads(raw);requests.append(b);assert len(b['tools'])==2 and b['model']=='long-fixture'
            question=[m['content'] for m in b['messages'] if m['role']=='user'][-1];has_tool=b['messages'][-1]['role']=='tool'
            assert '长论文按需读取' in question and 'PDF物理页 60' in question and '核心主张—证据表' in question
            assert 'representation improves under fixed conditions.' not in question
            self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
            def send(delta,reason=None):self.wfile.write(('data: '+json.dumps({'choices':[{'index':0,'delta':delta,'finish_reason':reason}]})+'\n\n').encode());self.wfile.flush()
            if not has_tool:
                pages=[60] if 'Explain last-page' in question else [1,60]
                for i,page in enumerate(pages):send({'tool_calls':[{'index':i,'id':f'read_{i}','type':'function','function':{'name':'read_paper_page','arguments':json.dumps({'page':page,'limit':8000})}}]})
                send({},'tool_calls')
            else:
                content='\n'.join(m['content'] for m in b['messages'] if m['role']=='tool');assert 'SYNTHETIC EVIDENCE' in content and 'representation improves under fixed conditions.' in content
                if 'STOP_LONG' in question:
                    for _ in range(20):send({'content':'Partial final text. '});time.sleep(.2)
                    send({},'stop')
                else:send({'content':'# Long fixture report\n\nPartial material, no invented full coverage.'},'stop')
            self.wfile.write(b'data: [DONE]\n\n');self.wfile.flush()
        except (BrokenPipeError,ConnectionResetError):pass
        except Exception as e:errors.append(type(e).__name__+': '+str(e))
server=ThreadingHTTPServer(('127.0.0.1',0),Service);threading.Thread(target=server.serve_forever,daemon=True).start()
out=Path('artifacts/validation/paper-long');out.mkdir(parents=True,exist_ok=True)
try:
    with tempfile.TemporaryDirectory(prefix='zwen-paper-long-') as folder:
        path=Path(folder)/'long.pdf';pdf(path)
        extracted=subprocess.run(['pdftotext',str(path),'-'],check=True,capture_output=True).stdout;assert len(extracted)>256*1024
        env=os.environ.copy();env.update(ZWEN_DATA_DIR=folder,ZWEN_SMOKE='paper-long',ZWEN_PROVIDER_TEST_KEY='synthetic',ZWEN_PROVIDER_TEST_BASE=f'http://127.0.0.1:{server.server_port}/v1',ZWEN_PAPER_FIXTURE_PDF=str(path),ZWEN_SMOKE_TIMEOUT_SECS='45')
        p=subprocess.run([str(Path(sys.argv[1]).resolve()),'show'],env=env,capture_output=True,text=True,timeout=55)
        (out/'stdout.txt').write_text(p.stdout);(out/'stderr.txt').write_text(p.stderr)
        rows=[json.loads(s.split(':',1)[1]) for s in p.stdout.splitlines() if s.startswith('FOUNDATION_SMOKE:')]
        (out/'model-diagnostics.json').write_text(json.dumps({'errors':errors,'requests':len(requests),'lastRoles':[m.get('role') for m in requests[-1]['messages']] if requests else []},indent=2))
        assert p.returncode==0 and any(r.get('ok') for r in rows),(p.returncode,rows,errors,len(requests),p.stderr[-800:]);assert not errors,errors
        assert 5<=len(requests)<=6,len(requests)
        (out/'result.json').write_text(json.dumps({'native':rows,'requests':len(requests),'realModel':False,'extractedBytes':len(extracted)},indent=2));print('Native long paper PASS:',rows)
finally:server.shutdown();server.server_close()
