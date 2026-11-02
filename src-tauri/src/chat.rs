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
fn run(app:tauri::AppHandle,state:Shared,mut journal:Journal,mut turn:Turn,config:zwen_core::provider::ProviderConfig,key:zwen_core::provider::ApiKey,messages:Vec<zwen_core::completion::Message>,mut consume_ids:Vec<String>,attach_dir:std::path::PathBuf,cancel:watch::Receiver<bool>)->Result<Reply,ClientError>{
 let mut emit_at=Instant::now();let mut save_at=Instant::now();let mut first=true;
 let result=(||{
  let runtime=tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|_|zwen_transport::Error::Network)?;
  runtime.block_on(async{
   let client=zwen_transport::client()?;
   zwen_transport::stream_messages(&client,&config,&key,&messages,cancel,|part|{
    turn.answer.push_str(part);
    // Screenshots are consumed once the model actually starts answering. A
    // request rejected before the first token (429, network, …) keeps them so
    // the user can retry without re-capturing.
    if !consume_ids.is_empty(){
     let mut store=AttachmentStore::open(&attach_dir).ok();
     if let Some(store)=store.as_mut(){
      let mut all=true;
      for id in consume_ids.iter(){
       match store.remove(id){Ok(())=>{},Err(zwen_core::Error::AttachmentMissing)=>{},Err(_)=>{all=false;}}
      }
      if all{consume_ids.clear();}
     }
     let _=app.emit("attachments-changed",());
    }
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
struct Prepared {
 turn:Turn,config:zwen_core::provider::ProviderConfig,
 messages:Vec<zwen_core::completion::Message>,consume_ids:Vec<String>,
}
fn prepare_turn(journal:&mut Journal,dir:&std::path::Path,config:&zwen_core::provider::ProviderConfig,request_id:&str,prompt:String,conversation_id:Option<&str>,isolated:bool)->Result<Prepared,ClientError>{
 let profile=serde_json::to_string(&config).map_err(|_|ClientError::storage())?;
  // Screenshot questions carry bounded base64 payloads to the configured vision
  // model. Preparation happens here, off the UI thread; a failed payload aborts
  // before the question is persisted so no half-attached turn is stored.
 let conversation_id=if isolated{None}else{conversation_id};
 // Isolation is enforced before opening attachment storage. A selection action
 // must neither inspect nor consume screenshots waiting in the ordinary draft.
 let vision=if isolated{None}else{
  let store=AttachmentStore::open(dir)?;
  let attached=store.list()?;
  if attached.is_empty(){None}else{
   let image_model=config.image_model().ok_or_else(||error("image_model_missing","截图尚未接入模型；请先在模型设置中配置图片模型"))?.to_owned();
   let mut images=Vec::new();
   for attachment in &attached{images.push(BASE64_STANDARD.encode(store.model_payload(&attachment.id)?));}
   Some((image_model,images,attached.iter().map(|a|a.id.clone()).collect::<Vec<_>>()))
  }
 };
  let request_config=match &vision{Some((model,_,_))=>config.with_model(model)?,None=>config.clone()};
  let mut messages=journal.context(conversation_id,&profile,prompt.len())?;let count=messages.len()/2;
  let request_model=request_config.model().to_owned();
  let turn=journal.begin_in(&request_id,&prompt,&request_model,conversation_id,&profile,count)?;
  let (images,consume_ids)=vision.map(|(_,images,ids)|(images,ids)).unwrap_or_default();
 messages.push(zwen_core::completion::Message{role:zwen_core::completion::Role::User,content:prompt,images});
 Ok(Prepared{turn,config:request_config,messages,consume_ids})
}
enum StartScope {Conversation(Option<String>),Selection{selection:String,action_id:String,expected_revision:u64}}
#[tauri::command]
pub async fn start_chat(app:tauri::AppHandle,storage:tauri::State<'_,SharedStorage>,chat:tauri::State<'_,Shared>,request_id:String,prompt:String,conversation_id:Option<String>)->Result<Reply,ClientError>{
 start(app,storage.inner().clone(),chat.inner().clone(),request_id,prompt,StartScope::Conversation(conversation_id)).await
}
#[tauri::command]
pub async fn start_selection_chat(app:tauri::AppHandle,storage:tauri::State<'_,SharedStorage>,chat:tauri::State<'_,Shared>,inbox:tauri::State<'_,crate::selection::Shared>,request_id:String,selection:String,action_id:String,expected_revision:u64)->Result<Reply,ClientError>{
 if inbox.lock().map_err(|_|ClientError::window())?.frozen_text()!=Some(selection.as_str()){
  return Err(error("selection_changed","待处理选区状态已改变，请重新选择操作"));
 }
 start(app,storage.inner().clone(),chat.inner().clone(),request_id,String::new(),StartScope::Selection{selection,action_id,expected_revision}).await
}
async fn start(app:tauri::AppHandle,storage:SharedStorage,state:Shared,request_id:String,prompt:String,scope:StartScope)->Result<Reply,ClientError>{
 tauri::async_runtime::spawn_blocking(move||{
  let mut s=state.lock().map_err(|_|ClientError::storage())?;
  if s.active.is_some(){return Err(error("chat_busy","当前请求尚未结束，请先停止"));}
  if s.revoked.contains(&request_id){return Err(error("chat_cancelled","此请求已取消"));}
  if s.last.as_ref().is_some_and(|r|!r.saved){return Err(error("chat_unsaved","上一条回答尚未保存，请先重试保存"));}
  let config=provider::configured(&storage)?;
  // Environment wins (run-with-key and smoke paths); otherwise the keyring.
  let key=match config.read_key(){
   Ok(key)=>key,
   Err(_)=>match crate::secret::read_model_key(){
    Ok(Some(value))=>zwen_core::provider::ApiKey::new(&value).map_err(|_|error("invalid_provider","钥匙环中的密钥无效；请在设置中重新保存"))?,
    _=>return Err(error("invalid_provider","未找到模型密钥；请在设置中保存到系统钥匙环，或用启动环境提供")),
   },
  };
  let dir=directory(&storage)?;
  let mut journal=Journal::open(&dir.join("chat.sqlite3"))?;
  let (prompt,conversation_id,isolated)=match scope{
   StartScope::Conversation(conversation)=>(prompt,conversation,false),
   StartScope::Selection{selection,action_id,expected_revision}=>(crate::quick_actions::prepare_request(&storage,&request_id,&selection,&action_id,expected_revision)?,None,true),
  };
  let prepared=prepare_turn(&mut journal,&dir,&config,&request_id,prompt,conversation_id.as_deref(),isolated)?;
  let reply=Reply{turn:prepared.turn.clone(),saved:true};s.last=Some(reply.clone());
  let (cancel,rx)=watch::channel(false);let id=request_id;let worker=state.clone();
  let handle=tauri::async_runtime::spawn_blocking(move||run(app,worker,journal,prepared.turn,prepared.config,key,prepared.messages,prepared.consume_ids,dir,rx));
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

#[cfg(test)]mod tests{
 use super::*;
 use zwen_core::provider::ProviderConfig;
 use std::{path::PathBuf,sync::atomic::{AtomicU64,Ordering}};
 static NEXT:AtomicU64=AtomicU64::new(0);
 struct Scratch(PathBuf);
 impl Scratch{
  fn new()->Self{let path=PathBuf::from("/tmp").join(format!("zwen-selection-chat-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));std::fs::create_dir(&path).unwrap();Self(path)}
 }
 impl Drop for Scratch{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
 fn config()->ProviderConfig{ProviderConfig::build("http://127.0.0.1:1","text-fixture","ZWEN_TEST_NEVER_READ",Some("vision-fixture")).unwrap()}
 fn history(dir:&std::path::Path,config:&ProviderConfig)->Journal{
  let mut journal=Journal::open(&dir.join("chat.sqlite3")).unwrap();
  let profile=serde_json::to_string(config).unwrap();
  let mut old=journal.begin_in("old-conversation","private old context","text-fixture",None,&profile,0).unwrap();
  old.answer="old answer".into();old.state=TurnState::Complete;old.revision=1;journal.checkpoint(&old).unwrap();journal
 }
 fn screenshot(dir:&std::path::Path)->String{
  let source=dir.join("source.png");std::fs::write(&source,include_bytes!("../icons/icon.png")).unwrap();
  let id="a".repeat(32);let mut store=AttachmentStore::open(dir).unwrap();store.import(&id,&source).unwrap();id
 }
 #[test]fn selection_request_has_new_conversation_and_no_screenshot_or_prior_context(){
  let dir=Scratch::new();let config=config();let mut journal=history(&dir.0,&config);let attachment=screenshot(&dir.0);
  let prepared=prepare_turn(&mut journal,&dir.0,&config,"selection-request","only this selection".into(),Some("old-conversation"),true).unwrap();
  assert_eq!(prepared.turn.conversation_id,"selection-request");assert_eq!(prepared.turn.context_turns,0);
  assert_eq!(prepared.config.model(),"text-fixture");assert_eq!(prepared.messages.len(),1);
  assert_eq!(prepared.messages[0].content,"only this selection");assert!(prepared.messages[0].images.is_empty());
  assert!(prepared.consume_ids.is_empty());
  assert_eq!(AttachmentStore::open(&dir.0).unwrap().list().unwrap()[0].id,attachment);
 }
 #[test]fn isolated_text_request_does_not_open_broken_attachment_storage(){
  let dir=Scratch::new();let config=config();let mut journal=history(&dir.0,&config);
  std::fs::write(dir.0.join("attachment.sqlite3"),b"synthetic broken attachment database").unwrap();
  let prepared=prepare_turn(&mut journal,&dir.0,&config,"isolated-text","selected text".into(),None,true).unwrap();
  assert_eq!(prepared.messages.len(),1);assert!(prepared.messages[0].images.is_empty());
  assert_eq!(std::fs::read(dir.0.join("attachment.sqlite3")).unwrap(),b"synthetic broken attachment database");
 }
 #[test]fn ordinary_chat_keeps_context_and_vision_attachment_routing(){
  let dir=Scratch::new();let config=config();let mut journal=history(&dir.0,&config);let attachment=screenshot(&dir.0);
  let prepared=prepare_turn(&mut journal,&dir.0,&config,"normal-request","question".into(),Some("old-conversation"),false).unwrap();
  assert_eq!(prepared.turn.conversation_id,"old-conversation");assert_eq!(prepared.turn.context_turns,1);
  assert_eq!(prepared.config.model(),"vision-fixture");assert_eq!(prepared.messages.len(),3);
  assert_eq!(prepared.messages[0].content,"private old context");assert_eq!(prepared.messages[1].content,"old answer");
  assert_eq!(prepared.messages[2].content,"question");assert_eq!(prepared.messages[2].images.len(),1);
  assert_eq!(prepared.consume_ids,vec![attachment]);
 }
}
