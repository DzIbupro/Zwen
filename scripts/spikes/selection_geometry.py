"""Synthetic GTK/AT-SPI geometry probe; never visits another process's widgets."""
import json,subprocess,sys,time,os
import gi
gi.require_version('Gtk','3.0');gi.require_version('Atspi','2.0')
from gi.repository import Gtk,GLib,Atspi
if len(sys.argv)>1 and sys.argv[1]=='--fixture':
 w=Gtk.Window(title='Zwen synthetic selection geometry');w.set_default_size(600,240)
 box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=8);box.set_border_width(28)
 label=Gtk.Label(label='Synthetic test only — no user material');box.pack_start(label,False,False,0)
 text=Gtk.TextView();text.get_buffer().set_text('A controlled selection for Zwen toolbar placement.\nSecond fixture line.')
 box.pack_start(text,True,True,0);w.add(box);w.connect('destroy',Gtk.main_quit);w.show_all()
 def select():
  b=text.get_buffer();b.select_range(b.get_iter_at_offset(13),b.get_iter_at_offset(22));text.grab_focus();return False
 GLib.timeout_add(500,select);GLib.timeout_add_seconds(12,lambda:(Gtk.main_quit(),False)[1]);Gtk.main();raise SystemExit()
p=subprocess.Popen([sys.executable,__file__,'--fixture'],stdout=subprocess.DEVNULL,stderr=subprocess.PIPE,env=dict(os.environ,GTK_MODULES='gail:atk-bridge',NO_AT_BRIDGE='0'))
try:
 desktop=Atspi.get_desktop(0);deadline=time.monotonic()+8;results=[];moved=False
 while time.monotonic()<deadline and not results:
  roots=[]
  for i in range(desktop.get_child_count()):
   app=desktop.get_child_at_index(i)
   if app and app.get_process_id()==p.pid:roots.append(app)
  if roots and '--float-fixture' in sys.argv and not moved:
   owned=next((w for w in json.loads(subprocess.check_output(['niri','msg','-j','windows'],text=True)) if w.get('pid')==p.pid),None)
   if owned:
    subprocess.run(['niri','msg','action','move-window-to-floating','--id',str(owned['id'])],check=True,stdout=subprocess.DEVNULL)
    subprocess.run(['niri','msg','action','move-floating-window','--id',str(owned['id']),'--x','500','--y','300'],check=True,stdout=subprocess.DEVNULL)
    moved=True;time.sleep(.25)
  pending=roots;count=0
  while pending and count<200:
   obj=pending.pop();count+=1
   iface=obj.get_text_iface()
   if iface and iface.get_n_selections()>0:
    selected=Atspi.Text.get_selection(iface,0)
    start,end=selected.start_offset,selected.end_offset
    coords={}
    for name,kind in [('screen',Atspi.CoordType.SCREEN),('window',Atspi.CoordType.WINDOW)]:
     r=iface.get_range_extents(start,end,kind);coords[name]={'x':r.x,'y':r.y,'width':r.width,'height':r.height}
    results.append({'pid':p.pid,'start':start,'end':end,'coordinates':coords})
   pending.extend(obj.get_child_at_index(i) for i in range(min(obj.get_child_count(),200-count)) if obj.get_child_at_index(i))
  if not results:time.sleep(.1)
 windows=json.loads(subprocess.check_output(['niri','msg','-j','windows'],text=True));own=[{k:v for k,v in w.items() if k in ('pid','layout','is_floating','is_focused','workspace_id')} for w in windows if w.get('pid')==p.pid]
 print(json.dumps({'ownedWindow':own,'syntheticOnly':True,'selectionRanges':results,'fixtureAlive':p.poll() is None,'accessibleApplicationCount':desktop.get_child_count()},ensure_ascii=False))
 if not results:raise SystemExit('No accessible synthetic selection geometry')
finally:
 p.terminate();_,err=p.communicate(timeout=5);print(err.decode(errors='replace'),file=sys.stderr)
