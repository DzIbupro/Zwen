use std::{path::PathBuf, sync::atomic::{AtomicU64, Ordering}};
use study_assistant_core::{Store, Error};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
 fn new() -> Self { let p=std::env::temp_dir().join(format!("study-core-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));std::fs::create_dir(&p).unwrap();Self(p) }
 fn db(&self) -> PathBuf { self.0.join("state.sqlite3") }
}
impl Drop for Scratch { fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);} }
#[test] fn new_store_is_empty(){let t=Scratch::new();let s=Store::open(&t.db()).unwrap();assert_eq!(s.load().unwrap().text,"");assert_eq!(s.load().unwrap().revision,0);}
#[test] fn unicode_and_sql_are_restored_after_reopening(){
 let t=Scratch::new();let text="中文 🌿\n'; DROP TABLE drafts; --\n$E=mc^2$";
 {let mut s=Store::open(&t.db()).unwrap();let d=s.save(0,text).unwrap();assert_eq!(d.text,text);assert_eq!(d.revision,1);}
 let d=Store::open(&t.db()).unwrap().load().unwrap();assert_eq!(d.text,text);assert_eq!(d.revision,1);
}
#[test] fn stale_writer_does_not_overwrite_saved_text(){
 let t=Scratch::new();let mut a=Store::open(&t.db()).unwrap();let mut b=Store::open(&t.db()).unwrap();
 a.save(0,"first").unwrap();assert!(matches!(b.save(0,"stale"),Err(Error::Conflict)));assert_eq!(a.load().unwrap().text,"first");
 assert_eq!(b.save(1,"second").unwrap().revision,2);
}
#[test] fn oversized_utf8_is_rejected_without_erasing_draft(){
 let t=Scratch::new();let mut s=Store::open(&t.db()).unwrap();s.save(0,"keep").unwrap();
 assert!(matches!(s.save(1,&"字".repeat(90_000)),Err(Error::TooLarge)));assert_eq!(s.load().unwrap().text,"keep");
}
#[test] fn newer_schema_is_not_downgraded(){
 let t=Scratch::new();let c=rusqlite::Connection::open(t.db()).unwrap();c.pragma_update(None,"user_version",99).unwrap();drop(c);
 assert!(matches!(Store::open(&t.db()),Err(Error::UnsupportedSchema)));
 let c=rusqlite::Connection::open(t.db()).unwrap();assert_eq!(c.pragma_query_value(None,"user_version",|r|r.get::<_,i64>(0)).unwrap(),99);
}
#[test] fn corrupt_database_is_not_replaced(){
 let t=Scratch::new();let bytes=b"this is not a SQLite database";std::fs::write(t.db(),bytes).unwrap();
 assert!(Store::open(&t.db()).is_err());assert_eq!(std::fs::read(t.db()).unwrap(),bytes);
}

#[cfg(unix)]
#[test] fn database_is_private_to_the_user(){
 use std::os::unix::fs::PermissionsExt;
 let t=Scratch::new();let mut s=Store::open(&t.db()).unwrap();s.save(0,"private").unwrap();
 assert_eq!(std::fs::metadata(t.db()).unwrap().permissions().mode() & 0o077,0);
}
