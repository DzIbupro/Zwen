//! Inbox access only. A WebView cannot initiate system selection reads.
use std::sync::{Arc,Mutex};
use tauri::{Emitter,Manager};
use zwen_core::selection::{Inbox,Item,Mode,read_primary_text};
use crate::{ClientError,SharedStorage};
pub type Shared=Arc<Mutex<Inbox>>;
#[path="candidate_watcher.rs"] mod watcher;
static WATCHER:Mutex<watcher::Watcher>=Mutex::new(watcher::Watcher::new());
pub fn stop_candidate_watcher(){if let Ok(mut watcher)=WATCHER.lock(){watcher.stop();}}
pub fn candidate_callback(app:tauri::AppHandle,token:Option<&str>){
 if let Ok(mut watcher)=WATCHER.lock(){if watcher.accepts(token){capture_guarded(app,Mode::Candidate,token.map(str::to_owned));}}
}
#[derive(serde::Serialize)]pub struct Snapshot{reading:bool,item:Option<Item>}
#[tauri::command]
pub fn claim_selection(state:tauri::State<'_,Shared>)->Result<Snapshot,ClientError>{
 let mut inbox=state.lock().map_err(|_|ClientError::window())?;
 Ok(Snapshot{reading:inbox.reading(),item:inbox.claim()})
}
#[tauri::command]
pub fn acknowledge_selection(state:tauri::State<'_,Shared>,id:u32)->Result<(),ClientError>{
 if state.lock().map_err(|_|ClientError::window())?.acknowledge(id){Ok(())}else{Err(ClientError{code:"selection_changed",message:"待处理选区状态已改变，请重新读取".into()})}
}
/// Manage one owned PRIMARY watcher. Its child is killed and reaped when
/// consent is revoked or the application exits. Callbacks carry its generation.
// --- candidate watcher persistence ---

#[tauri::command]
pub fn candidate_set_enabled(app:tauri::AppHandle,enabled:bool)->Result<bool,ClientError>{
 let mut watcher=WATCHER.lock().map_err(|_|ClientError::window())?;
 // Revoke consent immediately, even if persisting the new setting fails.
 if !enabled{watcher.stop();crate::windows::new_intent();}
 let state=app.state::<crate::SharedStorage>();
 let dir=state.directory.as_ref().map_err(Clone::clone)?;
 let mut builder=std::fs::DirBuilder::new();builder.recursive(true);
 #[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
 builder.create(dir).map_err(|_|crate::ClientError::storage())?;
 let connection=zwen_core::open_private_database(&dir.join("settings.sqlite3"),|connection|{
  let version:i64=connection.pragma_query_value(None,"user_version",|r|r.get(0))?;
  if version!=0&&version!=1{return Err(zwen_core::Error::UnsupportedSchema);}
  if version==0{
   let tx=connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
   tx.execute_batch("CREATE TABLE settings(key TEXT PRIMARY KEY,value TEXT NOT NULL);PRAGMA user_version = 1;")?;
   tx.commit()?;
  }
  Ok(())
 })?;
 connection.execute("INSERT INTO settings(key,value) VALUES('candidate_watcher',?1) ON CONFLICT(key) DO UPDATE SET value=?1",[if enabled{"on"}else{"off"}]).map_err(zwen_core::Error::Database)?;
 if enabled{
  if let Err(error)=start_watcher(&mut watcher){
   let _=connection.execute("UPDATE settings SET value='off' WHERE key='candidate_watcher'",[]);
   return Err(error);
  }
 }
 Ok(enabled)
}

#[tauri::command]
pub fn candidate_get_enabled(storage:tauri::State<'_,SharedStorage>)->Result<bool,ClientError>{
 let dir=storage.directory.as_ref().map_err(Clone::clone)?;
 let path=dir.join("settings.sqlite3");
 if !path.exists(){return Ok(false);}
 let connection=rusqlite::Connection::open(&path).map_err(|_|crate::ClientError::storage())?;
 let version:i64=connection.pragma_query_value(None,"user_version",|r|r.get(0)).unwrap_or(0);
 if version!=1{return Ok(false);}
 use rusqlite::OptionalExtension;
 let value:String=connection.query_row("SELECT value FROM settings WHERE key='candidate_watcher'",[],|r|r.get(0)).optional().map_err(|_|crate::ClientError::storage())?.unwrap_or_else(||"off".into());
 Ok(value=="on")
}

pub fn restore_candidate_watcher_if_enabled(app:&tauri::AppHandle){
 match candidate_get_enabled_inner(app){
  Ok(true)=>{let _=candidate_set_enabled(app.clone(),true);},
  _=>{}
 }
}

fn candidate_get_enabled_inner(app:&tauri::AppHandle)->Result<bool,zwen_core::Error>{
 use rusqlite::OptionalExtension;
 let state=app.state::<crate::SharedStorage>();
 let dir=state.directory.as_ref().map_err(|_|zwen_core::Error::Io(std::io::Error::new(std::io::ErrorKind::Other,"no dir")))?;
 let path=dir.join("settings.sqlite3");
 if !path.exists(){return Ok(false);}
 let connection=rusqlite::Connection::open(&path).map_err(zwen_core::Error::Database)?;
 let value:String=connection.query_row("SELECT value FROM settings WHERE key='candidate_watcher'",[],|r|r.get(0)).optional().map_err(zwen_core::Error::Database)?.unwrap_or_else(||"off".into());
 Ok(value=="on")
}

fn start_watcher(watcher:&mut watcher::Watcher)->Result<(),ClientError>{
 if watcher.enabled(){return Ok(());}
 let self_exe=std::env::current_exe().map_err(|_|ClientError::window())?;
 let token=format!("{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
 let mut command=std::process::Command::new("wl-paste");
 command.args(["--primary","--watch"]).arg(self_exe).arg("candidate-callback").arg(&token)
  .stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
 watcher.start(&mut command,token).map_err(|_|ClientError{code:"watcher_unavailable",message:"自动划词监听未能启动；请确认 wl-paste 可用后重试".into()})
}

pub fn capture(app:tauri::AppHandle,mode:Mode){capture_guarded(app,mode,None);}
fn candidate_allowed(token:&Option<String>)->bool{
 token.as_deref().is_none_or(|token|WATCHER.lock().is_ok_and(|mut watcher|watcher.accepts(Some(token))))
}
fn capture_guarded(app:tauri::AppHandle,mode:Mode,token:Option<String>){
 let shared=app.state::<Shared>().inner().clone();
 let (id,reading)=match shared.lock(){Ok(mut inbox)=>{let id=inbox.begin(mode);(id,inbox.reading())},Err(_)=>(None,false)};
 if id.is_none()&&(reading||mode==Mode::Candidate){return;}
 let intent=crate::windows::new_intent();
 let _=app.emit("selection-ready",());
 tauri::async_runtime::spawn(async move{
  if let Some(id)=id{
   let read_token=token.clone();
   let worker=tauri::async_runtime::spawn_blocking(move||{
    if !candidate_allowed(&read_token){return Err(zwen_core::selection::SelectionError::Unavailable);}
    read_primary_text()
   }).await;
   let result=worker.unwrap_or(Err(zwen_core::selection::SelectionError::Unavailable));
   // Revalidate under the watcher lock at delivery. Disable/re-enable cannot
   // publish an old generation's text or race a successful finish.
   let delivered={
    let mut watcher=WATCHER.lock().ok();
    let allowed=token.as_deref().is_none_or(|token|watcher.as_mut().is_some_and(|watcher|watcher.accepts(Some(token))));
    if let Ok(mut inbox)=shared.lock(){if allowed{inbox.finish(id,result);}else{inbox.cancel_read(id);}}
    allowed
   };
   let _=app.emit("selection-ready",());
   if !delivered{return;}
  }
  // Capture ends before any focus/activation request.
  if candidate_allowed(&token){let _=crate::windows::activate_if_current(app,true,intent).await;}
 });
}
