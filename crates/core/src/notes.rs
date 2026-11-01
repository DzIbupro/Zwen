//! Local Markdown knowledge base. The .md file is the authority for content and
//! identity (stable id = filename); `notes.sqlite3` is a derived, rebuildable
//! index. Writes are temp-file → rename → index transaction; an orphan file is
//! adopted by rebuild, never deleted on sight.
use crate::{Error,open_private_database};
use rusqlite::{Connection,TransactionBehavior,params};
use serde::Serialize;
use std::{fs,io::Write,path::{Path,PathBuf},sync::atomic::{AtomicU64,Ordering},time::{SystemTime,UNIX_EPOCH}};

pub const MAX_NOTE_BYTES:usize=256*1024;
pub const MAX_TITLE_BYTES:usize=200;
pub const MAX_SOURCE_BYTES:usize=64;
pub const MAX_MODEL_BYTES:usize=200;
pub const MAX_NOTE_ATTACHMENTS:usize=6;
pub const MAX_SEARCH_QUERY:usize=256;
pub const MAX_LIST:usize=200;
const ATTACH_DIR:&str="attachments";
const TEMP_COUNTER:AtomicU64=AtomicU64::new(0);

#[derive(Clone,Debug,PartialEq,Serialize)]
#[serde(rename_all="camelCase")]
pub struct NoteMeta{pub id:String,pub title:String,pub created_at:i64,pub source:String,pub model:String,pub attachments:Vec<String>,#[serde(skip_serializing_if="Option::is_none")] pub request:Option<String>}

/// Chat request ids follow the journal's rules; anything else is dropped so a
/// hand-edited front matter cannot smuggle arbitrary values into links.
pub fn valid_request(value:&str)->bool{!value.is_empty()&&value.len()<=100&&value.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_')}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Note{pub meta:NoteMeta,pub body:String}

pub fn valid_id(id:&str)->bool{id.len()==32&&id.bytes().all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_uppercase())}

pub fn generate_id()->String{
 let nanos=SystemTime::now().duration_since(UNIX_EPOCH).map(|d|d.as_nanos() as u64).unwrap_or(0);
 let counter=TEMP_COUNTER.fetch_add(1,Ordering::Relaxed);
 format!("{:016x}{:016x}",nanos,counter)
}

/// Whitelist filenames: ASCII alphanumerics, CJK, dot, dash, underscore. Anything
/// else collapses to `_`; separators can never survive, so the value is safe to
/// join under the note's attachment directory.
pub fn sanitize_filename(name:&str)->String{
 let mut out=String::new();
 for ch in name.chars(){
  if ch.is_ascii_alphanumeric()||matches!(ch,'.'|'-'|'_')||('\u{4e00}'..='\u{9fff}').contains(&ch){out.push(ch);}
  else if ch.is_ascii_graphic(){out.push('_');}
 }
 let truncated:String=out.chars().take(80).collect();
 let trimmed=truncated.trim_matches(|c|c=='.'||c=='_').to_string();
 if trimmed.is_empty(){"file".to_string()}else{trimmed}
}

fn front_matter(meta:&NoteMeta)->String{
 let list=serde_json::to_string(&meta.attachments).unwrap_or_else(|_|"[]".to_string());
 let request=meta.request.as_deref().map(|value|format!("\nrequest: {value}")).unwrap_or_default();
 format!("---\nid: {}\ncreated: {}\nsource: {}\nmodel: {}\ntitle: {}\nattachments: {}{}\n---\n",meta.id,meta.created_at,meta.source,meta.model,meta.title,list,request)
}

/// Splits the stored file into metadata and body. The body starts after the
/// front matter block; a missing block means the whole file is the body.
fn split_file(id:&str,bytes:&[u8])->Result<(NoteMeta,String),Error>{
 let text=String::from_utf8(bytes.to_vec()).map_err(|_|Error::InvalidNote)?;
 if let Some(rest)=text.strip_prefix("---\n"){
  if let Some(end)=rest.find("\n---\n"){
   let mut meta=NoteMeta{id:id.to_string(),title:"未命名笔记".to_string(),created_at:0,source:String::new(),model:String::new(),attachments:Vec::new(),request:None};
   for line in rest[..end].lines(){
    let Some((key,value))=line.split_once(':')else{continue};
    match key{
     "id" if value.trim()==id=>{},
     "id"=>return Err(Error::InvalidNote),
     "created"|"createdAt"=>meta.created_at=value.trim().parse().unwrap_or(0),
     "source"=>{let v=value.trim();if v.len()<=MAX_SOURCE_BYTES{meta.source=v.to_string();}}
     "model"=>{let v=value.trim();if v.len()<=MAX_MODEL_BYTES{meta.model=v.to_string();}}
     "title"=>{let v=value.trim();if !v.is_empty()&&v.len()<=MAX_TITLE_BYTES{meta.title=v.to_string();}}
     "attachments"=>{if let Ok(list)=serde_json::from_str::<Vec<String>>(value.trim()){meta.attachments=list.into_iter().take(MAX_NOTE_ATTACHMENTS).collect();}}
     "request"=>{let v=value.trim();if valid_request(v){meta.request=Some(v.to_string());}}
     _=>{}
    }
   }
   let body=rest[end+5..].to_string();
   return Ok((meta,body));
  }
 }
 Ok((NoteMeta{id:id.to_string(),title:"未命名笔记".to_string(),created_at:0,source:String::new(),model:String::new(),attachments:Vec::new(),request:None},text))
}

fn read_all(connection:&Connection)->Result<Vec<NoteMeta>,Error>{
 let mut statement=connection.prepare("SELECT CASE WHEN length(id)=32 THEN id END,title,created_at,CASE WHEN length(source)<=64 THEN source END,CASE WHEN length(model)<=200 THEN model END FROM notes ORDER BY created_at DESC,id DESC LIMIT ?1")?;
 let rows=statement.query_map([MAX_LIST as i64],|r|Ok((r.get::<_,Option<String>>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?,r.get::<_,Option<String>>(3)?,r.get::<_,Option<String>>(4)?)))?;
 let mut notes=Vec::new();
 for row in rows{
  let(id,title,created_at,source,model)=row?;
  let id=id.ok_or(Error::InvalidNote)?;
  if !valid_id(&id)||title.is_empty(){return Err(Error::InvalidNote);}
  notes.push(NoteMeta{id,title,created_at,source:source.unwrap_or_default(),model:model.unwrap_or_default(),attachments:Vec::new(),request:None});
 }
 Ok(notes)
}

pub struct NoteStore{connection:Connection,directory:PathBuf}
impl NoteStore{
 pub fn open(data_dir:&Path)->Result<Self,Error>{
  create_dir(data_dir)?;
  let directory=data_dir.join("notes");create_dir(&directory)?;create_dir(&directory.join(ATTACH_DIR))?;
  let connection=open_private_database(&data_dir.join("notes.sqlite3"),|connection|{
   let version:i64=connection.pragma_query_value(None,"user_version",|r|r.get(0))?;
   if version!=0&&version!=1{return Err(Error::UnsupportedSchema);}
   if version==0{
    let tx=connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute_batch("CREATE TABLE notes(
     id TEXT PRIMARY KEY,
     title TEXT NOT NULL,
     created_at INTEGER NOT NULL,
     source TEXT NOT NULL DEFAULT '',
     model TEXT NOT NULL DEFAULT ''
    );PRAGMA user_version = 1;")?;
    tx.commit()?;
   }
   read_all(connection)?;Ok(())
  })?;
  Ok(Self{connection,directory})
 }
 pub fn create(&mut self,title:&str,body:&str,source:&str,model:&str,attachments:&[(String,Vec<u8>)],request:Option<&str>)->Result<NoteMeta,Error>{
  let request=match request{Some(value) if valid_request(value)=>Some(value.to_string()),_=>None};
  let title=title.trim();
  let title=if title.is_empty(){"未命名笔记".to_string()}else if title.len()<=MAX_TITLE_BYTES{title.to_string()}else{return Err(Error::InvalidNote)};
  if body.len()>MAX_NOTE_BYTES||source.len()>MAX_SOURCE_BYTES||model.len()>MAX_MODEL_BYTES{return Err(Error::InvalidNote);}
  if attachments.len()>MAX_NOTE_ATTACHMENTS{return Err(Error::InvalidNote);}
  let mut seen=std::collections::BTreeSet::new();
  let mut files=Vec::new();
  for (name,bytes) in attachments{
   let name=sanitize_filename(name);
   if name.is_empty()||bytes.len() as u64>crate::attachment::MAX_IMAGE_BYTES||!seen.insert(name.clone()){return Err(Error::InvalidNote);}
   files.push((name,bytes.clone()));
  }
  let id=generate_id();
  let created_at=SystemTime::now().duration_since(UNIX_EPOCH).map(|d|d.as_secs() as i64).unwrap_or(0);
  let meta=NoteMeta{id:id.clone(),title,created_at,source:source.to_string(),model:model.to_string(),attachments:files.iter().map(|(name,_)|format!("{id}-{name}")).collect(),request};
  let mut names=Vec::new();
  for (name,bytes) in &files{
   let stored=format!("{id}-{name}");
   write_atomic(&self.directory.join(ATTACH_DIR).join(&stored),bytes)?;
   names.push(stored);
  }
  let mut document=front_matter(&meta).into_bytes();document.extend_from_slice(body.as_bytes());
  if let Err(error)=write_atomic(&self.directory.join(format!("{id}.md")),&document){
   for name in &names{let _=fs::remove_file(self.directory.join(ATTACH_DIR).join(name));}
   return Err(error);
  }
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  if let Err(error)=tx.execute("INSERT INTO notes(id,title,created_at,source,model) VALUES(?1,?2,?3,?4,?5)",params![meta.id,meta.title,meta.created_at,meta.source,meta.model]){
   let _=fs::remove_file(self.directory.join(format!("{id}.md")));
   for name in &names{let _=fs::remove_file(self.directory.join(ATTACH_DIR).join(name));}
   return Err(Error::Database(error));
  }
  tx.commit()?;
  Ok(meta)
 }
 pub fn list(&self,query:&str)->Result<Vec<NoteMeta>,Error>{
  let query=query.trim();
  if query.len()>MAX_SEARCH_QUERY{return Err(Error::InvalidNote);}
  if query.is_empty(){return read_all(&self.connection);}
  let escaped=query.replace('\\',"\\\\").replace('%',"\\%").replace('_',"\\_");
  let pattern=format!("%{escaped}%");
  let mut ids=Vec::new();
  {
   let mut statement=self.connection.prepare("SELECT id FROM notes WHERE title LIKE ?1 ESCAPE '\\' ORDER BY created_at DESC,id DESC LIMIT ?2")?;
   let rows=statement.query_map(params![pattern,MAX_LIST as i64],|r|r.get::<_,String>(0))?;
   for row in rows{ids.push(row?);}
  }
  if ids.len()<MAX_LIST{
   let mut statement=self.connection.prepare("SELECT id FROM notes WHERE title NOT LIKE ?1 ESCAPE '\\' ORDER BY created_at DESC,id DESC")?;
   let rows=statement.query_map(params![pattern],|r|r.get::<_,String>(0))?;
   for row in rows{
    if ids.len()>=MAX_LIST{break;}
    let id=row?;
    let bytes=fs::read(self.directory.join(format!("{id}.md"))).unwrap_or_default();
    if bytes.len()<=MAX_NOTE_BYTES{
     let text=String::from_utf8_lossy(&bytes);
     if text.contains(query){ids.push(id);}
    }
   }
  }
  let mut notes=Vec::new();
  for id in ids{
   let (meta,_)={let bytes=fs::read(self.directory.join(format!("{id}.md"))).map_err(|_|Error::NoteMissing)?;split_file(&id,&bytes)?};
   notes.push(meta);
  }
  Ok(notes)
 }
 pub fn read(&self,id:&str)->Result<Note,Error>{
  if !valid_id(id){return Err(Error::NoteMissing);}
  let (meta,body)=self.load_file(id)?;
  Ok(Note{meta,body})
 }
 pub fn delete(&mut self,id:&str)->Result<(),Error>{
  if !valid_id(id){return Err(Error::NoteMissing);}
  // A vanished file is already gone as far as the authority goes; the row is
  // still removed so no ghost entry stays in the list.
  let attachments=match self.load_file(id){Ok((meta,_))=>meta.attachments,Err(_)=>Vec::new()};
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  let changed=tx.execute("DELETE FROM notes WHERE id=?1",[id])?;
  tx.commit()?;
  if changed==0{return Err(Error::NoteMissing);}
  let _=fs::remove_file(self.directory.join(format!("{id}.md")));
  for name in attachments{let _=fs::remove_file(self.directory.join(ATTACH_DIR).join(name));}
  Ok(())
 }
 /// In-app manual edit: rewrites title and body in place. Identity, creation
 /// time, source, model, request link and attachment files are preserved.
 pub fn update(&mut self,id:&str,title:&str,body:&str)->Result<NoteMeta,Error>{
  if !valid_id(id){return Err(Error::NoteMissing);}
  let title=title.trim();
  if title.is_empty()||title.len()>MAX_TITLE_BYTES||body.len()>MAX_NOTE_BYTES{return Err(Error::InvalidNote);}
  let (mut meta,_)=self.load_file(id)?;
  meta.title=title.to_string();
  let mut document=front_matter(&meta).into_bytes();document.extend_from_slice(body.as_bytes());
  write_atomic(&self.directory.join(format!("{id}.md")),&document)?;
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  tx.execute("UPDATE notes SET title=?1 WHERE id=?2",params![meta.title,id])?;
  tx.commit()?;
  Ok(meta)
 }
 /// Rebuilds the index from the files on disk. Orphan files are adopted (the
 /// .md is the authority); index rows whose file disappeared are dropped.
 pub fn rebuild_index(&mut self)->Result<usize,Error>{
  let mut found=Vec::new();
  for entry in fs::read_dir(&self.directory)?{
   let entry=entry?;
   if !entry.file_type()?.is_file(){continue;}
   let Some(name)=entry.file_name().to_str().map(str::to_owned)else{continue};
   let Some(id)=name.strip_suffix(".md")else{continue};
   if !valid_id(id){continue;}
   found.push(id.to_string());
  }
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  tx.execute("DELETE FROM notes",[])?;
  let mut count=0usize;
  for id in &found{
   let bytes=match fs::read(self.directory.join(format!("{id}.md"))){Ok(bytes)=>bytes,Err(_)=>continue};
   let (meta,_)=match split_file(id,&bytes){Ok(value)=>value,Err(_)=>continue};
   if tx.execute("INSERT INTO notes(id,title,created_at,source,model) VALUES(?1,?2,?3,?4,?5)",params![meta.id,meta.title,meta.created_at,meta.source,meta.model]).is_ok(){count+=1;}
  }
  tx.commit()?;
  Ok(count)
 }
 fn load_file(&self,id:&str)->Result<(NoteMeta,String),Error>{
  let bytes=fs::read(self.directory.join(format!("{id}.md"))).map_err(|_|Error::NoteMissing)?;
  if bytes.len()>MAX_NOTE_BYTES+4096{return Err(Error::InvalidNote);}
  split_file(id,&bytes)
 }
}

fn create_dir(path:&Path)->Result<(),Error>{
 let mut builder=fs::DirBuilder::new();builder.recursive(true);
 #[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
 builder.create(path)?;Ok(())
}

fn write_atomic(path:&Path,bytes:&[u8])->Result<(),Error>{
 let mut temporary=path.as_os_str().to_os_string();
 temporary.push(format!(".{}.tmp",TEMP_COUNTER.fetch_add(1,Ordering::Relaxed)));
 let temporary=PathBuf::from(temporary);
 let mut options=fs::OpenOptions::new();options.write(true).create(true).truncate(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
 let mut file=options.open(&temporary)?;file.write_all(bytes)?;file.sync_all()?;drop(file);
 fs::rename(&temporary,path)?;
 #[cfg(unix)]{use std::os::unix::fs::PermissionsExt;fs::set_permissions(path,fs::Permissions::from_mode(0o600))?;}
 Ok(())
}
