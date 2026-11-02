//! Same-process, non-focusing GTK layer-shell selection controls.
//! The short input probe yields pointer proximity, never selection bounds.
#[path = "selection_bar_geometry.rs"]
mod geometry;

pub struct BarAction { pub id: String, pub name: String }

#[cfg(target_os = "linux")]
mod native {
    use super::*;
    use gtk::{gdk, glib, prelude::*};
    use glib::translate::ToGlibPtr;
    use std::{cell::RefCell, sync::{OnceLock,atomic::{AtomicU64, Ordering}}, time::{Duration,Instant}};
    type Window = *mut gtk::ffi::GtkWindow;
    // layer-shell registers GObject callbacks; its code must remain loaded even
    // after the last toolbar or this thread's Runtime has been dropped.
    static LAYER_LIBRARY:OnceLock<Result<libloading::Library,String>>=OnceLock::new();
    struct Layer {
        _library: &'static libloading::Library,
        init: unsafe extern "C" fn(Window),
        namespace: unsafe extern "C" fn(Window, *const libc::c_char),
        monitor: unsafe extern "C" fn(Window, *mut gdk::ffi::GdkMonitor),
        layer: unsafe extern "C" fn(Window, i32),
        keyboard: unsafe extern "C" fn(Window, i32),
        zone: unsafe extern "C" fn(Window, i32),
        anchor: unsafe extern "C" fn(Window, i32, i32),
        margin: unsafe extern "C" fn(Window, i32, i32),
    }
    impl Layer {
        fn load() -> Result<Self, String> {
            // Function pointers remain valid because the library is retained by this runtime.
            unsafe {
                let lib = LAYER_LIBRARY.get_or_init(||libloading::Library::new("libgtk-layer-shell.so.0").map_err(|e|e.to_string())).as_ref().map_err(Clone::clone)?;
                let supported: unsafe extern "C" fn()->i32 = *lib.get(b"gtk_layer_is_supported\0").map_err(|e|e.to_string())?;
                if supported()==0 { return Err("Wayland layer-shell protocol unavailable".into()); }
                macro_rules! sym { ($s:literal) => { *lib.get(concat!($s,"\0").as_bytes()).map_err(|e|e.to_string())? }; }
                Ok(Self { init:sym!("gtk_layer_init_for_window"), namespace:sym!("gtk_layer_set_namespace"),
                    monitor:sym!("gtk_layer_set_monitor"), layer:sym!("gtk_layer_set_layer"),
                    keyboard:sym!("gtk_layer_set_keyboard_mode"), zone:sym!("gtk_layer_set_exclusive_zone"),
                    anchor:sym!("gtk_layer_set_anchor"), margin:sym!("gtk_layer_set_margin"), _library:lib })
            }
        }
        fn configure(&self, w:&gtk::Window, monitor:&gdk::Monitor, probe:bool) {
            w.set_decorated(false); w.set_resizable(false); w.set_accept_focus(false); w.set_focus_on_map(false);
            unsafe {
                let p=w.to_glib_none().0;
                (self.init)(p); (self.namespace)(p,c"zwen-selection".as_ptr());
                (self.monitor)(p,monitor.to_glib_none().0); (self.layer)(p,3);
                (self.keyboard)(p,0); (self.zone)(p,0);
                // GtkLayerShellEdge: LEFT=0 RIGHT=1 TOP=2 BOTTOM=3.
                (self.anchor)(p,0,1); (self.anchor)(p,2,1);
                if probe { (self.anchor)(p,1,1); (self.anchor)(p,3,1); }
            }
        }
        fn place(&self,w:&gtk::Window,x:i32,y:i32) { unsafe {
            (self.margin)(w.to_glib_none().0,0,x); (self.margin)(w.to_glib_none().0,2,y);
        }}
    }
    #[derive(Clone)]
    struct Anchor {monitor:gdk::Monitor,x:f64,y:f64}
    struct PendingBar {app:tauri::AppHandle,id:u32,revision:u64,actions:std::rc::Rc<Vec<BarAction>>,pid:u32}
    #[derive(Default)]
    struct Runtime {
        layer:Option<std::rc::Rc<Layer>>, probes:Vec<gtk::Window>, bar:Option<gtk::Window>,
        probe_gate:geometry::ProbeGate,
        token:u64,anchor:Option<geometry::PrimedAnchor<Anchor>>,pending:Option<PendingBar>,
        buttons:Vec<(String,gtk::Button)>, selection:Option<u32>,
        monitor_handlers:Vec<(gdk::Monitor,glib::SignalHandlerId)>,
    }
    thread_local! { static UI: RefCell<Runtime> = RefCell::new(Runtime::default()); }
    static REQUEST:AtomicU64=AtomicU64::new(0);
    fn current(token:u64)->bool { geometry::accepts(REQUEST.load(Ordering::SeqCst),token) }
    fn clear() {
        // Extract ownership before any GTK call: unmap/destroy can synchronously emit signals.
        let (probes,bar,handlers,buttons)=UI.with(|ui| {
            let mut r=ui.borrow_mut();r.probe_gate.cancel();r.selection=None;r.anchor=None;r.pending=None;
            (std::mem::take(&mut r.probes),r.bar.take(),std::mem::take(&mut r.monitor_handlers),std::mem::take(&mut r.buttons))
        });
        for (monitor,handler) in handlers {monitor.disconnect(handler);}
        destroy_probes(probes);
        if let Some(w)=bar {unsafe {w.destroy();}}
        drop(buttons);
    }
    fn destroy_probes(probes:Vec<gtk::Window>) {
        // Revoke every input surface before destroying any of them.
        for w in &probes {w.input_shape_combine_region(Some(&gtk::cairo::Region::create()));}
        for w in probes {unsafe {w.destroy();}}
    }
    fn with_layer(f:impl FnOnce(&Layer)) {
        let layer=UI.with(|ui|ui.borrow().layer.clone());
        if let Some(layer)=layer {f(&layer);}
    }
    #[track_caller]
    fn cancel(token:u64) {
        #[cfg(feature="desktop-smoke")]eprintln!("BAR_TRACE cancel token={token} at {}",std::panic::Location::caller());
        if REQUEST.compare_exchange(token,token+1,Ordering::SeqCst,Ordering::SeqCst).is_err() {return;}
        clear();
    }
    pub fn dismiss(app:&tauri::AppHandle) {
        let token=REQUEST.fetch_add(1,Ordering::SeqCst)+1;
        let _=app.run_on_main_thread(move||{ if current(token) { clear(); }});
    }
    pub fn prime(app:&tauri::AppHandle,id:u32,pid:u32) {
        let token=REQUEST.fetch_add(1,Ordering::SeqCst)+1;let app2=app.clone();
        let _=app.run_on_main_thread(move||begin_prime(app2,token,id,pid));
    }
    pub fn discard_prime(app:&tauri::AppHandle,id:u32) {
        let _=app.run_on_main_thread(move|| {
            let token=UI.with(|ui| {let r=ui.borrow();if r.selection==Some(id){Some(r.token)}else{None}});
            if let Some(token)=token {cancel(token);}
        });
    }
    pub fn show(app:&tauri::AppHandle,id:u32,revision:u64,actions:Vec<BarAction>,pid:u32) {
        let token=REQUEST.load(Ordering::SeqCst);let a=app.clone();
        let _=app.run_on_main_thread(move|| {
            if !current(token)||!crate::selection::bar_is_current(&a,id,pid) {return;}
            let accepted=UI.with(|ui| {
                let mut r=ui.borrow_mut();
                if r.token!=token||r.bar.is_some()||!r.anchor.as_ref().is_some_and(|p|p.matches(id,pid)&&!p.expired()) {return false;}
                r.pending=Some(PendingBar{app:a,id,revision,actions:std::rc::Rc::new(actions),pid});true
            });
            #[cfg(feature="desktop-smoke")]eprintln!("BAR_TRACE show id={id} token={token} accepted={accepted}");
            if accepted {maybe_show(token);}
        });
    }
    fn maybe_show(token:u64) {
        if !current(token) {return;}
        let ready=UI.with(|ui| {
            let mut r=ui.borrow_mut();
            let point=r.pending.as_ref().and_then(|p|r.anchor.as_ref().and_then(|anchor|anchor.point_for(p.id,p.pid))).cloned();
            point.and_then(|point|r.pending.take().map(|pending|(point,pending)))
        });
        if let Some((point,p))=ready {
            if crate::selection::bar_is_current(&p.app,p.id,p.pid) {
                build_bar(p.app,token,p.id,p.revision,p.actions,p.pid,point.monitor,point.x,point.y,style());
            }else{cancel(token);}
        }
    }
    fn style()->gtk::CssProvider {
        let css=gtk::CssProvider::new();
        let _=css.load_from_data(b".zwen-probe {background-color:rgba(0,0,0,0);} .zwen-selection {background:#effbf2; color:#123d23; border:1px solid #6daf80; border-radius:10px;} .zwen-selection button {background:#d8f2df; color:#153e23; border:0; border-radius:6px; padding:5px 7px; min-height:22px; box-shadow:none;} .zwen-selection button:hover {background:#bce4c8;}");
        css
    }
    fn begin_prime(app:tauri::AppHandle,token:u64,id:u32,pid:u32) {
        if !current(token)||!crate::selection::anchor_is_current(&app,id,pid) {return;}
        clear();
        if !crate::selection::anchor_is_current(&app,id,pid) {#[cfg(feature="desktop-smoke")]eprintln!("BAR_TRACE prime rejected id={id}");return;}
        #[cfg(feature="desktop-smoke")]eprintln!("BAR_TRACE prime begin id={id} token={token}");
        if gtk::init().is_err() { eprintln!("selection bar: GTK unavailable"); return; }
        let Some(display)=gdk::Display::default() else { return; };
        let ready=UI.with(|ui| {
            let mut r=ui.borrow_mut(); r.selection=Some(id);r.token=token;r.anchor=Some(geometry::PrimedAnchor::new(id,pid));r.probe_gate.start();
            if r.layer.is_none() { match Layer::load() { Ok(l)=>r.layer=Some(std::rc::Rc::new(l)),Err(e)=>{eprintln!("selection bar unavailable: {e}");return false;} } }
            true
        });
        if !ready { return; }
        let css=style();
        let started=Instant::now();
        for i in 0..display.n_monitors() {
            if started.elapsed()>=Duration::from_millis(200) {cancel(token);break;}
            if !current(token)||!UI.with(|ui|ui.borrow().probe_gate.active()) {break;}
            let Some(monitor)=display.monitor(i) else {continue;};
            let h=monitor.connect_invalidate(move |_|cancel(token));
            let g=monitor.connect_notify_local(Some("geometry"),move |_,_|cancel(token));
            let s=monitor.connect_notify_local(Some("scale-factor"),move |_,_|cancel(token));
            UI.with(|ui| {let mut r=ui.borrow_mut();for handler in [h,g,s] {r.monitor_handlers.push((monitor.clone(),handler));}});
            let w=gtk::Window::new(gtk::WindowType::Toplevel); w.set_app_paintable(true);
            if let Some(screen)=gtk::prelude::WidgetExt::screen(&w) { if let Some(visual)=screen.rgba_visual() { w.set_visual(Some(&visual)); } }
            w.style_context().add_class("zwen-probe"); w.style_context().add_provider(&css,600);
            with_layer(|layer|layer.configure(&w,&monitor,true));
            w.add_events(gdk::EventMask::ENTER_NOTIFY_MASK|gdk::EventMask::BUTTON_PRESS_MASK|gdk::EventMask::BUTTON_RELEASE_MASK);
            let a=app.clone();let m=monitor.clone();
            w.connect_enter_notify_event(move|_,event| {
                if !current(token) {return glib::Propagation::Proceed;}
                let probing=UI.with(|ui|ui.borrow_mut().probe_gate.claim());
                if !probing {return glib::Propagation::Proceed;}
                let (x,y)=event.position();
                #[cfg(feature="desktop-smoke")]eprintln!("BAR_TRACE enter id={id} token={token}");
                // Clear synchronously, before any idle callback or next input dispatch.
                let probes=UI.with(|ui|std::mem::take(&mut ui.borrow_mut().probes));
                destroy_probes(probes);
                if current(token)&&crate::selection::anchor_is_current(&a,id,pid) {
                    UI.with(|ui| {if let Some(anchor)=ui.borrow_mut().anchor.as_mut(){anchor.capture(Anchor{monitor:m.clone(),x,y});}});
                    glib::idle_add_local_once(move||maybe_show(token));
                }else{cancel(token);}
                glib::Propagation::Proceed
            });
            w.connect_button_press_event(move|_,_|{cancel(token);glib::Propagation::Proceed});
            w.connect_button_release_event(move|_,_|{cancel(token);glib::Propagation::Proceed});
            if !current(token)||!UI.with(|ui|ui.borrow().probe_gate.active()) {unsafe {w.destroy();}break;}
            UI.with(|ui|ui.borrow_mut().probes.push(w.clone())); w.show_all();
        }
        // Maximum probe lifetime. This does not claim a compositor cannot swallow an intervening click.
        glib::timeout_add_local_once(Duration::from_millis(200).saturating_sub(started.elapsed()),move|| {
            if current(token)&&UI.with(|ui|ui.borrow().probe_gate.active()) {
                UI.with(|ui| {if let Some(anchor)=ui.borrow_mut().anchor.as_mut(){anchor.expire();}});
                cancel(token);
            }
        });
        let a=app.clone();
        glib::timeout_add_local(Duration::from_millis(120),move|| {
            if !current(token) {return glib::ControlFlow::Break;}
            let visible=UI.with(|ui|ui.borrow().bar.is_some());
            let valid=if visible {crate::selection::bar_is_current(&a,id,pid)}else{crate::selection::anchor_is_current(&a,id,pid)};
            if !valid {cancel(token);return glib::ControlFlow::Break;}
            glib::ControlFlow::Continue
        });
    }
    fn button(app:&tauri::AppHandle,token:u64,id:u32,revision:u64,pid:u32,key:&str,label:&str)->gtk::Button {
        let b=gtk::Button::with_label(&label.chars().take(18).collect::<String>()); b.set_can_focus(false);
        let css=gtk::CssProvider::new();
        let _=css.load_from_data(b"button {background:#d8f2df; color:#153e23; border:0; border-radius:6px; padding:5px 7px; min-height:22px; box-shadow:none;} button:hover {background:#bce4c8;}");
        b.style_context().add_provider(&css,600);
        let app=app.clone();let key=key.to_string();let action=key.clone();
        b.connect_clicked(move|_| {
            if !current(token) {return;}
            if !crate::selection::bar_is_current(&app,id,pid) {cancel(token);return;}
            cancel(token);crate::selection::toolbar_action(&app,id,&action,revision);
        });
        UI.with(|ui|ui.borrow_mut().buttons.push((key,b.clone())));b
    }
    #[allow(clippy::too_many_arguments)]
    fn build_bar(app:tauri::AppHandle,token:u64,id:u32,revision:u64,actions:std::rc::Rc<Vec<BarAction>>,pid:u32,monitor:gdk::Monitor,x:f64,y:f64,css:gtk::CssProvider) {
        let w=gtk::Window::new(gtk::WindowType::Toplevel);w.set_size_request(320,44);w.style_context().add_class("zwen-selection");w.style_context().add_provider(&css,600);
        let outer=gtk::Box::new(gtk::Orientation::Vertical,3);outer.set_border_width(5);
        let row=gtk::Box::new(gtk::Orientation::Horizontal,3);
        for (key,label) in [("ask","问一问 Zwen"),("translate","翻译"),("explain","详细解释")] {
            let b=button(&app,token,id,revision,pid,key,label);
            b.set_sensitive(key=="ask"||actions.iter().any(|a|a.id==key));
            row.pack_start(&b,false,false,0);
        }
        let other=gtk::ToggleButton::with_label("其他");other.set_can_focus(false); row.pack_start(&other,false,false,0);
        UI.with(|ui|ui.borrow_mut().buttons.push(("__other".into(),other.clone().upcast::<gtk::Button>())));
        row.pack_start(&button(&app,token,id,revision,pid,"__dismiss","×"),false,false,0);
        outer.pack_start(&row,false,false,0);
        let menu=gtk::Box::new(gtk::Orientation::Vertical,3);
        for action in actions.iter().filter(|a|!matches!(a.id.as_str(),"ask"|"translate"|"explain"|"__settings"|"__dismiss")) {
            menu.pack_start(&button(&app,token,id,revision,pid,&action.id,&action.name),false,false,0);
        }
        menu.pack_start(&button(&app,token,id,revision,pid,"__settings","管理快捷提示词"),false,false,0);
        let scroll=gtk::ScrolledWindow::new(None::<&gtk::Adjustment>,None::<&gtk::Adjustment>);
        scroll.set_policy(gtk::PolicyType::Never,gtk::PolicyType::Automatic);scroll.set_max_content_height(250);scroll.set_propagate_natural_height(true);scroll.add(&menu);
        outer.pack_start(&scroll,false,false,0);w.add(&outer);outer.show_all();scroll.hide();scroll.set_no_show_all(true);
        with_layer(|layer|layer.configure(&w,&monitor,false));
        if !current(token) {unsafe {w.destroy();}return;}
        UI.with(|ui|ui.borrow_mut().bar=Some(w.clone()));
        let geometry=monitor.geometry();let (_,natural)=w.preferred_size();
        let (left,top)=geometry::position(x,y,geometry.width(),geometry.height(),natural.width,natural.height);
        with_layer(|layer|layer.place(&w,left,top));
        w.show_all();scroll.hide();
        let s=scroll.clone();let ww=w.clone();let m=monitor.clone();
        other.connect_toggled(move|b| {
            if !current(token) {return;}
            s.set_visible(b.is_active());
            let (_,n)=ww.preferred_size();let g=m.geometry();let (l,t)=geometry::expanded_position(left,top,g.width(),g.height(),n.width,n.height);
            with_layer(|layer|layer.place(&ww,l,t));
        });
        let a=app.clone();
        glib::timeout_add_local(Duration::from_millis(120),move|| {
            if !current(token) {return glib::ControlFlow::Break;}
            if !crate::selection::bar_is_current(&a,id,pid) {cancel(token);return glib::ControlFlow::Break;}
            glib::ControlFlow::Continue
        });
    }
    #[cfg(feature="desktop-smoke")]
    pub fn smoke_inspect(app:&tauri::AppHandle) {
        use tauri::{Emitter,Manager};
        let a=app.clone();let _=app.run_on_main_thread(move||UI.with(|ui|{
            let r=ui.borrow();let (w,h)=r.bar.as_ref().map(|w|(w.allocated_width(),w.allocated_height())).unwrap_or((0,0));
            let _=a.emit("selection-bar-state",serde_json::json!({"mainVisible":a.get_webview_window("main").and_then(|w|w.is_visible().ok()).unwrap_or(false),"anchorKind":"pointer-proximity","buttons":r.buttons.iter().map(|(id,b)|serde_json::json!({"id":id,"mapped":b.is_mapped(),"sensitive":b.is_sensitive()})).collect::<Vec<_>>(),"visible":r.bar.is_some(),"probing":!r.probes.is_empty(),"selectionId":r.selection,"width":w,"height":h,"keyboardMode":"none","exclusiveZone":0}));
        }));
    }
    #[cfg(feature="desktop-smoke")]
    pub fn smoke_click(app:&tauri::AppHandle,action_id:String) {
        let _=app.run_on_main_thread(move|| {
            let button=UI.with(|ui|ui.borrow().buttons.iter().find(|(id,_)|id==&action_id).map(|(_,b)|b.clone()));
            if let Some(button)=button {if button.is_sensitive()&&button.is_mapped() {button.emit_clicked();}}
        });
    }
}
#[cfg(target_os="linux")]
pub use native::{show,dismiss,prime,discard_prime};
#[cfg(all(target_os="linux",feature="desktop-smoke"))]
pub use native::{smoke_click,smoke_inspect};
#[cfg(not(target_os="linux"))]
pub fn show(_: &tauri::AppHandle,_:u32,_:u64,_:Vec<BarAction>,_:u32) {}
#[cfg(not(target_os="linux"))]
pub fn dismiss(_: &tauri::AppHandle) {}

#[cfg(not(target_os="linux"))]
pub fn prime(_: &tauri::AppHandle,_:u32,_:u32) {}
#[cfg(not(target_os="linux"))]
pub fn discard_prime(_: &tauri::AppHandle,_:u32) {}
