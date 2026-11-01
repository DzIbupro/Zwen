"""Synthetic knowledge-base flow over an owned window and temp databases."""
import json,os,sqlite3,subprocess,sys,tempfile,time
from pathlib import Path
binary=Path(sys.argv[1]).resolve()
assert subprocess.run(['pgrep','-x','zwen'],capture_output=True).returncode!=0,'Save and quit existing Zwen before isolated native tests'
root=Path(tempfile.mkdtemp(prefix='zwen-knowledge-'))
ORPHAN='abcdef0123456789abcdef0123456789'
ATTACH='ab'*16

# Seed a chat attachment that will be referenced (and must NOT be consumed by收藏).
attachments=root/'attachments';attachments.mkdir()
(attachments/f'{ATTACH}.png').write_bytes(b'\x89PNG\r\n\x1a\nseeded-bytes')
db=sqlite3.connect(root/'attachment.sqlite3')
db.executescript("CREATE TABLE attachments(id TEXT PRIMARY KEY,width INTEGER NOT NULL,height INTEGER NOT NULL,bytes INTEGER NOT NULL,created_at INTEGER NOT NULL);PRAGMA user_version=1;")
db.execute('INSERT INTO attachments VALUES(?,?,?,?,?)',(ATTACH,64,48,28,123));db.commit();db.close()
# Seed an orphan Markdown note: the file is the authority and rebuild must adopt it.
notes=root/'notes';(notes/'attachments').mkdir(parents=True)
(notes/f'{ORPHAN}.md').write_text(f"---\nid: {ORPHAN}\ncreated: 1700000000\nsource: chat\nmodel: model-b\ntitle: 孤儿笔记\nattachments: []\n---\n孤儿正文",encoding='utf-8')

db=sqlite3.connect(root/'chat.sqlite3')
db.executescript("CREATE TABLE turns(id INTEGER PRIMARY KEY,request_id TEXT NOT NULL UNIQUE,conversation_id TEXT NOT NULL,provider_profile TEXT NOT NULL,context_turns INTEGER NOT NULL,prompt TEXT NOT NULL,model TEXT NOT NULL,answer TEXT NOT NULL DEFAULT '',revision INTEGER NOT NULL DEFAULT 0 CHECK(revision>=0),state TEXT NOT NULL DEFAULT 'streaming',error TEXT,created_at INTEGER NOT NULL DEFAULT (unixepoch()));PRAGMA user_version=2;")
db.execute("INSERT INTO turns(request_id,conversation_id,provider_profile,context_turns,prompt,model,answer,revision,state) VALUES('seedturn0001','conv1','p',0,'原始问题文本','fixture-model','原始回答内容',1,'complete')")
db.commit();db.close()

env=os.environ.copy()
env.update({'ZWEN_DATA_DIR':str(root),'ZWEN_SMOKE':'knowledge','ZWEN_SMOKE_TIMEOUT_SECS':'90'})
process=subprocess.Popen([str(binary)],env=env,stdout=subprocess.PIPE,stderr=(root/'stderr.log').open('wb'),bufsize=0,start_new_session=True)
selector=__import__('selectors').DefaultSelector();selector.register(process.stdout,__import__('selectors').EVENT_READ);buffer=b'';rows=[];passed=False
try:
 began=time.monotonic()
 while time.monotonic()-began<60:
  for key,_ in selector.select(.1):
   chunk=os.read(key.fd,65536);buffer+=chunk
   while b'\n' in buffer:
    line,buffer=buffer.split(b'\n',1);text=line.decode(errors='replace');print(text,flush=True)
    if text.startswith('FOUNDATION_SMOKE:'):
     event=json.loads(text.split(':',1)[1]);rows.append(event)
     if event.get('ok'):passed=True
  if process.poll() is not None:break
 process.wait(timeout=5)
 stderr=(root/'stderr.log').read_text(errors='replace')[-1500:]
 assert process.returncode==0 and passed,(process.returncode,rows,stderr)
 # The chat attachment is copied into the note, never consumed by收藏.
 assert (attachments/f'{ATTACH}.png').exists(),'收藏 consumed the chat attachment'
 # The saved note and its copy were deleted with the note; the orphan md stays.
 saved_files=[p for p in notes.glob('*.md') if p.name!=f'{ORPHAN}.md']
 assert not saved_files,f'saved note files remain: {saved_files}'
 remain=[p for p in (notes/'attachments').glob('*') if p.is_file()]
 assert not remain,f'note attachment copies remain: {remain}'
 assert (notes/f'{ORPHAN}.md').exists(),'orphan note was deleted'
 db=sqlite3.connect(root/'attachment.sqlite3');assert db.execute('SELECT count(*) FROM attachments').fetchone()[0]==1;db.close()
 db=sqlite3.connect(root/'chat.sqlite3');assert db.execute("SELECT prompt,answer FROM turns WHERE request_id='seedturn0001'").fetchone()==('原始问题文本','原始回答内容');db.close()
 for path in root.glob('*.sqlite3*'):assert b'fixture-token' not in path.read_bytes() if path.exists() else True
 summary={'passed':True,'exit':process.returncode,'events':rows}
 Path('artifacts/validation').mkdir(parents=True,exist_ok=True)
 Path('artifacts/validation/knowledge.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2))
 print(json.dumps(summary,ensure_ascii=False))
finally:
 if process.poll() is None:
  __import__('signal').SIGTERM;os.killpg(process.pid,__import__('signal').SIGTERM);process.wait(timeout=5)
 selector.close()
