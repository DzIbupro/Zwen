//! One native-owned serial round; renderer polling never owns execution.
use crate::{ClientError,SharedStorage,provider,organize_job::{self,JobState}};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle,Manager};
use tokio::sync::watch;
use zwen_core::{notes::NoteStore,organize::{OrganizeStore,OrganizeRound},provider::ProviderConfig};

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct View{pub round:OrganizeRound,pub status:String,pub model:String,pub base_url:String,pub error:String}
struct Session{view:View,config:ProviderConfig,cancel:Option<watch::Sender<bool>>}
#[derive(Default)]
pub struct State(Mutex<Option<Session>>);
fn busy()->ClientError{ClientError{code:"organize_busy",message:"已有整理任务在进行；请先停止或等待完成".into()}}
fn stale()->ClientError{ClientError{code:"organize_stale",message:"整理确认已失效，请重新检查本轮范围与模型".into()}}
fn dir(storage:&SharedStorage)->Result<std::path::PathBuf,ClientError>{storage.directory.as_ref().cloned().map_err(Clone::clone)}
fn running(status:&str)->bool{matches!(status,"running"|"stopping")}
fn stopped(round:OrganizeRound,config:&ProviderConfig)->View{
 let status=if round.undone{"undone"}else if round.completed==round.note_ids.len(){"complete"}else{"interrupted"};
 View{round,status:status.into(),model:config.model().into(),base_url:config.base_url().into(),error:String::new()}
}

#[tauri::command]
pub async fn organize_prepare(app:AppHandle,storage:tauri::State<'_,SharedStorage>,note_ids:Option<Vec<String>>)->Result<View,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||{
  let state=app.state::<State>();let mut session=state.0.lock().map_err(|_|ClientError::storage())?;
  if session.as_ref().is_some_and(|s|running(&s.view.status)){return Err(busy());}
  let config=provider::configured(&storage)?;
  let path=dir(&storage)?;let notes=NoteStore::open(&path)?;
  let ids=match note_ids{Some(ids)=>ids,None=>notes.all_ids()?};
  // Only IDs are collected here. No body/key/model call before confirmation.
  let profile=serde_json::to_string(&config).map_err(|_|ClientError::storage())?;
  let round=OrganizeStore::open(&path)?.create_round(&ids,&profile)?;
  let mut view=stopped(round,&config);view.status="prepared".into();
  *session=Some(Session{view:view.clone(),config,cancel:None});Ok(view)
 }).await.map_err(|_|ClientError::storage())?
}

#[tauri::command]
pub async fn organize_round_status(app:AppHandle,storage:tauri::State<'_,SharedStorage>)->Result<Option<View>,ClientError>{
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||{
  let state=app.state::<State>();let mut session=state.0.lock().map_err(|_|ClientError::storage())?;
  if let Some(s)=session.as_ref(){return Ok(Some(s.view.clone()));}
  if let Some(round)=OrganizeStore::open(&dir(&storage)?)?.latest_round()?{
   let config:ProviderConfig=serde_json::from_str(&round.profile).map_err(|_|ClientError::storage())?;
   let view=stopped(round,&config);
   *session=Some(Session{view:view.clone(),config,cancel:None});return Ok(Some(view));
  }Ok(None)
 }).await.map_err(|_|ClientError::storage())?
}

#[tauri::command]
pub async fn organize_round_start(app:AppHandle,storage:tauri::State<'_,SharedStorage>,round_id:String)->Result<View,ClientError>{
 let worker_storage=storage.inner().clone();let handle=app.clone();
 let (view,config,rx)=tauri::async_runtime::spawn_blocking(move||{
  let job=handle.state::<JobState>();let mut job=job.0.lock().map_err(|_|ClientError::storage())?;
  if *job{return Err(busy());}
  let state=handle.state::<State>();let mut guard=state.0.lock().map_err(|_|ClientError::storage())?;
  let s=guard.as_mut().ok_or_else(stale)?;
  if s.view.round.id!=round_id||running(&s.view.status)||s.view.round.undone||s.view.round.completed==s.view.round.note_ids.len(){return Err(stale());}
  // Explicit start confirms the displayed saved model, including after restart.
  // Current settings never silently replace this round's provider.
  let (tx,rx)=watch::channel(false);s.cancel=Some(tx);s.view.status="running".into();s.view.error.clear();*job=true;
  Ok((s.view.clone(),s.config.clone(),rx))
 }).await.map_err(|_|ClientError::storage())??;
 let worker_app=app.clone();
 let id=view.round.id.clone();
 tauri::async_runtime::spawn_blocking(move||run(worker_app,worker_storage,id,config,rx));
 Ok(view)
}

fn run(app:AppHandle,storage:SharedStorage,id:String,config:ProviderConfig,rx:watch::Receiver<bool>){
 let result=(||->Result<(),ClientError>{
  let path=dir(&storage)?;let key=organize_job::model_key(&config)?;
  loop{
   if *rx.borrow(){return Ok(());}
   let round=OrganizeStore::open(&path)?.round(&id)?;
   if round.completed==round.note_ids.len(){return Ok(());}
   let end=(round.completed+8).min(round.note_ids.len());let ids=&round.note_ids[round.completed..end];
   let notes=organize_job::collect_notes(&storage,ids)?;
   let answer=organize_job::run_prompt_text(&organize_job::build_prompt(&notes)?,&config,&key,rx.clone())?;
   let state=app.state::<State>();let mut guard=state.0.lock().map_err(|_|ClientError::storage())?;
   let s=guard.as_mut().ok_or_else(stale)?;
   // Stop and commit share this mutex; after stop returns no late result commits.
   if s.view.round.id!=id||*rx.borrow(){return Ok(());}
   organize_job::apply_answer(&storage,&config,&notes,&answer,Some(&id))?;
   s.view.round=OrganizeStore::open(&path)?.round(&id)?;
  }
 })();
 if let Ok(mut guard)=app.state::<State>().0.lock(){if let Some(s)=guard.as_mut(){if s.view.round.id==id{
  if let Ok(path)=dir(&storage){if let Ok(store)=OrganizeStore::open(&path){if let Ok(round)=store.round(&id){s.view.round=round;}}}
  s.view.status=if s.view.round.completed==s.view.round.note_ids.len(){"complete"}else if *rx.borrow(){"stopped"}else{"failed"}.into();
  s.view.error=result.err().map(|e|e.message).unwrap_or_default();s.cancel=None;
 }}}
 if let Ok(mut guard)=app.state::<JobState>().0.lock(){*guard=false;}
}

#[tauri::command]
pub fn organize_round_stop(app:AppHandle,round_id:String)->Result<(),ClientError>{
 let state=app.state::<State>();let mut guard=state.0.lock().map_err(|_|ClientError::storage())?;
 let s=guard.as_mut().ok_or_else(stale)?;if s.view.round.id!=round_id{return Err(stale());}
 if let Some(tx)=&s.cancel{let _=tx.send(true);s.view.status="stopping".into();}Ok(())
}
#[tauri::command]
pub async fn organize_round_undo(app:AppHandle,storage:tauri::State<'_,SharedStorage>,round_id:String)->Result<View,ClientError>{
 let storage=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||{
  let state=app.state::<State>();let mut guard=state.0.lock().map_err(|_|ClientError::storage())?;
  let s=guard.as_mut().ok_or_else(stale)?;if s.view.round.id!=round_id||running(&s.view.status){return Err(busy());}
  let mut store=OrganizeStore::open(&dir(&storage)?)?;store.undo_round(&round_id)?;s.view.round=store.round(&round_id)?;s.view.status="undone".into();Ok(s.view.clone())
 }).await.map_err(|_|ClientError::storage())?
}

/// Called on explicit quit and platform exit requests. Shares the commit lock.
pub fn cancel_all(app:&AppHandle){
 if let Some(state)=app.try_state::<State>(){if let Ok(mut guard)=state.0.lock(){if let Some(s)=guard.as_mut(){if let Some(tx)=&s.cancel{let _=tx.send(true);s.view.status="stopping".into();}}}}
}
