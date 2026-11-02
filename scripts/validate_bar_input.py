"""Human input check on an owned GTK fixture. Fake PRIMARY; no real model/data.
Run smoke binary, then use displayed synthetic text and click counter.
"""
import json,os,subprocess,sys,tempfile,time,threading,signal
from pathlib import Path
if len(sys.argv)>1 and sys.argv[1]=='--fixture':
 import gi
 gi.require_version('Gtk','3.0');from gi.repository import Gtk,Gdk,GLib
 root=Path(sys.argv[2]);counts={'drags':0,'checks':0,'presses':0};marks=[0];down=[False];start=[(0,0)];moved=[False]
 w=Gtk.Window(title='Zwen 划词输入测试（合成文字）');w.set_default_size(680,440)
 box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=16);box.set_border_width(24)
 label=Gtk.Label(label='请做2次“拖选后立刻点击检查按钮”，再做1次“选好后按住1秒再松开”。\n每次只点击一次检查按钮；最后点击完成。无需使用你的文档。');label.set_line_wrap(True);box.pack_start(label,False,False,0)
 text=Gtk.TextView();text.set_editable(False);text.set_wrap_mode(Gtk.WrapMode.WORD);text.get_buffer().set_text('这是专门用于测试的合成文字。请拖动鼠标选择其中几个词语，随后点击下面的检查按钮。\nZwen 的独立小操作条应出现在所选文字附近，而不是先打开聊天胶囊。')
 box.pack_start(text,True,True,0);status=Gtk.Label(label='已完成拖选：0；检查按钮点击：0');box.pack_start(status,False,False,0)
 def update():status.set_text(f'已完成拖选：{counts["drags"]}；检查按钮点击：{counts["checks"]}')
 def press(_widget,e):
  if e.button==1:down[0]=True;start[0]=(e.x,e.y);moved[0]=False
  return False
 def release(_widget,e):
  if e.button==1 and down[0]:
   down[0]=False
   if moved[0] and text.get_buffer().get_has_selection():counts['drags']+=1;update()
  return False
 def motion(_widget,e):
  if down[0] and (abs(e.x-start[0][0])+abs(e.y-start[0][1]))>=4:moved[0]=True
  return False
 def changed(buf,*_args):
  if buf.get_has_selection():marks[0]+=1;(root/'selection').write_text('合成选区-'+str(marks[0]))
 text.add_events(Gdk.EventMask.BUTTON_PRESS_MASK|Gdk.EventMask.BUTTON_RELEASE_MASK|Gdk.EventMask.POINTER_MOTION_MASK);text.connect('button-press-event',press);text.connect('button-release-event',release);text.connect('motion-notify-event',motion);text.get_buffer().connect('mark-set',changed)
 check=Gtk.Button(label='检查点击（每次拖选后只点一次）');check.connect('button-press-event',lambda *_:counts.__setitem__('presses',counts['presses']+1) or False)
 def clicked(*_args):counts['checks']+=1;update()
 check.connect('clicked',clicked);box.pack_start(check,False,False,0)
 def done(*_args):(root/'input-result.json').write_text(json.dumps(counts));Gtk.main_quit()
 finish=Gtk.Button(label='完成测试');finish.connect('clicked',done);box.pack_start(finish,False,False,0)
 w.add(box);w.connect('destroy',done);w.show_all();GLib.timeout_add_seconds(300,lambda:done() or False);Gtk.main();raise SystemExit()
binary=Path(sys.argv[1]).resolve();assert subprocess.run(['pgrep','-x','zwen'],capture_output=True).returncode!=0,'Close ordinary Zwen before isolated input test'
with tempfile.TemporaryDirectory(prefix='zwen-bar-input-') as folder:
 root=Path(folder);(root/'selection').write_text('');fake=root/'bin';fake.mkdir()
 helper=fake/'wl-paste';helper.write_text('''#!/usr/bin/python3
import os,sys,time,subprocess
from pathlib import Path
p=Path(os.environ['ZWEN_ACTIONS_FIXTURE_SELECTION_FILE'])
if '--watch' in sys.argv:
 last=None
 while True:
  current=p.read_text()
  if current!=last:
   last=current;subprocess.run(sys.argv[sys.argv.index('--watch')+1:],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
  time.sleep(.01)
else:sys.stdout.write(p.read_text())
''');helper.chmod(0o755)
 env=dict(os.environ,ZWEN_DATA_DIR=folder,ZWEN_SMOKE='bar-input',ZWEN_SMOKE_TIMEOUT_SECS='310',ZWEN_ACTIONS_FIXTURE_SELECTION_FILE=str(root/'selection'),PATH=str(fake)+':'+os.environ['PATH'])
 log=[];ready=threading.Event();out=Path('artifacts/validation');out.mkdir(parents=True,exist_ok=True)
 with (out/'bar-input-stderr.txt').open('w') as errors:
  app=subprocess.Popen([str(binary)],env=env,stdout=subprocess.PIPE,stderr=errors,text=True,start_new_session=True)
  def read():
   for line in app.stdout:
    log.append(line)
    if line.startswith('BAR_INPUT_READY'):ready.set()
  threading.Thread(target=read,daemon=True).start()
  fixture=None
  try:
   assert ready.wait(20),'test application not ready'
   fixture=subprocess.Popen([sys.executable,__file__,'--fixture',folder]);print('INPUT_FIXTURE_OPENED',flush=True);fixture.wait(timeout=310)
   result=json.loads((root/'input-result.json').read_text());result['barAppearances']=sum(s.startswith('BAR_INPUT_OBSERVED:') for s in log);result['syntheticOnly']=True
   (out/'bar-input.json').write_text(json.dumps(result,ensure_ascii=False,indent=2));(out/'bar-input-stdout.txt').write_text(''.join(log));print(json.dumps(result,ensure_ascii=False),flush=True)
  finally:
   if fixture and fixture.poll() is None:fixture.terminate();fixture.wait(timeout=5)
   if app.poll() is None:os.killpg(app.pid,signal.SIGTERM);app.wait(timeout=5)
