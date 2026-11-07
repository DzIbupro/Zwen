//! Explicit screenshot requests. One request owns one private path; no image is
//! uploaded, and a cancelled/expired/late file is never attached to a newer draft.
use crate::{ClientError,SharedStorage,windows};
use base64::{Engine as _,engine::general_purpose::STANDARD};
use serde::Serialize;
use std::{fs,path::{Path,PathBuf},sync::{Arc,Mutex,atomic::{AtomicU64,Ordering}},time::{Duration,SystemTime,UNIX_EPOCH}};
use tauri::{AppHandle,Emitter,Manager};
use zwen_core::{
 attachment::{Attachment,AttachmentStore,MAX_ATTACHMENTS,valid_id},
 screenshot::{CaptureError,cleanup_incoming,default_incoming},
};

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct CaptureView{state:&'static str,request_id:Option<String>,replace_id:Option<String>}
#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Item{#[serde(flatten)] attachment:Attachment,thumbnail:Option<String>,error:Option<String>}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct View{items:Vec<Item>,capture:CaptureView,notice:Option<String>,max:usize}

struct Pending{id:String,replace:Option<String>,cancelled:bool}
#[derive(Default)]
pub struct State{pending:Option<Pending>,notice:Option<String>,settled:Option<String>}
pub type Shared=Arc<Mutex<State>>;
static COUNTER:AtomicU64=AtomicU64::new(0);

fn error(code:&'static str,message:&str)->ClientError{ClientError{code,message:message.into()}}
fn storage_dir(storage:&SharedStorage)->Result<PathBuf,ClientError>{storage.directory.as_ref().map(Clone::clone).map_err(Clone::clone)}
fn ensure_private_dir(path:&Path)->Result<(),ClientError>{
 let mut builder=fs::DirBuilder::new();builder.recursive(true);
 #[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
 builder.create(path).map_err(|_|ClientError::storage())
}
fn capture_timeout()->Option<Duration>{
 #[cfg(feature="desktop-smoke")]{
  if let Ok(value)=std::env::var("ZWEN_SCREENSHOT_TIMEOUT_MS"){if let Ok(ms)=value.parse::<u64>(){return Some(Duration::from_millis(ms.clamp(300,60_000)));}}
 }
 None
}
fn request_id()->String{
 use std::io::Read;
 let mut bytes=[0u8;16];
 let read=fs::File::open("/dev/urandom").and_then(|mut file|file.read_exact(&mut bytes)).is_ok();
 if !read{
  let nanos=SystemTime::now().duration_since(UNIX_EPOCH).map(|duration|duration.as_nanos()).unwrap_or(0);
  let value=nanos^((u128::from(std::process::id()))<<64)^u128::from(COUNTER.fetch_add(1,Ordering::Relaxed));
  bytes=value.to_le_bytes();
 }
 bytes.iter().map(|byte|format!("{byte:02x}")).collect()
}

fn capture_view(state:&State)->CaptureView{
 match &state.pending{
  Some(pending)=>CaptureView{state:"pending",request_id:Some(pending.id.clone()),replace_id:pending.replace.clone()},
  None=>CaptureView{state:"idle",request_id:None,replace_id:None},
 }
}
fn view(storage:&SharedStorage,shared:&Shared)->Result<View,ClientError>{
 // Match commit lock order: items and pending state must describe one instant.
 let state=shared.lock().map_err(|_|ClientError::window())?;
 let directory=storage_dir(storage)?;
 let store=AttachmentStore::open(&directory)?;
 let items=store.list()?.into_iter().map(|attachment|match store.thumbnail(&attachment.id){
  Ok(bytes)=>Item{attachment,thumbnail:Some(format!("data:image/png;base64,{}",STANDARD.encode(bytes))),error:None},
  Err(error)=>Item{attachment,thumbnail:None,error:Some(error.to_string())},
 }).collect();
 let (capture,notice)=(capture_view(&state),state.notice.clone());
 Ok(View{items,capture,notice,max:MAX_ATTACHMENTS})
}

fn notify(app:&AppHandle,shared:&Shared,notice:String){
 if let Ok(mut state)=shared.lock(){state.notice=Some(notice);}
 let _=app.emit("attachments-changed",());
}
fn cancel_pending(shared:&Shared,notice:&str)->bool{
 let Ok(mut state)=shared.lock()else{return false;};
 let Some(pending)=state.pending.as_mut()else{return false;};
 pending.cancelled=true;state.notice=Some(notice.into());true
}
fn finish(shared:&Shared,id:&str,notice:Option<String>){
 let Ok(mut state)=shared.lock()else{return;};
 let current=state.pending.as_ref().map(|pending|pending.id.as_str());
 if current==Some(id){
  state.pending=None;state.settled=Some(id.into());
  if let Some(notice)=notice{state.notice=Some(notice);}
 }else if current.is_none()&&state.settled.as_deref()==Some(id){
  if let Some(notice)=notice{state.notice=Some(notice);}
 }
}
// Before import there is no committed result: only this pending owner may
// finish, so a late hide failure cannot overwrite a newer request's notice.
fn abandon_before_capture(shared:&Shared,id:&str,notice:&str)->bool{
 let Ok(mut state)=shared.lock()else{return false;};
 if state.pending.as_ref().is_none_or(|pending|pending.id!=id){return false;}
 state.pending=None;state.notice=Some(notice.into());true
}
fn cancelled(shared:&Shared,id:&str)->bool{
 match shared.lock(){Ok(state)=>state.pending.as_ref().is_none_or(|pending|pending.id!=id||pending.cancelled),Err(_)=>true}
}
fn spawn_activate(app:&AppHandle){
 let app=app.clone();tauri::async_runtime::spawn(async move{let _=windows::activate(app,true).await;});
}

pub fn start_from_app(app:&AppHandle,replace:Option<String>){
 let storage=app.state::<SharedStorage>().inner().clone();
 let shared=app.state::<Shared>().inner().clone();
 start(app.clone(),storage,shared,replace);
}

pub fn start(app:AppHandle,storage:SharedStorage,shared:Shared,replace:Option<String>){
 // Re-trigger owns a new selector; the previous process observes cancellation.
 // Re-triggering means "restart the capture": drop the stale wait and begin
 // a fresh request instead of only reopening the capsule.
 if cancel_pending(&shared,"已取消上一次截图等待"){
  let _=app.emit("attachments-changed",());
 }
 let replace=match replace{
  Some(id) if valid_id(&id)=>Some(id),
  Some(_)=>{notify(&app,&shared,"待替换的截图无效；未开始截图".into());spawn_activate(&app);return;}
  None=>None,
 };
 let directory=match storage_dir(&storage){Ok(directory)=>directory,Err(error)=>{notify(&app,&shared,error.message);spawn_activate(&app);return;}};
 let incoming=default_incoming(&directory);
 if ensure_private_dir(&incoming).is_err(){notify(&app,&shared,ClientError::storage().message);spawn_activate(&app);return;}
 // No request can be pending here: abandoned request files from earlier
 // sessions are never attachable and are removed before a new capture.
 let _=cleanup_incoming(&incoming,Duration::ZERO);
 let id=request_id();let path=incoming.join(format!("{id}.png"));let intent=windows::new_intent();
 {
  let Ok(mut state)=shared.lock()else{return;};
  // The stale entry above stays Some until its worker exits; the new request
  // takes the slot now and the old worker sees an id mismatch and stops.
  state.settled=None;state.pending=Some(Pending{id:id.clone(),replace:replace.clone(),cancelled:false});
  state.notice=None;
 }
 let _=app.emit("attachments-changed",());
 tauri::async_runtime::spawn(run(app,storage,shared,id,path,intent,replace));
}

async fn run(app:AppHandle,storage:SharedStorage,shared:Shared,id:String,path:PathBuf,intent:u64,replace:Option<String>){
 // The assistant hides while Niri's interactive selection UI is up: a visible
 // window would obscure (and could contaminate) the region being captured.
 // hide_if_current keeps user window actions authoritative; the same intent
 // re-presents the capsule once the request settles, in every outcome.
 match windows::hide_if_current(app.clone(),intent).await{
  Ok(true)=>{},
  hidden=>{
   let failed=hidden.is_err();
   let notice=if failed{"Zwen 窗口未能隐藏；未开始截图，请重试"}else{"窗口状态已变化；已取消本次截图，草稿已保留"};
   if abandon_before_capture(&shared,&id,notice){let _=app.emit("attachments-changed",());}
   if failed{let _=windows::activate_if_current(app,true,intent).await;}
   return;
  }
 }
 let worker_shared=shared.clone();let worker_id=id.clone();let worker_path=path.clone();
 let result=tauri::async_runtime::spawn_blocking(move||{
  zwen_core::screenshot::capture(&worker_path,capture_timeout(),||cancelled(&worker_shared,&worker_id))
 }).await.unwrap_or(Err(CaptureError::Io));
 match result{
  Ok(())=>{
   let import_storage=storage.clone();let import_id=id.clone();let import_path=path.clone();let import_replace=replace.clone();
   let import_shared=shared.clone();
   let imported=tauri::async_runtime::spawn_blocking(move||->Result<Attachment,ClientError>{
    // Hold the ownership lock through the commit: cancel/restart either wins
    // before this boundary or observes an already committed capture.
    with_current_capture(&import_shared,&import_id,||{
    let directory=storage_dir(&import_storage)?;
    let mut store=AttachmentStore::open(&directory)?;
    let attachment=match import_replace{
     Some(old)=>store.replace(&import_id,&import_path,&old)?,
     None=>store.import(&import_id,&import_path)?,
    };
    Ok(attachment)
    })
   }).await;
   match imported{
    Ok(Ok(_))=>finish(&shared,&id,Some("截图已附加到草稿 · 未发送".into())),
    Ok(Err(error))=>{let _=fs::remove_file(&path);finish(&shared,&id,Some(error.message));}
    Err(_)=>{let _=fs::remove_file(&path);finish(&shared,&id,Some("截图保存未完成，请重试".into()));}
   }
  }
  Err(CaptureError::Cancelled)=>finish(&shared,&id,Some(CaptureError::Cancelled.to_string())),
  Err(error)=>finish(&shared,&id,Some(error.to_string())),
 }
 // A late writer can still land after expiry; remove it unless a newer capture
 // owns the directory now. This only touches 32-hex request names.
 if shared.lock().is_ok_and(|state|state.pending.is_none()){
  if let Some(directory)=path.parent(){let _=cleanup_incoming(directory,Duration::ZERO);}
 }
 let _=app.emit("attachments-changed",());
 let _=windows::activate_if_current(app,true,intent).await;
}

#[tauri::command]
pub async fn get_attachments(storage:tauri::State<'_,SharedStorage>,state:tauri::State<'_,Shared>)->Result<View,ClientError>{
 let storage=storage.inner().clone();let shared=state.inner().clone();
 tauri::async_runtime::spawn_blocking(move||view(&storage,&shared)).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub fn capture_screenshot(app:AppHandle,storage:tauri::State<'_,SharedStorage>,state:tauri::State<'_,Shared>,replace_id:Option<String>)->Result<(),ClientError>{
 if let Some(id)=&replace_id{if !valid_id(id){return Err(error("invalid_attachment","待替换的截图无效"));}}
 start(app,storage.inner().clone(),state.inner().clone(),replace_id);Ok(())
}
#[tauri::command]
pub fn cancel_screenshot(app:AppHandle,state:tauri::State<'_,Shared>)->Result<(),ClientError>{
 if cancel_pending(state.inner(),"截图已取消"){let _=app.emit("attachments-changed",());}
 Ok(())
}
#[tauri::command]
pub async fn remove_attachment(app:AppHandle,storage:tauri::State<'_,SharedStorage>,state:tauri::State<'_,Shared>,id:String)->Result<(),ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||{
  let directory=storage_dir(&storage)?;let mut store=AttachmentStore::open(&directory)?;store.remove(&id)?;Ok::<(),ClientError>(())
 }).await.map_err(|_|ClientError::storage())??;
 if let Ok(mut capture)=state.lock(){capture.notice=Some("截图已从草稿移除".into());}
 let _=app.emit("attachments-changed",());
 Ok(())
}

fn with_current_capture<T>(shared:&Shared,id:&str,commit:impl FnOnce()->Result<T,ClientError>)->Result<T,ClientError>{
 let mut state=shared.lock().map_err(|_|ClientError::window())?;
 if !may_import(&state,id){return Err(error("capture_cancelled","截图请求已取消；未附加图片"));}
 let result=commit();if result.is_ok(){state.pending=None;state.settled=Some(id.into());}drop(state);result
}
fn may_import(state:&State,id:&str)->bool{state.pending.as_ref().is_some_and(|pending|pending.id==id&&!pending.cancelled)}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn late_hide_failure_does_not_clear_newer_pending_or_completed_request(){
  let shared=Shared::default();{
   let mut state=shared.lock().unwrap();state.pending=Some(Pending{id:"new".into(),replace:None,cancelled:false});state.notice=Some("new status".into());
  }
  assert!(!abandon_before_capture(&shared,"old","old failure"));
  {let state=shared.lock().unwrap();assert_eq!(state.pending.as_ref().unwrap().id,"new");assert_eq!(state.notice.as_deref(),Some("new status"));}
  shared.lock().unwrap().pending=None;
  assert!(!abandon_before_capture(&shared,"old","old failure"));assert_eq!(shared.lock().unwrap().notice.as_deref(),Some("new status"));
 }
 #[test]fn committed_capture_is_no_longer_cancellable(){
  let shared=Shared::default();shared.lock().unwrap().pending=Some(Pending{id:"done".into(),replace:None,cancelled:false});
  with_current_capture(&shared,"done",||Ok(())).unwrap();
  assert!(!cancel_pending(&shared,"cancelled"),"a committed image must not be reported as cancelled");
  assert!(shared.lock().unwrap().pending.is_none());
 }
 #[test]fn cancellation_and_commit_share_one_boundary(){
  use std::{sync::mpsc,thread};
  let shared=Shared::default();shared.lock().unwrap().pending=Some(Pending{id:"old".into(),replace:None,cancelled:false});
  cancel_pending(&shared,"cancelled");
  assert!(with_current_capture(&shared,"old",||->Result<(),ClientError>{panic!("cancelled capture committed")}).is_err());
  shared.lock().unwrap().pending.as_mut().unwrap().cancelled=false;
  let (entered,ready)=mpsc::channel();let (release,resume)=mpsc::channel();let worker=shared.clone();
  let commit=thread::spawn(move||with_current_capture(&worker,"old",||{entered.send(()).unwrap();resume.recv().unwrap();Ok(())}));
  ready.recv().unwrap();let cancellation=shared.clone();let (cancelled,observed)=mpsc::channel();
  let cancel=thread::spawn(move||{cancel_pending(&cancellation,"cancelled");cancelled.send(()).unwrap();});
  assert!(observed.recv_timeout(Duration::from_millis(30)).is_err(),"cancellation crossed an active commit");
  release.send(()).unwrap();assert!(commit.join().unwrap().is_ok());cancel.join().unwrap();observed.recv().unwrap();
  assert!(!may_import(&shared.lock().unwrap(),"old"));
 }
 #[test]fn obsolete_or_cancelled_capture_cannot_import(){
  let mut state=State::default();assert!(!may_import(&state,"old"));
  state.pending=Some(Pending{id:"old".into(),replace:None,cancelled:false});assert!(may_import(&state,"old"));
  state.pending.as_mut().unwrap().cancelled=true;assert!(!may_import(&state,"old"));
  state.pending=Some(Pending{id:"new".into(),replace:None,cancelled:false});assert!(!may_import(&state,"old"));assert!(may_import(&state,"new"));
 }
}

/// Exit invalidates every pending selector without reactivating a window.
pub fn shutdown(app:&AppHandle){
 windows::new_intent();if let Some(state)=app.try_state::<Shared>(){let _=cancel_pending(state.inner(),"截图已取消");}
 zwen_core::screenshot::shutdown_capture_processes();
 if let Some(storage)=app.try_state::<SharedStorage>(){if let Ok(directory)=storage_dir(storage.inner()){let _=cleanup_incoming(&default_incoming(&directory),Duration::ZERO);}}
}

#[cfg(test)]mod settled_tests{use super::*;
 #[test]fn old_completion_cannot_replace_new_committed_notice(){let shared=Shared::default();shared.lock().unwrap().pending=Some(Pending{id:"new".into(),replace:None,cancelled:false});with_current_capture(&shared,"new",||Ok(())).unwrap();finish(&shared,"new",Some("new saved".into()));finish(&shared,"old",Some("old error".into()));assert_eq!(shared.lock().unwrap().notice.as_deref(),Some("new saved"));}
}
