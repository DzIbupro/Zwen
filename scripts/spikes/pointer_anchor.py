"""One-shot transparent layer-shell pointer-enter probe, no screenshots/input log.
It destroys all surfaces on first enter, or within 200ms. Never grabs keyboard.
Experimental pointer proximity only, not exact selected text geometry.
"""
import gi,json,subprocess
gi.require_version('Gtk','3.0');gi.require_version('GtkLayerShell','0.1')
from gi.repository import Gtk,Gdk,GLib,GtkLayerShell

def focus():
 try:return json.loads(subprocess.check_output(['niri','msg','-j','focused-window'],text=True)).get('pid')
 except Exception:return None
before=focus();windows=[];result=None

def finish():
 for w in windows:w.destroy()
 Gtk.main_quit();return False

def enter(widget,event,index,geometry):
 global result
 if result is not None:return False
 result={'kind':'pointer-proximity','monitorIndex':index,'localX':event.x,'localY':event.y,'outputX':geometry.x,'outputY':geometry.y,'width':geometry.width,'height':geometry.height}
 GLib.idle_add(finish);return False

provider=Gtk.CssProvider();provider.load_from_data(b'.zwen-pointer-probe { background-color: rgba(0,0,0,0); }')
screen=Gdk.Screen.get_default();display=Gdk.Display.get_default()
for i in range(display.get_n_monitors()):
 monitor=display.get_monitor(i);w=Gtk.Window();w.set_decorated(False);w.set_accept_focus(False);w.set_app_paintable(True)
 visual=screen.get_rgba_visual()
 if visual:w.set_visual(visual)
 style=w.get_style_context();style.add_class('zwen-pointer-probe');style.add_provider(provider,600)
 GtkLayerShell.init_for_window(w);GtkLayerShell.set_namespace(w,'zwen-pointer-prototype');GtkLayerShell.set_monitor(w,monitor)
 GtkLayerShell.set_layer(w,GtkLayerShell.Layer.OVERLAY);GtkLayerShell.set_keyboard_mode(w,GtkLayerShell.KeyboardMode.NONE);GtkLayerShell.set_exclusive_zone(w,0)
 for edge in [GtkLayerShell.Edge.LEFT,GtkLayerShell.Edge.RIGHT,GtkLayerShell.Edge.TOP,GtkLayerShell.Edge.BOTTOM]:GtkLayerShell.set_anchor(w,edge,True)
 w.add_events(Gdk.EventMask.ENTER_NOTIFY_MASK);w.connect('enter-notify-event',enter,i,monitor.get_geometry());windows.append(w);w.show_all()
GLib.timeout_add(200,finish);Gtk.main()
print(json.dumps({'pointer':result,'focusPreserved':before==focus(),'maxLifetimeMs':200}))
