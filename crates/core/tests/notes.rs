use std::{path::PathBuf,sync::atomic::{AtomicU64,Ordering}};
use zwen_core::notes::{NoteStore,MAX_NOTE_ATTACHMENTS,MAX_NOTE_BYTES,NoteMeta};
use zwen_core::Error;

static NEXT:AtomicU64=AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch{
 fn new()->Self{let path=std::env::temp_dir().join(format!("zwen-notes-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));std::fs::create_dir(&path).unwrap();Self(path)}
}
impl Drop for Scratch{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}

fn meta_from(list:&[NoteMeta],index:usize)->NoteMeta{list[list.len()-1-index].clone()}

#[test] fn create_read_roundtrip_keeps_chinese_and_metadata(){
 let scratch=Scratch::new();let mut store=NoteStore::open(&scratch.0).unwrap();
 let meta=store.create("正则化的直觉","## 问题\n\n什么是 L1 与 L2？\n\n## 回答\n\nL1 让参数变零。","chat","glm-4.7-flash",&[]).unwrap();
 assert_eq!(meta.title,"正则化的直觉");
 assert_eq!(meta.attachments,Vec::<String>::new());
 assert!(String::from_utf8(std::fs::read(scratch.0.join("notes").join(format!("{}.md",meta.id))).unwrap()).unwrap().contains("L1 让参数"));
 let note=store.read(&meta.id).unwrap();
 assert_eq!(note.body,"## 问题\n\n什么是 L1 与 L2？\n\n## 回答\n\nL1 让参数变零。");
 assert_eq!(note.meta.source,"chat");
 assert_eq!(note.meta.model,"glm-4.7-flash");
 assert!(note.meta.created_at>0);
}

#[test] fn search_matches_title_and_body_substrings_and_escapes_wildcards(){
 let scratch=Scratch::new();let mut store=NoteStore::open(&scratch.0).unwrap();
 store.create("正则化笔记","正文包含 中文子串 与 100% 数据。","chat","", &[]).unwrap();
 store.create("另一篇","完全无关的内容","chat","",&[]).unwrap();
 let by_title=store.list("正则化").unwrap();
 assert_eq!(by_title.len(),1);
 let by_body=store.list("中文子串").unwrap();
 assert_eq!(by_body.len(),1);
 // LIKE wildcards in the query are literals.
 assert_eq!(store.list("00%").unwrap().len(),1);
 assert_eq!(store.list("不存在xyz").unwrap().len(),0);
 assert_eq!(store.list("").unwrap().len(),2);
}

#[test] fn attachments_are_copied_sanitized_and_referenced(){
 let scratch=Scratch::new();let mut store=NoteStore::open(&scratch.0).unwrap();
 let bytes=vec![137u8,80,78,71,1,2,3];
 let meta=store.create("带图笔记","看图说话","chat","",&[("截图 2026.png".to_string(),bytes.clone()),("../../escape.png".to_string(),bytes.clone())]).unwrap();
 assert_eq!(meta.attachments.len(),2);
 for name in &meta.attachments{
  assert!(name.starts_with(&format!("{}-",meta.id)));
  let path=scratch.0.join("notes").join("attachments").join(name);
  assert_eq!(std::fs::read(&path).unwrap(),bytes);
  assert!(!name.contains("..")&&!name.contains('/'));
 }
 let note=store.read(&meta.id).unwrap();
 assert_eq!(note.meta.attachments,meta.attachments);
}

#[test] fn delete_removes_row_files_and_survives_missing_files(){
 let scratch=Scratch::new();let mut store=NoteStore::open(&scratch.0).unwrap();
 let meta=store.create("待删除","内容","chat","",&[("a.png".to_string(),b"png".to_vec())]).unwrap();
 assert!(store.delete(&meta.id).is_ok());
 assert!(!scratch.0.join("notes").join(format!("{}.md",meta.id)).exists());
 assert!(!scratch.0.join("notes").join("attachments").join(format!("{}-a.png",meta.id)).exists());
 assert!(matches!(store.read(&meta.id),Err(Error::NoteMissing)));
 assert!(matches!(store.delete(&meta.id),Err(Error::NoteMissing)));
 // A row whose file vanished still deletes cleanly (authority already gone).
 let meta2=store.create("文件丢失","内容","chat","",&[]).unwrap();
 std::fs::remove_file(scratch.0.join("notes").join(format!("{}.md",meta2.id))).unwrap();
 assert!(store.delete(&meta2.id).is_ok());
}

#[test] fn rebuild_adopts_orphans_and_drops_missing_rows(){
 let scratch=Scratch::new();let mut store=NoteStore::open(&scratch.0).unwrap();
 let kept=store.create("在册笔记","保留内容","chat","model-a",&[]).unwrap();
 let gone=store.create("将被删除文件","内容","chat","",&[]).unwrap();
 std::fs::remove_file(scratch.0.join("notes").join(format!("{}.md",gone.id))).unwrap();
 let orphan_id="abcdef0123456789abcdef0123456789";
 let orphan=format!("---\nid: {orphan_id}\ncreated: 1700000000\nsource: chat\nmodel: model-b\ntitle: 孤儿笔记\nattachments: []\n---\n孤儿正文");
 std::fs::write(scratch.0.join("notes").join(format!("{orphan_id}.md")),orphan).unwrap();
 let adopted=store.rebuild_index().unwrap();
 assert_eq!(adopted,2);
 let list=store.list("").unwrap();
 assert_eq!(list.len(),2);
 let titles:Vec<_>=list.iter().map(|m|m.title.as_str()).collect();
 assert!(titles.contains(&"孤儿笔记")&&titles.contains(&"在册笔记"));
 assert!(titles.iter().all(|t|*t!="将被删除文件"));
 let orphan_note=store.read(orphan_id).unwrap();
 assert_eq!(orphan_note.meta.model,"model-b");
 assert_eq!(orphan_note.body,"孤儿正文");
 assert!(list.iter().any(|m|m.id==kept.id));
}

#[test] fn limits_reject_without_writing(){
 let scratch=Scratch::new();let mut store=NoteStore::open(&scratch.0).unwrap();
 assert!(matches!(store.create("t",&"x".repeat(MAX_NOTE_BYTES+1),"chat","",&[]),Err(Error::InvalidNote)));
 assert!(matches!(store.create("t","b","chat","",&(0..=MAX_NOTE_ATTACHMENTS).map(|i|("a.png".to_string(),vec![0u8])).collect::<Vec<_>>()),Err(Error::InvalidNote)));
 assert!(matches!(store.create(&"t".repeat(300),"b","chat","",&[]),Err(Error::InvalidNote)));
 assert!(matches!(store.list(&"q".repeat(300)),Err(Error::InvalidNote)));
 assert!(matches!(store.read("short"),Err(Error::NoteMissing)));
 assert!(matches!(store.read("../escape"),Err(Error::NoteMissing)));
 let list=store.list("").unwrap();
 assert!(list.is_empty());
}

#[test] fn corrupt_index_is_reported_and_files_are_untouched(){
 let scratch=Scratch::new();{let mut store=NoteStore::open(&scratch.0).unwrap();store.create("标题","正文","chat","",&[]).unwrap();}
 let db=scratch.0.join("notes.sqlite3");
 std::fs::write(&db,b"not a sqlite database").unwrap();
 assert!(matches!(NoteStore::open(&scratch.0),Err(Error::UnsupportedSchema))||matches!(NoteStore::open(&scratch.0),Err(Error::Database(_))));
 let md=scratch.0.join("notes");
 let count=std::fs::read_dir(md).unwrap().filter(|e|e.as_ref().unwrap().file_name().to_string_lossy().ends_with(".md")).count();
 assert_eq!(count,1,"note files must never be deleted because of index trouble");
}

#[test] fn reopening_preserves_index_and_deduplicates_names(){
 let scratch=Scratch::new();
 {
  let mut store=NoteStore::open(&scratch.0).unwrap();
  let meta=store.create("重复名","正文","chat","",&[("same.png".to_string(),b"1".to_vec())]).unwrap();
  store.create("重复名2","正文2","chat","",&[("same.png".to_string(),b"2".to_vec())]).unwrap();
  let list=store.list("").unwrap();
  assert_eq!(list.len(),2);
  let first=store.read(&meta.id).unwrap();
  assert_eq!(std::fs::read(scratch.0.join("notes").join("attachments").join(&first.meta.attachments[0])).unwrap(),b"1");
  assert!(scratch.0.join("notes").join(format!("{}.md",meta.id)).exists());
 }
 {let store=NoteStore::open(&scratch.0).unwrap();assert_eq!(store.list("").unwrap().len(),2);}
}
