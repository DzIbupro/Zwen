"""Quit an owned native app while a fake writer holds a partial request file."""
import os,subprocess,sys,tempfile,time
from pathlib import Path
binary=Path(sys.argv[1]).resolve();assert subprocess.run(['pgrep','-x','zwen'],capture_output=True).returncode!=0,'Save and quit user Zwen first'
with tempfile.TemporaryDirectory(prefix='zwen-capture-exit-') as folder:
 root=Path(folder);helpers=root/'bin';helpers.mkdir()
 scripts={'niri':'raise SystemExit(2)','slurp':"print('0,0 64x48')",'grim':"Path(sys.argv[-1]).write_bytes(b'partial');(root/'writer.pid').write_text(str(os.getpid()));time.sleep(30)"}
 for name,body in scripts.items():
  path=helpers/name;path.write_text('#!/usr/bin/python3\nimport os,sys,time\nfrom pathlib import Path\nroot=Path(__file__).resolve().parent.parent\n'+body+'\n');path.chmod(0o700)
 env=os.environ.copy();env.update(ZWEN_DATA_DIR=folder,ZWEN_SMOKE='capture-exit',PATH=str(helpers)+':'+env['PATH'])
 p=subprocess.Popen([str(binary)],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
 try:
  began=time.monotonic()
  while not (root/'writer.pid').exists() and p.poll() is None and time.monotonic()-began<8:time.sleep(.02)
  assert (root/'writer.pid').exists(),'writer did not start'
  writer=(root/'writer.pid').read_text().strip();assert list((root/'attachments'/'incoming').glob('*.png')),'no partial fixture'
  child_env=env.copy();child_env.pop('ZWEN_SMOKE');subprocess.run([str(binary),'quit'],env=child_env,capture_output=True,timeout=8,check=True)
  stdout,stderr=p.communicate(timeout=8);assert p.returncode==0,(stdout,stderr[-1500:])
  state=Path('/proc')/writer/'stat';assert not state.exists() or state.read_text().split(')')[1].strip().startswith('Z'),'writer survived quit'
  assert not list((root/'attachments'/'incoming').glob('*.png')),'partial file survived quit'
  print('{"passed":true,"quitKilledWriter":true,"partialRemoved":true}')
 finally:
  if p.poll() is None:p.terminate();p.wait(timeout=5)
