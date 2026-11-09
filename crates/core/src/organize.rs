//! AI organizing data model: topics, note memberships, and revertible batches.
//! The knowledge notes stay the evidence; every AI write is tied to a batch so
//! it can be undone, and manual rows are never touched by batch undo.
use crate::{Error,open_private_database};
use rusqlite::{Connection,OptionalExtension,TransactionBehavior,params};
use serde::Serialize;
use std::path::Path;
mod rounds;
pub use rounds::OrganizeRound;

pub const MAX_TOPIC_NAME:usize=80;
pub const MAX_COLUMN:usize=80;
pub const MAX_BATCH_NOTES:usize=8;
pub const MAX_BATCH_TOPICS:usize=16;
pub const MAX_BATCH_ASSIGN:usize=32;
pub const MAX_SCOPE:usize=64;

#[derive(Clone,Debug,PartialEq,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Topic{pub id:String,pub name:String,pub column:String,pub created_by:String,pub batch_id:Option<String>,pub created_at:i64,#[serde(rename="locked")] pub locked:bool}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct TopicAssignments{pub topic:Topic,pub note_ids:Vec<String>,pub count:i64}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Relation{pub id:String,pub from_topic:String,pub to_topic:String,pub reason:String,pub evidence:String,pub added_by:String}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Batch{pub id:String,pub model:String,status:std::cell::Cell<&'static str>,pub created_at:i64,finished_at:std::cell::Cell<Option<i64>>}
impl Batch{
 pub fn status(&self)->Option<&'static str>{Some(self.status.get())}
 pub fn finished_at(&self)->Option<i64>{self.finished_at.get()}
}

pub fn valid_name(name:&str)->bool{let name=name.trim();!name.is_empty()&&name.len()<=MAX_TOPIC_NAME&&!name.chars().any(char::is_control)}
pub fn valid_column(column:&str)->bool{let column=column.trim();column.len()<=MAX_COLUMN&&!column.chars().any(char::is_control)}

/// FNV-1a 64: cheap content fingerprint so stale AI assignments (note edited
/// after organizing) can be detected without pulling in a hash dependency.
pub fn content_hash(bytes:&[u8])->u64{
 let mut hash:u64=0xcbf29ce484222325;
 for byte in bytes{hash^=u64::from(*byte);hash=hash.wrapping_mul(0x100000001b3);}
 hash
}

fn connection_init(connection:&mut Connection)->Result<(),Error>{
 let version:i64=connection.pragma_query_value(None,"user_version",|r|r.get(0))?;
 if version!=0&&version!=1{return Err(Error::UnsupportedSchema);}
 if version==0{
  let tx=connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  tx.execute_batch("CREATE TABLE topics(
   id TEXT PRIMARY KEY,
   name TEXT NOT NULL UNIQUE,
   column_name TEXT NOT NULL DEFAULT '',
   created_by TEXT NOT NULL,
   batch_id TEXT,
   locked INTEGER NOT NULL DEFAULT 0,
   created_at INTEGER NOT NULL
  );
  CREATE TABLE memberships(
   note_id TEXT NOT NULL,
   topic_id TEXT NOT NULL,
   added_by TEXT NOT NULL,
   batch_id TEXT,
   note_hash INTEGER NOT NULL,
   created_at INTEGER NOT NULL,
   PRIMARY KEY(note_id,topic_id)
  );
  CREATE TABLE relations(
   id TEXT PRIMARY KEY,
   from_topic TEXT NOT NULL,
   to_topic TEXT NOT NULL,
   reason TEXT NOT NULL,
   evidence TEXT NOT NULL,
   added_by TEXT NOT NULL,
   batch_id TEXT,
   created_at INTEGER NOT NULL,
   UNIQUE(from_topic,to_topic)
  );
  CREATE TABLE meta(
   key TEXT PRIMARY KEY,
   value TEXT NOT NULL
  );
  CREATE TABLE batches(
   id TEXT PRIMARY KEY,
   model TEXT NOT NULL,
   scope TEXT NOT NULL,
   status TEXT NOT NULL,
   changeset TEXT NOT NULL DEFAULT '[]',
   error TEXT,
   created_at INTEGER NOT NULL,
   finished_at INTEGER
  );PRAGMA user_version = 1;")?;
  tx.commit()?;
 }
 // Compatible schema-1 extension: each AI relationship tracks its own source
 // body even when a user has made both memberships manual.
 let has_hash:i64=connection.query_row("SELECT count(*) FROM pragma_table_info('relations') WHERE name='note_hash'",[],|r|r.get(0))?;
 if has_hash==0{connection.execute("ALTER TABLE relations ADD COLUMN note_hash INTEGER",[])?;}
 connection.execute_batch("CREATE TABLE IF NOT EXISTS organize_rounds(id TEXT PRIMARY KEY, profile TEXT NOT NULL, scope TEXT NOT NULL, completed INTEGER NOT NULL DEFAULT 0, batches TEXT NOT NULL DEFAULT '[]', undone INTEGER NOT NULL DEFAULT 0)")?;
 Ok(())
}

pub struct OrganizeStore{connection:Connection}
impl OrganizeStore{
 pub fn open(data_dir:&Path)->Result<Self,Error>{
  let mut builder=std::fs::DirBuilder::new();builder.recursive(true);
  #[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
  builder.create(data_dir)?;
  let connection=open_private_database(&data_dir.join("organize.sqlite3"),connection_init)?;
  let mut store=Self{connection};
  // File edits from other Markdown tools invalidate derived AI evidence too.
  if data_dir.join("notes").is_dir(){
   let notes=crate::notes::NoteStore::open(data_dir)?;
   let ids:Vec<String>={let mut q=store.connection.prepare("SELECT note_id FROM memberships UNION SELECT evidence FROM relations")?;let rows=q.query_map([],|r|r.get(0))?;rows.collect::<Result<_,_>>()?};
   for id in ids{match notes.read(&id){
    Ok(note)=>store.invalidate_note(&id,Some(content_hash(note.body.as_bytes())))?,
    Err(Error::NoteMissing)=>store.invalidate_note(&id,None)?,
    // An unreadable authority is not proof that the user deleted the note.
    Err(error)=>return Err(error),
   }}
  }
  Ok(store)
 }

 /// Applies one validated AI batch atomically. `assignments` must only reference
 /// note ids from `scope`; unknown topics are created (created_by='ai'). The
 /// changeset recorded on the batch is enough to undo everything it did.
 pub fn apply_batch(&mut self,model:&str,scope:&[String],assignments:&[(String,u64,Vec<String>)])->Result<Batch,Error>{
  self.apply_batch_details(model,scope,assignments,&[],&[])
 }
 /// Optional model columns and shared-note evidenced relations commit with the
 /// membership batch. Existing manual columns/relations are never overwritten.
 pub fn apply_batch_details(&mut self,model:&str,scope:&[String],assignments:&[(String,u64,Vec<String>)],columns:&[(String,String)],relations:&[(String,String,String,String)])->Result<Batch,Error>{
  self.apply_batch_in_round(None,model,scope,assignments,columns,relations)
 }
 fn apply_batch_in_round(&mut self,round_id:Option<&str>,model:&str,scope:&[String],assignments:&[(String,u64,Vec<String>)],columns:&[(String,String)],relations:&[(String,String,String,String)])->Result<Batch,Error>{
  if model.is_empty()||model.len()>200{return Err(Error::InvalidChat);}
  if scope.len()>MAX_SCOPE||assignments.len()>MAX_BATCH_ASSIGN{return Err(Error::InvalidChat);}
  let scope_set:std::collections::BTreeSet<&String>=scope.iter().collect();
  let mut seen_notes=std::collections::BTreeSet::new();
  let mut cleaned:Vec<(String,u64,Vec<String>)>=Vec::new();
  for (note,hash,topics) in assignments{
   if !scope_set.contains(note)||!seen_notes.insert(note.clone()){return Err(Error::InvalidChat);}
   let mut names:Vec<String>=Vec::new();
   for name in topics{
    let name=name.trim();
    if !valid_name(name){return Err(Error::InvalidChat);}
    if !names.iter().any(|existing:&String|existing.as_str()==name){names.push(name.to_string());}
   }
   cleaned.push((note.clone(),*hash,names));
  }
  let total_topics:usize=cleaned.iter().map(|(_,_,topics)|topics.len()).sum();
  if total_topics>MAX_BATCH_TOPICS{return Err(Error::InvalidChat);}
  if columns.len()>MAX_BATCH_TOPICS||relations.len()>MAX_BATCH_ASSIGN{return Err(Error::InvalidChat);}
  for (name,column) in columns{
   if !valid_name(name)||!valid_column(column)||!cleaned.iter().any(|(_,_,names)|names.contains(name)){return Err(Error::InvalidChat);}
  }
  for (from,to,reason,evidence) in relations{
   if from==to||reason.trim().is_empty()||reason.len()>200||!cleaned.iter().any(|(note,_,names)|note==evidence&&names.contains(from)&&names.contains(to)){return Err(Error::InvalidChat);}
  }
  let id=crate::notes::generate_id();
  let created_at=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d|d.as_secs() as i64).unwrap_or(0);
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  if let Some(round)=round_id{rounds::advance(&tx,round,scope,&id)?;}
  tx.execute("INSERT INTO batches(id,model,scope,status,created_at) VALUES(?1,?2,?3,'running',?4)",params![id,model,serde_json::to_string(scope).map_err(|_|Error::InvalidChat)?,created_at])?;
  let mut changeset:Vec<serde_json::Value>=Vec::new();
  for (note,hash,topics) in &cleaned{
   for name in topics{
    let existing:Option<(String,String,String,i64)>=tx.query_row("SELECT id,column_name,created_by,locked FROM topics WHERE name=?1",[name],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
    let (topic_id,_created)=match existing{
     Some((_,_,_,1))=>continue,
     Some((topic_id,_,_,_))=>(topic_id,false),
     None=>{
      let topic_id=crate::notes::generate_id();
      let column=columns.iter().find(|(topic,_)|topic==name).map(|(_,column)|column.trim()).unwrap_or("");
      tx.execute("INSERT INTO topics(id,name,column_name,created_by,batch_id,created_at) VALUES(?1,?2,?3,'ai',?4,?5)",params![topic_id,name,column,id,created_at])?;
      changeset.push(serde_json::json!({"op":"create_topic","id":topic_id,"name":name}));
      (topic_id,true)
     }
    };
    let already:Option<i64>=tx.query_row("SELECT 1 FROM memberships WHERE note_id=?1 AND topic_id=?2",params![note,topic_id],|r|r.get(0)).optional()?;
    if already.is_some(){continue;}
    tx.execute("INSERT INTO memberships(note_id,topic_id,added_by,batch_id,note_hash,created_at) VALUES(?1,?2,'ai',?3,?4,?5)",params![note,topic_id,id,i64::from_ne_bytes(hash.to_ne_bytes()),created_at])?;
    changeset.push(serde_json::json!({"op":"assign","note":note,"topic":topic_id,"name":name}));
   }
  }
  for (from,to,reason,evidence) in relations{
   let pair:Option<(String,String)>=tx.query_row("SELECT a.id,b.id FROM topics a,topics b WHERE a.name=?1 AND b.name=?2 AND a.locked=0 AND b.locked=0 AND EXISTS(SELECT 1 FROM memberships WHERE note_id=?3 AND topic_id=a.id) AND EXISTS(SELECT 1 FROM memberships WHERE note_id=?3 AND topic_id=b.id)",params![from,to,evidence],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
   if let Some((from,to))=pair{
    let relation=crate::notes::generate_id();
    let hash=cleaned.iter().find(|(note,_,_)|note==evidence).ok_or(Error::InvalidChat)?.1;
    let inserted=tx.execute("INSERT OR IGNORE INTO relations(id,from_topic,to_topic,reason,evidence,added_by,batch_id,created_at,note_hash) VALUES(?1,?2,?3,?4,?5,'ai',?6,?7,?8)",params![relation,from,to,reason.trim(),evidence,id,created_at,i64::from_ne_bytes(hash.to_ne_bytes())])?;
    if inserted>0{changeset.push(serde_json::json!({"op":"create_relation","id":relation,"evidence":evidence}));}
   }
  }
  tx.execute("UPDATE batches SET status='applied',changeset=?1,finished_at=?2 WHERE id=?3",params![serde_json::to_string(&changeset).map_err(|_|Error::InvalidChat)?,created_at,id])?;
  tx.commit()?;
  Ok(Batch{id,model:model.to_string(),status:std::cell::Cell::new("applied"),created_at,finished_at:std::cell::Cell::new(Some(created_at))})
 }

 /// Undoes exactly one AI batch: removes its memberships and its empty
 /// AI-created topics. Manual rows and topics are never touched.
 /// Renames a topic; identity (id), column and memberships survive.
 pub fn rename_topic(&mut self,topic_id:&str,name:&str)->Result<(),Error>{
  if !crate::notes::valid_id(topic_id){return Err(Error::InvalidChat);}
  if !valid_name(name){return Err(Error::InvalidChat);}
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  let exists:Option<String>=tx.query_row("SELECT id FROM topics WHERE name=?1",[name],|r|r.get(0)).optional()?;
  if exists.is_some(){return Err(Error::InvalidChat);}
  tx.execute("UPDATE topics SET name=?1,created_by='manual',batch_id=NULL WHERE id=?2",[name,topic_id])?;
  tx.commit()?;Ok(())
 }
 /// Moves a topic to another column (重新归类).
 pub fn set_topic_column(&mut self,topic_id:&str,column:&str)->Result<(),Error>{
  if !crate::notes::valid_id(topic_id){return Err(Error::InvalidChat);}
  if !valid_column(column){return Err(Error::InvalidChat);}
  self.connection.execute("UPDATE topics SET column_name=?1,created_by='manual',batch_id=NULL WHERE id=?2",params![column,topic_id])?;
  Ok(())
 }
 /// Merges source into target: memberships move (added_by preserved), the
 /// source topic is removed unless the target is locked.
 pub fn merge_topics(&mut self,source:&str,target:&str)->Result<(),Error>{
  if !crate::notes::valid_id(source)||!crate::notes::valid_id(target)||source==target{return Err(Error::InvalidChat);}
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  let locked:i64=tx.query_row("SELECT locked FROM topics WHERE id=?1",[target],|r|r.get(0)).optional()?.ok_or(Error::InvalidChat)?;
  if locked==1{return Err(Error::InvalidChat);}
  let moved:Vec<(String,String,i64)>={
   let mut statement=tx.prepare("SELECT note_id,added_by,note_hash FROM memberships WHERE topic_id=?1")?;
   let rows=statement.query_map([source],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
   rows.collect::<Result<Vec<_>,_>>()?
  };
  for (note_id,added_by,note_hash) in moved{
   tx.execute("INSERT OR IGNORE INTO memberships(note_id,topic_id,added_by,batch_id,note_hash,created_at) VALUES(?1,?2,?3,NULL,?4,?5)",params![note_id,target,added_by,note_hash,created_now()])?;
  }
  tx.execute("UPDATE memberships SET added_by='manual',batch_id=NULL WHERE topic_id=?1",[target])?;
  tx.execute("UPDATE topics SET created_by='manual',batch_id=NULL WHERE id=?1",[target])?;
  tx.execute("DELETE FROM memberships WHERE topic_id=?1",[source])?;
  // Relations touching the merged-away topic would dangle; drop them.
  tx.execute("DELETE FROM relations WHERE from_topic=?1 OR to_topic=?1",[source])?;
  tx.execute("DELETE FROM topics WHERE id=?1",[source])?;
  tx.commit()?;Ok(())
 }
 /// Splits a note out of one topic into another (created on demand, manual).
 pub fn split_membership(&mut self,note_id:&str,from:&str,to_name:&str,column:&str)->Result<String,Error>{
  if !crate::notes::valid_id(note_id)||!crate::notes::valid_id(from)||!valid_name(to_name)||!valid_column(column){return Err(Error::InvalidChat);}
  let hash:i64=self.connection.query_row("SELECT note_hash FROM memberships WHERE note_id=?1 AND topic_id=?2",params![note_id,from],|r|r.get(0)).optional()?.ok_or(Error::InvalidChat)?;
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  let mut to_id=tx.query_row("SELECT id FROM topics WHERE name=?1",[to_name],|r|r.get(0)).optional()?;
  if to_id.is_none(){
   to_id=Some(crate::notes::generate_id());
   tx.execute("INSERT INTO topics(id,name,column_name,created_by,created_at) VALUES(?1,?2,?3,'manual',?4)",params![to_id,to_name,column,created_now()])?;
  }
  let to_id=to_id.unwrap();
  tx.execute("DELETE FROM memberships WHERE note_id=?1 AND topic_id=?2",params![note_id,from])?;
  tx.execute("INSERT OR IGNORE INTO memberships(note_id,topic_id,added_by,batch_id,note_hash,created_at) VALUES(?1,?2,'manual',NULL,?3,?4) ON CONFLICT(note_id,topic_id) DO UPDATE SET added_by='manual',batch_id=NULL",params![note_id,to_id,hash,created_now()])?;
  tx.execute("UPDATE topics SET created_by='manual',batch_id=NULL WHERE id=?1",[&to_id])?;
  cleanup_relations(&tx)?;
  tx.commit()?;Ok(to_id)
 }
 /// Deletes a topic with its memberships and touching relations. Note bodies
 /// are never touched (the note is the evidence and survives).
 pub fn delete_topic(&mut self,topic_id:&str)->Result<(),Error>{
  if !crate::notes::valid_id(topic_id){return Err(Error::InvalidChat);}
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  tx.execute("DELETE FROM memberships WHERE topic_id=?1",[topic_id])?;
  tx.execute("DELETE FROM relations WHERE from_topic=?1 OR to_topic=?1",[topic_id])?;
  tx.execute("DELETE FROM topics WHERE id=?1",[topic_id])?;
  tx.commit()?;Ok(())
 }
 pub fn set_topic_locked(&mut self,topic_id:&str,locked:bool)->Result<(),Error>{
  if !crate::notes::valid_id(topic_id){return Err(Error::InvalidChat);}
  self.connection.execute("UPDATE topics SET locked=?1,created_by='manual',batch_id=NULL WHERE id=?2",params![i64::from(locked),topic_id])?;
  Ok(())
 }
 /// Adds a topic-to-topic relation. Both topics must exist and the evidence
 /// note must actually belong to both — the note is the evidence.
 pub fn add_relation(&mut self,from_topic:&str,to_topic:&str,reason:&str,evidence_note:&str)->Result<String,Error>{
  if !crate::notes::valid_id(from_topic)||!crate::notes::valid_id(to_topic)||from_topic==to_topic{return Err(Error::InvalidChat);}
  if reason.trim().is_empty()||reason.len()>200{return Err(Error::InvalidChat);}
  let from_member:i64=self.connection.query_row("SELECT count(*) FROM memberships WHERE topic_id=?1 AND note_id=?2",params![from_topic,evidence_note],|r|r.get(0))?;
  let to_member:i64=self.connection.query_row("SELECT count(*) FROM memberships WHERE topic_id=?1 AND note_id=?2",params![to_topic,evidence_note],|r|r.get(0))?;
  if from_member==0||to_member==0{return Err(Error::InvalidChat);}
  let id=crate::notes::generate_id();
  let id=self.connection.query_row("INSERT INTO relations(id,from_topic,to_topic,reason,evidence,added_by,created_at) VALUES(?1,?2,?3,?4,?5,'manual',?6) ON CONFLICT(from_topic,to_topic) DO UPDATE SET reason=excluded.reason,evidence=excluded.evidence,added_by='manual',batch_id=NULL RETURNING id",params![id,from_topic,to_topic,reason.trim(),evidence_note,created_now()],|r|r.get::<_,String>(0))?;
  Ok(id)
 }
 pub fn remove_relation(&mut self,relation_id:&str)->Result<(),Error>{
  self.connection.execute("DELETE FROM relations WHERE id=?1",[relation_id])?;
  Ok(())
 }
 pub fn list_relations(&self)->Result<Vec<Relation>,Error>{
  let mut statement=self.connection.prepare("SELECT id,from_topic,to_topic,reason,evidence,added_by FROM relations ORDER BY created_at")?;
  let rows=statement.query_map([],|r|Ok(Relation{id:r.get(0)?,from_topic:r.get(1)?,to_topic:r.get(2)?,reason:r.get(3)?,evidence:r.get(4)?,added_by:r.get(5)?}))?;
  rows.collect::<Result<Vec<_>,_>>().map_err(Into::into)
 }
 pub fn auto_organize(&self)->Result<bool,Error>{
  let value:String=self.connection.query_row("SELECT value FROM meta WHERE key='auto_organize'",[],|r|r.get(0)).optional()?.unwrap_or_else(||"off".into());
  Ok(value=="on")
 }
 pub fn auto_organize_for(&self,profile:&str)->Result<bool,Error>{
  let approved:Option<String>=self.connection.query_row("SELECT value FROM meta WHERE key='auto_organize_profile'",[],|r|r.get(0)).optional()?;
  Ok(self.auto_organize()?&&approved.as_deref()==Some(profile))
 }
 pub fn set_auto_organize_for(&mut self,on:bool,profile:&str)->Result<(),Error>{
  if profile.is_empty()||profile.len()>8192{return Err(Error::InvalidProvider);}
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  tx.execute("INSERT INTO meta(key,value) VALUES('auto_organize_profile',?1) ON CONFLICT(key) DO UPDATE SET value=?1",[profile])?;
  tx.execute("INSERT INTO meta(key,value) VALUES('auto_organize',?1) ON CONFLICT(key) DO UPDATE SET value=?1",[if on{"on"}else{"off"}])?;
  tx.commit()?;Ok(())
 }
 pub fn set_auto_organize(&self,on:bool)->Result<(),Error>{
  self.connection.execute("INSERT INTO meta(key,value) VALUES('auto_organize',?1) ON CONFLICT(key) DO UPDATE SET value=?1",[if on {"on"}else{"off"}])?;
  Ok(())
 }
 pub fn undo_batch(&mut self,batch_id:&str)->Result<(),Error>{
  self.undo_batches(&[batch_id.to_string()])
 }

 /// Undo a chronological group newest first in one transaction. Validation or
 /// SQL failure in any member rolls back all members, including audit status.
 pub fn undo_batches(&mut self,batch_ids:&[String])->Result<(),Error>{
  let mut seen=std::collections::HashSet::new();
  if batch_ids.is_empty()||batch_ids.iter().any(|id|!crate::notes::valid_id(id)||!seen.insert(id)){
   return Err(Error::InvalidChat);
  }
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  for batch_id in batch_ids.iter().rev(){undo_batch_in(&tx,batch_id)?;}
  tx.commit()?;Ok(())
 }

 /// Invalidate only AI derivations on edit; deletion removes all references
 /// to the vanished evidence, while batch audit records remain intact.
 pub fn invalidate_note(&mut self,note:&str,current_hash:Option<u64>)->Result<(),Error>{
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  match current_hash{
   Some(hash)=>{
    tx.execute("UPDATE memberships SET added_by='manual',batch_id=NULL WHERE note_id=?1 AND EXISTS(SELECT 1 FROM relations WHERE added_by='manual' AND evidence=memberships.note_id AND (from_topic=memberships.topic_id OR to_topic=memberships.topic_id))",[note])?;
    tx.execute("DELETE FROM relations WHERE evidence=?1 AND added_by='ai' AND (note_hash IS NULL OR note_hash!=?2)",params![note,i64::from_ne_bytes(hash.to_ne_bytes())])?;
    tx.execute("DELETE FROM memberships WHERE note_id=?1 AND added_by='ai' AND note_hash!=?2",params![note,i64::from_ne_bytes(hash.to_ne_bytes())])?;
   },
   None=>{tx.execute("DELETE FROM relations WHERE evidence=?1",[note])?;tx.execute("DELETE FROM memberships WHERE note_id=?1",[note])?;prune_empty(&tx)?;},
  }
  cleanup_relations(&tx)?;
  tx.commit()?;Ok(())
 }

 /// Remove historical empty topics on refresh, without touching batch audit rows.
 pub fn prune_empty_topics(&mut self)->Result<(),Error>{let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;prune_empty(&tx)?;cleanup_relations(&tx)?;tx.commit()?;Ok(())}

 /// Bounded names-only context for consistent future organizing, without note bodies.
 pub fn topic_catalog(&self,limit:usize)->Result<Vec<Topic>,Error>{
  if limit==0||limit>128{return Err(Error::InvalidChat);}
  let mut q=self.connection.prepare("SELECT id,name,column_name,created_by,batch_id,created_at,locked FROM topics WHERE locked=0 ORDER BY (SELECT count(*) FROM memberships WHERE topic_id=topics.id) DESC,column_name,name LIMIT ?1")?;
  let rows=q.query_map([limit as i64],|r|Ok(Topic{id:r.get(0)?,name:r.get(1)?,column:r.get(2)?,created_by:r.get(3)?,batch_id:r.get(4)?,created_at:r.get(5)?,locked:r.get::<_,i64>(6)?!=0}))?;Ok(rows.collect::<Result<Vec<_>,_>>()?)
 }
 pub fn list_topics(&self)->Result<Vec<TopicAssignments>,Error>{
  let mut statement=self.connection.prepare("SELECT id,name,column_name,created_by,batch_id,created_at,locked FROM topics ORDER BY column_name,name")?;
  let rows=statement.query_map([],|r|Ok(Topic{id:r.get(0)?,name:r.get(1)?,column:r.get(2)?,created_by:r.get(3)?,batch_id:r.get(4)?,created_at:r.get(5)?,locked:r.get::<_,i64>(6)?==1}))?;
  let mut topics=Vec::new();
  for row in rows{
   let topic=row?;
   let count:i64=self.connection.query_row("SELECT count(*) FROM memberships WHERE topic_id=?1",[&topic.id],|r|r.get(0))?;
   let note_ids=self.note_ids(&topic.id)?;
   topics.push(TopicAssignments{topic,note_ids,count});
  }
  Ok(topics)
 }

 pub fn note_ids(&self,topic_id:&str)->Result<Vec<String>,Error>{
  let mut statement=self.connection.prepare("SELECT note_id FROM memberships WHERE topic_id=?1 ORDER BY created_at,note_id")?;
  let rows=statement.query_map([topic_id],|r|r.get::<_,String>(0))?;
  rows.collect::<Result<Vec<_>,_>>().map_err(Into::into)
 }

 /// AI assignments whose stored note hash no longer matches the note body are
 /// stale (the note was edited after organizing) and must be recomputed.
 /// Test/diagnostic escape hatch for direct row seeding. Not used by the app.
 #[doc(hidden)]
 pub fn raw_execute(&mut self,sql:&str,p:impl rusqlite::Params)->Result<usize,Error>{
  Ok(self.connection.execute(sql,p)?)
 }

 pub fn stale_assignments(&self,current:&[(String,u64)])->Result<Vec<String>,Error>{
  let mut stale=Vec::new();
  for (note,hash) in current{
   let hash_bytes=i64::from_ne_bytes(hash.to_ne_bytes());
   let rows:i64=self.connection.query_row("SELECT count(*) FROM memberships WHERE note_id=?1 AND added_by='ai' AND note_hash!=?2",params![note.as_str(),hash_bytes],|r|r.get(0))?;
   if rows>0{stale.push(note.clone());}
  }
  Ok(stale)
 }
}

fn created_now()->i64{std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d|d.as_secs() as i64).unwrap_or(0)}

fn prune_empty(connection:&Connection)->Result<(),Error>{connection.execute("DELETE FROM topics WHERE NOT EXISTS(SELECT 1 FROM memberships WHERE topic_id=topics.id)",[])?;Ok(())}

fn cleanup_relations(connection:&Connection)->Result<(),Error>{
 connection.execute("DELETE FROM relations WHERE NOT EXISTS(SELECT 1 FROM topics WHERE id=from_topic) OR NOT EXISTS(SELECT 1 FROM topics WHERE id=to_topic) OR NOT EXISTS(SELECT 1 FROM memberships WHERE note_id=evidence AND topic_id=from_topic) OR NOT EXISTS(SELECT 1 FROM memberships WHERE note_id=evidence AND topic_id=to_topic)",[])?;
 Ok(())
}

fn undo_batch_in(tx:&rusqlite::Transaction<'_>,batch_id:&str)->Result<(),Error>{
  let status:String=tx.query_row("SELECT status FROM batches WHERE id=?1",[batch_id],|r|r.get(0)).optional()?.ok_or(Error::InvalidChat)?;
  if status!="applied"{return Err(Error::InvalidChat);}
  tx.execute("DELETE FROM relations WHERE batch_id=?1",[batch_id])?;
  tx.execute("UPDATE memberships SET added_by='manual',batch_id=NULL WHERE batch_id=?1 AND EXISTS(SELECT 1 FROM relations WHERE added_by='manual' AND evidence=memberships.note_id AND (from_topic=memberships.topic_id OR to_topic=memberships.topic_id))",[batch_id])?;
  tx.execute("DELETE FROM memberships WHERE batch_id=?1",[batch_id])?;
  let ai_topics:String=tx.query_row("SELECT coalesce(json_group_array(id),'[]') FROM topics WHERE created_by='ai' AND batch_id=?1",[batch_id],|r|r.get::<_,String>(0)).optional()?.unwrap_or_else(||"[]".to_string());
  let ids:Vec<String>=serde_json::from_str(&ai_topics).map_err(|_|Error::InvalidChat)?;
  for topic_id in ids{
   let used:i64=tx.query_row("SELECT count(*) FROM memberships WHERE topic_id=?1",[&topic_id],|r|r.get(0))?;
   if used==0{tx.execute("DELETE FROM topics WHERE id=?1",[&topic_id])?;}
  }
  cleanup_relations(&tx)?;
  tx.execute("UPDATE batches SET status='undone',finished_at=?1 WHERE id=?2",params![created_now(),batch_id])?;
  Ok(())
}
