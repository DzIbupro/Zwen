"""Throwaway native toolbar positioning/focus probe at a SYNTHETIC anchor.
Not integrated with Zwen or real selections. No model requests.
"""
import gi,json,subprocess
gi.require_version('Gtk','3.0');gi.require_version('GtkLayerShell','0.1')
from gi.repository import Gtk,Gdk,GLib,GtkLayerShell

def focused_pid():
 try:return json.loads(subprocess.check_output(['niri','msg','-j','focused-window'],text=True)).get('pid')
 except Exception:return None
before=focused_pid()
if not GtkLayerShell.is_supported():raise SystemExit('Layer shell is unavailable')
w=Gtk.Window(title='Zwen synthetic independent toolbar');w.set_decorated(False);w.set_resizable(False);w.set_accept_focus(False)
GtkLayerShell.init_for_window(w);GtkLayerShell.set_namespace(w,'zwen-selection-prototype');GtkLayerShell.set_layer(w,GtkLayerShell.Layer.OVERLAY)
GtkLayerShell.set_keyboard_mode(w,GtkLayerShell.KeyboardMode.NONE);GtkLayerShell.set_exclusive_zone(w,0)
GtkLayerShell.set_anchor(w,GtkLayerShell.Edge.TOP,True);GtkLayerShell.set_anchor(w,GtkLayerShell.Edge.LEFT,True)
GtkLayerShell.set_margin(w,GtkLayerShell.Edge.TOP,300);GtkLayerShell.set_margin(w,GtkLayerShell.Edge.LEFT,600)
box=Gtk.Box(spacing=4);box.set_border_width(5)
for label in ['问一问 Zwen','翻译','详细解释','其他']:
 b=Gtk.Button(label=label);b.set_can_focus(False);b.connect('clicked',lambda button:print(json.dumps({'clicked':button.get_label()},ensure_ascii=False),flush=True));box.pack_start(b,False,False,0)
w.add(box);w.show_all()
def inspect():
 after=focused_pid();print(json.dumps({'syntheticAnchorOnly':True,'layerShellSupported':True,'keyboardMode':'none','exclusiveZone':GtkLayerShell.get_exclusive_zone(w),'focusPreserved':before==after,'size':[w.get_allocated_width(),w.get_allocated_height()]},ensure_ascii=False),flush=True);Gtk.main_quit();return False
GLib.timeout_add(700,inspect);Gtk.main();w.destroy()
