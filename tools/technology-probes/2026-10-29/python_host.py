"""Throwaway GTK4/WebKit6 runtime probe, no remote model or user data."""
import gi,json,time,sys,os
from pathlib import Path
gi.require_version('Gtk','4.0');gi.require_version('WebKit','6.0')
from gi.repository import Gtk,WebKit,GLib
root=Path(__file__).resolve().parent
app=Gtk.Application(application_id='org.example.StudyTechSpike');started=time.monotonic();fired=False

def event(stage,**kwargs): print(json.dumps(dict(stage=stage,elapsed=round(time.monotonic()-started,3),**kwargs)),flush=True)
def activate(app):
 global fired
 app.hold();win=Gtk.ApplicationWindow(application=app,title='Study Assistant · Python GTK4 技术验证');win.set_default_size(900,850)
 web=WebKit.WebView();win.set_child(web)
 def title(w,*_):
  global fired
  t=w.get_title() or ''
  if t.startswith('SPIKE_'):event('page',title=t)
  if t.startswith('SPIKE_READY:') and not fired:
   checks=json.loads(t.split(':',1)[1]);assert all(checks[k] for k in ['table','formula','image','graph','code']) and checks['ticks']==50,checks
   fired=True
   
   if os.environ.get('SPIKE_SNAPSHOT')=='1':GLib.timeout_add(200,snapshot)
   if os.environ.get('SPIKE_VISUAL_ONLY')=='1':
    GLib.timeout_add_seconds(2,done);return
   GLib.timeout_add_seconds(1,fault)
   GLib.timeout_add_seconds(4,hidden)
   GLib.timeout_add_seconds(20,restore)
   GLib.timeout_add_seconds(24,cycles)
   GLib.timeout_add_seconds(29,done)
 def snapshot():
  def saved(w,result):
   try:w.get_snapshot_finish(result).save_to_png(str(root/'results'/'webkit6-render.png'));event('snapshot_saved')
   except Exception as e:event('snapshot_failed',error=str(e))
  web.get_snapshot(WebKit.SnapshotRegion.FULL_DOCUMENT,WebKit.SnapshotOptions.NONE,None,saved);return False
 def fault():web.evaluate_javascript('window.runFaultSuite()',-1,None,None,None,None,None);event('fault_requested');return False
 def hidden():win.set_visible(False);event('hidden');return False
 def restore():win.present();event('restored');return False
 def cycles():
  state={'n':0}
  def step():
   state['n']+=1;win.set_visible(state['n']%2==0)
   if state['n']==100:event('cycles',count=100);win.present();return False
   return True
  GLib.timeout_add(20,step);return False
 def done():event('done',ready=fired);app.quit();return False
 def crashed(w,reason):event('web_process_terminated',reason=str(reason));return False
 web.connect('notify::title',title);web.connect('web-process-terminated',crashed)
 web.load_uri((root/'web'/'index.html').as_uri());win.present();event('started')
 GLib.timeout_add_seconds(60,lambda:(event('timeout'),app.quit(),False)[-1])
app.connect('activate',activate);app.run([])
