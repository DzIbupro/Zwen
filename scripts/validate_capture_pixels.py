"""Pixel proof inside an isolated nested Niri containing only synthetic Zwen.
Never captures the host desktop or reads/writes its clipboard.
"""
import json,os,struct,subprocess,sys,tempfile,time,zlib
from pathlib import Path

def pixels(path):
 b=path.read_bytes();assert b[:8]==b'\x89PNG\r\n\x1a\n';pos=8;packed=b''
 while pos<len(b):
  n=struct.unpack('>I',b[pos:pos+4])[0];kind=b[pos+4:pos+8];data=b[pos+8:pos+8+n];pos+=n+12
  if kind==b'IHDR':w,h,depth,color,_,_,interlace=struct.unpack('>IIBBBBB',data);assert depth==8 and color in (2,6) and interlace==0
  if kind==b'IDAT':packed+=data
 raw=zlib.decompress(packed);channels=3 if color==2 else 4;stride=w*channels;previous=bytearray(stride);result=[];offset=0
 for y in range(h):
  filt=raw[offset];offset+=1;row=bytearray(raw[offset:offset+stride]);offset+=stride
  for i in range(stride):
   a=row[i-channels] if i>=channels else 0;b=previous[i];c=previous[i-channels] if i>=channels else 0
   p=a+b-c;pa,pb,pc=abs(p-a),abs(p-b),abs(p-c);predict=a if pa<=pb and pa<=pc else b if pb<=pc else c
   row[i]=(row[i]+(0 if filt==0 else a if filt==1 else b if filt==2 else (a+b)//2 if filt==3 else predict))&255
  result.extend(tuple(row[i:i+3]) for i in range(0,stride,channels));previous=row
 return result

def inside(binary,root):
 assert os.environ['NIRI_SOCKET']!=os.environ.get('ZWEN_PARENT_NIRI_SOCKET'),'must use nested compositor'
 outputs=json.loads(subprocess.check_output(['niri','msg','-j','outputs']));logical=next(o['logical'] for o in outputs.values() if o.get('logical'));region=f"{logical['x']},{logical['y']} {logical['width']}x{logical['height']}"
 helpers=root/'bin';helpers.mkdir();selector=helpers/'slurp';selector.write_text('#!/bin/sh\nprintf \'%s\\n\' \''+region+"'\n");selector.chmod(0o700)
 env=os.environ.copy();env.update(ZWEN_SMOKE='capture-pixels',ZWEN_SMOKE_TIMEOUT_SECS='45',ZWEN_DATA_DIR=str(root/'data'),PATH=str(helpers)+':'+env['PATH'])
 p=subprocess.Popen([str(binary)],env=env,stdout=subprocess.PIPE,stderr=(root/'zwen.log').open('w'),text=True);events=[];proofs=[]
 try:
  for line in p.stdout:
   if line.startswith('CAPTURE_PIXEL_STAGE:'):
    phase=json.loads(line.split(':',1)[1])['phase'];assert phase in ['baseline','restored-full','restored-capsule','restored-hidden'];subprocess.run(['grim','-g',region,str(root/(phase+'.png'))],check=True,timeout=5)
   if line.startswith('CAPTURE_FRAME_PROOF:'):proofs.append(json.loads(line.split(':',1)[1]))
   if line.startswith('FOUNDATION_SMOKE:'):events.append(json.loads(line.split(':',1)[1]))
  p.wait(timeout=5);assert p.returncode==0 and events[-1].get('ok'),events
  baseline=pixels(root/'baseline.png');assert sum(c[0]>200 and c[1]<40 for c in baseline)>100,'baseline window never rendered'
  for phase in ['restored-full','restored-capsule','restored-hidden']:
   assert sum(c[0]>200 and c[1]<40 for c in pixels(root/(phase+'.png')))>100,phase+' UI was not restored'
  counts=[]
  for id in events[-1]['ids']:
   image=pixels(root/'data'/'attachments'/f'{id}.png');bad=sum(any(abs(a-b)>1 for a,b in zip(c,(18,52,86))) for c in image);counts.append(bad)
  assert counts==[0,0,0],counts
  assert proofs and all(x.get('complete') and x.get('alphaMax')==0 for x in proofs),proofs
  result={'passed':True,'nonBackgroundPixels':counts,'scenes':['full','capsule','already-hidden'],'restoredUI':True,'actualScale':logical['scale'],'proofs':proofs};(root/'result.json').write_text(json.dumps(result))
 except Exception as e:(root/'result.json').write_text(json.dumps({'passed':False,'error':str(e),'events':events,'proofs':proofs}))
 finally:
  if p.poll() is None:p.terminate();p.wait(timeout=5)
  subprocess.run(['niri','msg','action','quit','--skip-confirmation'],capture_output=True,timeout=5)

if len(sys.argv)>1 and sys.argv[1]=='--inside':inside(Path(sys.argv[2]),Path(sys.argv[3]));raise SystemExit()
binary=Path(sys.argv[1]).resolve();assert subprocess.run(['pgrep','-x','zwen'],capture_output=True).returncode!=0,'Save and quit user Zwen first'
with tempfile.TemporaryDirectory(prefix='zwen-nested-pixels-') as folder:
 root=Path(folder);config=root/'config.kdl';scale=sys.argv[2] if len(sys.argv)>2 else '1';assert scale in ('1','1.5');config.write_text('output "winit" { scale '+scale+'; }\n'+'layout { background-color "#123456"; focus-ring { off; }; border { off; }; shadow { off; }; }\nanimations { window-close { duration-ms 1500; curve "ease-out-cubic"; }; }\nhotkey-overlay { skip-at-startup; }\n')
 env=os.environ.copy();env['ZWEN_PARENT_NIRI_SOCKET']=env.get('NIRI_SOCKET','')
 p=subprocess.Popen(['niri','-c',str(config),'--','/usr/bin/python3',str(Path(__file__).resolve()),'--inside',str(binary),str(root)],env=env,stdout=(root/'niri.log').open('w'),stderr=subprocess.STDOUT,start_new_session=True)
 try:
  began=time.monotonic()
  while not (root/'result.json').exists() and p.poll() is None and time.monotonic()-began<55:time.sleep(.1)
  assert (root/'result.json').exists(),(root/'niri.log').read_text()[-2000:]
  result=json.loads((root/'result.json').read_text());result['scale']=scale;print(json.dumps(result));assert result['passed'],result;assert abs(result['actualScale']-float(scale))<.001,result
  Path('artifacts/validation').mkdir(parents=True,exist_ok=True);Path('artifacts/validation/capture-pixels-'+scale+'.json').write_text(json.dumps(result,indent=2))
 finally:
  if p.poll() is None:p.terminate()
  try:p.wait(timeout=5)
  except subprocess.TimeoutExpired:p.kill();p.wait(timeout=5)
