//! Explicit screenshot requests. One request owns one private path; no image is
//! uploaded, and a cancelled/expired/late file is never attached to a newer draft.
use crate::{ClientError,SharedStorage,windows};
use base64::{Engine as _,engine::general_purpose::STANDARD};
use serde::Serialize;
use std::{fs,path::{Path,PathBuf},sync::{Arc,Mutex,atomic::{AtomicU64,Ordering}},time::{Duration,SystemTime,UNIX_EPOCH}};
use tauri::{AppHandle,Emitter,Manager};
use zwen_core::{
 attachment::{Attachment,AttachmentStore,MAX_ATTACHMENTS,valid_id},
 screenshot::{CaptureError,DEFAULT_CAPTURE_TIMEOUT,cleanup_incoming,default_incoming},
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
pub struct State{pending:Option<Pending>,notice:Option<String>}
pub type Shared=Arc<Mutex<State>>;
static COUNTER:AtomicU64=AtomicU64::new(0);

fn error(code:&'static str,message:&str)->ClientError{ClientError{code,message:message.into()}}
fn storage_dir(storage:&SharedStorage)->Result<PathBuf,ClientError>{storage.directory.as_ref().map(Clone::clone).map_err(Clone::clone)}
fn ensure_private_dir(path:&Path)->Result<(),ClientError>{
 let mut builder=fs::DirBuilder::new();builder.recursive(true);
 #[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
 builder.create(path).map_err(|_|ClientError::storage())
}
fn capture_timeout()->Duration{
 #[cfg(feature="desktop-smoke")]{
  if let Ok(value)=std::env::var("ZWEN_SCREENSHOT_TIMEOUT_MS"){if let Ok(ms)=value.parse::<u64>(){return Duration::from_millis(ms.clamp(300,60_000));}}
 }
 DEFAULT_CAPTURE_TIMEOUT
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
 let directory=storage_dir(storage)?;
 let store=AttachmentStore::open(&directory)?;
 let items=store.list()?.into_iter().map(|attachment|match store.thumbnail(&attachment.id){
  Ok(bytes)=>Item{attachment,thumbnail:Some(format!("data:image/png;base64,{}",STANDARD.encode(bytes))),error:None},
  Err(error)=>Item{attachment,thumbnail:None,error:Some(error.to_string())},
 }).collect();
 let (capture,notice)={let state=shared.lock().map_err(|_|ClientError::window())?;(capture_view(&state),state.notice.clone())};
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
  state.pending=None;
  if let Some(notice)=notice{state.notice=Some(notice);}
 }else if current.is_none(){
  if let Some(notice)=notice{state.notice=Some(notice);}
 }
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
 if cancel_pending(&shared,"已取消上一次截图等待；若 Niri 截图界面仍打开，可按 Esc 关闭"){
  let _=app.emit("attachments-changed",());
  spawn_activate(&app);
  return;
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
  if state.pending.is_some(){return;}
  state.pending=Some(Pending{id:id.clone(),replace:replace.clone(),cancelled:false});
  state.notice=None;
 }
 let _=app.emit("attachments-changed",());
 tauri::async_runtime::spawn(run(app,storage,shared,id,path,intent,replace));
}

async fn run(app:AppHandle,storage:SharedStorage,shared:Shared,id:String,path:PathBuf,intent:u64,replace:Option<String>){
 // Keep the pending/cancel state visible; a late result never overrides a newer
 // user window action because both activations share this intent token.
 if windows::activate_if_current(app.clone(),true,intent).await.is_err(){
  finish(&shared,&id,Some("窗口操作未完成，截图未开始".into()));
  let _=app.emit("attachments-changed",());
  return;
 }
 let worker_shared=shared.clone();let worker_id=id.clone();let worker_path=path.clone();
 let result=tauri::async_runtime::spawn_blocking(move||{
  zwen_core::screenshot::capture(&worker_path,capture_timeout(),||cancelled(&worker_shared,&worker_id))
 }).await.unwrap_or(Err(CaptureError::Io));
 match result{
  Ok(())=>{
   let import_storage=storage.clone();let import_id=id.clone();let import_path=path.clone();let import_replace=replace.clone();
   let imported=tauri::async_runtime::spawn_blocking(move||->Result<Attachment,ClientError>{
    let directory=storage_dir(&import_storage)?;
    let mut store=AttachmentStore::open(&directory)?;
    let attachment=match import_replace{
     Some(old)=>store.replace(&import_id,&import_path,&old)?,
     None=>store.import(&import_id,&import_path)?,
    };
    Ok(attachment)
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
 if cancel_pending(state.inner(),"已取消等待截图；若 Niri 截图界面仍打开，可按 Esc 关闭"){let _=app.emit("attachments-changed",());}
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
