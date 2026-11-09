//! Native chat commands. Screenshots route to the configured vision model.
use crate::{SharedStorage,ClientError,provider};
use base64::{Engine as _,engine::general_purpose::STANDARD as BASE64_STANDARD};
use serde::Serialize;
use std::{collections::VecDeque,sync::{Arc,Mutex},time::{Instant,Duration}};
use tauri::Emitter;
use tokio::sync::watch;
use zwen_core::{attachment::AttachmentStore,journal::{Journal,Turn,TurnState},completion::FinishReason};
#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct RetryState {attempt:u32,next_retry_at:u64,reason:&'static str}
#[derive(Clone,Serialize)]
pub struct Reply {#[serde(flatten)] turn:Turn,saved:bool,#[serde(skip_serializing_if="Option::is_none")]retry:Option<RetryState>}
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
fn run(app:tauri::AppHandle,state:Shared,mut journal:Journal,mut turn:Turn,config:zwen_core::provider::ProviderConfig,key:zwen_core::provider::ApiKey,messages:Vec<zwen_core::completion::Message>,consume_ids:Vec<String>,attach_dir:std::path::PathBuf,cancel:watch::Receiver<bool>)->Result<Reply,ClientError>{
 let mut emit_at=Instant::now();let mut save_at=Instant::now();let mut first=true;
 let result=(||{
  let runtime=tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|_|zwen_transport::Error::Network)?;
  runtime.block_on(async{
   let client=zwen_transport::client()?;
   use zwen_transport::retry::{stream_messages_with_retry,RetryPolicy,StreamEvent};
   stream_messages_with_retry(&client,&config,&key,&messages,cancel,RetryPolicy::default(),|event|{
    match event {
     StreamEvent::Retrying(status)=>{
      turn.revision+=1;journal.checkpoint(&turn).map_err(|_|())?;
      let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis().min(u64::MAX as u128) as u64;
      let delay=status.delay.as_millis().min(u64::MAX as u128) as u64;
      publish(&app,&state,Reply{turn:turn.clone(),saved:true,retry:Some(RetryState{attempt:status.attempt,next_retry_at:now.saturating_add(delay),reason:status.reason})})?;
     }
     StreamEvent::Attempt=>{
      turn.revision+=1;journal.checkpoint(&turn).map_err(|_|())?;
      publish(&app,&state,Reply{turn:turn.clone(),saved:true,retry:None})?;
     }
     StreamEvent::Text(part)=>{
      turn.answer.push_str(part);
      let emit=first||emit_at.elapsed()>=Duration::from_millis(80);let save=first||save_at.elapsed()>=Duration::from_millis(500);
      if emit||save{turn.revision+=1;}
      if save{journal.checkpoint(&turn).map_err(|_|())?;save_at=Instant::now();}
      if emit{publish(&app,&state,Reply{turn:turn.clone(),saved:save,retry:None})?;emit_at=Instant::now();first=false;}
     }
    }
    Ok(())
   }).await
  })
 })();
 turn.revision+=1;
 turn.state=match result{Ok(FinishReason::Stop)=>TurnState::Complete,Ok(FinishReason::Length)=>TurnState::Limited,Ok(FinishReason::Filtered)=>TurnState::Filtered,Err(zwen_transport::Error::Cancelled)=>TurnState::Stopped,Err(_)=>TurnState::Failed};
 turn.error=match &result{Err(zwen_transport::Error::Cancelled)=>None,Err(e)=>Some(e.to_string()),Ok(FinishReason::Length)=>Some("达到服务输出限制，回答可能不完整".into()),Ok(FinishReason::Filtered)=>Some("服务限制了本轮输出".into()),_=>None};
 let saved=journal.checkpoint(&turn).is_ok();if saved&&turn.state==TurnState::Complete&&!consume_ids.is_empty(){
  // Failed/partial requests must retain their exact screenshots for manual recovery.
  if let Ok(mut store)=AttachmentStore::open(&attach_dir){for id in &consume_ids{let _=store.remove(id);}}
  let _=app.emit("attachments-changed",());
 }
 let reply=Reply{turn,saved,retry:None};
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
enum StartScope {Paper{source:String,options:zwen_core::paper::ReadingOptions},Conversation(Option<String>),Selection{selection:String,action_id:String,expected_revision:u64}}
pub async fn start_paper(app:tauri::AppHandle,storage:SharedStorage,state:Shared,request_id:String,source:String,options:zwen_core::paper::ReadingOptions,question:Option<String>)->Result<Reply,ClientError>{
 start(app,storage,state,request_id,question.unwrap_or_default(),StartScope::Paper{source,options}).await
}
fn bound_document(dir:&std::path::Path,id:&str)->Result<Option<(zwen_core::paper::Source,zwen_core::paper::ReadingOptions)>,ClientError>{
 let document=if dir.join("papers-v2/library.sqlite3").exists(){zwen_core::paper::Library::open(dir).map_err(crate::paper::error)?.session(id).map_err(crate::paper::error)?}else{None};
 if id.starts_with("paper-")&&document.is_none(){return Err(error("paper_source_missing","论文来源或会话绑定丢失，已停止发送；请重新导入并新建分析"));}Ok(document)
}
fn prepare_document(journal:&mut Journal,dir:&std::path::Path,config:&zwen_core::provider::ProviderConfig,request_id:&str,prompt:String,conversation:Option<&str>,source:&zwen_core::paper::Source,options:&zwen_core::paper::ReadingOptions)->Result<Prepared,ClientError>{
 let _=dir; // File access stays in the paper adapter; screenshots are never inspected here.
 let full=source.request(options,&prompt).map_err(crate::paper::error)?;
 let profile=serde_json::to_string(config).map_err(|_|ClientError::storage())?;
 let mut messages=match journal.context(conversation,&profile,full.len()){
  Ok(messages)=>messages,Err(zwen_core::Error::ContextTooLarge)=>Vec::new(),Err(e)=>return Err(e.into())
 };let count=messages.len()/2;
 let turn=journal.begin_in(request_id,&prompt,config.model(),conversation,&profile,count)?;
 messages.push(zwen_core::completion::Message{role:zwen_core::completion::Role::User,content:full,images:Vec::new()});
 Ok(Prepared{turn,config:config.clone(),messages,consume_ids:Vec::new()})
}
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
  let dir=directory(&storage)?;
  let mut journal=Journal::open(&dir.join("chat.sqlite3"))?;
  let mut document=None;let mut new_binding=None;
  if !matches!(&scope,StartScope::Paper{..})&&request_id.starts_with("paper-"){return Err(error("invalid_paper_session","论文请求标识保留给论文分析入口"));}
  let (prompt,conversation_id,isolated)=match scope{
   StartScope::Paper{source,options}=>{
    if !request_id.starts_with("paper-"){return Err(error("invalid_paper_session","论文会话标识无效"));}
    let lib=crate::paper::library(&storage)?;let paper=lib.get(&source).map_err(crate::paper::error)?;
    let question=if prompt.trim().is_empty(){"请按阅读要求分析这篇论文。"}else{prompt.trim()};
    let label=format!("论文阅读 · {}\n\n{}",paper.meta.name,question);
    paper.request(&options,&label).map_err(crate::paper::error)?;
    new_binding=Some((source,options.clone()));document=Some((paper,options));(label,None,true)
   },
   StartScope::Conversation(conversation)=>{
    if let Some(id)=conversation.as_deref(){document=bound_document(&dir,id)?;}
    (prompt,conversation,false)
   },
   StartScope::Selection{selection,action_id,expected_revision}=>(crate::quick_actions::prepare_request(&storage,&request_id,&selection,&action_id,expected_revision)?,None,true),
  };
  // A document may use only an explicitly configured environment key or a
  // service-bound keyring entry, never an unbound key for another endpoint.
  let key=match config.read_key(){
   Ok(key)=>key,
   Err(_)=>match if document.is_some(){crate::secret::read_paper_key(&config)}else{crate::secret::read_model_key()}{
    Ok(Some(value))=>zwen_core::provider::ApiKey::new(&value).map_err(|_|error("invalid_provider","钥匙环中的密钥无效；请在设置中重新保存"))?,
    _=>return Err(error("invalid_provider","未找到当前服务的密钥；请在模型设置中重新保存对应密钥，或用配置的环境变量提供")),
   },
  };
  let prepared=if let Some((source,options))=document{prepare_document(&mut journal,&dir,&config,&request_id,prompt,conversation_id.as_deref(),&source,&options)?}else{prepare_turn(&mut journal,&dir,&config,&request_id,prompt,conversation_id.as_deref(),isolated)?};
  if let Some((id,options))=new_binding{
   if let Err(e)=crate::paper::library(&storage)?.bind(&prepared.turn.conversation_id,&id,&options){
    let mut failed=prepared.turn;failed.revision+=1;failed.state=TurnState::Failed;failed.error=Some(e.to_string());let _=journal.checkpoint(&failed);return Err(crate::paper::error(e));
   }
  }
  let reply=Reply{turn:prepared.turn.clone(),saved:true,retry:None};s.last=Some(reply.clone());
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
  let persisted=journal.latest()?.map(|turn|Reply{turn,saved:true,retry:None});
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

#[derive(serde::Deserialize)]
#[serde(tag="kind",rename_all="lowercase",rename_all_fields="camelCase",deny_unknown_fields)]
pub enum HistoryDeleteScope{Selected{request_ids:Vec<String>},All{}}
fn delete_history_scope(dir:&std::path::Path,state:&Shared,scope:&HistoryDeleteScope)->Result<usize,ClientError>{
 let mut s=state.lock().map_err(|_|ClientError::storage())?;
 if s.active.is_some()||s.last.as_ref().is_some_and(|reply|!reply.saved){return Err(error("history_busy","请先停止生成并保存回答，再删除历史"));}
 let ids=match scope{HistoryDeleteScope::Selected{request_ids}=>Some(request_ids.as_slice()),HistoryDeleteScope::All{}=>None};
 let mut journal=Journal::open(&dir.join("chat.sqlite3"))?;let count=journal.delete_history(ids)?;
 if s.last.as_ref().is_some_and(|reply|ids.is_none_or(|ids|ids.iter().any(|id|id==&reply.turn.request_id))){s.last=None;}
 Ok(count)
}
fn delete_saved_turn(dir:&std::path::Path,state:&Shared,request_id:&str)->Result<(),ClientError>{delete_history_scope(dir,state,&HistoryDeleteScope::Selected{request_ids:vec![request_id.into()]}).map(|_|())}
#[tauri::command]
pub async fn delete_chat_history(storage:tauri::State<'_,SharedStorage>,chat:tauri::State<'_,Shared>,scope:HistoryDeleteScope)->Result<usize,ClientError>{
 let storage=storage.inner().clone();let state=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||delete_history_scope(&directory(&storage)?,&state,&scope)).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn delete_chat_turn(storage:tauri::State<'_,SharedStorage>,chat:tauri::State<'_,Shared>,request_id:String)->Result<(),ClientError>{
 let storage=storage.inner().clone();let state=chat.inner().clone();
 tauri::async_runtime::spawn_blocking(move||delete_saved_turn(&directory(&storage)?,&state,&request_id)).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn get_chat_history(storage:tauri::State<'_,SharedStorage>,page:usize)->Result<Vec<zwen_core::journal::Summary>,ClientError>{
 let storage=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||{let offset=page.checked_mul(20).ok_or(zwen_core::Error::InvalidChat)?;let journal=Journal::open(&directory(&storage)?.join("chat.sqlite3"))?;journal.history(offset,21).map_err(Into::into)}).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn get_chat_turn(storage:tauri::State<'_,SharedStorage>,chat:tauri::State<'_,Shared>,request_id:String)->Result<Option<Reply>,ClientError>{
 let storage=storage.inner().clone();let state=chat.inner().clone();tauri::async_runtime::spawn_blocking(move||{
  if request_id.len()>100{return Err(zwen_core::Error::InvalidChat.into());}
  let s=state.lock().map_err(|_|ClientError::storage())?;let journal=Journal::open(&directory(&storage)?.join("chat.sqlite3"))?;let saved=journal.find(&request_id)?.map(|turn|Reply{turn,saved:true,retry:None});
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
 #[test]fn history_bulk_scope_rejects_ambiguous_payloads_and_preserves_cache_on_failure(){
  for payload in [r#"{"kind":"all","requestIds":["a"]}"#,r#"{"kind":"selected"}"#,r#"{"kind":"unknown"}"#]{assert!(serde_json::from_str::<HistoryDeleteScope>(payload).is_err());}
  assert!(matches!(serde_json::from_str::<HistoryDeleteScope>(r#"{"kind":"selected","requestIds":["a"]}"#).unwrap(),HistoryDeleteScope::Selected{..}));
  let scratch=Scratch::new();let path=scratch.0.join("chat.sqlite3");let mut j=Journal::open(&path).unwrap();let mut last=None;
  for id in ["a","b"]{let mut t=j.begin(id,"q","m").unwrap();t.state=TurnState::Complete;t.revision=1;j.checkpoint(&t).unwrap();last=Some(t);}
  let state=Shared::default();state.lock().unwrap().last=Some(Reply{turn:last.unwrap(),saved:true,retry:None});
  let c=rusqlite::Connection::open(&path).unwrap();c.execute_batch("CREATE TRIGGER reject_b BEFORE DELETE ON turns WHEN OLD.request_id='b' BEGIN SELECT RAISE(ABORT,'fixture'); END").unwrap();
  assert!(delete_history_scope(&scratch.0,&state,&HistoryDeleteScope::All{}).is_err());assert_eq!(state.lock().unwrap().last.as_ref().unwrap().turn.request_id,"b");assert_eq!(j.history(0,20).unwrap().len(),2);
  c.execute_batch("DROP TRIGGER reject_b").unwrap();assert_eq!(delete_history_scope(&scratch.0,&state,&HistoryDeleteScope::Selected{request_ids:vec!["a".into()]}).unwrap(),1);assert!(state.lock().unwrap().last.is_some());
  assert_eq!(delete_history_scope(&scratch.0,&state,&HistoryDeleteScope::All{}).unwrap(),1);assert!(state.lock().unwrap().last.is_none());
 }
 #[test]fn history_delete_protects_live_and_unsaved_state_and_clears_deleted_cache(){
  let scratch=Scratch::new();let path=scratch.0.join("chat.sqlite3");let mut j=Journal::open(&path).unwrap();let mut t=j.begin("delete-me","q","m").unwrap();t.state=TurnState::Complete;t.revision=1;j.checkpoint(&t).unwrap();
  let state=Shared::default();state.lock().unwrap().last=Some(Reply{turn:t.clone(),saved:false,retry:None});assert!(delete_saved_turn(&scratch.0,&state,"delete-me").is_err());assert!(j.find("delete-me").unwrap().is_some());
  {let mut s=state.lock().unwrap();s.last.as_mut().unwrap().saved=true;let(cancel,_)=watch::channel(false);s.active=Some(Active{id:"live".into(),cancel,handle:None});}
  assert!(delete_saved_turn(&scratch.0,&state,"delete-me").is_err());state.lock().unwrap().active=None;
  delete_saved_turn(&scratch.0,&state,"delete-me").unwrap();assert!(state.lock().unwrap().last.is_none());assert!(j.find("delete-me").unwrap().is_none());
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
 #[test]fn paper_context_resends_source_without_opening_draft_screenshots(){
  let dir=Scratch::new();let config=config();let mut journal=history(&dir.0,&config);
  std::fs::write(dir.0.join("attachment.sqlite3"),b"invalid screenshot database").unwrap();
  let source=zwen_core::paper::Source{meta:zwen_core::paper::Metadata{id:"0123456789abcdef0123456789abcdef".into(),name:"paper.pdf".into(),page_count:1,readable_pages:1},pages:vec!["original paper evidence".into()]};
  let p=prepare_document(&mut journal,&dir.0,&config,"paper-followup","解释证据".into(),Some("old-conversation"),&source,&Default::default()).unwrap();
  assert_eq!(p.messages.len(),3);assert!(p.messages[2].content.contains("original paper evidence"));
  assert!(p.messages.iter().all(|m|m.images.is_empty()));assert!(p.consume_ids.is_empty());
  assert_eq!(p.config.model(),config.model());assert_eq!(p.turn.prompt,"解释证据");
  assert_eq!(std::fs::read(dir.0.join("attachment.sqlite3")).unwrap(),b"invalid screenshot database");
 }

 #[test]fn missing_paper_binding_never_falls_back_to_ordinary_chat(){
  let dir=Scratch::new();assert!(bound_document(&dir.0,"paper-missing").is_err());assert!(bound_document(&dir.0,"ordinary").unwrap().is_none());
  let _=zwen_core::paper::Library::open(&dir.0).unwrap();assert!(bound_document(&dir.0,"paper-missing").is_err());
 }
 #[test]fn large_source_followup_can_drop_history_but_keeps_original(){
  let dir=Scratch::new();let config=config();let mut journal=history(&dir.0,&config);
  let source=zwen_core::paper::Source{meta:zwen_core::paper::Metadata{id:"0123456789abcdef0123456789abcdef".into(),name:"paper.pdf".into(),page_count:1,readable_pages:1},pages:vec!["x".repeat(245000)]};
  let first=prepare_document(&mut journal,&dir.0,&config,"paper-large","分析".into(),None,&source,&Default::default()).unwrap();
  let mut turn=first.turn;turn.revision+=1;turn.answer="y".repeat(32768);turn.state=TurnState::Complete;journal.checkpoint(&turn).unwrap();
  let follow=prepare_document(&mut journal,&dir.0,&config,"followup","解释原文".into(),Some("paper-large"),&source,&Default::default()).unwrap();assert_eq!(follow.turn.context_turns,0);assert!(follow.messages[0].content.contains(&"x".repeat(245000)));
 }

}
