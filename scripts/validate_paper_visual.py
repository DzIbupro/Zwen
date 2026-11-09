"""20 actual PNG pages in request, including vector formula/diagram without text."""
import base64,io,json,os,subprocess,sys,tempfile,threading,time
from pathlib import Path
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
import gi
gi.require_version('GdkPixbuf','2.0')
from gi.repository import GdkPixbuf

def pdf(path):
    count=20;font=count*2+3
    objects=[b'<< /Type /Catalog /Pages 2 0 R >>',f'<< /Type /Pages /Kids [{" ".join(f"{3+2*i} 0 R" for i in range(count))}] /Count {count} >>'.encode()]
    for i in range(count):
        color=(i+1)/20
        diagram=f'{color} 0 1 rg 50 500 150 80 re f 0 1 0 rg 350 500 150 80 re f 0 0 0 RG 2 w 200 540 m 350 540 l 340 535 l 350 540 m 340 545 l S '
        # Visible formula is vector ink, independent of substitute system fonts.
        formula='0 0 0 RG 2 w 60 710 m 80 730 l 60 730 m 80 710 l 95 720 m 115 720 l 105 710 m 105 730 l 130 730 m 140 720 l 150 730 l 140 720 m 140 710 l 165 716 m 185 716 l 165 724 m 185 724 l 200 730 m 220 730 l 200 710 l 220 710 l S '
        # An invisible text layer on 19 pages simulates OCR-assisted PDFs. Last
        # page has no text layer; every page still has a visible full-page graph.
        text=f'BT /F1 18 Tf 3 Tr 50 790 Td (PHYSICAL PAGE {i+1}) Tj ET ' if i<19 else ''
        content=(diagram+formula+text).encode()
        objects += [f'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 {font} 0 R >> >> /Contents {4+2*i} 0 R >>'.encode(),b'<< /Length '+str(len(content)).encode()+b' >>\nstream\n'+content+b'\nendstream']
    objects.append(b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>');data=b'%PDF-1.4\n';offsets=[0]
    for n,obj in enumerate(objects,1):offsets.append(len(data));data+=f'{n} 0 obj\n'.encode()+obj+b'\nendobj\n'
    xref=len(data);data+=f'xref\n0 {len(offsets)}\n0000000000 65535 f \n'.encode()+b''.join(f'{x:010d} 00000 n \n'.encode() for x in offsets[1:])+f'trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode();path.write_bytes(data)

def check_page(data,page):
    assert data.startswith(b'\x89PNG\r\n\x1a\n')
    loader=GdkPixbuf.PixbufLoader.new_with_type('png');loader.write(data);loader.close();p=loader.get_pixbuf();w,h=p.get_width(),p.get_height();assert h==3000 and 2100<w<2130,(w,h)
    pixels=p.get_pixels();stride=p.get_rowstride();channels=p.get_n_channels()
    def pixel(x,y):k=int((842-y)*h/842)*stride+int(x*w/595)*channels;return pixels[k:k+3]
    rgb=pixel(100,565);assert abs(rgb[0]-round(page*255/20))<=3 and rgb[1]<3 and rgb[2]>252,(page,list(rgb))
    ink=0
    for y in range(705,742,2):
        for x in range(50,450,2):
            if max(pixel(x,y))<80:ink+=1
    assert ink>10,(page,'formula absent',ink)

requests=[];errors=[]
class Service(BaseHTTPRequestHandler):
    def log_message(self,*args):pass
    def do_POST(self):
        try:
            raw=self.rfile.read(int(self.headers['Content-Length']));assert len(raw)<=16*1024*1024
            b=json.loads(raw);requests.append(b);assert b['model']=='qwen3.7-flash'
            last=b['messages'][-1]['content']
            if isinstance(last,list):
                images=[p['image_url']['url'] for p in last if p.get('type')=='image_url'];assert len(images)==20
                text=next(p['text'] for p in last if p.get('type')=='text');assert '共20张' in text and '图像20对应PDF物理页20' in text
                for page,url in enumerate(images,1):
                    assert url.startswith('data:image/png;base64,');data=base64.b64decode(url.split(',',1)[1],validate=True);check_page(data,page)
                    if len(requests)==1 and page in [1,20]:(out/f'page-{page}.png').write_bytes(data)
                assert '不得仅凭正文重建后冒充看过原图' in text
            else:assert 'TEXT_ONLY_CONTROL' in last and '图表图像未读取' in last
            self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
            text='# Visual fixture report\n\nInput contains complete pages and visible formula/diagram.'
            for i in range(0,len(text),18):
                self.wfile.write(('data: '+json.dumps({'choices':[{'delta':{'content':text[i:i+18]},'finish_reason':None}]})+'\n\n').encode());self.wfile.flush();time.sleep(.02)
            self.wfile.write(b'data: {"choices":[{"delta":{},"finish_reason":"stop"}]}\n\ndata: [DONE]\n\n');self.wfile.flush()
        except (BrokenPipeError,ConnectionResetError):pass
        except Exception as e:errors.append(type(e).__name__+': '+str(e))
server=ThreadingHTTPServer(('127.0.0.1',0),Service);threading.Thread(target=server.serve_forever,daemon=True).start();out=Path('artifacts/validation/paper-visual');out.mkdir(parents=True,exist_ok=True)
try:
    with tempfile.TemporaryDirectory(prefix='zwen-paper-visual-') as folder:
        path=Path(folder)/'visual.pdf';pdf(path)
        env=os.environ.copy();env.update(ZWEN_DATA_DIR=folder,ZWEN_SMOKE='paper-visual',ZWEN_PROVIDER_TEST_KEY='synthetic',ZWEN_PROVIDER_TEST_BASE=f'http://127.0.0.1:{server.server_port}/v1',ZWEN_PAPER_FIXTURE_PDF=str(path),ZWEN_SMOKE_TIMEOUT_SECS='80')
        p=subprocess.run([str(Path(sys.argv[1]).resolve()),'show'],env=env,capture_output=True,text=True,timeout=90);(out/'stdout.txt').write_text(p.stdout);(out/'stderr.txt').write_text(p.stderr)
        rows=[json.loads(s.split(':',1)[1]) for s in p.stdout.splitlines() if s.startswith('FOUNDATION_SMOKE:')]
        (out/'diagnostics.json').write_text(json.dumps({'errors':errors,'requests':len(requests),'native':rows},indent=2))
        assert p.returncode==0 and any(r.get('ok') for r in rows),(p.returncode,rows,errors);assert not errors,errors;assert len(requests)==4,len(requests)
        (out/'result.json').write_text(json.dumps({'native':rows,'requests':4,'realModel':False,'pageCountPerVisualRequest':20,'imageHeight':3000,'pixelChecks':True},indent=2));print('Native visual paper PASS:',rows)
finally:server.shutdown();server.server_close()
