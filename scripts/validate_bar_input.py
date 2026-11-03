"""Human input check on an owned GTK fixture. Fake PRIMARY; no real model/data.
Run smoke binary, then use displayed synthetic text and click counter.
"""
import hashlib,json,os,subprocess,sys,tempfile,time,threading,signal
from pathlib import Path
if len(sys.argv)>1 and sys.argv[1]=='--fixture':
 import gi
 gi.require_version('Gtk','3.0');from gi.repository import Gtk,Gdk,GLib
 root=Path(sys.argv[2]);counts={'drags':0,'checks':0,'presses':0};marks=[0];down=[False];start=[(0,0)];moved=[False];last_selection=[None]
 def record(event,**details):
  with (root/'fixture-events.jsonl').open('a') as stream:stream.write(json.dumps({'timeNs':time.monotonic_ns(),'event':event,**details},ensure_ascii=False)+'\n')
 def selected(buf):
  bounds=buf.get_selection_bounds()
  return buf.get_text(*bounds,True) if bounds else ''
 def selection_details(buf):
  value=selected(buf)
  return {'fixtureSelectionId':marks[0],'chars':len(value),'bytes':len(value.encode()),'sha256':hashlib.sha256(value.encode()).hexdigest()}

 w=Gtk.Window(title='Zwen 划词输入测试（合成文字）');w.set_default_size(680,440)
 box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=16);box.set_border_width(24)
 label=Gtk.Label(label='依次测试单汉字、单字母、2–4字、长句，每种各做快速拖选和按住1秒再松开。\n每次观察是否出现小条，再点一次检查；重复同长度时先取消旧选区。最后点完成。');label.set_line_wrap(True);box.pack_start(label,False,False,0)
 text=Gtk.TextView();text.set_editable(False);text.set_wrap_mode(Gtk.WrapMode.WORD);text.get_buffer().set_text('中   a   中文   abcd\n这是专门用于测试的合成长句，请拖动选择不同长度的文字。\nSynthetic sentence for comparing short and long selections.')
 box.pack_start(text,True,True,0);status=Gtk.Label(label='已完成拖选：0；检查按钮点击：0');box.pack_start(status,False,False,0)
 def update():status.set_text(f'已完成拖选：{counts["drags"]}；检查按钮点击：{counts["checks"]}')
 def press(_widget,e):
  if e.button==1:
   down[0]=True;start[0]=(e.x,e.y);moved[0]=False;record('text-press',x=e.x,y=e.y,**selection_details(text.get_buffer()))
  return False
 def release(_widget,e):
  if e.button==1 and down[0]:
   down[0]=False
   record('text-release',x=e.x,y=e.y,moved=moved[0],**selection_details(text.get_buffer()))
   GLib.idle_add(lambda:record('after-release',**selection_details(text.get_buffer())) or False)
   if moved[0] and text.get_buffer().get_has_selection():counts['drags']+=1;update()
  return False
 def motion(_widget,e):
  if down[0] and not moved[0] and (abs(e.x-start[0][0])+abs(e.y-start[0][1]))>=4:
   moved[0]=True;record('drag-threshold',x=e.x,y=e.y)
  return False
 def changed(buf,*_args):
  value=selected(buf)
  if value==last_selection[0]:return
  last_selection[0]=value;marks[0]+=1
  # Atomic replacement avoids observing a truncated file between GTK mark events.
  pending=root/'selection.pending';pending.write_text(value);pending.replace(root/'selection')
  record('selection-changed',syntheticText=value,**selection_details(buf))
 text.add_events(Gdk.EventMask.BUTTON_PRESS_MASK|Gdk.EventMask.BUTTON_RELEASE_MASK|Gdk.EventMask.POINTER_MOTION_MASK);text.connect('button-press-event',press);text.connect('button-release-event',release);text.connect('motion-notify-event',motion);text.get_buffer().connect('mark-set',changed)
 check=Gtk.Button(label='检查点击（每次拖选后只点一次）')
 def check_press(*_args):
  counts['presses']+=1;record('check-press',**selection_details(text.get_buffer()));return False
 check.connect('button-press-event',check_press)
 def clicked(*_args):counts['checks']+=1;record('check-click',**selection_details(text.get_buffer()));update()
 check.connect('clicked',clicked);box.pack_start(check,False,False,0)
 def done(*_args):(root/'input-result.json').write_text(json.dumps(counts));Gtk.main_quit()
 finish=Gtk.Button(label='完成测试');finish.connect('clicked',done);box.pack_start(finish,False,False,0)
 w.add(box);w.connect('destroy',done);w.show_all();GLib.timeout_add_seconds(300,lambda:done() or False);Gtk.main();raise SystemExit()
binary=Path(sys.argv[1]).resolve();assert subprocess.run(['pgrep','-x','zwen'],capture_output=True).returncode!=0,'Close ordinary Zwen before isolated input test'
with tempfile.TemporaryDirectory(prefix='zwen-bar-input-') as folder:
 root=Path(folder);(root/'selection').write_text('');fake=root/'bin';fake.mkdir()
 helper=fake/'wl-paste';helper.write_text('''#!/usr/bin/python3
import hashlib,json,os,sys,time,subprocess
from pathlib import Path
p=Path(os.environ['ZWEN_ACTIONS_FIXTURE_SELECTION_FILE'])
def record(event,value,**details):
 with (p.parent/'helper-events.jsonl').open('a') as stream:stream.write(json.dumps({'timeNs':time.monotonic_ns(),'event':event,'chars':len(value),'bytes':len(value.encode()),'sha256':hashlib.sha256(value.encode()).hexdigest(),**details})+'\\n')
if '--watch' in sys.argv:
 last=None
 while True:
  current=p.read_text()
  if current!=last:
   last=current;record('watch-change',current);result=subprocess.run(sys.argv[sys.argv.index('--watch')+1:],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL);record('callback-return',current,returncode=result.returncode)
  time.sleep(.01)
else:
 value=p.read_text();record('primary-read',value);sys.stdout.write(value)
''');helper.chmod(0o755)
 env=dict(os.environ,ZWEN_DATA_DIR=folder,ZWEN_SMOKE='bar-input',ZWEN_SMOKE_TIMEOUT_SECS='310',ZWEN_ACTIONS_FIXTURE_SELECTION_FILE=str(root/'selection'),PATH=str(fake)+':'+os.environ['PATH'])
 env.pop('ZWEN_ACTIONS_FIXED_ANCHOR',None)
 log=[];ready=threading.Event();out=Path('artifacts/validation');out.mkdir(parents=True,exist_ok=True)
 with (out/'bar-input-stderr.txt').open('w') as errors, (out/'bar-input-app-events.jsonl').open('w') as app_events:
  event_lock=threading.Lock()
  app=subprocess.Popen([str(binary)],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,start_new_session=True)
  def read():
   for line in app.stdout:
    log.append(line)
    with event_lock:
     app_events.write(json.dumps({'timeNs':time.monotonic_ns(),'stream':'stdout','line':line.rstrip('\n')})+'\n');app_events.flush()
    if line.startswith('BAR_INPUT_READY'):ready.set()
  def read_errors():
   for line in app.stderr:
    errors.write(line);errors.flush()
    with event_lock:
     app_events.write(json.dumps({'timeNs':time.monotonic_ns(),'stream':'stderr','line':line.rstrip('\n')})+'\n');app_events.flush()
  readers=[threading.Thread(target=read,daemon=True),threading.Thread(target=read_errors,daemon=True)]
  for reader in readers:reader.start()
  fixture=None
  try:
   assert ready.wait(20),'test application not ready'
   fixture=subprocess.Popen([sys.executable,__file__,'--fixture',folder]);print('INPUT_FIXTURE_OPENED',flush=True);fixture.wait(timeout=310)
   result=json.loads((root/'input-result.json').read_text());result['barAppearances']=sum(s.startswith('BAR_INPUT_OBSERVED:') for s in log);result['syntheticOnly']=True
   result['automaticVerdict']=False
   result['scope']='Synthetic PRIMARY with real pointer probe; manually correlate per-selection events with BAR_TRACE and visible toolbar. Aggregate appearances do not prove every trial passed.'
   (out/'bar-input.json').write_text(json.dumps(result,ensure_ascii=False,indent=2));(out/'bar-input-stdout.txt').write_text(''.join(log));print(json.dumps(result,ensure_ascii=False),flush=True)
  finally:
   if fixture and fixture.poll() is None:fixture.terminate();fixture.wait(timeout=5)
   if app.poll() is None:os.killpg(app.pid,signal.SIGTERM);app.wait(timeout=5)
   for reader in readers:reader.join(timeout=2)
   for event_file in ['fixture-events.jsonl','helper-events.jsonl']:
    source=root/event_file
    if source.exists():(out/('bar-input-'+event_file)).write_text(source.read_text())
   (out/'bar-input-stdout.txt').write_text(''.join(log))
