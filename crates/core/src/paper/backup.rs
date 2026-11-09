//! Verified, reusable pre-migration snapshots. Source data is never rewritten here.
use crate::Error;
use rusqlite::{Connection,OpenFlags};
use sha2::{Digest,Sha256};
use std::{fs,path::{Path,PathBuf}};
fn identity(c:&Connection)->Result<String,Error>{
 let mut hash=Sha256::new();hash.update(b"zwen-paper-library-v1");
 for sql in ["SELECT id,name,pages FROM sources ORDER BY id","SELECT conversation,source,options FROM sessions ORDER BY conversation"]{
  hash.update(sql.as_bytes());let mut q=c.prepare(sql)?;let columns=q.column_count();let mut rows=q.query([])?;
  while let Some(row)=rows.next()?{for n in 0..columns{let text:String=row.get(n)?;hash.update((text.len() as u64).to_le_bytes());hash.update(text.as_bytes());}}
 }
 Ok(format!("{:x}",hash.finalize()))
}
fn verify(path:&Path,expected:&str)->Result<(),Error>{
 let c=Connection::open_with_flags(path,OpenFlags::SQLITE_OPEN_READ_ONLY)?;c.set_limit(rusqlite::limits::Limit::SQLITE_LIMIT_LENGTH,(super::MAX_TEXT_BYTES+65536) as i32)?;
 let version:i64=c.pragma_query_value(None,"user_version",|r|r.get(0))?;let check:String=c.query_row("PRAGMA quick_check",[],|r|r.get(0))?;
 if version!=1||check!="ok"||identity(&c)?!=expected{return Err(Error::UnsupportedSchema);}Ok(())
}
pub(super) fn ensure(c:&Connection,dir:&Path)->Result<(),Error>{
 let hash=identity(c)?;let final_path=dir.join(format!("library-before-v2-{hash}.sqlite3"));
 if final_path.exists(){return verify(&final_path,&hash);}
 let temp=final_path.with_extension("tmp");
 // Only this exact staging name is ours; preserve every completed backup.
 if temp.exists(){let m=fs::symlink_metadata(&temp)?;if !m.is_file()||m.file_type().is_symlink(){return Err(Error::UnsupportedSchema);}fs::remove_file(&temp)?;}
 let mut o=fs::OpenOptions::new();o.write(true).create_new(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;o.mode(0o600);}
 drop(o.open(&temp)?);
 struct Staging(PathBuf);impl Drop for Staging{fn drop(&mut self){let _=fs::remove_file(&self.0);}}
 let _staging=Staging(temp.clone());
 c.execute("VACUUM INTO ?1",[temp.to_string_lossy().as_ref()])?;verify(&temp,&hash)?;
 fs::File::open(&temp)?.sync_all()?;fs::rename(&temp,&final_path)?;
 #[cfg(unix)]fs::File::open(dir)?.sync_all()?;
 Ok(())
}
