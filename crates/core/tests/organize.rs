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

#[test] fn governance_rename_merge_split_lock_and_relations(){
 use zwen_core::organize::OrganizeStore;
 let scratch=Scratch::new();let mut store=OrganizeStore::open(&scratch.0).unwrap();
 let note="a".repeat(32);let note2="b".repeat(32);
 let scope=scope(&[&note,&note2]);
 let hash=content_hash(b"body");
 store.apply_batch("m",&scope,&[
  (note.clone(),hash,vec!["旧主题".into(),"相邻主题".into()]),
  (note2.clone(),hash,vec!["相邻主题".into()]),
 ]).unwrap();
 let topics=store.list_topics().unwrap();
 let old_id=topics.iter().find(|t|t.topic.name=="旧主题").unwrap().topic.id.clone();
 let other_id=topics.iter().find(|t|t.topic.name=="相邻主题").unwrap().topic.id.clone();
 // relation with real two-sided evidence
 let rel=store.add_relation(&old_id,&other_id,"同源于该笔记",&note).unwrap();
 assert!(store.list_relations().unwrap().iter().any(|r|r.id==rel));
 assert!(matches!(store.add_relation(&other_id,&other_id,"自环",&note),Err(Error::InvalidChat)));
 assert!(matches!(store.add_relation(&old_id,&other_id,"证据不在两侧",&note2),Err(Error::InvalidChat)));
 // rename keeps id and memberships
 store.rename_topic(&old_id,"新主题").unwrap();
 assert!(matches!(store.rename_topic(&old_id,"相邻主题"),Err(Error::InvalidChat)));
 // lock target blocks merge; unlock allows it and drops dangling relations
 store.set_topic_locked(&other_id,true).unwrap();
 assert!(matches!(store.merge_topics(&old_id,&other_id),Err(Error::InvalidChat)));
 store.set_topic_locked(&other_id,false).unwrap();
 store.merge_topics(&old_id,&other_id).unwrap();
 assert!(store.list_topics().unwrap().iter().all(|t|t.topic.name!="新主题"));
 assert!(store.list_relations().unwrap().is_empty(),"relations of merged topic must not dangle");
 // split one note into a new manual topic, then relate with two-sided evidence
 store.split_membership(&note2,&other_id,"拆分主题","机器学习").unwrap();
 let split=store.list_topics().unwrap().iter().find(|t|t.topic.name=="拆分主题").unwrap().clone();
 assert_eq!(split.note_ids,vec![note2.clone()]);
 assert_eq!(split.topic.created_by,"manual");
 // re-assign note2 to 相邻主题 so the relation has two-sided evidence
 store.apply_batch("m",&scope,&[(note2.clone(),hash,vec!["相邻主题".into()])]).unwrap();
 let rel2=store.add_relation(&other_id,&split.topic.id,"笔记同源",&note2).unwrap();
 assert!(store.list_relations().unwrap().iter().any(|r|r.id==rel2));
 store.remove_relation(&rel2).unwrap();
 assert!(store.list_relations().unwrap().is_empty());
 // column move + auto toggle
 store.set_topic_column(&split.topic.id,"机器学习").unwrap();
 assert!(store.list_topics().unwrap().iter().find(|t|t.topic.id==split.topic.id).unwrap().topic.column=="机器学习");
 assert!(!store.auto_organize().unwrap());
 store.set_auto_organize(true).unwrap();
 assert!(store.auto_organize().unwrap());
}

#[test] fn locked_topics_reject_ai_memberships_and_manual_governance_survives_undo(){
 let s=Scratch::new();let mut store=OrganizeStore::open(&s.0).unwrap();
 let a="a".repeat(32);let b="b".repeat(32);
 let batch=store.apply_batch("m",&[a.clone()],&[(a.clone(),1,vec!["old".into()])]).unwrap();
 let id=store.list_topics().unwrap()[0].topic.id.clone();
 store.rename_topic(&id,"manual").unwrap();store.set_topic_column(&id,"column").unwrap();store.set_topic_locked(&id,true).unwrap();
 store.apply_batch("m",&[b.clone()],&[(b,1,vec!["manual".into()])]).unwrap();
 assert_eq!(store.note_ids(&id).unwrap(),vec![a]);
 store.undo_batch(&batch.id).unwrap();let topics=store.list_topics().unwrap();
 assert!(topics.iter().any(|t|t.topic.id==id&&t.topic.name=="manual"&&t.topic.column=="column"&&t.topic.locked));
}

#[test] fn auto_consent_is_bound_to_the_provider_and_legacy_flags_fail_closed(){
 let s=Scratch::new();let mut store=OrganizeStore::open(&s.0).unwrap();
 store.set_auto_organize(true).unwrap();assert!(!store.auto_organize_for("profile-a").unwrap());
 store.set_auto_organize_for(true,"profile-a").unwrap();
 assert!(store.auto_organize_for("profile-a").unwrap());assert!(!store.auto_organize_for("profile-b").unwrap());
 store.set_auto_organize_for(false,"profile-a").unwrap();assert!(!store.auto_organize_for("profile-a").unwrap());
}

#[test] fn split_into_existing_membership_preserves_manual_row_on_undo_and_drops_invalid_relation(){
 let s=Scratch::new();let mut store=OrganizeStore::open(&s.0).unwrap();let note="a".repeat(32);
 let batch=store.apply_batch("m",&[note.clone()],&[(note.clone(),1,vec!["A".into(),"B".into()])]).unwrap();
 let topics=store.list_topics().unwrap();let a=topics.iter().find(|t|t.topic.name=="A").unwrap().topic.id.clone();let b=topics.iter().find(|t|t.topic.name=="B").unwrap().topic.id.clone();
 store.add_relation(&a,&b,"evidence",&note).unwrap();store.split_membership(&note,&a,"B","").unwrap();assert!(store.list_relations().unwrap().is_empty());
 store.invalidate_note(&note,Some(2)).unwrap();assert_eq!(store.note_ids(&b).unwrap(),vec![note.clone()]);
 store.undo_batch(&batch.id).unwrap();assert_eq!(store.note_ids(&b).unwrap(),vec![note]);
}
#[test] fn merged_memberships_survive_original_batch_undo(){
 let s=Scratch::new();let mut store=OrganizeStore::open(&s.0).unwrap();let note="a".repeat(32);
 let batch=store.apply_batch("m",&[note.clone()],&[(note.clone(),1,vec!["A".into(),"B".into()])]).unwrap();
 let topics=store.list_topics().unwrap();let a=topics.iter().find(|t|t.topic.name=="A").unwrap().topic.id.clone();let b=topics.iter().find(|t|t.topic.name=="B").unwrap().topic.id.clone();
 store.merge_topics(&a,&b).unwrap();store.undo_batch(&batch.id).unwrap();assert_eq!(store.note_ids(&b).unwrap(),vec![note]);
}

#[test] fn structured_batch_applies_columns_and_evidenced_relations_and_undoes_them(){
 let s=Scratch::new();let mut store=OrganizeStore::open(&s.0).unwrap();let note="a".repeat(32);
 let scope=vec![note.clone()];let assignments=vec![(note.clone(),1,vec!["A".into(),"B".into()])];
 let batch=store.apply_batch_details("m",&scope,&assignments,&[("A".into(),"column".into())],&[("A".into(),"B".into(),"inferred".into(),note.clone())]).unwrap();
 let topics=store.list_topics().unwrap();assert!(topics.iter().any(|t|t.topic.name=="A"&&t.topic.column=="column"));
 let relations=store.list_relations().unwrap();assert_eq!(relations.len(),1);assert_eq!(relations[0].added_by,"ai");assert_eq!(relations[0].evidence,note);
 store.undo_batch(&batch.id).unwrap();assert!(store.list_relations().unwrap().is_empty());
}
#[test] fn structured_batch_rejects_invented_evidence_without_partial_writes(){
 let s=Scratch::new();let mut store=OrganizeStore::open(&s.0).unwrap();let note="a".repeat(32);
 assert!(store.apply_batch_details("m",&[note.clone()],&[(note,1,vec!["A".into(),"B".into()])],&[],&[("A".into(),"B".into(),"reason".into(),"b".repeat(32))]).is_err());
 assert!(store.list_topics().unwrap().is_empty());
}

#[test] fn manually_confirmed_ai_relation_and_support_survive_batch_undo(){
 let s=Scratch::new();let mut store=OrganizeStore::open(&s.0).unwrap();let note="a".repeat(32);
 let batch=store.apply_batch_details("m",&[note.clone()],&[(note.clone(),1,vec!["A".into(),"B".into()])],&[],&[("A".into(),"B".into(),"AI reason".into(),note.clone())]).unwrap();
 let original=store.list_relations().unwrap()[0].clone();
 assert_eq!(store.add_relation(&original.from_topic,&original.to_topic,"manual confirmation",&note).unwrap(),original.id);
 store.undo_batch(&batch.id).unwrap();let relation=store.list_relations().unwrap();assert_eq!(relation.len(),1);assert_eq!(relation[0].added_by,"manual");assert_eq!(relation[0].reason,"manual confirmation");
}

#[test] fn ai_relation_hash_invalidates_even_when_its_memberships_are_manual(){
 let s=Scratch::new();let mut store=OrganizeStore::open(&s.0).unwrap();let note="a".repeat(32);
 store.apply_batch_details("m",&[note.clone()],&[(note.clone(),1,vec!["A".into(),"B".into()])],&[],&[("A".into(),"B".into(),"AI reason".into(),note.clone())]).unwrap();
 store.raw_execute("UPDATE memberships SET added_by='manual',batch_id=NULL",[]).unwrap();
 store.invalidate_note(&note,Some(1)).unwrap();assert_eq!(store.list_relations().unwrap().len(),1);
 store.invalidate_note(&note,Some(2)).unwrap();assert!(store.list_relations().unwrap().is_empty());assert!(store.list_topics().unwrap().iter().all(|t|t.count==1));
}

#[test]fn deleting_evidence_updates_all_counts_removes_empty_topics_and_keeps_undo_valid(){
 let s=Scratch::new();let mut store=OrganizeStore::open(&s.0).unwrap();let a="a".repeat(32);let b="b".repeat(32);
 let batch=store.apply_batch("m",&[a.clone(),b.clone()],&[(a.clone(),1,vec!["共有".into(),"唯一".into()]),(b.clone(),1,vec!["共有".into()])]).unwrap();
 let topics=store.list_topics().unwrap();let shared=topics.iter().find(|t|t.topic.name=="共有").unwrap().topic.id.clone();let unique=topics.iter().find(|t|t.topic.name=="唯一").unwrap().topic.id.clone();
 store.set_topic_locked(&unique,true).unwrap();store.add_relation(&shared,&unique,"共同证据",&a).unwrap();
 store.invalidate_note(&a,None).unwrap();let remaining=store.list_topics().unwrap();assert_eq!(remaining.len(),1);assert_eq!(remaining[0].count,1);assert_eq!(remaining[0].note_ids,vec![b.clone()]);assert!(store.list_relations().unwrap().is_empty());
 store.invalidate_note(&a,None).unwrap();assert_eq!(store.list_topics().unwrap().len(),1);
 store.undo_batch(&batch.id).unwrap();assert!(store.list_topics().unwrap().is_empty());
}

#[test]fn cleanup_failure_rolls_back_membership_and_relation_removal(){
 let s=Scratch::new();let mut store=OrganizeStore::open(&s.0).unwrap();let a="a".repeat(32);
 store.apply_batch("m",&[a.clone()],&[(a.clone(),1,vec!["A".into(),"B".into()])]).unwrap();let topics=store.list_topics().unwrap();
 store.add_relation(&topics[0].topic.id,&topics[1].topic.id,"证据",&a).unwrap();
 store.raw_execute("CREATE TRIGGER fail_cleanup BEFORE DELETE ON topics BEGIN SELECT RAISE(ABORT,'fixture'); END",[]).unwrap();
 assert!(store.invalidate_note(&a,None).is_err());assert_eq!(store.list_topics().unwrap().len(),2);assert_eq!(store.list_topics().unwrap()[0].count,1);assert_eq!(store.list_relations().unwrap().len(),1);
}
#[test]fn refresh_prunes_historical_empty_topics_and_dangling_relations(){
 let s=Scratch::new();let mut store=OrganizeStore::open(&s.0).unwrap();let a="a".repeat(32);
 store.apply_batch("m",&[a.clone()],&[(a.clone(),1,vec!["A".into(),"B".into()])]).unwrap();let topics=store.list_topics().unwrap();store.add_relation(&topics[0].topic.id,&topics[1].topic.id,"证据",&a).unwrap();
 store.raw_execute("DELETE FROM memberships",[]).unwrap();store.prune_empty_topics().unwrap();assert!(store.list_topics().unwrap().is_empty());assert!(store.list_relations().unwrap().is_empty());store.prune_empty_topics().unwrap();
}

#[test]fn markdown_note_deletion_drives_topic_cleanup_without_removing_other_notes(){
 let s=Scratch::new();let mut notes=zwen_core::notes::NoteStore::open(&s.0).unwrap();let a=notes.create("A","正文A","chat","fixture",&[],None).unwrap();let b=notes.create("B","正文B","chat","fixture",&[],None).unwrap();
 let mut store=OrganizeStore::open(&s.0).unwrap();store.apply_batch("m",&[a.id.clone(),b.id.clone()],&[(a.id.clone(),content_hash("正文A".as_bytes()),vec!["共有".into(),"仅A".into()]),(b.id.clone(),content_hash("正文B".as_bytes()),vec!["共有".into()])]).unwrap();
 notes.delete(&a.id).unwrap();assert!(notes.read(&a.id).is_err());assert_eq!(notes.read(&b.id).unwrap().body,"正文B");let topics=store.list_topics().unwrap();assert_eq!(topics.len(),1);assert_eq!(topics[0].count,1);
 notes.delete(&b.id).unwrap();assert!(store.list_topics().unwrap().is_empty());
}

#[test] fn undo_batches_reverses_all_commits_and_keeps_manual_evidence(){
 let s=Scratch::new();let mut store=OrganizeStore::open(&s.0).unwrap();let a="a".repeat(32);let b="b".repeat(32);
 let first=store.apply_batch_details("m",&[a.clone()],&[(a.clone(),1,vec!["A".into(),"B".into()])],&[],&[("A".into(),"B".into(),"AI".into(),a.clone())]).unwrap();
 let relation=store.list_relations().unwrap()[0].clone();store.add_relation(&relation.from_topic,&relation.to_topic,"已手工确认",&a).unwrap();
 store.set_topic_locked(&relation.from_topic,true).unwrap();
 let second=store.apply_batch("m",&[b.clone()],&[(b.clone(),1,vec!["B".into(),"C".into()])]).unwrap();
 store.undo_batches(&[first.id.clone(),second.id.clone()]).unwrap();
 let topics=store.list_topics().unwrap();assert_eq!(topics.len(),2);assert!(topics.iter().all(|t|t.note_ids==vec![a.clone()]));
 assert!(topics.iter().any(|t|t.topic.id==relation.from_topic&&t.topic.locked));
 let relations=store.list_relations().unwrap();assert_eq!(relations.len(),1);assert_eq!(relations[0].added_by,"manual");assert_eq!(relations[0].reason,"已手工确认");
 assert!(store.undo_batch(&first.id).is_err());assert!(store.undo_batch(&second.id).is_err());
}

#[test] fn undo_batches_midway_sql_failure_rolls_back_every_batch_and_status(){
 let s=Scratch::new();let mut store=OrganizeStore::open(&s.0).unwrap();let a="a".repeat(32);let b="b".repeat(32);
 let first=store.apply_batch("m",&[a.clone()],&[(a.clone(),1,vec!["A".into()])]).unwrap();
 let second=store.apply_batch("m",&[b.clone()],&[(b.clone(),1,vec!["B".into()])]).unwrap();
 // Reverse order removes B first; failure on A must restore B as well.
 store.raw_execute("CREATE TRIGGER fail_older BEFORE DELETE ON topics WHEN OLD.name='A' BEGIN SELECT RAISE(ABORT,'fixture'); END",[]).unwrap();
 assert!(store.undo_batches(&[first.id.clone(),second.id.clone()]).is_err());
 let topics=store.list_topics().unwrap();assert_eq!(topics.len(),2);assert!(topics.iter().any(|t|t.note_ids==vec![a.clone()]));assert!(topics.iter().any(|t|t.note_ids==vec![b.clone()]));
 store.raw_execute("DROP TRIGGER fail_older",[]).unwrap();
 drop(store);let mut store=OrganizeStore::open(&s.0).unwrap();
 store.undo_batches(&[first.id,second.id]).unwrap();assert!(store.list_topics().unwrap().is_empty());
}

#[test] fn undo_batches_rejects_empty_duplicate_missing_or_undone_scope_without_partial_changes(){
 let s=Scratch::new();let mut store=OrganizeStore::open(&s.0).unwrap();let a="a".repeat(32);let b="b".repeat(32);
 let first=store.apply_batch("m",&[a.clone()],&[(a.clone(),1,vec!["A".into()])]).unwrap();
 let second=store.apply_batch("m",&[b.clone()],&[(b.clone(),1,vec!["B".into()])]).unwrap();
 assert!(store.undo_batches(&[]).is_err());
 assert!(store.undo_batches(&[first.id.clone(),first.id.clone()]).is_err());
 assert!(store.undo_batches(&["bad".into(),second.id.clone()]).is_err());
 assert!(store.undo_batches(&["0".repeat(32),second.id.clone()]).is_err());
 assert_eq!(store.list_topics().unwrap().len(),2);
 store.undo_batch(&first.id).unwrap();
 assert!(store.undo_batches(&[first.id,second.id.clone()]).is_err());
 assert_eq!(store.list_topics().unwrap().len(),1);store.undo_batch(&second.id).unwrap();
}
