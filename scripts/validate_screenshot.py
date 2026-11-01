"""Synthetic Niri screenshot flow. No real screenshot, screen read or user data."""
import json,os,selectors,signal,sqlite3,struct,subprocess,sys,tempfile,time,zlib
from pathlib import Path
binary=Path(sys.argv[1]).resolve()
assert subprocess.run(['pgrep','-x','zwen'],capture_output=True).returncode!=0,'Save and quit existing Zwen before isolated native tests'
root=Path(tempfile.mkdtemp(prefix='zwen-screenshot-'))
helpers=root/'bin';helpers.mkdir()
fake=helpers/'niri'
fake.write_text('''#!/usr/bin/python3
import pathlib,shutil,subprocess,sys,time
root=pathlib.Path(__file__).resolve().parent.parent
args=sys.argv[1:]
if args[:2]==['msg','event-stream']:
    print('{"ConfigLoaded":{}}',flush=True)
    time.sleep(30)
    sys.exit(0)
if args[:2]==['msg','-j']:
    sys.exit(2)
if args[:3]==['msg','action','screenshot']:
    counter=root/'counter'
    n=int(counter.read_text())+1 if counter.exists() else 1
    counter.write_text(str(n))
    path=args[-1]
    if n in (1,2):
        shutil.copyfile(root/'image.png',path)
    elif n==3:
        time.sleep(.8)
        shutil.copyfile(root/'image.png',path)
    elif n==4:
        subprocess.Popen([sys.executable,'-c','import shutil,sys,time;time.sleep(1.2);shutil.copyfile(sys.argv[1],sys.argv[2])',str(root/'image.png'),path],start_new_session=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    else:
        shutil.copyfile(root/'image.png',path)
    sys.exit(0)
sys.exit(2)
''')
fake.chmod(0o700)

def png(width,height):
 row=b'\x00'+bytes([80,120,90,255])*width
 raw=row*height
 def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
 return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',width,height,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(raw))+chunk(b'IEND',b'')

(root/'image.png').write_bytes(png(64,48))
db=sqlite3.connect(root/'state.sqlite3')
db.executescript("CREATE TABLE drafts(id INTEGER PRIMARY KEY CHECK(id=1),text TEXT NOT NULL,revision INTEGER NOT NULL CHECK(revision>=0));INSERT INTO drafts VALUES(1,'合成旧草稿',0);PRAGMA user_version=1;")
db.close()

env=os.environ.copy()
env.update({'PATH':str(helpers)+':'+env['PATH'],'ZWEN_DATA_DIR':str(root),'ZWEN_SMOKE':'screenshot','ZWEN_SCREENSHOT_TIMEOUT_MS':'600','NIRI_SOCKET':str(root/'fake.sock')})
process=subprocess.Popen([str(binary),'screenshot'],env=env,stdout=subprocess.PIPE,stderr=(root/'stderr.log').open('wb'),bufsize=0,start_new_session=True)
selector=selectors.DefaultSelector();selector.register(process.stdout,selectors.EVENT_READ);buffer=b'';events=[];passed=False
try:
 began=time.monotonic()
 while time.monotonic()-began<40:
  for key,_ in selector.select(.1):
   chunk=os.read(key.fd,65536);buffer+=chunk
   while b'\n' in buffer:
    line,buffer=buffer.split(b'\n',1);text=line.decode(errors='replace');print(text,flush=True)
    if text.startswith('SCREENSHOT_PROGRESS:'):events.append({'progress':json.loads(text.split(':',1)[1])})
    if text.startswith('FOUNDATION_SMOKE:'):
     event=json.loads(text.split(':',1)[1]);events.append(event)
     if event.get('ok'):passed=True
  if process.poll() is not None:break
 process.wait(timeout=5)
 stderr=(root/'stderr.log').read_text(errors='replace')[-2000:]
 assert process.returncode==0 and passed,(process.returncode,events,stderr)
 db=sqlite3.connect(root/'state.sqlite3');draft=db.execute('SELECT text FROM drafts').fetchone()[0];db.close()
 assert '合成旧草稿' in draft,draft
 chat=root/'chat.sqlite3'
 if chat.exists():
  db=sqlite3.connect(chat);assert db.execute('SELECT count(*) FROM turns').fetchone()[0]==0;db.close()
 attachment=root/'attachment.sqlite3'
 if attachment.exists():
  db=sqlite3.connect(attachment);assert db.execute('SELECT count(*) FROM attachments').fetchone()[0]==0;db.close()
 summary={'passed':True,'exit':process.returncode,'events':events,'draftPreserved':True,'orphanRequestFiles':len(list((root/'attachments'/'incoming').glob('*.png'))) if (root/'attachments'/'incoming').exists() else 0}
 Path('artifacts/validation').mkdir(parents=True,exist_ok=True)
 Path('artifacts/validation/screenshot.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2))
 print(json.dumps(summary,ensure_ascii=False),flush=True)
finally:
 if process.poll() is None:
  os.killpg(process.pid,signal.SIGTERM);process.wait(timeout=5)
 selector.close()
