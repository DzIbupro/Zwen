"""Disposable GTK-only synthetic paint probe. Never captures the desktop."""
import gi,json,time
gi.require_version('Gtk','3.0')
from gi.repository import Gtk,Gdk,GLib
import cairo
w=Gtk.Window(title='Zwen synthetic transparent-frame probe');w.set_default_size(240,120);w.set_decorated(False)
visual=w.get_screen().get_rgba_visual();assert visual;w.set_visual(visual)
child=Gtk.Label(label='SYNTHETIC ONLY');w.add(child);w.show_all();began=time.monotonic();state={'submitted':None,'alpha':None}
def start():
 child.hide();w.set_app_paintable(True)
 def draw(_w,cr):
  cr.set_operator(cairo.OPERATOR_SOURCE);cr.set_source_rgba(0,0,0,0);cr.paint();state['submitted']=w.get_frame_clock().get_frame_counter();return True
 w.connect('draw',draw);w.queue_draw();return False
def poll():
 frame=state['submitted'];clock=w.get_frame_clock()
 if frame is not None:
  timings=clock.get_timings(frame)
  if timings and timings.get_complete():
   surface=cairo.ImageSurface(cairo.FORMAT_ARGB32,240,120);cr=cairo.Context(surface);w.draw(cr);surface.flush();data=bytes(surface.get_data());state['alpha']=max(data[3::4]);
   print(json.dumps({'complete':True,'alphaMax':state['alpha'],'frame':frame,'presentation':timings.get_presentation_time(),'elapsed':time.monotonic()-began}),flush=True);w.destroy();Gtk.main_quit();return False
 if time.monotonic()-began>4:
  print(json.dumps({'complete':False,'submitted':frame}),flush=True);w.destroy();Gtk.main_quit();return False
 return True
GLib.timeout_add(300,start);GLib.timeout_add(16,poll);Gtk.main()
