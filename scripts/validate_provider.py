"""Owned native process, isolated metadata database and a synthetic key. No requests."""
import os,subprocess,tempfile,json,sys
from pathlib import Path
binary=Path(sys.argv[1]).resolve()
out=Path('artifacts/validation');out.mkdir(parents=True,exist_ok=True)
with tempfile.TemporaryDirectory(prefix='zwen-provider-native-') as folder:
 env=os.environ.copy();env.update(ZWEN_DATA_DIR=folder,ZWEN_SMOKE='provider',ZWEN_PROVIDER_TEST_KEY='synthetic-test-token')
 p=subprocess.run([str(binary)],env=env,capture_output=True,text=True,timeout=40)
 (out/'provider-stdout.txt').write_text(p.stdout);(out/'provider-stderr.txt').write_text(p.stderr)
 rows=[json.loads(line.split(':',1)[1]) for line in p.stdout.splitlines() if line.startswith('FOUNDATION_SMOKE:')]
 assert p.returncode==0 and any(r.get('ok') and r.get('mode')=='provider' for r in rows),(p.returncode,p.stdout,p.stderr)
 assert b'synthetic-test-token' not in Path(folder,'provider.sqlite3').read_bytes()
 print('Native provider fixture PASS:',rows)
