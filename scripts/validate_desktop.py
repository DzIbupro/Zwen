"""Native foundation smoke; only an explicitly smoke-enabled binary and synthetic data.
Usage: python3 scripts/validate_desktop.py /absolute/path/to/smoke-binary
"""
import json, os, selectors, sqlite3, subprocess, sys, tempfile, time
from pathlib import Path

binary = Path(sys.argv[1]).resolve()
output = Path('artifacts/validation'); output.mkdir(parents=True, exist_ok=True)

def sample(root_pid):
    parents={}
    for p in Path('/proc').iterdir():
        if not p.name.isdigit():continue
        try:
            s=(p/'stat').read_text();parents[int(p.name)]=int(s[s.rfind(')')+2:].split()[1])
        except (OSError,ValueError):pass
    ids={root_pid}
    while True:
        more={pid for pid,parent in parents.items() if parent in ids}
        if more.issubset(ids):break
        ids|=more
    pss=rss=0
    for pid in ids:
        try:
            for line in Path(f'/proc/{pid}/smaps_rollup').read_text().splitlines():
                if line.startswith('Pss:'):pss+=int(line.split()[1])
                if line.startswith('Rss:'):rss+=int(line.split()[1])
        except OSError:pass
    return {'pss_MiB':round(pss/1024,2),'rss_MiB':round(rss/1024,2),'processes':len(ids)}

with tempfile.TemporaryDirectory(prefix='study-desktop-smoke-') as folder:
    summaries=[]
    for mode in ['edit','restore','quit']:
        env=os.environ.copy();env['STUDY_ASSISTANT_DATA_DIR']=str(Path(folder)/('quit' if mode=='quit' else 'close'));env['STUDY_ASSISTANT_SMOKE']=mode
        start=time.monotonic();buffer=b'';events=[];samples=[]
        with (output/f'{mode}-stderr.txt').open('wb') as errors:
            p=subprocess.Popen([str(binary)],env=env,stdout=subprocess.PIPE,stderr=errors,bufsize=0)
            sel=selectors.DefaultSelector();sel.register(p.stdout,selectors.EVENT_READ)
            while time.monotonic()-start<40:
                for key,_ in sel.select(.2):
                    chunk=os.read(key.fd,65536)
                    if not chunk:continue
                    buffer+=chunk
                    while b'\n' in buffer:
                        line,buffer=buffer.split(b'\n',1);text=line.decode(errors='replace')
                        if text.startswith('FOUNDATION_SMOKE:'):
                            events.append({'seconds':round(time.monotonic()-start,3),'value':text.split(':',1)[1]});print(text,flush=True)
                if p.poll() is not None:break
                samples.append({'seconds':round(time.monotonic()-start,3),**sample(p.pid)})
            else:p.terminate()
            p.wait(timeout=5)
        good=[]
        for event in events:
            try:d=json.loads(event['value'])
            except ValueError:continue
            if d.get('ok'):good.append(d)
        success=p.returncode==0 and any(d.get('mode')=='restore' for d in good)
        if mode=='quit':
            with sqlite3.connect(Path(env['STUDY_ASSISTANT_DATA_DIR'])/'state.sqlite3') as db:
                saved=db.execute('SELECT text FROM drafts WHERE id=1').fetchone()[0]
            success=p.returncode==0 and saved.endswith('关闭前追加')
        if mode=='edit':success=success and any(d.get('mode')=='edit' for d in good) and any(e['value'].startswith('hidden:') for e in events)
        result={'mode':mode,'exit':p.returncode,'passed':success,'duration':round(time.monotonic()-start,2),'peak_pss_MiB':max((s['pss_MiB'] for s in samples),default=0),'peak_rss_MiB':max((s['rss_MiB'] for s in samples),default=0)}
        (output/f'{mode}.json').write_text(json.dumps({'summary':result,'events':events,'samples':samples},indent=2))
        print(result,flush=True);summaries.append(result)
        assert success,'Native smoke failed; inspect artifacts/validation'
    (output/'summary.json').write_text(json.dumps(summaries,indent=2))
