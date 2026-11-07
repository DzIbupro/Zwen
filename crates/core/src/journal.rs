use crate::{Error,open_private_database,MAX_DRAFT_BYTES};
use rusqlite::{Connection,OptionalExtension,params};
use serde::Serialize;
use std::path::Path;
#[derive(Clone,Debug,PartialEq,Serialize)]
#[serde(rename_all="snake_case")]
pub enum TurnState {Streaming,Complete,Stopped,Failed,Interrupted,Limited,Filtered}
impl TurnState {
 fn as_str(&self)->&str{match self{Self::Streaming=>"streaming",Self::Complete=>"complete",Self::Stopped=>"stopped",Self::Failed=>"failed",Self::Interrupted=>"interrupted",Self::Limited=>"limited",Self::Filtered=>"filtered"}}
 fn parse(s:&str)->Result<Self,Error>{Ok(match s{"streaming"=>Self::Streaming,"complete"=>Self::Complete,"stopped"=>Self::Stopped,"failed"=>Self::Failed,"interrupted"=>Self::Interrupted,"limited"=>Self::Limited,"filtered"=>Self::Filtered,_=>return Err(Error::InvalidChat)})}
}
#[derive(Clone,Debug,PartialEq,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Turn {pub conversation_id:String,#[serde(skip_serializing)] pub provider_profile:String,pub context_turns:usize,pub request_id:String,pub prompt:String,pub model:String,pub answer:String,pub revision:u64,pub state:TurnState,pub error:Option<String>,pub created_at:i64}
pub struct Journal {connection:Connection}
// Guard allocations even when a database is externally malformed.
const COLUMNS:&str="CASE WHEN length(request_id)<=100 THEN request_id END,CASE WHEN length(CAST(prompt AS BLOB))<=262144 THEN prompt END,CASE WHEN length(CAST(model AS BLOB))<=200 THEN model END,CASE WHEN length(CAST(answer AS BLOB))<=262144 THEN answer END,revision,substr(state,1,32),CASE WHEN length(CAST(error AS BLOB))<=500 THEN error END,created_at,CASE WHEN length(conversation_id)<=100 THEN conversation_id END,CASE WHEN length(CAST(provider_profile AS BLOB))<=8192 THEN provider_profile END,context_turns";
fn read(connection:&Connection,id:Option<&str>)->Result<Option<Turn>,Error>{
 let tail=if id.is_some(){"WHERE request_id=?1"}else{"WHERE ?1 IS NULL ORDER BY id DESC LIMIT 1"};
 let sql=format!("SELECT {COLUMNS} FROM turns {tail}");
 let row=connection.query_row(&sql,[id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,i64>(4)?,r.get::<_,String>(5)?,r.get::<_,Option<String>>(6)?,r.get::<_,i64>(7)?,r.get::<_,String>(8)?,r.get::<_,String>(9)?,r.get::<_,i64>(10)?))).optional()?;
 row.map(|(request_id,prompt,model,answer,revision,state,error,created_at,conversation_id,provider_profile,context_turns)|Ok(Turn{conversation_id,provider_profile,context_turns:usize::try_from(context_turns).map_err(|_|Error::InvalidChat)?,request_id,prompt,model,answer,revision:u64::try_from(revision).map_err(|_|Error::InvalidChat)?,state:TurnState::parse(&state)?,error,created_at})).transpose()
}
impl Journal {
 pub fn binding(&self,conversation:&str,profile:&str)->Result<(),Error>{
  let saved:Option<String>=self.connection.query_row("SELECT CASE WHEN length(CAST(provider_profile AS BLOB))<=8192 THEN provider_profile END FROM turns WHERE conversation_id=?1 ORDER BY id LIMIT 1",[conversation],|r|r.get(0)).optional()?;
  if saved.as_deref()!=Some(profile){return Err(Error::ConversationChanged);}Ok(())
 }
 pub fn begin_in(&mut self,id:&str,prompt:&str,model:&str,conversation:Option<&str>,profile:&str,count:usize)->Result<Turn,Error>{
  if id.is_empty()||id.len()>100||!id.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_')||prompt.trim().is_empty()||prompt.len()>MAX_DRAFT_BYTES||model.is_empty()||model.len()>200||profile.len()>8192||count>8{return Err(Error::InvalidChat);}
  if let Some(conversation)=conversation{self.binding(conversation,profile)?;}
  self.connection.execute("INSERT INTO turns(request_id,conversation_id,provider_profile,context_turns,prompt,model) VALUES(?1,?2,?3,?4,?5,?6)",params![id,conversation.unwrap_or(id),profile,count as i64,prompt,model])?;
  read(&self.connection,Some(id))?.ok_or(Error::InvalidChat)
 }
 pub fn context(&self,conversation:Option<&str>,profile:&str,prompt_bytes:usize)->Result<Vec<crate::completion::Message>,Error>{
  use crate::completion::{Message,Role};
  let Some(conversation)=conversation else{return Ok(vec![]);};self.binding(conversation,profile)?;
  let mut statement=self.connection.prepare("SELECT CASE WHEN length(request_id)<=100 THEN request_id END FROM turns WHERE conversation_id=?1 AND answer!='' AND state!='streaming' ORDER BY id DESC LIMIT 8")?;
  let ids=statement.query_map([conversation],|r|r.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;
  let mut budget=MAX_DRAFT_BYTES.checked_sub(prompt_bytes).ok_or(Error::ContextTooLarge)?;let mut chosen=Vec::new();
  for id in ids{let t=read(&self.connection,Some(&id))?.ok_or(Error::InvalidChat)?;let size=t.prompt.len()+t.answer.len();if size>budget{if chosen.is_empty(){return Err(Error::ContextTooLarge);}break;}budget-=size;chosen.push(t);}
  let mut messages=Vec::new();for t in chosen.into_iter().rev(){messages.push(Message{role:Role::User,content:t.prompt,images:Vec::new()});messages.push(Message{role:Role::Assistant,content:t.answer,images:Vec::new()});}Ok(messages)
 }
 pub fn history(&self,offset:usize,limit:usize)->Result<Vec<Summary>,Error>{
  if limit==0||limit>30||offset>100000{return Err(Error::InvalidChat);}
  let mut statement=self.connection.prepare("SELECT CASE WHEN length(request_id)<=100 THEN request_id END,CASE WHEN length(conversation_id)<=100 THEN conversation_id END,substr(prompt,1,240),CASE WHEN length(CAST(model AS BLOB))<=200 THEN model END,substr(state,1,32),created_at FROM turns ORDER BY id DESC LIMIT ?1 OFFSET ?2")?;
  let rows=statement.query_map(params![limit as i64,offset as i64],|r|Ok((r.get::<_,String>(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get::<_,String>(4)?,r.get(5)?)))?;
  rows.map(|row|{let(request_id,conversation_id,title,model,state,created_at)=row?;Ok(Summary{request_id,conversation_id,title,model,state:TurnState::parse(&state)?,created_at})}).collect()
 }
 /// Delete one terminal record; the independent Markdown library is untouched.
 pub fn delete_completed(&mut self,id:&str)->Result<bool,Error>{self.delete_history(Some(&[id.to_owned()])).map(|n|n>0)}
 /// A selected set or all terminal history is removed atomically, across pages.
 pub fn delete_history(&mut self,ids:Option<&[String]>)->Result<usize,Error>{
  if let Some(ids)=ids{if ids.is_empty()||ids.iter().any(|id|id.is_empty()||id.len()>100||!id.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_')){return Err(Error::InvalidChat);}}
  let tx=self.connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
  let changed=if let Some(ids)=ids{
   {let mut check=tx.prepare("SELECT CASE WHEN length(state)<=32 THEN state END FROM turns WHERE request_id=?1")?;
    for id in ids{let state:Option<String>=check.query_row([id],|r|r.get(0)).optional()?;if let Some(state)=state{if TurnState::parse(&state)?==TurnState::Streaming{return Err(Error::ChatConflict);}}}}
   let mut remove=tx.prepare("DELETE FROM turns WHERE request_id=?1")?;let mut count=0;for id in ids{count+=remove.execute([id])?;}count
  }else{
   {let mut check=tx.prepare("SELECT DISTINCT CASE WHEN length(state)<=32 THEN state END FROM turns")?;let rows=check.query_map([],|r|r.get::<_,String>(0))?;for state in rows{if TurnState::parse(&state?)?==TurnState::Streaming{return Err(Error::ChatConflict);}}}
   tx.execute("DELETE FROM turns",[])?
  };
  tx.commit()?;Ok(changed)
 }
 pub fn find(&self,id:&str)->Result<Option<Turn>,Error>{read(&self.connection,Some(id))}

 pub fn open(path:&Path)->Result<Self,Error>{
  let connection=open_private_database(path,|c|{
   let version:i64=c.pragma_query_value(None,"user_version",|r|r.get(0))?;
   if version!=0&&version!=2{return Err(Error::UnsupportedSchema);}
   if version==0{let tx=c.transaction()?;tx.execute_batch("CREATE TABLE turns(id INTEGER PRIMARY KEY,request_id TEXT NOT NULL UNIQUE,conversation_id TEXT NOT NULL,provider_profile TEXT NOT NULL,context_turns INTEGER NOT NULL,prompt TEXT NOT NULL,model TEXT NOT NULL,answer TEXT NOT NULL DEFAULT '',revision INTEGER NOT NULL DEFAULT 0 CHECK(revision>=0),state TEXT NOT NULL DEFAULT 'streaming',error TEXT,created_at INTEGER NOT NULL DEFAULT (unixepoch()));CREATE INDEX turns_conversation ON turns(conversation_id,id);PRAGMA user_version=2;")?;tx.commit()?;}
   read(c,None)?;Ok(())
  })?;Ok(Self{connection})
 }
 pub fn begin(&mut self,id:&str,prompt:&str,model:&str)->Result<Turn,Error>{self.begin_in(id,prompt,model,None,"",0)}
 pub fn latest(&self)->Result<Option<Turn>,Error>{read(&self.connection,None)}
 pub fn checkpoint(&mut self,t:&Turn)->Result<(),Error>{
  if t.answer.len()>MAX_DRAFT_BYTES||t.error.as_ref().is_some_and(|e|e.len()>500){return Err(Error::InvalidChat);}
  let revision=i64::try_from(t.revision).map_err(|_|Error::InvalidChat)?;
  let changed=self.connection.execute("UPDATE turns SET answer=?1,revision=?2,state=?3,error=?4 WHERE request_id=?5 AND revision<?2 AND state='streaming'",params![t.answer,revision,t.state.as_str(),t.error,t.request_id])?;
  if changed==0&&read(&self.connection,Some(&t.request_id))?.as_ref()!=Some(t){return Err(Error::ChatConflict);}
  Ok(())
 }
 pub fn recover(&mut self)->Result<(),Error>{self.connection.execute("UPDATE turns SET state='interrupted',revision=revision+1,error='上次生成未完整结束，已恢复最近保存的内容' WHERE state='streaming'",[])?;Ok(())}
}

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Summary {pub request_id:String,pub conversation_id:String,pub title:String,pub model:String,pub state:TurnState,pub created_at:i64}
