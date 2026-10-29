import json,statistics
from pathlib import Path
root=Path(__file__).parent/'results';groups={}
for p in sorted(root.glob('*-formal-*/summary.json')):
 d=json.loads(p.read_text());kind=d['label'].rsplit('-formal-',1)[0];groups.setdefault(kind,[]).append(d)
for kind,rows in groups.items():
 print(kind,len(rows),'runs')
 for key in ['peak_pss_mib','peak_rss_mib','ready_pss_mib','hidden_pss_mib','restored_pss_mib']:
  v=[r[key] for r in rows if key in r]
  if v:print(' ',key,'median',round(statistics.median(v),1),'range',min(v),max(v))
 print(' passed',all(r['done'] and r['ready'] and r['cycles'] and r['exit']==0 for r in rows))
