//! Inbox access only. A WebView cannot initiate system selection reads.
use std::sync::{Arc,Mutex};
use tauri::{Emitter,Manager};
use zwen_core::selection::{Inbox,Item,Mode,read_primary_text};
use crate::{ClientError,SharedStorage};
pub type Shared=Arc<Mutex<Inbox>>;
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
/// Spawns a detached PRIMARY watcher (wl-paste --primary --watch) that feeds
/// captured selections back as candidate material. Only one watcher per app;
/// the thread dies with the process.
// --- candidate watcher persistence ---

#[tauri::command]
pub fn candidate_set_enabled(app:tauri::AppHandle,enabled:bool)->Result<bool,ClientError>{
 if enabled{start_candidate_watcher(app.clone());}
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
  Ok(true)=>start_candidate_watcher(app.clone()),
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

pub fn start_candidate_watcher(app:tauri::AppHandle){
 tauri::async_runtime::spawn_blocking(move||{
  let self_exe=std::env::current_exe().unwrap_or_default();
  let _=std::process::Command::new("wl-paste").args(["--primary","--watch",self_exe.to_string_lossy().into_owned().as_str(),"candidate-callback"])
   .stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn()
   .map(|child|std::thread::spawn(move||{let _=child;}));
 });
}

pub fn capture(app:tauri::AppHandle,mode:Mode){
 let shared=app.state::<Shared>().inner().clone();
 let (id,reading)=match shared.lock(){Ok(mut inbox)=>{let id=inbox.begin(mode);(id,inbox.reading())},Err(_)=>(None,false)};
 if id.is_none()&&reading{return;}
 let intent=crate::windows::new_intent();
 let _=app.emit("selection-ready",());
 tauri::async_runtime::spawn(async move{
  if let Some(id)=id{
   let worker=tauri::async_runtime::spawn_blocking(read_primary_text).await;
   let result=worker.unwrap_or(Err(zwen_core::selection::SelectionError::Unavailable));
   if let Ok(mut inbox)=shared.lock(){inbox.finish(id,result);}
   let _=app.emit("selection-ready",());
  }
  // Capture ends before any focus/activation request.
  let _=crate::windows::activate_if_current(app,true,intent).await;
 });
}
