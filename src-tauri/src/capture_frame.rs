//! Submit an empty RGBA root buffer before unmapping the assistant.
//! All GTK values stay on the main thread; a lease restores every widget flag.
use crate::{ClientError,windows};
use gtk::{prelude::*,glib};
use std::{cell::{Cell,RefCell},rc::Rc,time::{Duration,Instant}};
use tauri::{AppHandle,Manager};
struct Blank{token:u64,window:gtk::ApplicationWindow,children:Vec<(gtk::Widget,bool)>,paintable:bool,handler:glib::SignalHandlerId}
thread_local!{static BLANK:RefCell<Option<Blank>>=const{RefCell::new(None)};}
fn restore_main(token:u64){BLANK.with(|slot|{
 let mut slot=slot.borrow_mut();if !slot.as_ref().is_some_and(|b|b.token==token){return;}
 let b=slot.take().unwrap();b.window.disconnect(b.handler);b.window.set_app_paintable(b.paintable);
 for (child,visible) in b.children{child.set_visible(visible);}b.window.queue_draw();
});}
fn ready(frame:Option<i64>,baseline:i64,acknowledged:bool)->bool{frame.is_some_and(|f|f>baseline)&&acknowledged}
pub async fn blank(app:AppHandle,intent:u64)->Result<Option<u64>,ClientError>{
 let (tx,rx)=tokio::sync::oneshot::channel();
 let handle=app.clone();app.run_on_main_thread(move||{
  if tx.is_closed(){return;}
  if !windows::current(intent){let _=tx.send(Ok(None));return;}
  let Some(web)=handle.get_webview_window("main")else{let _=tx.send(Err(ClientError::window()));return;};
  let Ok(window)=web.gtk_window()else{let _=tx.send(Err(ClientError::window()));return;};
  if !window.display().type_().name().contains("Wayland"){let _=tx.send(Ok(None));return;}
  if !gtk::prelude::WidgetExt::is_visible(&window){let _=tx.send(Ok(None));return;}
  let Some(clock)=window.frame_clock()else{let _=tx.send(Err(ClientError::window()));return;};
  if window.visual().is_none_or(|v|v.depth()!=32){let _=tx.send(Err(ClientError::window()));return;}
  let previous=BLANK.with(|s|s.borrow().as_ref().map(|b|b.token));if let Some(token)=previous{restore_main(token);}
  let baseline=clock.frame_counter();let painted=Rc::new(Cell::new(None));let mark=painted.clone();let draw_clock=clock.clone();
  let paintable=window.is_app_paintable();let children=window.children().into_iter().map(|c|{let visible=c.is_visible();c.hide();(c,visible)}).collect();
  window.set_app_paintable(true);
  let handler=window.connect_draw(move|_,cr|{
   cr.set_operator(gtk::cairo::Operator::Source);cr.set_source_rgba(0.,0.,0.,0.);
   if cr.paint().is_ok()&&mark.get().is_none(){mark.set(Some(draw_clock.frame_counter()));}glib::Propagation::Stop
  });
  BLANK.with(|s|*s.borrow_mut()=Some(Blank{token:intent,window:window.clone(),children,paintable,handler}));window.queue_draw();
  let began=Instant::now();let mut tx=Some(tx);let mut ack:Option<crate::wayland_ack::Ack>=None;
  glib::timeout_add_local(Duration::from_millis(8),move||{
   let frame=painted.get();#[cfg(feature="desktop-smoke")]let timing=frame.and_then(|f|clock.timings(f));
   // The paint signal has returned before this main-loop timer runs, so GDK's
   // after-paint commit precedes this sync request on the same connection.
   if frame.is_some()&&ack.is_none(){ack=crate::wayland_ack::Ack::new(&window.display());}
   let done=ready(frame,baseline,ack.as_ref().is_some_and(|a|a.done()));
   let stale=!windows::current(intent)||tx.as_ref().is_none_or(|t|t.is_closed());
   if stale||began.elapsed()>Duration::from_secs(2){
    #[cfg(feature="desktop-smoke")]{use tauri::Emitter;let _=handle.emit("capture-frame-proof",serde_json::json!({"failed":true,"stale":stale,"baseline":baseline,"painted":frame,"counter":clock.frame_counter(),"complete":timing.as_ref().map(|t|t.is_complete()),"presentation":timing.as_ref().and_then(|t|t.presentation_time()).map(|v|v.get())}));}
    restore_main(intent);if let Some(tx)=tx.take(){let _=tx.send(Err(ClientError::window()));}return glib::ControlFlow::Break;}
   if done{
    #[cfg(feature="desktop-smoke")]{
     use tauri::Emitter;
     let mut surface=gtk::cairo::ImageSurface::create(gtk::cairo::Format::ARgb32,window.allocated_width(),window.allocated_height()).ok();
     let alpha=surface.as_mut().and_then(|s|{let cr=gtk::cairo::Context::new(&*s).ok()?;window.draw(&cr);drop(cr);s.flush();Some(s.data().ok()?.chunks_exact(4).map(|p|p[3]).max().unwrap_or(255))});
     let _=handle.emit("capture-frame-proof",serde_json::json!({"frame":frame,"complete":true,"waylandSync":true,"alphaMax":alpha}));
    }
    if let Some(tx)=tx.take(){if tx.send(Ok(Some(intent))).is_err(){restore_main(intent);}}return glib::ControlFlow::Break;
   }
   glib::ControlFlow::Continue
  });
 }).map_err(|_|ClientError::window())?;
 match tokio::time::timeout(Duration::from_secs(3),rx).await{Ok(Ok(result))=>result,_=>Err(ClientError::window())}
}
pub async fn restore(app:AppHandle,token:Option<u64>){
 if let Some(token)=token{let (tx,rx)=tokio::sync::oneshot::channel();if app.run_on_main_thread(move||{restore_main(token);let _=tx.send(());}).is_ok(){let _=rx.await;}}
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn old_or_unacknowledged_frames_never_satisfy_barrier(){assert!(!ready(Some(2),2,true));assert!(!ready(Some(3),2,false));assert!(!ready(None,2,true));assert!(ready(Some(3),2,true));}
}
