"""Native tray registration over D-Bus; isolated app data, no model requests."""
import json,os,subprocess,sys,tempfile,time
from pathlib import Path
binary=Path(sys.argv[1]).resolve()
assert subprocess.run(['pgrep','-x','zwen'],capture_output=True).returncode!=0,'Save and quit existing Zwen first'
def items():
 raw=subprocess.check_output(['busctl','--user','--json=short','get-property','org.kde.StatusNotifierWatcher','/StatusNotifierWatcher','org.kde.StatusNotifierWatcher','RegisteredStatusNotifierItems'],text=True)
 return set(json.loads(raw)['data'])
before=items()
with tempfile.TemporaryDirectory(prefix='zwen-tray-') as folder:
 env=os.environ.copy();env['ZWEN_DATA_DIR']=folder
 p=subprocess.Popen([str(binary)],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
 try:
  found=None
  for _ in range(100):
   for item in items()-before:
    bus,_,suffix=item.partition('/');path='/'+suffix
    title=subprocess.check_output(['busctl','--user','get-property',bus,path,'org.kde.StatusNotifierItem','Title'],text=True).strip()
    if 'zwen' in title.lower():found=(bus,path,title);break
   if found:break
   if p.poll() is not None:break
   time.sleep(.05)
  assert found,'Zwen tray registration not found'
  bus,path,title=found
  menu=subprocess.check_output(['busctl','--user','get-property',bus,path,'org.kde.StatusNotifierItem','Menu'],text=True).strip()
  assert menu!='o "/"',menu
  # Native application CLI requests the existing frontend save/quit workflow.
  time.sleep(.5)
  subprocess.run([str(binary),'quit'],env=env,check=True,capture_output=True)
  stdout,stderr=p.communicate(timeout=10)
  assert p.returncode==0,(p.returncode,stderr[-1000:])
  for _ in range(50):
   if not (items()-before):break
   time.sleep(.05)
  assert not (items()-before),'tray remains after quit'
  result={'passed':True,'title':title,'menu':menu,'registered':True,'removedOnQuit':True,'exit':p.returncode}
  out=Path('artifacts/validation');out.mkdir(parents=True,exist_ok=True);(out/'tray.json').write_text(json.dumps(result,ensure_ascii=False,indent=2));print(json.dumps(result,ensure_ascii=False))
 finally:
  if p.poll() is None:p.terminate();p.wait(timeout=5)
  if p.stderr and not p.stderr.closed:print('STDERR',p.stderr.read(),flush=True)
