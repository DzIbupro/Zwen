//! Local Markdown knowledge base. The .md file is the authority for content and
//! identity (stable id = filename); `notes.sqlite3` is a derived, rebuildable
//! index. Writes are temp-file → rename → index transaction; an orphan file is
//! adopted by rebuild, never deleted on sight.
use crate::{Error,open_private_database};
use rusqlite::{Connection,TransactionBehavior,params};
use serde::Serialize;
use std::{fs,io::{Read,Write},path::{Path,PathBuf},sync::atomic::{AtomicU64,Ordering},time::{SystemTime,UNIX_EPOCH}};

pub const MAX_NOTE_BYTES:usize=256*1024;
// Includes the maximum supported source URL, metadata and six attachment names.
const MAX_DOCUMENT_BYTES:usize=MAX_NOTE_BYTES+8192;
pub const MAX_TITLE_BYTES:usize=200;
pub const MAX_SOURCE_BYTES:usize=64;
pub const MAX_MODEL_BYTES:usize=200;
pub const MAX_NOTE_ATTACHMENTS:usize=6;
pub const MAX_SEARCH_QUERY:usize=256;
pub const MAX_LIST:usize=200;
const ATTACH_DIR:&str="attachments";
static TEMP_COUNTER:AtomicU64=AtomicU64::new(0);

#[derive(Clone,Debug,PartialEq,Serialize)]
#[serde(rename_all="camelCase")]
pub struct NoteMeta{pub id:String,pub title:String,pub created_at:i64,pub source:String,pub model:String,pub attachments:Vec<String>,#[serde(skip_serializing_if="Option::is_none")] pub request:Option<String>,#[serde(skip_serializing_if="Option::is_none")] pub source_title:Option<String>,#[serde(skip_serializing_if="Option::is_none")] pub source_url:Option<String>,#[serde(skip_serializing_if="Option::is_none")] pub source_loc:Option<String>}

/// Chat request ids follow the journal's rules; anything else is dropped so a
/// hand-edited front matter cannot smuggle arbitrary values into links.
pub fn valid_text_field(value:&str,max:usize)->bool{!value.is_empty()&&value.len()<=max&&!value.chars().any(char::is_control)}
pub fn valid_source_url(value:&str)->bool{
 (value.starts_with("https://")||value.starts_with("http://"))&&value.len()<=2048&&value.bytes().all(|b|b.is_ascii_graphic())
}
pub fn sanitize_filename_export(name:&str)->String{let cleaned=sanitize_filename(name);if cleaned.is_empty(){"未命名".to_string()}else{cleaned}}
pub fn valid_request(value:&str)->bool{!value.is_empty()&&value.len()<=100&&value.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_')}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Note{pub meta:NoteMeta,pub body:String}

fn valid_attachment_name(id:&str,name:&str)->bool{
 name.starts_with(&format!("{id}-"))&&!name.contains('/')&&!name.contains('\\')&&name.len()>33&&name.len()<=300&&!name.chars().any(char::is_control)
}
pub fn valid_id(id:&str)->bool{id.len()==32&&id.bytes().all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_uppercase())}

pub fn generate_id()->String{
 let nanos=SystemTime::now().duration_since(UNIX_EPOCH).map(|d|d.as_nanos() as u64).unwrap_or(0);
 let counter=TEMP_COUNTER.fetch_add(1,Ordering::Relaxed);
 format!("{:016x}{:016x}",nanos,counter)
}

/// Whitelist filenames: ASCII alphanumerics, CJK, dot, dash, underscore. Anything
/// else collapses to `_`; separators can never survive, so the value is safe to
/// join under the note's attachment directory.
pub fn sanitize_filename(name:&str)->String{sanitize_filename_inner(name)}
fn sanitize_filename_inner(name:&str)->String{
 let mut out=String::new();
 for ch in name.chars(){
  if ch.is_ascii_alphanumeric()||matches!(ch,'.'|'-'|'_')||('\u{4e00}'..='\u{9fff}').contains(&ch){out.push(ch);}
  else if ch.is_ascii_graphic(){out.push('_');}
 }
 let truncated:String=out.chars().take(80).collect();
 let trimmed=truncated.trim_matches(|c|c=='.'||c=='_').to_string();
 if trimmed.is_empty(){"file".to_string()}else{trimmed}
}

pub fn front_matter_text(meta:&NoteMeta)->String{front_matter(meta)}
fn front_matter(meta:&NoteMeta)->String{
 let list=serde_json::to_string(&meta.attachments).unwrap_or_else(|_|"[]".to_string());
 let request=meta.request.as_deref().map(|value|format!("\nrequest: {value}")).unwrap_or_default();
 let source_title=meta.source_title.as_deref().map(|value|format!("\nsource_title: {value}")).unwrap_or_default();
 let source_url=meta.source_url.as_deref().map(|value|format!("\nsource_url: {value}")).unwrap_or_default();
 let source_loc=meta.source_loc.as_deref().map(|value|format!("\nsource_loc: {value}")).unwrap_or_default();
 format!("---\nid: {}\ncreated: {}\nsource: {}\nmodel: {}\ntitle: {}\nattachments: {}{}{}{}{}\n---\n",meta.id,meta.created_at,meta.source,meta.model,meta.title,list,request,source_title,source_url,source_loc)
}

/// Splits the stored file into metadata and body. The body starts after the
/// front matter block; a missing block means the whole file is the body.
fn split_file(id:&str,bytes:&[u8])->Result<(NoteMeta,String),Error>{
 let text=String::from_utf8(bytes.to_vec()).map_err(|_|Error::InvalidNote)?;
 if let Some(rest)=text.strip_prefix("---\n"){
  if let Some(end)=rest.find("\n---\n"){
   let mut meta=NoteMeta{id:id.to_string(),title:"未命名笔记".to_string(),created_at:0,source:String::new(),model:String::new(),attachments:Vec::new(),request:None,source_title:None,source_url:None,source_loc:None};
   for line in rest[..end].lines(){
    let Some((key,value))=line.split_once(':')else{continue};
    match key{
     "id" if value.trim()==id=>{},
     "id"=>return Err(Error::InvalidNote),
     "created"|"createdAt"=>meta.created_at=value.trim().parse().unwrap_or(0),
     "source"=>{let v=value.trim();if v.len()<=MAX_SOURCE_BYTES{meta.source=v.to_string();}}
     "model"=>{let v=value.trim();if v.len()<=MAX_MODEL_BYTES{meta.model=v.to_string();}}
     "title"=>{let v=value.trim();if !v.is_empty()&&v.len()<=MAX_TITLE_BYTES{meta.title=v.to_string();}}
     "attachments"=>{if let Ok(list)=serde_json::from_str::<Vec<String>>(value.trim()){meta.attachments=list.into_iter().filter(|name|valid_attachment_name(id,name)).take(MAX_NOTE_ATTACHMENTS).collect();}}
     "request"=>{let v=value.trim();if valid_request(v){meta.request=Some(v.to_string());}}
     "source_title"=>{let v=value.trim();if valid_text_field(v,200){meta.source_title=Some(v.to_string());}}
     "source_url"=>{let v=value.trim();if valid_source_url(v){meta.source_url=Some(v.to_string());}}
     "source_loc"=>{let v=value.trim();if valid_text_field(v,200){meta.source_loc=Some(v.to_string());}}
     _=>{}
    }
   }
   let body=rest[end+5..].to_string();
   return Ok((meta,body));
  }
 }
 Ok((NoteMeta{id:id.to_string(),title:"未命名笔记".to_string(),created_at:0,source:String::new(),model:String::new(),attachments:Vec::new(),request:None,source_title:None,source_url:None,source_loc:None},text))
}

fn read_all(connection:&Connection)->Result<Vec<NoteMeta>,Error>{
 let mut statement=connection.prepare("SELECT CASE WHEN length(id)=32 THEN id END,title,created_at,CASE WHEN length(source)<=64 THEN source END,CASE WHEN length(model)<=200 THEN model END FROM notes ORDER BY created_at DESC,id DESC LIMIT ?1")?;
 let rows=statement.query_map([MAX_LIST as i64],|r|Ok((r.get::<_,Option<String>>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?,r.get::<_,Option<String>>(3)?,r.get::<_,Option<String>>(4)?)))?;
 let mut notes=Vec::new();
 for row in rows{
  let(id,title,created_at,source,model)=row?;
  let id=id.ok_or(Error::InvalidNote)?;
  if !valid_id(&id)||title.is_empty(){return Err(Error::InvalidNote);}
  notes.push(NoteMeta{id,title,created_at,source:source.unwrap_or_default(),model:model.unwrap_or_default(),attachments:Vec::new(),request:None,source_title:None,source_url:None,source_loc:None});
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
 /// Explicit user recovery only: preserve corrupt index bytes and sidecars,
 /// then derive a new index from the authoritative Markdown files.
 pub fn rebuild_from_files(data_dir:&Path)->Result<usize,Error>{
  match Self::open(data_dir){
   Ok(mut store)=>return store.rebuild_index(),
   Err(Error::Database(rusqlite::Error::SqliteFailure(code,_))) if matches!(code.code,rusqlite::ErrorCode::DatabaseCorrupt|rusqlite::ErrorCode::NotADatabase)=>(),
   Err(Error::InvalidNote)=>(),
   Err(error)=>return Err(error),
  }
  let stamp=generate_id();let mut moved:Vec<(PathBuf,PathBuf)>=Vec::new();
  for suffix in ["","-wal","-shm"]{
   let source=data_dir.join(format!("notes.sqlite3{suffix}"));
   if !source.exists(){continue;}
   if fs::symlink_metadata(&source)?.file_type().is_symlink(){return Err(Error::InvalidNote);}
   let target=data_dir.join(format!("notes.sqlite3.corrupt-{stamp}{suffix}"));
   if let Err(error)=fs::rename(&source,&target){
    for (source,target) in moved.iter().rev(){let _=fs::rename(target,source);}
    return Err(error.into());
   }
   moved.push((source,target));
  }
  Self::open(data_dir)?.rebuild_index()
 }
 pub fn create(&mut self,title:&str,body:&str,source:&str,model:&str,attachments:&[(String,Vec<u8>)],request:Option<&str>)->Result<NoteMeta,Error>{
  create_dir(&self.directory)?;create_dir(&self.directory.join(ATTACH_DIR))?;
  let request=match request{Some(value) if valid_request(value)=>Some(value.to_string()),_=>None};
  let title=title.trim();
  let title=if title.is_empty(){"未命名笔记".to_string()}else if title.len()<=MAX_TITLE_BYTES{title.to_string()}else{return Err(Error::InvalidNote)};
  if title.chars().chain(source.chars()).chain(model.chars()).any(char::is_control)||body.len()>MAX_NOTE_BYTES||source.len()>MAX_SOURCE_BYTES||model.len()>MAX_MODEL_BYTES{return Err(Error::InvalidNote);}
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
  let meta=NoteMeta{id:id.clone(),title,created_at,source:source.to_string(),model:model.to_string(),attachments:files.iter().map(|(name,_)|format!("{id}-{name}")).collect(),request,source_title:None,source_url:None,source_loc:None};
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
    if let Ok(bytes)=read_document(&self.directory,&id){
     let text=String::from_utf8_lossy(&bytes);
     if text.contains(query){ids.push(id);}
    }
   }
  }
  let mut notes=Vec::new();
  for id in ids{
   let (meta,_)=self.load_file(&id)?;
   notes.push(meta);
  }
  Ok(notes)
 }
 /// Complete enumeration for backup/export. UI list limits never truncate it.
 pub fn all_ids(&self)->Result<Vec<String>,Error>{
  let mut q=self.connection.prepare("SELECT id FROM notes ORDER BY created_at DESC,id DESC")?;
  let rows=q.query_map([],|r|r.get::<_,String>(0))?;
  let ids=rows.collect::<Result<Vec<_>,_>>()?;
  if ids.iter().any(|id|!valid_id(id)){return Err(Error::InvalidNote);}
  Ok(ids)
 }
 /// Original bytes retain external Markdown metadata during export.
 pub fn document(&self,id:&str)->Result<Vec<u8>,Error>{
  let bytes=read_document(&self.directory,id)?;
  split_file(id,&bytes)?;
  Ok(bytes)
 }
 pub fn read(&self,id:&str)->Result<Note,Error>{
  if !valid_id(id){return Err(Error::NoteMissing);}
  let (meta,body)=self.load_file(id)?;
  Ok(Note{meta,body})
 }
 pub fn delete(&mut self,id:&str)->Result<(),Error>{
  create_dir(&self.directory)?;create_dir(&self.directory.join(ATTACH_DIR))?;
  if !valid_id(id){return Err(Error::NoteMissing);}
  // A vanished file is already gone as far as the authority goes; the row is
  // still removed so no ghost entry stays in the list.
  let attachments=match self.load_file(id){Ok((meta,_))=>meta.attachments,Err(_)=>Vec::new()};
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  let changed=tx.execute("DELETE FROM notes WHERE id=?1",[id])?;
  if changed==0{return Err(Error::NoteMissing);}
  match fs::remove_file(self.directory.join(format!("{id}.md"))){
   Ok(())=>(),Err(error) if error.kind()==std::io::ErrorKind::NotFound=>(),Err(error)=>return Err(error.into()),
  }
  tx.commit()?;
  crate::organize::OrganizeStore::open(self.directory.parent().ok_or(Error::InvalidNote)?)?.invalidate_note(id,None)?;
  for name in attachments{let _=fs::remove_file(self.directory.join(ATTACH_DIR).join(name));}
  Ok(())
 }
 /// In-app manual edit: rewrites title and body in place. Identity, creation
 /// time, source, model, request link and attachment files are preserved.
 pub fn update(&mut self,id:&str,title:&str,body:&str)->Result<NoteMeta,Error>{
  self.update_with_source(id,title,body,None)
 }
 /// Body and source changes share one validated Markdown replacement.
 pub fn update_with_source(&mut self,id:&str,title:&str,body:&str,source:Option<&NoteMeta>)->Result<NoteMeta,Error>{
  if !valid_id(id){return Err(Error::NoteMissing);}
  let title=title.trim();
  if title.is_empty()||title.chars().any(char::is_control)||title.len()>MAX_TITLE_BYTES||body.len()>MAX_NOTE_BYTES{return Err(Error::InvalidNote);}
  let (mut meta,_)=self.load_file(id)?;
  meta.title=title.to_string();
  if let Some(source)=source{apply_source(&mut meta,source)?;}
  let mut document=front_matter(&meta).into_bytes();document.extend_from_slice(body.as_bytes());
  write_atomic(&self.directory.join(format!("{id}.md")),&document)?;
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  tx.execute("UPDATE notes SET title=?1 WHERE id=?2",params![meta.title,id])?;
  tx.commit()?;
  crate::organize::OrganizeStore::open(self.directory.parent().ok_or(Error::InvalidNote)?)?.invalidate_note(id,Some(crate::organize::content_hash(body.as_bytes())))?;
  Ok(meta)
 }
 /// Manual source metadata (title / URL / page-or-location). Unknown stays empty.
 pub fn update_source(&mut self,id:&str,meta_in:&NoteMeta)->Result<(),Error>{
  if !valid_id(id){return Err(Error::NoteMissing);}
  let (mut meta,body)=self.load_file(id)?;
  apply_source(&mut meta,meta_in)?;
  let mut document=front_matter(&meta).into_bytes();document.extend_from_slice(body.as_bytes());
  write_atomic(&self.directory.join(format!("{id}.md")),&document)?;
  Ok(())
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
   let bytes=match read_document(&self.directory,id){Ok(bytes)=>bytes,Err(_)=>continue};
   let (meta,_)=match split_file(id,&bytes){Ok(value)=>value,Err(_)=>continue};
   if tx.execute("INSERT INTO notes(id,title,created_at,source,model) VALUES(?1,?2,?3,?4,?5)",params![meta.id,meta.title,meta.created_at,meta.source,meta.model]).is_ok(){count+=1;}
  }
  tx.commit()?;
  Ok(count)
 }
 fn load_file(&self,id:&str)->Result<(NoteMeta,String),Error>{
  split_file(id,&read_document(&self.directory,id)?)
 }
}

/// All Markdown consumers share an allocation bound, including search,
/// rebuild and raw export. O_NONBLOCK avoids hanging on a substituted FIFO;
/// O_NOFOLLOW prevents a symlink swap between validation and opening.
fn read_document(directory:&Path,id:&str)->Result<Vec<u8>,Error>{
 if !valid_id(id){return Err(Error::NoteMissing);}
 let parent=fs::symlink_metadata(directory).map_err(|_|Error::NoteMissing)?;
 if !parent.is_dir()||parent.file_type().is_symlink(){return Err(Error::InvalidNote);}
 let path=directory.join(format!("{id}.md"));
 let metadata=fs::symlink_metadata(&path).map_err(|_|Error::NoteMissing)?;
 if !metadata.is_file()||metadata.file_type().is_symlink(){return Err(Error::InvalidNote);}
 let mut options=fs::OpenOptions::new();options.read(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.custom_flags(libc::O_NOFOLLOW|libc::O_NONBLOCK);}
 let file=options.open(path).map_err(|error|if error.kind()==std::io::ErrorKind::NotFound{Error::NoteMissing}else{Error::Io(error)})?;
 let metadata=file.metadata()?;
 if !metadata.is_file()||metadata.len()>MAX_DOCUMENT_BYTES as u64{return Err(Error::InvalidNote);}
 let mut bytes=Vec::new();
 file.take((MAX_DOCUMENT_BYTES+1) as u64).read_to_end(&mut bytes)?;
 if bytes.len()>MAX_DOCUMENT_BYTES{return Err(Error::InvalidNote);}
 Ok(bytes)
}

fn create_dir(path:&Path)->Result<(),Error>{
 if let Ok(meta)=fs::symlink_metadata(path){if !meta.is_dir()||meta.file_type().is_symlink(){return Err(Error::InvalidNote);}}
 let mut builder=fs::DirBuilder::new();builder.recursive(true);
 #[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
 builder.create(path)?;Ok(())
}

fn write_atomic(path:&Path,bytes:&[u8])->Result<(),Error>{
 let mut temporary=path.as_os_str().to_os_string();
 temporary.push(format!(".{}.tmp",TEMP_COUNTER.fetch_add(1,Ordering::Relaxed)));
 let temporary=PathBuf::from(temporary);
 let mut options=fs::OpenOptions::new();options.write(true).create_new(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
 let mut file=options.open(&temporary)?;file.write_all(bytes)?;file.sync_all()?;drop(file);
 fs::rename(&temporary,path)?;
 #[cfg(unix)]{use std::os::unix::fs::PermissionsExt;fs::set_permissions(path,fs::Permissions::from_mode(0o600))?;}
 Ok(())
}

fn apply_source(meta:&mut NoteMeta,meta_in:&NoteMeta)->Result<(),Error>{
  let validators:[(&mut Option<String>,Option<&str>,fn(&str)->bool);3]=[
   (&mut meta.source_title,meta_in.source_title.as_deref(),|v|valid_text_field(v,200)),
   (&mut meta.source_url,meta_in.source_url.as_deref(),valid_source_url),
   (&mut meta.source_loc,meta_in.source_loc.as_deref(),|v|valid_text_field(v,200)),
  ];
  for (target,value,valid) in validators{
   *target=match value{Some(v) if valid(v)=>Some(v.to_string()),Some(_)=>return Err(Error::InvalidNote),None=>None};
  }
 Ok(())
}
