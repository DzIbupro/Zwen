//! Knowledge library commands over the core Markdown store, plus topic queries
//! that join the organize index with the notes index.
use crate::{ClientError,SharedStorage};
use serde::Serialize;
use std::fs;
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
pub async fn notes_update(storage:tauri::State<'_,SharedStorage>,id:String,title:String,body:String)->Result<NoteMeta,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<NoteMeta,ClientError>{Ok(store(&storage)?.update(&id,&title,&body)?)}).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn notes_rebuild(storage:tauri::State<'_,SharedStorage>)->Result<usize,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||->Result<usize,ClientError>{Ok(store(&storage)?.rebuild_index()?)}).await.map_err(|_|ClientError::storage())?
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
 tauri::async_runtime::spawn_blocking(move||->Result<Vec<TopicAssignments>,ClientError>{Ok(organize_store(&storage)?.list_topics()?)}).await.map_err(|_|ClientError::storage())?
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
