//! Native chat commands. Screenshots route to the configured vision model.
use crate::{SharedStorage,ClientError,provider};
use base64::{Engine as _,engine::general_purpose::STANDARD as BASE64_STANDARD};
use serde::Serialize;
use std::{collections::VecDeque,sync::{Arc,Mutex},time::{Instant,Duration}};
use tauri::Emitter;
use tokio::sync::watch;
use zwen_core::{attachment::AttachmentStore,journal::{Journal,Turn,TurnState},completion::FinishReason};
#[derive(Clone,Serialize)]
pub struct Reply {#[serde(flatten)] turn:Turn,saved:bool}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Overview {reply:Option<Reply>,active_request_id:Option<String>}
struct Active {id:String,cancel:watch::Sender<bool>,handle:Option<tauri::async_runtime::JoinHandle<Result<Reply,ClientError>>>}
#[derive(Default)] pub struct State {active:Option<Active>,last:Option<Reply>,revoked:VecDeque<String>}
pub type Shared=Arc<Mutex<State>>;
fn error(code:&'static str,message:&str)->ClientError{ClientError{code,message:message.into()}}
fn directory(storage:&SharedStorage)->Result<std::path::PathBuf,ClientError>{
 let dir=storage.directory.as_ref().map_err(Clone::clone)?.clone();let mut b=std::fs::DirBuilder::new();b.recursive(true);
 #[cfg(unix)] {use std::os::unix::fs::DirBuilderExt;b.mode(0o700);}
 b.create(&dir).map_err(|_|ClientError::storage())?;Ok(dir)
}
fn publish(app:&tauri::AppHandle,state:&Shared,reply:Reply)->Result<(),()>{
 let mut s=state.lock().map_err(|_|())?;
 if s.active.as_ref().is_none_or(|a|a.id!=reply.turn.request_id){return Err(());}
 s.last=Some(reply.clone());app.emit("chat-update",reply).map_err(|_|())
}
fn run(app:tauri::AppHandle,state:Shared,mut journal:Journal,mut turn:Turn,config:zwen_core::provider::ProviderConfig,key:zwen_core::provider::ApiKey,messages:Vec<zwen_core::completion::Message>,cancel:watch::Receiver<bool>)->Result<Reply,ClientError>{
 let mut emit_at=Instant::now();let mut save_at=Instant::now();let mut first=true;
 let result=(||{
  let runtime=tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|_|zwen_transport::Error::Network)?;
  runtime.block_on(async{
   let client=zwen_transport::client()?;
   zwen_transport::stream_messages(&client,&config,&key,&messages,cancel,|part|{
    turn.answer.push_str(part);
    let emit=first||emit_at.elapsed()>=Duration::from_millis(80);let save=first||save_at.elapsed()>=Duration::from_millis(500);
    if emit||save{turn.revision+=1;}
    if save{journal.checkpoint(&turn).map_err(|_|())?;save_at=Instant::now();}
    if emit{publish(&app,&state,Reply{turn:turn.clone(),saved:save})?;emit_at=Instant::now();first=false;}
    Ok(())
   }).await
  })
 })();
 turn.revision+=1;
 turn.state=match result{Ok(FinishReason::Stop)=>TurnState::Complete,Ok(FinishReason::Length)=>TurnState::Limited,Ok(FinishReason::Filtered)=>TurnState::Filtered,Err(zwen_transport::Error::Cancelled)=>TurnState::Stopped,Err(_)=>TurnState::Failed};
 turn.error=match &result{Err(zwen_transport::Error::Cancelled)=>None,Err(e)=>Some(e.to_string()),Ok(FinishReason::Length)=>Some("达到服务输出限制，回答可能不完整".into()),Ok(FinishReason::Filtered)=>Some("服务限制了本轮输出".into()),_=>None};
 let saved=journal.checkpoint(&turn).is_ok();let reply=Reply{turn,saved};
 {
  let mut s=state.lock().map_err(|_|ClientError::storage())?;
  if s.active.as_ref().is_some_and(|a|a.id==reply.turn.request_id){s.last=Some(reply.clone());s.active=None;let _=app.emit("chat-update",&reply);}
 }
 if saved{Ok(reply)}else{Err(error("chat_unsaved","回答尚未成功保存，请保留窗口并重试保存"))}
}
#[tauri::command]
pub async fn start_chat(app:tauri::AppHandle,storage:tauri::State<'_,SharedStorage>,chat:tauri::State<'_,Shared>,request_id:String,prompt:String,conversation_id:Option<String>)->Result<Reply,ClientError>{
 let storage=storage.inner().clone();let state=chat.inner().clone();
 tauri::async_runtime::spawn_blocking(move||{
  let mut s=state.lock().map_err(|_|ClientError::storage())?;
  if s.active.is_some(){return Err(error("chat_busy","当前请求尚未结束，请先停止"));}
  if s.revoked.contains(&request_id){return Err(error("chat_cancelled","此请求已取消"));}
  if s.last.as_ref().is_some_and(|r|!r.saved){return Err(error("chat_unsaved","上一条回答尚未保存，请先重试保存"));}
  let config=provider::configured(&storage)?;let key=config.read_key()?;
  let dir=directory(&storage)?;
  let mut journal=Journal::open(&dir.join("chat.sqlite3"))?;
  let profile=serde_json::to_string(&config).map_err(|_|ClientError::storage())?;
  // Screenshot questions carry bounded base64 payloads to the configured vision
  // model. Preparation happens here, off the UI thread; a failed payload aborts
  // before the question is persisted so no half-attached turn is stored.
  let store=AttachmentStore::open(&dir)?;
  let attached=store.list()?;
  let vision=if attached.is_empty(){None}else{
   let image_model=config.image_model().ok_or_else(||error("image_model_missing","截图尚未接入模型；请先在模型设置中配置图片模型"))?.to_owned();
   let mut images=Vec::new();
   for attachment in &attached{images.push(BASE64_STANDARD.encode(store.model_payload(&attachment.id)?));}
   Some((image_model,images))
  };
  let request_config=match &vision{Some((model,_))=>config.with_model(model)?,None=>config.clone()};
  let mut messages=journal.context(conversation_id.as_deref(),&profile,prompt.len())?;let count=messages.len()/2;
  let request_model=request_config.model().to_owned();
  let turn=journal.begin_in(&request_id,&prompt,&request_model,conversation_id.as_deref(),&profile,count)?;
  let images=vision.map(|(_,images)|images).unwrap_or_default();
  messages.push(zwen_core::completion::Message{role:zwen_core::completion::Role::User,content:prompt,images});let reply=Reply{turn:turn.clone(),saved:true};s.last=Some(reply.clone());
  let (cancel,rx)=watch::channel(false);let id=request_id;let worker=state.clone();
  let handle=tauri::async_runtime::spawn_blocking(move||run(app,worker,journal,turn,request_config,key,messages,rx));
  s.active=Some(Active{id,cancel,handle:Some(handle)});Ok(reply)
 }).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn get_chat(storage:tauri::State<'_,SharedStorage>,chat:tauri::State<'_,Shared>)->Result<Overview,ClientError>{
 let storage=storage.inner().clone();let state=chat.inner().clone();
 tauri::async_runtime::spawn_blocking(move||{
  let mut s=state.lock().map_err(|_|ClientError::storage())?;
  // A storage outage must not hide the only remaining in-process answer.
  if s.active.is_none()&&s.last.as_ref().is_some_and(|r|!r.saved){return Ok(Overview{reply:s.last.clone(),active_request_id:None});}
  let mut journal=Journal::open(&directory(&storage)?.join("chat.sqlite3"))?;
  if s.active.is_none()&&!s.last.as_ref().is_some_and(|r|!r.saved){journal.recover()?;}
  let persisted=journal.latest()?.map(|turn|Reply{turn,saved:true});
  let reply=match (&s.last,persisted){(Some(live),Some(db))if live.turn.request_id==db.turn.request_id=>Some(if live.turn.revision>=db.turn.revision{live.clone()}else{db}),(Some(live),_)if !live.saved||s.active.is_some()=>Some(live.clone()),(_,db)=>db};
  s.last=reply.clone();Ok(Overview{reply,active_request_id:s.active.as_ref().map(|a|a.id.clone())})
 }).await.map_err(|_|ClientError::storage())?
}
async fn stop(app:tauri::AppHandle,storage:SharedStorage,state:Shared,id:Option<String>)->Result<Option<Reply>,ClientError>{
 let control=state.clone();let requested=id.clone();
 let handle=tauri::async_runtime::spawn_blocking(move||{
  let mut s=control.lock().map_err(|_|ClientError::storage())?;
  if let Some(id)=requested.as_ref(){if id.is_empty()||id.len()>100||!id.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_'){return Err(zwen_core::Error::InvalidChat.into());}s.revoked.push_back(id.clone());if s.revoked.len()>64{s.revoked.pop_front();}}
  if let Some(active)=s.active.as_mut(){if requested.as_ref().is_none_or(|id|id==&active.id){let _=active.cancel.send(true);return active.handle.take().map(Some).ok_or_else(||error("chat_stopping","请求正在停止，请稍后重试"));}}
  Ok(None)
 }).await.map_err(|_|ClientError::storage())??;
 if let Some(handle)=handle{
  if handle.await.is_err(){let mut s=state.lock().map_err(|_|ClientError::storage())?;s.active=None;if let Some(last)=s.last.as_mut(){last.turn.revision+=1;last.turn.state=TurnState::Failed;last.turn.error=Some("请求意外中断，已接收内容保留".into());last.saved=false;}}
 }
 tauri::async_runtime::spawn_blocking(move||{
  let mut s=state.lock().map_err(|_|ClientError::storage())?;
  if s.active.is_some(){return Ok(s.last.as_ref().filter(|r|id.as_ref().is_none_or(|id|id==&r.turn.request_id)).cloned());}
  if let Some(last)=s.last.as_mut(){if id.as_ref().is_none_or(|id|id==&last.turn.request_id)&&!last.saved{let mut journal=Journal::open(&directory(&storage)?.join("chat.sqlite3"))?;journal.checkpoint(&last.turn)?;last.saved=true;let _=app.emit("chat-update",&*last);}}
  Ok(s.last.as_ref().filter(|r|id.as_ref().is_none_or(|id|id==&r.turn.request_id)).cloned())
 }).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn stop_chat(app:tauri::AppHandle,storage:tauri::State<'_,SharedStorage>,chat:tauri::State<'_,Shared>,request_id:String)->Result<Option<Reply>,ClientError>{stop(app,storage.inner().clone(),chat.inner().clone(),Some(request_id)).await}
#[tauri::command]
pub async fn settle_chat(app:tauri::AppHandle,storage:tauri::State<'_,SharedStorage>,chat:tauri::State<'_,Shared>)->Result<Option<Reply>,ClientError>{stop(app,storage.inner().clone(),chat.inner().clone(),None).await}

#[tauri::command]
pub async fn get_chat_history(storage:tauri::State<'_,SharedStorage>,page:usize)->Result<Vec<zwen_core::journal::Summary>,ClientError>{
 let storage=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||{let offset=page.checked_mul(20).ok_or(zwen_core::Error::InvalidChat)?;let journal=Journal::open(&directory(&storage)?.join("chat.sqlite3"))?;journal.history(offset,21).map_err(Into::into)}).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn get_chat_turn(storage:tauri::State<'_,SharedStorage>,chat:tauri::State<'_,Shared>,request_id:String)->Result<Option<Reply>,ClientError>{
 let storage=storage.inner().clone();let state=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||{
  if request_id.len()>100{return Err(zwen_core::Error::InvalidChat.into());}
  let s=state.lock().map_err(|_|ClientError::storage())?;let journal=Journal::open(&directory(&storage)?.join("chat.sqlite3"))?;let saved=journal.find(&request_id)?.map(|turn|Reply{turn,saved:true});
  Ok(match (&s.last,saved){(Some(live),Some(db))if live.turn.request_id==request_id&&live.turn.revision>=db.turn.revision=>Some(live.clone()),(_,db)=>db})
 }).await.map_err(|_|ClientError::storage())?
}
