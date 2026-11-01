use std::{path::PathBuf,sync::atomic::{AtomicU64,Ordering}};
use zwen_core::organize::{OrganizeStore,content_hash,MAX_BATCH_TOPICS};
use zwen_core::Error;

static NEXT:AtomicU64=AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch{
 fn new()->Self{let path=std::env::temp_dir().join(format!("zwen-organize-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));std::fs::create_dir(&path).unwrap();Self(path)}
}
impl Drop for Scratch{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}

fn scope(ids:&[&str])->Vec<String>{ids.iter().map(|s|s.to_string()).collect()}

#[test] fn apply_creates_topics_and_memberships_with_changeset(){
 let scratch=Scratch::new();let mut store=OrganizeStore::open(&scratch.0).unwrap();
 let scope=scope(&["a".repeat(32).as_str(),"b".repeat(32).as_str()]);
 let hash=content_hash(b"body");
 let batch=store.apply_batch("model-a",&scope,&[
  ("a".repeat(32),hash,vec!["正则化".into(),"泛化".into()]),
  ("b".repeat(32),hash,vec!["正则化".into()]),
 ]).unwrap();
 assert_eq!(batch.status().unwrap(),"applied");
 let topics=store.list_topics().unwrap();
 assert_eq!(topics.len(),2,"deduped shared topic");
 assert!(topics.iter().any(|t|t.topic.name=="正则化"&&t.count==2));
 assert!(topics.iter().any(|t|t.topic.name=="泛化"&&t.topic.created_by=="ai"));
 let note_ids=store.note_ids(&topics.iter().find(|t|t.topic.name=="正则化").unwrap().topic.id).unwrap();
 assert!(note_ids.contains(&"a".repeat(32))&&note_ids.contains(&"b".repeat(32)));
}

#[test] fn assignments_outside_scope_or_invalid_are_rejected(){
 let scratch=Scratch::new();let mut store=OrganizeStore::open(&scratch.0).unwrap();
 let scope=scope(&["a".repeat(32).as_str()]);
 let hash=content_hash(b"body");
 assert!(matches!(store.apply_batch("m",&scope,&[("b".repeat(32),hash,vec!["t".into()])]),Err(Error::InvalidChat)));
 assert!(matches!(store.apply_batch("m",&scope,&[("a".repeat(32),hash,vec!["  ".into()])]),Err(Error::InvalidChat)));
 assert!(matches!(store.apply_batch("m",&scope,&[("a".repeat(32),hash,vec!["t\nx".into()])]),Err(Error::InvalidChat)));
 let many:Vec<String>=(0..=MAX_BATCH_TOPICS).map(|i|format!("主题{i}")).collect();
 assert!(matches!(store.apply_batch("m",&scope,&[("a".repeat(32),hash,many)]),Err(Error::InvalidChat)));
 assert!(store.list_topics().unwrap().is_empty());
}

#[test] fn undo_removes_only_its_own_ai_rows(){
 let scratch=Scratch::new();let mut store=OrganizeStore::open(&scratch.0).unwrap();
 let note="a".repeat(32);
 let scope=scope(&[&note]);
 let hash=content_hash(b"body");
 let batch=store.apply_batch("m",&scope,&[(note.clone(),hash,vec!["AI主题".into()])]).unwrap();
 store.insert_manual(note.clone(),"手工主题".into(),hash).unwrap();
 store.undo_batch(&batch.id).unwrap();
 let topics=store.list_topics().unwrap();
 assert!(topics.iter().all(|t|t.topic.name!="AI主题"),"empty AI topic must be removed");
 assert!(topics.iter().any(|t|t.topic.name=="手工主题"),"manual topic survives");
 let note_ids=store.note_ids(&topics.iter().find(|t|t.topic.name=="手工主题").unwrap().topic.id).unwrap();
 assert_eq!(note_ids,vec![note.clone()]);
 assert!(matches!(store.undo_batch(&batch.id),Err(Error::InvalidChat)),"double undo rejected");
 assert!(matches!(store.undo_batch("short"),Err(Error::InvalidChat)));
}

#[test] fn stale_assignments_are_detected_by_hash(){
 let scratch=Scratch::new();let mut store=OrganizeStore::open(&scratch.0).unwrap();
 let note="a".repeat(32);
 let scope=scope(&[&note]);
 let old_hash=content_hash(b"old body");
 store.apply_batch("m",&scope,&[(note.clone(),old_hash,vec!["主题".into()])]).unwrap();
 let current=vec![(note.clone(),content_hash(b"new body"))];
 assert_eq!(store.stale_assignments(&current).unwrap(),vec![note.clone()]);
 let unchanged=vec![(note.clone(),old_hash)];
 assert!(store.stale_assignments(&unchanged).unwrap().is_empty());
}

// helper used by the undo test to create a manual topic row
trait ManualSeed{fn insert_manual(&mut self,note:String,name:String,hash:u64)->Result<(),Error>;}
impl ManualSeed for OrganizeStore{
 fn insert_manual(&mut self,note:String,name:String,hash:u64)->Result<(),Error>{
  use rusqlite::params;
  let created=0i64;
  let topic_id=format!("{:032x}",created.wrapping_add(name.len() as i64)+1);
  self.raw_execute("INSERT OR IGNORE INTO topics(id,name,column_name,created_by,created_at) VALUES(?1,?2,'','manual',?3)",params![topic_id,name,created])?;
  self.raw_execute("INSERT OR IGNORE INTO memberships(note_id,topic_id,added_by,note_hash,created_at) VALUES(?1,?2,'manual',?3,?4)",params![note,topic_id,i64::from_ne_bytes(hash.to_ne_bytes()),created])?;
  Ok(())
 }
}
