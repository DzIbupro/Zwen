//! Durable identity and progress only. Runtime ownership and provider snapshots
//! belong to the desktop coordinator; no credentials are stored in this table.
use super::*;

#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct OrganizeRound{
 pub id:String,pub profile:String,pub note_ids:Vec<String>,pub completed:usize,
 pub batch_ids:Vec<String>,pub undone:bool,
}
fn load(connection:&Connection,id:&str)->Result<OrganizeRound,Error>{
 if !crate::notes::valid_id(id){return Err(Error::InvalidChat);}
 let (profile,scope,completed,batches,undone):(String,String,i64,String,bool)=connection.query_row(
  "SELECT profile,scope,completed,batches,undone FROM organize_rounds WHERE id=?1",[id],
  |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional()?.ok_or(Error::InvalidChat)?;
 let completed=usize::try_from(completed).map_err(|_|Error::InvalidChat)?;
 let note_ids:Vec<String>=serde_json::from_str(&scope).map_err(|_|Error::InvalidChat)?;
 let batch_ids:Vec<String>=serde_json::from_str(&batches).map_err(|_|Error::InvalidChat)?;
 if completed>note_ids.len(){return Err(Error::InvalidChat);}
 Ok(OrganizeRound{id:id.into(),profile,note_ids,completed,batch_ids,undone})
}
pub(super) fn advance(tx:&rusqlite::Transaction<'_>,id:&str,scope:&[String],batch:&str)->Result<(),Error>{
 let mut round=load(tx,id)?;
 let end=round.completed.saturating_add(MAX_BATCH_NOTES).min(round.note_ids.len());
 if round.undone||scope.is_empty()||scope!=&round.note_ids[round.completed..end]{return Err(Error::InvalidChat);}
 round.batch_ids.push(batch.into());
 tx.execute("UPDATE organize_rounds SET completed=?1,batches=?2 WHERE id=?3",params![end as i64,serde_json::to_string(&round.batch_ids).map_err(|_|Error::InvalidChat)?,id])?;
 Ok(())
}
impl OrganizeStore{
 pub fn latest_round(&self)->Result<Option<OrganizeRound>,Error>{
  let id:Option<String>=self.connection.query_row("SELECT id FROM organize_rounds ORDER BY rowid DESC LIMIT 1",[],|r|r.get(0)).optional()?;
  id.map(|id|self.round(&id)).transpose()
 }

 /// `profile` must be a non-secret identity of the confirmed provider config.
 pub fn create_round(&mut self,scope:&[String],profile:&str)->Result<OrganizeRound,Error>{
  let mut seen=std::collections::HashSet::new();
  if scope.is_empty()||scope.iter().any(|id|!crate::notes::valid_id(id)||!seen.insert(id))||profile.is_empty()||profile.len()>4096{
   return Err(Error::InvalidChat);
  }
  let id=crate::notes::generate_id();
  self.connection.execute("INSERT INTO organize_rounds(id,profile,scope) VALUES(?1,?2,?3)",params![id,profile,serde_json::to_string(scope).map_err(|_|Error::InvalidChat)?])?;
  self.round(&id)
 }
 pub fn round(&self,id:&str)->Result<OrganizeRound,Error>{load(&self.connection,id)}
 /// Exact next chunk only. Content, audit and durable progress commit together.
 pub fn apply_round_batch(&mut self,round:&str,model:&str,scope:&[String],assignments:&[(String,u64,Vec<String>)],columns:&[(String,String)],relations:&[(String,String,String,String)])->Result<Batch,Error>{
  self.apply_batch_in_round(Some(round),model,scope,assignments,columns,relations)
 }
 pub fn undo_round(&mut self,id:&str)->Result<(),Error>{
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  let round=load(&tx,id)?;
  if round.undone{return Err(Error::InvalidChat);}
  for batch in round.batch_ids.iter().rev(){super::undo_batch_in(&tx,batch)?;}
  tx.execute("UPDATE organize_rounds SET undone=1 WHERE id=?1",[id])?;
  tx.commit()?;Ok(())
 }
}
