//! Knowledge library commands over the core Markdown store, plus topic queries
//! that join the organize index with the notes index.
use crate::{ClientError,SharedStorage};
use serde::Serialize;
use std::{fs,path::{Path,PathBuf}};
use zwen_core::notes::{Note,NoteMeta,NoteStore};
use zwen_core::organize::{OrganizeStore,TopicAssignments};

fn dir_for(storage:&SharedStorage)->Result<std::path::PathBuf,ClientError>{storage.directory.as_ref().map(Clone::clone).map_err(Clone::clone)}
fn store(storage:&SharedStorage)->Result<NoteStore,ClientError>{
 let dir=storage.directory.as_ref().map_err(Clone::clone)?;
 let mut builder=fs::DirBuilder::new();builder.recursive(true);
 #[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
 builder.create(dir).map_err(|_|ClientError::storage())?;
 NoteStore::open(dir).map_err(Into::into)
}

#[derive(serde::Deserialize,Serialize)]
#[serde(rename_all="camelCase")]
pub struct SaveInput{title:String,body:String,source:String,model:String,attachment_ids:Vec<String>,request_id:Option<String>}

#[tauri::command]
pub async fn notes_save(storage:tauri::State<'_,SharedStorage>,input:SaveInput)->Result<NoteMeta,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<NoteMeta,ClientError>{
  let dir=storage.directory.as_ref().map_err(Clone::clone)?;
  let mut store=store(&storage)?;
  let mut files=Vec::new();
  for id in &input.attachment_ids{
   if !zwen_core::attachment::valid_id(id){return Err(zwen_core::Error::InvalidNote.into());}
   let bytes=fs::read(dir.join("attachments").join(format!("{id}.png"))).map_err(|_|zwen_core::Error::AttachmentMissing)?;
   if bytes.len() as u64>zwen_core::attachment::MAX_IMAGE_BYTES{return Err(zwen_core::Error::InvalidNote.into());}
   files.push((format!("{id}.png"),bytes));
  }
  store.create(&input.title,&input.body,&input.source,&input.model,&files,input.request_id.as_deref()).map_err(Into::into)
 }).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn notes_list(storage:tauri::State<'_,SharedStorage>,query:String)->Result<Vec<NoteMeta>,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<Vec<NoteMeta>,ClientError>{Ok(store(&storage)?.list(&query)?)}).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn notes_read(storage:tauri::State<'_,SharedStorage>,id:String)->Result<Note,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<Note,ClientError>{Ok(store(&storage)?.read(&id)?)}).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn notes_delete(storage:tauri::State<'_,SharedStorage>,id:String)->Result<(),ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<(),ClientError>{Ok(store(&storage)?.delete(&id)?)}).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn notes_update(storage:tauri::State<'_,SharedStorage>,id:String,title:String,body:String,source_title:Option<String>,source_url:Option<String>,source_loc:Option<String>)->Result<NoteMeta,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<NoteMeta,ClientError>{
  let mut store=store(&storage)?;
  let source=zwen_core::notes::NoteMeta{id:id.clone(),title:title.clone(),created_at:0,source:String::new(),model:String::new(),attachments:Vec::new(),request:None,source_title,source_url,source_loc};
  let meta=store.update_with_source(&id,&title,&body,Some(&source))?;
  Ok(meta)
 }).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn notes_update_source(storage:tauri::State<'_,SharedStorage>,id:String,source_title:Option<String>,source_url:Option<String>,source_loc:Option<String>)->Result<(),ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<(),ClientError>{
  store(&storage)?.update_source(&id,&zwen_core::notes::NoteMeta{id:id.clone(),title:String::new(),created_at:0,source:String::new(),model:String::new(),attachments:Vec::new(),request:None,source_title,source_url,source_loc})?;
  Ok(())
 }).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn notes_rebuild(storage:tauri::State<'_,SharedStorage>)->Result<usize,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<usize,ClientError>{Ok(NoteStore::rebuild_from_files(&dir_for(&storage)?)?)}).await.map_err(|_|ClientError::storage())?
}

pub fn organize_store(storage:&SharedStorage)->Result<OrganizeStore,ClientError>{
 let dir=storage.directory.as_ref().map_err(Clone::clone)?;
 let mut builder=fs::DirBuilder::new();builder.recursive(true);
 #[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
 builder.create(dir).map_err(|_|ClientError::storage())?;
 OrganizeStore::open(dir).map_err(Into::into)
}

#[tauri::command]
pub async fn organize_topics(storage:tauri::State<'_,SharedStorage>)->Result<Vec<TopicAssignments>,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<Vec<TopicAssignments>,ClientError>{let mut organize=organize_store(&storage)?;organize.prune_empty_topics()?;Ok(organize.list_topics()?)}).await.map_err(|_|ClientError::storage())?
}

/// Joins one topic's memberships with note metadata (cap 50, newest first).
#[tauri::command]
pub async fn organize_topic_notes(storage:tauri::State<'_,SharedStorage>,topic_id:String)->Result<Vec<Note>,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<Vec<Note>,ClientError>{
  let organize=organize_store(&storage)?;
  let ids=organize.note_ids(&topic_id)?;
  let notes_store=NoteStore::open(dir_for(&storage)?.as_path())?;
  let mut out=Vec::new();
  for id in ids.iter().rev().take(50){
   if let Ok(note)=notes_store.read(id){out.push(note);}
  }
  Ok(out)
 }).await.map_err(|_|ClientError::storage())?
}

#[tauri::command]
pub async fn organize_undo(storage:tauri::State<'_,SharedStorage>,batch_id:String)->Result<(),ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<(),ClientError>{Ok(organize_store(&storage)?.undo_batch(&batch_id)?)}).await.map_err(|_|ClientError::storage())?
}

/// Exports go to the user's Downloads folder; the .md is copied verbatim (the
/// file is the authority). Returns the written path for the UI to display.
pub fn export_target(stem:&str,id8:&str,home:&Path)->Result<PathBuf,ClientError>{
 let dir=home.join("Downloads").join("Zwen导出");
 let mut builder=fs::DirBuilder::new();builder.recursive(true);
 #[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
 builder.create(&dir).map_err(|_|ClientError::storage())?;
 let name=zwen_core::notes::sanitize_filename_export(stem);
 Ok(dir.join(format!("{name}-{id8}.md")))
}

#[tauri::command]
pub async fn note_export(storage:tauri::State<'_,SharedStorage>,id:String)->Result<String,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<String,ClientError>{
  let note_store=NoteStore::open(dir_for(&storage)?.as_path())?;
  let note=note_store.read(&id)?;
  let home=std::env::var("HOME").map_err(|_|ClientError{code:"export_failed",message:"未找到用户主目录；无法导出".into()})?;
  let target=export_target(&note.meta.title,&id,Path::new(&home))?;
  let document=note_store.document(&note.meta.id)?;
  fs::write(&target,document).map_err(|_|ClientError{code:"export_failed",message:"导出写入失败".into()})?;
  copy_note_attachments(&dir_for(&storage)?,target.parent().ok_or_else(ClientError::storage)?,&note)?;
  Ok(target.to_string_lossy().into_owned())
 }).await.map_err(|_|ClientError::storage())?
}

#[tauri::command]
pub async fn notes_export_all(storage:tauri::State<'_,SharedStorage>)->Result<String,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<String,ClientError>{
  let data_dir=dir_for(&storage)?;
  let note_store=NoteStore::open(data_dir.as_path())?;
  let home=std::env::var("HOME").map_err(|_|ClientError{code:"export_failed",message:"未找到用户主目录；无法导出".into()})?;
  let stamp=zwen_core::notes::generate_id();
  let target=Path::new(&home).join("Downloads").join(format!("Zwen导出-{stamp}"));
  let mut builder=fs::DirBuilder::new();builder.recursive(true);
  #[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
  builder.create(&target).map_err(|_|ClientError::storage())?;
  for id in note_store.all_ids()?{
   let note=note_store.read(&id)?;
   let name=format!("{}-{}",zwen_core::notes::sanitize_filename_export(&note.meta.title),id);
   let document=note_store.document(&note.meta.id)?;
   fs::write(target.join(format!("{name}.md")),document).map_err(|_|ClientError{code:"export_failed",message:"导出写入失败".into()})?;
   copy_note_attachments(&data_dir,&target,&note)?;
  }
  Ok(target.to_string_lossy().into_owned())
 }).await.map_err(|_|ClientError::storage())?
}

fn copy_note_attachments(data_dir:&Path,target:&Path,note:&Note)->Result<(),ClientError>{
 let source=data_dir.join("notes/attachments");
 if fs::symlink_metadata(&source).map_err(|_|ClientError::storage())?.file_type().is_symlink(){return Err(ClientError::storage());}
 let dest=target.join("attachments");
 if !note.meta.attachments.is_empty(){fs::create_dir_all(&dest).map_err(|_|ClientError::storage())?;}
 for name in &note.meta.attachments{
  let path=source.join(name);
  let meta=fs::symlink_metadata(&path).map_err(|_|ClientError::storage())?;
  if !meta.is_file()||meta.file_type().is_symlink(){return Err(ClientError::storage());}
  fs::copy(path,dest.join(name)).map_err(|_|ClientError::storage())?;
 }
 Ok(())
}

/// External links open in the system browser. Only http(s) is allowed and the
/// spawn is detached; the webview never navigates to the URL itself.
#[tauri::command]
pub fn open_external(url:String)->Result<(),ClientError>{
 let ok=url.len()<=2048&&(url.starts_with("https://")||url.starts_with("http://"))&&url.bytes().all(|b|b.is_ascii_graphic());
 if !ok{return Err(ClientError{code:"invalid_url",message:"仅支持 http(s) 链接".into()});}
 std::process::Command::new("xdg-open").arg(url).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().map_err(|_|ClientError{code:"open_failed",message:"系统浏览器打开失败".into()})?;
 Ok(())
}

#[tauri::command]
pub async fn organize_rename(storage:tauri::State<'_,SharedStorage>,topic_id:String,name:String)->Result<(),ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<(),ClientError>{Ok(organize_store(&storage)?.rename_topic(&topic_id,&name)?)}).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn organize_set_column(storage:tauri::State<'_,SharedStorage>,topic_id:String,column:String)->Result<(),ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<(),ClientError>{Ok(organize_store(&storage)?.set_topic_column(&topic_id,&column)?)}).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn organize_merge(storage:tauri::State<'_,SharedStorage>,source:String,target:String)->Result<(),ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<(),ClientError>{Ok(organize_store(&storage)?.merge_topics(&source,&target)?)}).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn organize_split(storage:tauri::State<'_,SharedStorage>,note_id:String,from:String,to_name:String,column:String)->Result<String,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<String,ClientError>{Ok(organize_store(&storage)?.split_membership(&note_id,&from,&to_name,&column)?)}).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn organize_set_locked(storage:tauri::State<'_,SharedStorage>,topic_id:String,locked:bool)->Result<(),ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<(),ClientError>{Ok(organize_store(&storage)?.set_topic_locked(&topic_id,locked)?)}).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn organize_delete_topic(storage:tauri::State<'_,SharedStorage>,topic_id:String)->Result<(),ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<(),ClientError>{Ok(organize_store(&storage)?.delete_topic(&topic_id)?)}).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn organize_add_relation(storage:tauri::State<'_,SharedStorage>,from_topic:String,to_topic:String,reason:String,evidence:String)->Result<String,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<String,ClientError>{Ok(organize_store(&storage)?.add_relation(&from_topic,&to_topic,&reason,&evidence)?)}).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn organize_remove_relation(storage:tauri::State<'_,SharedStorage>,relation_id:String)->Result<(),ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<(),ClientError>{Ok(organize_store(&storage)?.remove_relation(&relation_id)?)}).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn organize_list_relations(storage:tauri::State<'_,SharedStorage>)->Result<Vec<zwen_core::organize::Relation>,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<Vec<zwen_core::organize::Relation>,ClientError>{Ok(organize_store(&storage)?.list_relations()?)}).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn organize_set_auto(storage:tauri::State<'_,SharedStorage>,on:bool)->Result<(),ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<(),ClientError>{
  let mut store=organize_store(&storage)?;
  if !on{return Ok(store.set_auto_organize(false)?);}
  let profile=organize_profile(&storage)?;
  Ok(store.set_auto_organize_for(true,&profile)?)
 }).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn organize_get_auto(storage:tauri::State<'_,SharedStorage>)->Result<bool,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<bool,ClientError>{
  let store=organize_store(&storage)?;
  let Ok(profile)=organize_profile(&storage)else{return Ok(false)};
  Ok(store.auto_organize_for(&profile)?)
 }).await.map_err(|_|ClientError::storage())?
}

pub fn organize_profile(storage:&SharedStorage)->Result<String,ClientError>{
 serde_json::to_string(&crate::provider::configured(storage)?).map_err(|_|ClientError::storage())
}
