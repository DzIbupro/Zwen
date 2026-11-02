//! Local action configuration and immutable request/output bindings.
use crate::{ClientError,SharedStorage};
use rusqlite::{Connection,OptionalExtension,params,TransactionBehavior};
use serde::{Deserialize,Serialize};
use std::{fs,io::{Read,Write},path::{Path,Component},sync::Mutex};
use zwen_core::{journal::{Journal,TurnState},notes::NoteStore};
static LOCK:Mutex<()>=Mutex::new(());
fn error(code:&'static str,message:&str)->ClientError{ClientError{code,message:message.into()}}
fn invalid()->ClientError{error("invalid_quick_actions","快捷提示词配置无效或损坏，请检查原文件")}
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct QuickAction{pub id:String,pub name:String,pub prompt:String,pub output:String,pub enabled:bool}
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct QuickActionConfig{pub revision:u64,pub actions:Vec<QuickAction>,pub output_directory:String}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct ActionDocument{pub note_id:String,pub path:String}
fn defaults(directory:String)->QuickActionConfig{QuickActionConfig{revision:0,output_directory:directory,actions:vec![
 QuickAction{id:"translate".into(),name:"翻译".into(),prompt:"请将以下选中文字翻译成中文，保留原意：\n\n{{selection}}".into(),output:"display".into(),enabled:true},
 QuickAction{id:"explain".into(),name:"详细解释".into(),prompt:"请详细解释以下选中文字：\n\n{{selection}}".into(),output:"display".into(),enabled:true},
 QuickAction{id:"document".into(),name:"整理成文档".into(),prompt:"请将以下选中文字整理为结构清晰的 Markdown 文档：\n\n{{selection}}".into(),output:"document".into(),enabled:true}]}}
fn validate(c:&QuickActionConfig)->Result<(),ClientError>{
 let mut ids=std::collections::HashSet::new();
 if c.revision>i64::MAX as u64||c.actions.len()<2||c.actions.len()>32||!Path::new(&c.output_directory).is_absolute()||c.output_directory.len()>4096||c.output_directory.chars().any(char::is_control){return Err(invalid());}
 for a in &c.actions{if a.id.is_empty()||a.id.len()>64||!a.id.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_')||!ids.insert(a.id.as_str())||a.name.trim().is_empty()||a.name.len()>200||a.name.chars().any(char::is_control)||a.prompt.trim().is_empty()||a.prompt.len()>16384||a.prompt.contains('\0')||!matches!(a.output.as_str(),"display"|"document"){return Err(invalid());}}
 if !ids.contains("translate")||!ids.contains("explain")||serde_json::to_vec(c).map_err(|_|invalid())?.len()>600000{return Err(invalid());}Ok(())
}
fn connection(dir:&Path)->Result<Connection,ClientError>{
 let mut builder=fs::DirBuilder::new();builder.recursive(true);
 #[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
 builder.create(dir).map_err(|_|ClientError::storage())?;
 for suffix in ["","-wal","-shm"]{let path=dir.join(format!("quick-actions.sqlite3{suffix}"));match fs::symlink_metadata(path){Ok(meta) if !meta.is_file()||meta.file_type().is_symlink()=>return Err(invalid()),Ok(_)=>{},Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{},Err(_)=>return Err(ClientError::storage())}}
 Ok(zwen_core::open_private_database(&dir.join("quick-actions.sqlite3"),|c|{c.execute_batch("CREATE TABLE IF NOT EXISTS config(id INTEGER PRIMARY KEY CHECK(id=1),json TEXT NOT NULL);CREATE TABLE IF NOT EXISTS bindings(request_id TEXT PRIMARY KEY,selection TEXT NOT NULL,action_id TEXT NOT NULL,action_name TEXT NOT NULL,prompt TEXT NOT NULL,output_directory TEXT NOT NULL,output TEXT NOT NULL,note_id TEXT,path TEXT);")?;Ok(())})?)
}
fn read_config(c:&Connection,default_dir:&str)->Result<QuickActionConfig,ClientError>{
 let raw:Option<String>=c.query_row("SELECT CASE WHEN length(CAST(json AS BLOB))<=600000 THEN json END FROM config WHERE id=1",[],|r|r.get(0)).optional().map_err(|_|invalid())?;
 let value=match raw{Some(s)=>serde_json::from_str(&s).map_err(|_|invalid())?,None=>defaults(default_dir.into())};validate(&value)?;Ok(value)
}
fn default_directory()->Result<String,ClientError>{let home=std::env::var_os("HOME").ok_or_else(ClientError::storage)?;Ok(Path::new(&home).join("Documents/Zwen").to_string_lossy().into_owned())}
fn save_config(dir:&Path,default_dir:&str,expected:u64,actions:Vec<QuickAction>,output_directory:String)->Result<QuickActionConfig,ClientError>{
 let _lock=LOCK.lock().map_err(|_|ClientError::storage())?;let mut c=connection(dir)?;let tx=c.transaction_with_behavior(TransactionBehavior::Immediate).map_err(|_|ClientError::storage())?;let old=read_config(&tx,default_dir)?;
 if old.revision!=expected{return Err(error("quick_actions_conflict","快捷提示词已被其他窗口修改；当前编辑已保留，请重新读取后合并"));}
 let value=QuickActionConfig{revision:expected.checked_add(1).ok_or_else(invalid)?,actions,output_directory};validate(&value)?;let json=serde_json::to_string(&value).map_err(|_|invalid())?;tx.execute("INSERT INTO config(id,json) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET json=excluded.json",[json]).map_err(|_|ClientError::storage())?;tx.commit().map_err(|_|ClientError::storage())?;Ok(value)
}
#[tauri::command]
pub async fn load_quick_actions(storage:tauri::State<'_,SharedStorage>)->Result<QuickActionConfig,ClientError>{let storage=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||{let dir=storage.directory.as_ref().map_err(Clone::clone)?;read_config(&connection(dir)?,&default_directory()?) }).await.map_err(|_|ClientError::storage())?}
#[tauri::command]
pub async fn save_quick_actions(storage:tauri::State<'_,SharedStorage>,expected_revision:u64,actions:Vec<QuickAction>,output_directory:String)->Result<QuickActionConfig,ClientError>{let storage=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||save_config(storage.directory.as_ref().map_err(Clone::clone)?,&default_directory()?,expected_revision,actions,output_directory)).await.map_err(|_|ClientError::storage())?}
pub fn config_for_bar(storage:&SharedStorage)->Result<QuickActionConfig,ClientError>{
 let dir=storage.directory.as_ref().map_err(Clone::clone)?;
 read_config(&connection(dir)?,&default_directory()?)
}
/// Called only by the independent selection request path, before Journal::begin.
pub fn prepare_request(storage:&SharedStorage,request_id:&str,selection:&str,action_id:&str,expected_revision:u64)->Result<String,ClientError>{
 let dir=storage.directory.as_ref().map_err(Clone::clone)?;prepare_checked(dir,&default_directory()?,request_id,selection,action_id,Some(expected_revision))
}
#[cfg(test)]
fn prepare(dir:&Path,default_dir:&str,id:&str,selection:&str,action_id:&str)->Result<String,ClientError>{prepare_checked(dir,default_dir,id,selection,action_id,None)}
fn prepare_checked(dir:&Path,default_dir:&str,id:&str,selection:&str,action_id:&str,expected_revision:Option<u64>)->Result<String,ClientError>{
 let _lock=LOCK.lock().map_err(|_|ClientError::storage())?;
 if !zwen_core::notes::valid_request(id)||selection.trim().is_empty()||selection.len()>64*1024{return Err(invalid());}
 let c=connection(dir)?;let config=read_config(&c,default_dir)?;if expected_revision.is_some_and(|revision|revision!=config.revision){return Err(error("quick_actions_conflict","快捷提示词已变化，请在设置中重新读取后再选择动作；选区已保留"));}let action=config.actions.iter().find(|a|a.id==action_id&&a.enabled).ok_or_else(invalid)?;
 let prompt=if action.prompt.contains("{{selection}}"){action.prompt.replace("{{selection}}",selection)}else{format!("{}\n\n{}",action.prompt,selection)};
 if prompt.len()>zwen_core::MAX_DRAFT_BYTES{return Err(invalid());}
 c.execute("INSERT INTO bindings(request_id,selection,action_id,action_name,prompt,output_directory,output) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![id,selection,action.id,action.name,prompt,config.output_directory,action.output]).map_err(|_|error("action_request_conflict","此动作请求标识已使用"))?;Ok(prompt)
}
#[tauri::command]
pub async fn save_action_document(storage:tauri::State<'_,SharedStorage>,request_id:String,selection:String,action_id:String,action_name:String,output_directory:String)->Result<ActionDocument,ClientError>{let storage=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||save_document(storage.directory.as_ref().map_err(Clone::clone)?,&request_id,&selection,&action_id,&action_name,&output_directory)).await.map_err(|_|ClientError::storage())?}
fn save_document(dir:&Path,id:&str,selection:&str,action_id:&str,action_name:&str,output_directory:&str)->Result<ActionDocument,ClientError>{
 let _lock=LOCK.lock().map_err(|_|ClientError::storage())?;let c=connection(dir)?;
 let (saved_selection,saved_action,name,prompt,directory,output,note_id,path):(String,String,String,String,String,String,Option<String>,Option<String>)=c.query_row("SELECT selection,action_id,action_name,prompt,output_directory,output,note_id,path FROM bindings WHERE request_id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?))).map_err(|_|invalid())?;
 if saved_selection!=selection||saved_action!=action_id||name!=action_name||directory!=output_directory||output!="document"{return Err(invalid());}
 let turn=Journal::open(&dir.join("chat.sqlite3"))?.find(id)?.ok_or_else(invalid)?;
 if turn.state!=TurnState::Complete||turn.prompt!=prompt||turn.answer.trim().is_empty()||turn.error.is_some()||turn.context_turns!=0||turn.conversation_id!=id{return Err(error("action_incomplete","只有完整且已保存的独立动作回答才能保存文档"));}
 let mut store=NoteStore::open(dir)?;
 let body=format!("## 原始选区\n\n{selection}\n\n## {name}\n\n{}",turn.answer);
 let note_id=match note_id{Some(id)=>{store.read(&id)?;id},None=>{
  // Recover a note created before an interrupted binding-index update.
  let mut found=None;for candidate in store.all_ids()?{let note=store.read(&candidate)?;if note.meta.request.as_deref()==Some(id)&&note.meta.source=="selection-action"&&note.body==body{found=Some(candidate);break;}}
  let note_id=match found{Some(id)=>id,None=>store.create(&name,&body,"selection-action",&turn.model,&[],Some(id))?.id};c.execute("UPDATE bindings SET note_id=?2 WHERE request_id=?1",params![id,note_id]).map_err(|_|ClientError::storage())?;note_id
 }};
 let document=store.document(&note_id)?;
 if let Some(path)=path{let target=Path::new(&path);let parent=target.parent().ok_or_else(invalid)?;let handle=safe_directory(parent)?;let mut file=open_existing(&handle,target.file_name().ok_or_else(invalid)?)?;let mut content=Vec::new();Read::by_ref(&mut file).take(300000).read_to_end(&mut content).map_err(|_|export_error())?;if content!=document{return Err(export_error());}return Ok(ActionDocument{note_id,path});}
 let handle=safe_directory(Path::new(&directory))?;
 let stem=zwen_core::notes::sanitize_filename_export(&name);
 let stage=format!(".zwen-{}.tmp",zwen_core::notes::generate_id());
 let mut file=create_output(&handle,&stage).map_err(|_|export_error())?;
 let result=(||{
  file.write_all(&document).and_then(|_|file.sync_all()).map_err(|_|export_error())?;
  for suffix in 0..1000{
   let filename=format!("{stem}-{note_id}-{suffix}.md");
   match promote_output(&handle,&stage,&filename){
    Ok(())=>{},
    Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>{
     // A crash between export and binding update is recovered without making
     // another visible document; unrelated collisions are never overwritten.
     let same=open_existing(&handle,std::ffi::OsStr::new(&filename)).ok().and_then(|mut f|{let mut bytes=Vec::new();Read::by_ref(&mut f).take(300000).read_to_end(&mut bytes).ok()?;Some(bytes==document)}).unwrap_or(false);
     if !same{continue;}
    },
    Err(_)=>return Err(export_error())
   }
   handle.sync_all().map_err(|_|export_error())?;
   let path=Path::new(&directory).join(filename).to_string_lossy().into_owned();
   c.execute("UPDATE bindings SET path=?2 WHERE request_id=?1",params![id,path]).map_err(|_|ClientError::storage())?;
   return Ok(ActionDocument{note_id:note_id.clone(),path});
  }
  Err(export_error())
 })();
 unlink_output(&handle,&stage);result
}
fn export_error()->ClientError{error("action_export_failed","文档导出失败；回答和知识库笔记已保留，可单独重试保存")}
// Walk from / using directory descriptors: neither parent races nor symlink
// leaves can redirect the new file outside the explicitly chosen directory.
#[cfg(unix)]
fn safe_directory(path:&Path)->Result<fs::File,ClientError>{use std::os::fd::{AsRawFd,FromRawFd};use std::os::unix::ffi::OsStrExt;
 if !path.is_absolute(){return Err(invalid());}let mut current=fs::File::open("/").map_err(|_|export_error())?;
 for part in path.components(){match part{Component::RootDir=>continue,Component::Normal(name)=>{let name=std::ffi::CString::new(name.as_bytes()).map_err(|_|invalid())?;let fd=unsafe{libc::openat(current.as_raw_fd(),name.as_ptr(),libc::O_RDONLY|libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC)};
 let fd=if fd<0{let err=std::io::Error::last_os_error();if err.kind()!=std::io::ErrorKind::NotFound{return Err(export_error());}if unsafe{libc::mkdirat(current.as_raw_fd(),name.as_ptr(),0o700)}<0&&std::io::Error::last_os_error().kind()!=std::io::ErrorKind::AlreadyExists{return Err(export_error());}unsafe{libc::openat(current.as_raw_fd(),name.as_ptr(),libc::O_RDONLY|libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC)}}else{fd};if fd<0{return Err(export_error());}current=unsafe{fs::File::from_raw_fd(fd)};},_=>return Err(invalid())}}
 Ok(current)
}
#[cfg(unix)]fn create_output(dir:&fs::File,name:&str)->std::io::Result<fs::File>{use std::os::fd::{AsRawFd,FromRawFd};let name=std::ffi::CString::new(name)?;let fd=unsafe{libc::openat(dir.as_raw_fd(),name.as_ptr(),libc::O_WRONLY|libc::O_CREAT|libc::O_EXCL|libc::O_NOFOLLOW|libc::O_CLOEXEC,0o600)};if fd<0{Err(std::io::Error::last_os_error())}else{Ok(unsafe{fs::File::from_raw_fd(fd)})}}
#[cfg(unix)]fn promote_output(dir:&fs::File,stage:&str,name:&str)->std::io::Result<()>{use std::os::fd::AsRawFd;let stage=std::ffi::CString::new(stage)?;let name=std::ffi::CString::new(name)?;let result=unsafe{libc::linkat(dir.as_raw_fd(),stage.as_ptr(),dir.as_raw_fd(),name.as_ptr(),0)};if result<0{Err(std::io::Error::last_os_error())}else{Ok(())}}
#[cfg(unix)]fn unlink_output(dir:&fs::File,name:&str){use std::os::fd::AsRawFd;if let Ok(name)=std::ffi::CString::new(name){unsafe{libc::unlinkat(dir.as_raw_fd(),name.as_ptr(),0)};}}

#[cfg(unix)]fn open_existing(dir:&fs::File,name:&std::ffi::OsStr)->Result<fs::File,ClientError>{use std::os::fd::{AsRawFd,FromRawFd};use std::os::unix::ffi::OsStrExt;let name=std::ffi::CString::new(name.as_bytes()).map_err(|_|invalid())?;let fd=unsafe{libc::openat(dir.as_raw_fd(),name.as_ptr(),libc::O_RDONLY|libc::O_NOFOLLOW|libc::O_CLOEXEC|libc::O_NONBLOCK)};if fd<0{return Err(export_error());}let file=unsafe{fs::File::from_raw_fd(fd)};if !file.metadata().map_err(|_|export_error())?.is_file(){return Err(export_error());}Ok(file)}

#[cfg(test)]mod tests{
 use super::*;
 fn temp()->std::path::PathBuf{let dir=std::env::temp_dir().join(format!("zwen-actions-{}",zwen_core::notes::generate_id()));fs::create_dir_all(&dir).unwrap();dir}
 fn configured(dir:&Path,out:&Path){let c=defaults(out.to_string_lossy().into());save_config(dir,"/tmp/unused",0,c.actions,c.output_directory).unwrap();}
 fn request(dir:&Path,selection:&str,id:&str,state:TurnState)->String{let prompt=prepare(dir,"/tmp/unused",id,selection,"document").unwrap();let mut j=Journal::open(&dir.join("chat.sqlite3")).unwrap();let mut t=j.begin(id,&prompt,"isolated-test-model").unwrap();t.answer="完整回答".into();t.state=state;t.revision=1;j.checkpoint(&t).unwrap();prompt}
 #[test]fn stale_action_revision_cannot_bind_a_different_prompt(){let dir=temp();let out=dir.join("out");configured(&dir,&out);assert!(prepare_checked(&dir,"/tmp/unused","revision-test","text","document",Some(0)).is_err());let c=connection(&dir).unwrap();let count:i64=c.query_row("SELECT count(*) FROM bindings",[],|r|r.get(0)).unwrap();assert_eq!(count,0);assert!(prepare_checked(&dir,"/tmp/unused","revision-test","text","document",Some(1)).is_ok());fs::remove_dir_all(dir).unwrap();}
 #[test]fn config_bounds_conflicts_and_corruption(){let dir=temp();let c=defaults("/tmp/out".into());assert!(save_config(&dir,"/tmp/out",1,c.actions.clone(),c.output_directory.clone()).is_err());save_config(&dir,"/tmp/out",0,c.actions.clone(),c.output_directory.clone()).unwrap();assert!(save_config(&dir,"/tmp/out",0,c.actions,"/tmp/out".into()).is_err());connection(&dir).unwrap().execute("UPDATE config SET json='broken'",[]).unwrap();assert!(read_config(&connection(&dir).unwrap(),"/tmp/out").is_err());fs::remove_dir_all(dir).unwrap();}
 #[test]fn incomplete_or_unbound_requests_cannot_export(){let dir=temp();let out=dir.join("out");configured(&dir,&out);request(&dir,"选区","stopped",TurnState::Stopped);assert!(save_document(&dir,"stopped","选区","document","整理成文档",out.to_str().unwrap()).is_err());assert!(save_document(&dir,"other","选区","document","整理成文档",out.to_str().unwrap()).is_err());assert!(NoteStore::open(&dir).unwrap().all_ids().unwrap().is_empty());fs::remove_dir_all(dir).unwrap();}
 #[test]fn retry_exports_same_note_and_preserves_collision(){let dir=temp();let out=dir.join("out");configured(&dir,&out);request(&dir,"选区","complete",TurnState::Complete);fs::write(&out,"blocked").unwrap();assert!(save_document(&dir,"complete","选区","document","整理成文档",out.to_str().unwrap()).is_err());let store=NoteStore::open(&dir).unwrap();let ids=store.all_ids().unwrap();assert_eq!(ids.len(),1);fs::remove_file(&out).unwrap();fs::create_dir(&out).unwrap();let occupied=out.join(format!("整理成文档-{}-0.md",ids[0]));fs::write(&occupied,"keep").unwrap();let a=save_document(&dir,"complete","选区","document","整理成文档",out.to_str().unwrap()).unwrap();connection(&dir).unwrap().execute("UPDATE bindings SET path=NULL WHERE request_id='complete'",[]).unwrap();let b=save_document(&dir,"complete","选区","document","整理成文档",out.to_str().unwrap()).unwrap();assert_eq!(a.note_id,b.note_id);assert_eq!(a.path,b.path);assert_eq!(fs::read_to_string(occupied).unwrap(),"keep");assert_eq!(store.all_ids().unwrap().len(),1);fs::remove_dir_all(dir).unwrap();}
 #[cfg(unix)]#[test]fn symlink_output_directory_is_rejected(){use std::os::unix::fs::symlink;let dir=temp();let out=dir.join("out");let target=dir.join("target");fs::create_dir(&target).unwrap();symlink(&target,&out).unwrap();configured(&dir,&out);request(&dir,"选区","symlink",TurnState::Complete);assert!(save_document(&dir,"symlink","选区","document","整理成文档",out.to_str().unwrap()).is_err());assert_eq!(fs::read_dir(target).unwrap().count(),0);fs::remove_dir_all(dir).unwrap();}
 #[test]fn prompt_binding_rejects_mismatched_metadata_and_edited_journal(){let dir=temp();let out=dir.join("out");configured(&dir,&out);request(&dir,"选区","complete",TurnState::Complete);assert!(save_document(&dir,"complete","different","document","整理成文档",out.to_str().unwrap()).is_err());let c=rusqlite::Connection::open(dir.join("chat.sqlite3")).unwrap();c.execute("UPDATE turns SET prompt='forged'",[]).unwrap();assert!(save_document(&dir,"complete","选区","document","整理成文档",out.to_str().unwrap()).is_err());fs::remove_dir_all(dir).unwrap();}
}
