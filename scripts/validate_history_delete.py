"""Real native UI/IPC on synthetic history only; never invokes a model."""
import json,os,sqlite3,subprocess,sys,tempfile
from pathlib import Path
binary=Path(sys.argv[1]).resolve()
assert subprocess.run(['pgrep','-x','zwen'],capture_output=True).returncode!=0,'Save and quit existing Zwen first'
with tempfile.TemporaryDirectory(prefix='zwen-history-delete-') as folder:
 root=Path(folder)
 with sqlite3.connect(root/'state.sqlite3') as db:db.executescript("CREATE TABLE drafts(id INTEGER PRIMARY KEY CHECK(id=1),text TEXT NOT NULL,revision INTEGER NOT NULL CHECK(revision>=0));INSERT INTO drafts VALUES(1,'合成删除草稿',0);PRAGMA user_version=1;")
 with sqlite3.connect(root/'chat.sqlite3') as db:
  db.executescript("CREATE TABLE turns(id INTEGER PRIMARY KEY,request_id TEXT NOT NULL UNIQUE,conversation_id TEXT NOT NULL,provider_profile TEXT NOT NULL,context_turns INTEGER NOT NULL,prompt TEXT NOT NULL,model TEXT NOT NULL,answer TEXT NOT NULL,revision INTEGER NOT NULL,state TEXT NOT NULL,error TEXT,created_at INTEGER NOT NULL);CREATE INDEX turns_conversation ON turns(conversation_id,id);PRAGMA user_version=2;")
  for number,letter in enumerate(['a','b'],1):db.execute("INSERT INTO turns VALUES(?,?,?,?,?,?,?,?,?,?,?,?)",(number,'history-'+letter,'history-'+letter,'fixture',0,'问题 '+letter.upper(),'fixture','合成回答 '+letter,1,'complete',None,number))
 env=os.environ.copy();env.update({'ZWEN_DATA_DIR':folder,'ZWEN_SMOKE':'history-delete'})
 p=subprocess.Popen([str(binary)],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
 try:stdout,stderr=p.communicate(timeout=30)
 finally:
  if p.poll() is None:p.terminate();p.wait(timeout=5)
 events=[json.loads(line.split(':',1)[1]) for line in stdout.splitlines() if line.startswith('FOUNDATION_SMOKE:')]
 print(json.dumps({'exit':p.returncode,'events':events},ensure_ascii=False))
 assert p.returncode==0 and any(e.get('ok') for e in events),(stdout,stderr[-1500:])
 with sqlite3.connect(root/'chat.sqlite3') as db:assert db.execute('SELECT count(*) FROM turns').fetchone()[0]==0
 assert list((root/'notes').glob('*.md')),'Saved Markdown missing'
 Path('artifacts/validation').mkdir(parents=True,exist_ok=True);Path('artifacts/validation/history-delete.json').write_text(json.dumps({'passed':True,'events':events,'temporaryDataOnly':True,'evidence':'Native DOM driver and IPC; no physical mouse test'},ensure_ascii=False,indent=2))
