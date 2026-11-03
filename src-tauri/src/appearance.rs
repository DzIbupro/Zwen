//! Isolated local appearance preferences; never opens model or draft databases.
use crate::ClientError;
use serde::{Deserialize,Serialize};
use std::path::Path;
#[derive(Debug,Clone,Copy,PartialEq,Eq,Default,Deserialize,Serialize)]
#[serde(rename_all="lowercase")]
pub enum Theme { #[default] Light, Dark }
#[derive(Debug,Clone,Copy,PartialEq,Eq,Default,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Preferences { pub theme:Theme,pub sidebar_collapsed:bool }
use rusqlite::{Connection,OpenFlags,TransactionBehavior};
use std::{fs,path::PathBuf,sync::{Mutex,atomic::{AtomicBool,Ordering}}};
static LOCK:Mutex<()>=Mutex::new(());
static DARK:AtomicBool=AtomicBool::new(false);
const TABLE:&str="CREATE TABLE preferences(id INTEGER PRIMARY KEY CHECK(id=1),json TEXT NOT NULL CHECK(length(CAST(json AS BLOB))<=256))";
fn invalid()->ClientError{ClientError{code:"invalid_appearance",message:"外观设置无法读取；原有文件已保留，请重试或检查本机设置文件".into()}}
pub fn cached_theme()->Theme{if DARK.load(Ordering::SeqCst){Theme::Dark}else{Theme::Light}}
fn cache(value:Preferences){DARK.store(value.theme==Theme::Dark,Ordering::SeqCst);}
impl Theme {pub fn native(self)->tauri::Theme{match self{Self::Light=>tauri::Theme::Light,Self::Dark=>tauri::Theme::Dark}}}
fn path(dir:&Path)->Result<PathBuf,ClientError>{
 match fs::symlink_metadata(dir){Ok(m) if !m.is_dir()||m.file_type().is_symlink()=>return Err(invalid()),Ok(_)=>{},Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{},Err(_)=>return Err(ClientError::storage())}
 for suffix in ["","-wal","-shm","-journal"]{match fs::symlink_metadata(dir.join(format!("appearance.sqlite3{suffix}"))){Ok(m) if !m.is_file()||m.file_type().is_symlink()=>return Err(invalid()),Ok(_)=>{},Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{},Err(_)=>return Err(ClientError::storage())}}
 Ok(dir.join("appearance.sqlite3"))
}
fn open_guard(path:&Path,create:bool)->Result<fs::File,ClientError>{
 let mut options=fs::OpenOptions::new();options.read(true).write(create);
 if create{options.create_new(true);}
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600).custom_flags(libc::O_NOFOLLOW|libc::O_NONBLOCK|libc::O_CLOEXEC);}
 let file=options.open(path).map_err(|_|ClientError::storage())?;
 let meta=file.metadata().map_err(|_|ClientError::storage())?;
 if !meta.is_file(){return Err(invalid());}
 #[cfg(unix)]{use std::os::unix::fs::MetadataExt;if meta.nlink()!=1{return Err(invalid());}}
 file.lock().map_err(|_|ClientError::storage())?;Ok(file)
}
fn connect(path:&Path,write:bool)->Result<Connection,ClientError>{
 let mode=if write{OpenFlags::SQLITE_OPEN_READ_WRITE}else{OpenFlags::SQLITE_OPEN_READ_ONLY};
 let c=Connection::open_with_flags(path,mode|OpenFlags::SQLITE_OPEN_NOFOLLOW|OpenFlags::SQLITE_OPEN_NO_MUTEX).map_err(|_|invalid())?;
 c.busy_timeout(std::time::Duration::from_secs(2)).map_err(|_|ClientError::storage())?;
 c.set_limit(rusqlite::limits::Limit::SQLITE_LIMIT_LENGTH,4096).map_err(|_|invalid())?;Ok(c)
}
fn read(c:&Connection)->Result<Preferences,ClientError>{
 let version:i64=c.pragma_query_value(None,"user_version",|r|r.get(0)).map_err(|_|invalid())?;
 if version!=1{return Err(invalid());}
 let objects:Vec<(String,String,String)>=c.prepare("SELECT type,name,sql FROM sqlite_master WHERE name NOT LIKE 'sqlite_%'").map_err(|_|invalid())?.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(|_|invalid())?.collect::<Result<_,_>>().map_err(|_|invalid())?;
 if objects!=vec![("table".into(),"preferences".into(),TABLE.into())]{return Err(invalid());}
 let count:i64=c.query_row("SELECT count(*) FROM preferences",[],|r|r.get(0)).map_err(|_|invalid())?;
 if count!=1{return Err(invalid());}
 let json:String=c.query_row("SELECT CASE WHEN length(CAST(json AS BLOB))<=256 THEN json END FROM preferences WHERE id=1",[],|r|r.get(0)).map_err(|_|invalid())?;
 serde_json::from_str(&json).map_err(|_|invalid())
}
fn load(dir:&Path)->Result<Preferences,ClientError>{
 let _lock=LOCK.lock().map_err(|_|ClientError::storage())?;let path=path(dir)?;
 if !path.try_exists().map_err(|_|ClientError::storage())?{return Ok(Preferences::default());}
 let _guard=open_guard(&path,false)?;read(&connect(&path,false)?)
}
fn save(dir:&Path,value:Preferences)->Result<Preferences,ClientError>{
 let _lock=LOCK.lock().map_err(|_|ClientError::storage())?;let path=path(dir)?;
 let mut builder=fs::DirBuilder::new();builder.recursive(true);
 #[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
 builder.create(dir).map_err(|_|ClientError::storage())?;
 let new=!path.try_exists().map_err(|_|ClientError::storage())?;
 let _guard=open_guard(&path,new)?;let mut c=connect(&path,true)?;
 let tx=c.transaction_with_behavior(TransactionBehavior::Immediate).map_err(|_|invalid())?;
 if new{tx.execute_batch(TABLE).map_err(|_|invalid())?;tx.pragma_update(None,"user_version",1).map_err(|_|invalid())?;tx.execute("INSERT INTO preferences VALUES(1,?1)",[serde_json::to_string(&Preferences::default()).map_err(|_|invalid())?]).map_err(|_|invalid())?;}
 read(&tx)?;
 tx.execute("UPDATE preferences SET json=?1 WHERE id=1",[serde_json::to_string(&value).map_err(|_|invalid())?]).map_err(|_|ClientError::storage())?;
 tx.commit().map_err(|_|ClientError::storage())?;cache(value);Ok(value)
}
/// All interpolated data is an enum or boolean, never arbitrary file contents.
pub fn initialization_script(value:Preferences)->String{format!("window.__ZWEN_APPEARANCE__={};",serde_json::to_string(&value).expect("enum-only appearance"))}
pub fn initialize(directory:&Result<PathBuf,ClientError>)->Preferences{
 let value=directory.as_ref().map_err(Clone::clone).and_then(|dir|load(dir)).unwrap_or_default();cache(value);value
}
#[tauri::command]
pub async fn load_appearance(storage:tauri::State<'_,crate::SharedStorage>)->Result<Preferences,ClientError>{
 let storage=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||load(storage.directory.as_ref().map_err(Clone::clone)?)).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn save_appearance(app:tauri::AppHandle,storage:tauri::State<'_,crate::SharedStorage>,preferences:Preferences)->Result<Preferences,ClientError>{
 let storage=storage.inner().clone();let saved=tauri::async_runtime::spawn_blocking(move||save(storage.directory.as_ref().map_err(Clone::clone)?,preferences)).await.map_err(|_|ClientError::storage())??;
 // Read the current cache when this callback runs: an older save callback must
 // never undo a newer committed theme. No activation, remap or geometry change.
 let a=app.clone();let _=app.run_on_main_thread(move||{use tauri::Manager;if let Some(w)=a.get_webview_window("main"){let _=w.set_theme(Some(cached_theme().native()));}crate::selection_bar::refresh_theme();});Ok(saved)
}
#[cfg(test)]
mod tests {
 use super::*;
 fn temp()->std::path::PathBuf {let p=std::env::temp_dir().join(format!("zwen-appearance-{}-{}",std::process::id(),zwen_core::notes::generate_id()));std::fs::create_dir_all(&p).unwrap();p}
 #[test]fn defaults_and_roundtrip(){let d=temp();assert_eq!(load(&d).unwrap(),Preferences::default());let value=Preferences{theme:Theme::Dark,sidebar_collapsed:true};assert_eq!(save(&d,value).unwrap(),value);assert_eq!(load(&d).unwrap(),value);std::fs::remove_dir_all(d).unwrap();}
 #[test]fn ipc_rejects_unknown_fields_themes_and_nonboolean_sidebar(){for raw in [r#"{"theme":"system","sidebarCollapsed":false}"#,r#"{"theme":"dark","sidebarCollapsed":0}"#,r#"{"theme":"dark","sidebarCollapsed":true,"key":"private"}"#]{assert!(serde_json::from_str::<Preferences>(raw).is_err());}}
 #[test]fn corrupt_and_future_databases_are_preserved(){for future in [false,true]{let d=temp();let path=d.join("appearance.sqlite3");if future{let c=rusqlite::Connection::open(&path).unwrap();c.pragma_update(None,"user_version",99).unwrap();}else{std::fs::write(&path,b"keep invalid bytes").unwrap();}let before=std::fs::read(&path).unwrap();assert!(load(&d).is_err());assert!(save(&d,Preferences::default()).is_err());assert_eq!(std::fs::read(&path).unwrap(),before);std::fs::remove_dir_all(d).unwrap();}}
 #[cfg(unix)]#[test]fn nonregular_and_symlink_database_and_sidecars_are_rejected(){use std::os::unix::fs::symlink;for suffix in ["","-wal","-shm","-journal"]{let d=temp();let target=d.join("untouched");std::fs::write(&target,b"keep").unwrap();symlink(&target,d.join(format!("appearance.sqlite3{suffix}"))).unwrap();assert!(load(&d).is_err());assert!(save(&d,Preferences::default()).is_err());assert_eq!(std::fs::read(&target).unwrap(),b"keep");std::fs::remove_dir_all(d).unwrap();}let d=temp();std::fs::create_dir(d.join("appearance.sqlite3")).unwrap();assert!(load(&d).is_err());std::fs::remove_dir_all(d).unwrap();}
 #[test]fn malformed_preferences_are_preserved(){let d=temp();save(&d,Preferences::default()).unwrap();let path=d.join("appearance.sqlite3");let c=rusqlite::Connection::open(&path).unwrap();c.execute("UPDATE preferences SET json='{}'",[]).unwrap();drop(c);assert!(load(&d).is_err());assert!(save(&d,Preferences::default()).is_err());let c=rusqlite::Connection::open(&path).unwrap();assert_eq!(c.query_row("SELECT json FROM preferences",[],|r|r.get::<_,String>(0)).unwrap(),"{}");drop(c);std::fs::remove_dir_all(d).unwrap();}
}
