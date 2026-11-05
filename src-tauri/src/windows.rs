use serde::Serialize;
use std::sync::{Arc,Mutex,atomic::{AtomicU64,Ordering}};
static WINDOW_INTENT:AtomicU64=AtomicU64::new(0);
pub fn new_intent()->u64{WINDOW_INTENT.fetch_add(1,Ordering::SeqCst).wrapping_add(1)}
fn current(intent:u64)->bool{WINDOW_INTENT.load(Ordering::SeqCst)==intent}
use zwen_core::surface::{CapsuleBounds,SurfaceState};
use tauri::{Emitter,Manager,LogicalSize};
use crate::{ClientError,niri};
#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Snapshot {pub mode:&'static str,pub generation:u64,pub bounds:CapsuleBounds,pub scale_factor:f64}
pub struct Controller {state:SurfaceState,bounds:CapsuleBounds,placement:Option<niri::Placement>}
pub type Shared=Arc<Mutex<Controller>>;
impl Controller {
    pub fn new(capsule:bool)->Self{let mut state=SurfaceState::new();state.set_mode(capsule);Self{state,bounds:CapsuleBounds::new(1920.0,1080.0),placement:None}}
    fn snapshot(&self,window:&tauri::WebviewWindow)->Snapshot{Snapshot{mode:if self.state.is_capsule(){"capsule"}else{"full"},generation:self.state.generation(),bounds:self.bounds,scale_factor:window.scale_factor().unwrap_or(1.0)}}
}
pub fn snapshot(app:&tauri::AppHandle)->Result<Snapshot,ClientError>{
 let window=app.get_webview_window("main").ok_or_else(ClientError::window)?;
 let shared=app.state::<Shared>();let guard=shared.lock().map_err(|_|ClientError::window())?;Ok(guard.snapshot(&window))
}
pub fn activate(app:tauri::AppHandle,capsule:bool)->impl std::future::Future<Output=Result<Snapshot,ClientError>>{
 crate::selection::dismiss_bar(&app);
 let intent=new_intent();activate_if_current(app,capsule,intent)
}
pub async fn activate_if_current(app:tauri::AppHandle,capsule:bool,intent:u64)->Result<Snapshot,ClientError>{
 tauri::async_runtime::spawn_blocking(move||{
    // Query the reading display before showing/focusing our own surface.
    let placement=if capsule{niri::focused()}else{None};
    let window=app.get_webview_window("main").ok_or_else(ClientError::window)?;
    let shared=app.state::<Shared>();let mut c=shared.lock().map_err(|_|ClientError::window())?;
    if !current(intent){return Ok(c.snapshot(&window));}
    window.hide().map_err(|_|ClientError::window())?;
    window.set_resizable(true).map_err(|_|ClientError::window())?;
    window.set_max_size(None::<tauri::Size>).map_err(|_|ClientError::window())?;
    window.set_min_size(None::<tauri::Size>).map_err(|_|ClientError::window())?;
    if capsule {
      let fallback=window.current_monitor().ok().flatten().map(|m|(m.size().width as f64/m.scale_factor(),m.size().height as f64/m.scale_factor())).unwrap_or((1920.0,1080.0));
      let (w,h)=placement.as_ref().map(|p|(p.width,p.height)).unwrap_or(fallback);
      c.bounds=CapsuleBounds::new(w,h);c.placement=placement;
      window.set_decorations(false).map_err(|_|ClientError::window())?;
      window.as_ref().set_size(LogicalSize::new(c.bounds.width,c.bounds.initial_height)).and_then(|_|window.set_size(LogicalSize::new(c.bounds.width,c.bounds.initial_height))).map_err(|_|ClientError::window())?;
      window.set_min_size(Some(LogicalSize::new(c.bounds.width,c.bounds.initial_height))).map_err(|_|ClientError::window())?;
      window.set_max_size(Some(LogicalSize::new(c.bounds.width,c.bounds.initial_height))).map_err(|_|ClientError::window())?;
      window.set_resizable(true).map_err(|_|ClientError::window())?;
    } else {
      window.set_decorations(true).map_err(|_|ClientError::window())?;
      window.set_min_size(Some(LogicalSize::new(480,560))).map_err(|_|ClientError::window())?;
      window.as_ref().set_size(LogicalSize::new(900,850)).and_then(|_|window.set_size(LogicalSize::new(900,850))).map_err(|_|ClientError::window())?;
      if let Some(p)=&c.placement {p.tile();}
    }
    c.state.set_mode(capsule);
    let snapshot=c.snapshot(&window);
    // Announce the new surface while the window is still unmapped so the
    // renderer switches layouts first; showing earlier painted a stale
    // full-window strip (the ghosted titlebar) into the capsule frame.
    let _=window.emit("surface-changed",&snapshot);
    std::thread::sleep(std::time::Duration::from_millis(120));
    if !current(intent){return Ok(c.snapshot(&window));}
    window.show().map_err(|_|ClientError::window())?;
    if capsule {let width=c.bounds.width;if let Some(p)=&mut c.placement {p.place(width);}else{let _=window.center();}}
    let _=window.set_focus();Ok(snapshot)
 }).await.map_err(|_|ClientError::window())?
}
pub async fn resize(app:tauri::AppHandle,height:f64,generation:u64)->Result<(),ClientError>{
 tauri::async_runtime::spawn_blocking(move||{
    let shared=app.state::<Shared>();let c=shared.lock().map_err(|_|ClientError::window())?;
    if !c.state.accept_resize(generation){return Ok(());}
    let window=app.get_webview_window("main").ok_or_else(ClientError::window)?;
    let height=c.bounds.height(height);let width=c.bounds.width;
    window.set_resizable(true).map_err(|_|ClientError::window())?;
    window.set_min_size(None::<tauri::Size>).map_err(|_|ClientError::window())?;
    window.set_max_size(None::<tauri::Size>).map_err(|_|ClientError::window())?;
    window.as_ref().set_size(LogicalSize::new(width,height)).and_then(|_|window.set_size(LogicalSize::new(width,height))).map_err(|_|ClientError::window())?;
    window.set_min_size(Some(LogicalSize::new(width,height))).map_err(|_|ClientError::window())?;
    window.set_max_size(Some(LogicalSize::new(width,height))).map_err(|_|ClientError::window())?;
    // Size changes leave the compositor position untouched after a user drag.
    Ok(())
 }).await.map_err(|_|ClientError::window())?
}

// Scale events must never take this mutex on the GTK event thread.
pub fn refresh_scale(app:tauri::AppHandle){
 tauri::async_runtime::spawn_blocking(move||{
  let Some(window)=app.get_webview_window("main") else{return;};
  let shared=app.state::<Shared>();let Ok(mut c)=shared.lock() else{return;};
  let capsule=c.state.is_capsule();c.state.set_mode(capsule);
  let snapshot=c.snapshot(&window);let _=window.emit("surface-changed",snapshot);
 });
}

// Hide/recover share the geometry lock with activation. New intent is reserved at
// request time, so a late capture cannot win even if its worker queues afterward.
pub fn hide(app:tauri::AppHandle)->impl std::future::Future<Output=Result<(),ClientError>>{present(app,false)}
pub fn recover(app:tauri::AppHandle)->impl std::future::Future<Output=Result<(),ClientError>>{present(app,true)}
fn present(app:tauri::AppHandle,reload:bool)->impl std::future::Future<Output=Result<(),ClientError>>{
 let intent=new_intent();
 async move{tauri::async_runtime::spawn_blocking(move||{
  let shared=app.state::<Shared>();let _guard=shared.lock().map_err(|_|ClientError::window())?;
  if !current(intent){return Ok(());}
  let window=app.get_webview_window("main").ok_or_else(ClientError::window)?;
  if reload{window.reload().map_err(|_|ClientError::window())?;window.show().map_err(|_|ClientError::window())?;let _=window.set_focus();}
  else{window.hide().map_err(|_|ClientError::window())?;}
  Ok(())
 }).await.map_err(|_|ClientError::window())?}
}

// Hide only when this request still owns the latest intent. Used before an
// interactive screenshot so the assistant cannot obscure the captured region;
// a user window action always wins over a stale hide.
pub async fn hide_if_current(app:tauri::AppHandle,intent:u64)->Result<bool,ClientError>{
 #[cfg(feature="desktop-smoke")]
 match std::env::var("ZWEN_CAPTURE_HIDE_FIXTURE_MODE").as_deref(){
  Ok("failure")=>return Err(ClientError::window()),
  Ok("stale")=>{new_intent();return Ok(false);},
  _=>{}
 }
 tauri::async_runtime::spawn_blocking(move||{
  let window=app.get_webview_window("main").ok_or_else(ClientError::window)?;
  let shared=app.state::<Shared>();let _guard=shared.lock().map_err(|_|ClientError::window())?;
  if !current(intent){return Ok(false);}
  window.hide().map_err(|_|ClientError::window())?;Ok(true)
 }).await.map_err(|_|ClientError::window())?
}
