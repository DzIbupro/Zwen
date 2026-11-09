//! Bounded local screenshot attachments. Nothing here uploads a picture.
use crate::{Error,open_private_database};
use png::{BitDepth,ColorType,Decoder,Encoder,Limits,Transformations};
use rusqlite::{Connection,OptionalExtension,TransactionBehavior,params};
use serde::Serialize;
use std::{fs,io::{Cursor,Read,Write},path::{Path,PathBuf},sync::atomic::{AtomicU64,Ordering}};

pub const MAX_ATTACHMENTS:usize=3;
pub const MAX_IMAGE_BYTES:u64=24*1024*1024;
pub const MAX_TOTAL_BYTES:u64=48*1024*1024;
pub const MAX_IMAGE_PIXELS:u64=10_000_000;
pub const MAX_IMAGE_DIM:u32=8192;
/// Conservative request bounds: the official GLM-4.6V-Flash page documents no
/// numeric image limits, so the legacy GLM-4V limits (5MB, 6000×6000) are
/// undercut. Verified only against synthetic services so far.
pub const MAX_REQUEST_IMAGE_BYTES:u64=4*1024*1024;
pub const MAX_REQUEST_IMAGE_DIM:u32=6000;
const MIN_REQUEST_DIM:u32=256;
const THUMB_MAX:u32=480;
const MAX_THUMB_BYTES:u64=2*1024*1024;
const PNG_SIGNATURE:[u8;8]=[137,80,78,71,13,10,26,10];

#[derive(Clone,Debug,PartialEq,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Attachment {pub id:String,pub width:u32,pub height:u32,pub bytes:u64,pub created_at:i64}

pub struct AttachmentStore {connection:Connection,directory:PathBuf}
impl AttachmentStore {
 pub fn open(directory:&Path)->Result<Self,Error>{
  create_private_dir(directory)?;
  let attachments=directory.join("attachments");create_private_dir(&attachments)?;create_private_dir(&attachments.join("incoming"))?;
  let connection=open_private_database(&directory.join("attachment.sqlite3"),|connection|{
   let version:i64=connection.pragma_query_value(None,"user_version",|r|r.get(0))?;
   if version!=0&&version!=1{return Err(Error::UnsupportedSchema);}
   if version==0{
    let tx=connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute_batch("CREATE TABLE attachments(
     id TEXT PRIMARY KEY,
     width INTEGER NOT NULL,
     height INTEGER NOT NULL,
     bytes INTEGER NOT NULL,
     created_at INTEGER NOT NULL
    );PRAGMA user_version = 1;")?;
    tx.commit()?;
   }
   read_all(connection)?;Ok(())
  })?;
  Ok(Self{connection,directory:attachments})
 }
 pub fn list(&self)->Result<Vec<Attachment>,Error>{read_all(&self.connection)}
 pub fn import(&mut self,id:&str,source:&Path)->Result<Attachment,Error>{self.import_inner(id,source,None)}
 /// Replace an existing attachment in one transaction. A failed import keeps the
 /// old row and files; a successful one removes them after the new row commits.
 pub fn replace(&mut self,id:&str,source:&Path,replaced:&str)->Result<Attachment,Error>{
  if id==replaced{return Err(Error::InvalidImage);}
  self.import_inner(id,source,Some(replaced))
 }
 fn import_inner(&mut self,id:&str,source:&Path,replaced:Option<&str>)->Result<Attachment,Error>{
  if !valid_id(id)||replaced.is_some_and(|old|!valid_id(old)||old==id){return Err(Error::InvalidImage);}
  let meta=fs::metadata(source)?;
  if !meta.is_file()||meta.len()==0||meta.len()>MAX_IMAGE_BYTES{return Err(Error::InvalidImage);}
  let bytes=read_bounded(source,MAX_IMAGE_BYTES)?;
  if bytes.len() as u64>MAX_IMAGE_BYTES{return Err(Error::InvalidImage);}
  let (width,height,thumbnail)=render_thumbnail(&bytes)?;
  let image=self.directory.join(format!("{id}.png"));
  let preview=self.directory.join(format!("{id}.thumb.png"));
  if image.exists()||preview.exists(){return Err(Error::InvalidImage);}
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  let (count,total):(i64,i64)=match replaced{
   Some(old)=>tx.query_row("SELECT count(*),coalesce(sum(bytes),0) FROM attachments WHERE id!=?1",[old],|r|Ok((r.get(0)?,r.get(1)?)))?,
   None=>tx.query_row("SELECT count(*),coalesce(sum(bytes),0) FROM attachments",[],|r|Ok((r.get(0)?,r.get(1)?)))?,
  };
  let size=i64::try_from(meta.len()).map_err(|_|Error::InvalidImage)?;
  if count>=MAX_ATTACHMENTS as i64||total.checked_add(size).is_none_or(|value|value>MAX_TOTAL_BYTES as i64){return Err(Error::AttachmentLimit);}
  let created_at:i64=tx.query_row("INSERT INTO attachments(id,width,height,bytes,created_at) VALUES(?1,?2,?3,?4,unixepoch()) RETURNING created_at",params![id,width,height,size],|r|r.get(0))?;
  let removed=if let Some(old)=replaced{tx.execute("DELETE FROM attachments WHERE id=?1",[old])?==1}else{false};
  if let Err(error)=write_private(&preview,&thumbnail){return Err(error);}
  if let Err(error)=move_private(source,&image){let _=fs::remove_file(&preview);return Err(error);}
  if let Err(error)=tx.commit(){let _=fs::remove_file(&image);let _=fs::remove_file(&preview);return Err(error.into());}
  if removed{
   if let Some(old)=replaced{
    let _=fs::remove_file(self.directory.join(format!("{old}.png")));
    let _=fs::remove_file(self.directory.join(format!("{old}.thumb.png")));
   }
  }
  Ok(Attachment{id:id.to_owned(),width,height,bytes:meta.len(),created_at})
 }
 pub fn remove(&mut self,id:&str)->Result<(),Error>{
  if !valid_id(id){return Err(Error::AttachmentMissing);}
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  let changed=tx.execute("DELETE FROM attachments WHERE id=?1",[id])?;
  tx.commit()?;
  if changed==1{
   let _=fs::remove_file(self.directory.join(format!("{id}.png")));
   let _=fs::remove_file(self.directory.join(format!("{id}.thumb.png")));
  }
  Ok(())
 }
 /// Returns a bounded data-URI payload; regenerates a missing preview from the
 /// original instead of trusting a partial file.
 pub fn thumbnail(&self,id:&str)->Result<Vec<u8>,Error>{
  if !valid_id(id){return Err(Error::AttachmentMissing);}
  let exists:Option<i64>=self.connection.query_row("SELECT 1 FROM attachments WHERE id=?1",[id],|r|r.get(0)).optional()?;
  if exists.is_none(){return Err(Error::AttachmentMissing);}
  let preview=self.directory.join(format!("{id}.thumb.png"));
  if let Ok(bytes)=read_bounded(&preview,MAX_THUMB_BYTES){if bytes.starts_with(&PNG_SIGNATURE){return Ok(bytes);}}
  let image=read_bounded(&self.directory.join(format!("{id}.png")),MAX_IMAGE_BYTES).map_err(|_|Error::AttachmentMissing)?;
  if image.len() as u64>MAX_IMAGE_BYTES{return Err(Error::InvalidImage);}
  let (_,_,thumbnail)=render_thumbnail(&image)?;
  write_private(&preview,&thumbnail)?;
  Ok(thumbnail)
 }
 /// Bounded PNG payload for a model request: at most 4 MiB and 6000px per side.
 /// The stored original is never modified; a payload that cannot fit even at
 /// the minimum size is rejected instead of sent oversized.
 pub fn model_payload(&self,id:&str)->Result<Vec<u8>,Error>{
  if !valid_id(id){return Err(Error::AttachmentMissing);}
  let exists:Option<i64>=self.connection.query_row("SELECT 1 FROM attachments WHERE id=?1",[id],|r|r.get(0)).optional()?;
  if exists.is_none(){return Err(Error::AttachmentMissing);}
  let image=read_bounded(&self.directory.join(format!("{id}.png")),MAX_IMAGE_BYTES)?;
  let (width,height,rgba)=decode_rgba(&image)?;
  let source_width=width as usize;let source_height=height as usize;
  let mut scale=1.0f64;
  if width>MAX_REQUEST_IMAGE_DIM{scale=scale.min(MAX_REQUEST_IMAGE_DIM as f64/width as f64);}
  if height>MAX_REQUEST_IMAGE_DIM{scale=scale.min(MAX_REQUEST_IMAGE_DIM as f64/height as f64);}
  let mut target_width=((width as f64*scale).round() as u32).max(1);
  let mut target_height=((height as f64*scale).round() as u32).max(1);
  loop{
   let small=downscale(&rgba,source_width,source_height,target_width as usize,target_height as usize);
   let encoded=encode_png(target_width,target_height,&small)?;
   if encoded.len() as u64<=MAX_REQUEST_IMAGE_BYTES{return Ok(encoded);}
   if target_width<=MIN_REQUEST_DIM&&target_height<=MIN_REQUEST_DIM{return Err(Error::InvalidImage);}
   target_width=(target_width/2).max(MIN_REQUEST_DIM);
   target_height=(target_height/2).max(MIN_REQUEST_DIM);
  }
 }
}

pub fn valid_id(id:&str)->bool{id.len()==32&&id.bytes().all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_uppercase())}

fn read_all(connection:&Connection)->Result<Vec<Attachment>,Error>{
 let mut statement=connection.prepare("SELECT CASE WHEN length(id)=32 THEN id END,width,height,bytes,created_at FROM attachments ORDER BY created_at,id")?;
 let rows=statement.query_map([],|r|Ok((r.get::<_,Option<String>>(0)?,r.get::<_,i64>(1)?,r.get::<_,i64>(2)?,r.get::<_,i64>(3)?,r.get::<_,i64>(4)?)))?;
 let mut attachments=Vec::new();
 for row in rows{
  let(id,width,height,bytes,created_at)=row?;
  let id=id.ok_or(Error::InvalidImage)?;
  let width=u32::try_from(width).map_err(|_|Error::InvalidImage)?;
  let height=u32::try_from(height).map_err(|_|Error::InvalidImage)?;
  let bytes=u64::try_from(bytes).map_err(|_|Error::InvalidImage)?;
  if !valid_id(&id)||width==0||height==0||u64::from(width)*u64::from(height)>MAX_IMAGE_PIXELS||bytes>MAX_IMAGE_BYTES{return Err(Error::InvalidImage);}
  attachments.push(Attachment{id,width,height,bytes,created_at});
 }
 Ok(attachments)
}

fn render_thumbnail(bytes:&[u8])->Result<(u32,u32,Vec<u8>),Error>{
 let (width,height,rgba)=decode_rgba(bytes)?;
 let (thumb_width,thumb_height)=thumbnail_size(width,height);
 let small=downscale(&rgba,width as usize,height as usize,thumb_width as usize,thumb_height as usize);
 let encoded=encode_png(thumb_width,thumb_height,&small)?;
 if encoded.len() as u64>MAX_THUMB_BYTES{return Err(Error::InvalidImage);}
 Ok((width,height,encoded))
}

fn decode_rgba(bytes:&[u8])->Result<(u32,u32,Vec<u8>),Error>{
 if bytes.len()<PNG_SIGNATURE.len()||bytes[..PNG_SIGNATURE.len()]!=PNG_SIGNATURE{return Err(Error::InvalidImage);}
 let mut decoder=Decoder::new_with_limits(Cursor::new(bytes),Limits{bytes:64*1024*1024});
 decoder.set_transformations(Transformations::normalize_to_color8());
 let mut reader=decoder.read_info().map_err(|_|Error::InvalidImage)?;
 let (width,height)=reader.info().size();
 if width==0||height==0||width>MAX_IMAGE_DIM||height>MAX_IMAGE_DIM||u64::from(width)*u64::from(height)>MAX_IMAGE_PIXELS{return Err(Error::InvalidImage);}
 let size=reader.output_buffer_size().ok_or(Error::InvalidImage)?;
 if size>MAX_IMAGE_PIXELS as usize*4{return Err(Error::InvalidImage);}
 let mut pixels=vec![0u8;size];
 let output=reader.next_frame(&mut pixels).map_err(|_|Error::InvalidImage)?;
 let (color,depth)=reader.output_color_type();
 if depth!=BitDepth::Eight{return Err(Error::InvalidImage);}
 let rgba=to_rgba(&pixels[..output.buffer_size()],color,output.width as usize,output.height as usize)?;
 Ok((width,height,rgba))
}

fn encode_png(width:u32,height:u32,rgba:&[u8])->Result<Vec<u8>,Error>{
 let mut encoded=Vec::new();
 {
  let mut encoder=Encoder::new(&mut encoded,width,height);
  encoder.set_color(ColorType::Rgba);encoder.set_depth(BitDepth::Eight);
  let mut writer=encoder.write_header().map_err(|_|Error::InvalidImage)?;
  writer.write_image_data(rgba).map_err(|_|Error::InvalidImage)?;
  writer.finish().map_err(|_|Error::InvalidImage)?;
 }
 Ok(encoded)
}

fn to_rgba(data:&[u8],color:ColorType,width:usize,height:usize)->Result<Vec<u8>,Error>{
 let pixels=width.checked_mul(height).ok_or(Error::InvalidImage)?;
 let mut out=Vec::with_capacity(pixels.checked_mul(4).ok_or(Error::InvalidImage)?);
 match color{
  ColorType::Rgba=>{if data.len()<pixels*4{return Err(Error::InvalidImage);}out.extend_from_slice(&data[..pixels*4]);}
  ColorType::Rgb=>{
   if data.len()<pixels*3{return Err(Error::InvalidImage);}
   for pixel in data[..pixels*3].chunks_exact(3){out.extend_from_slice(&[pixel[0],pixel[1],pixel[2],255]);}
  }
  ColorType::Grayscale=>{
   if data.len()<pixels{return Err(Error::InvalidImage);}
   for &value in &data[..pixels]{out.extend_from_slice(&[value,value,value,255]);}
  }
  ColorType::GrayscaleAlpha=>{
   if data.len()<pixels*2{return Err(Error::InvalidImage);}
   for pixel in data[..pixels*2].chunks_exact(2){out.extend_from_slice(&[pixel[0],pixel[0],pixel[0],pixel[1]]);}
  }
  _=>return Err(Error::InvalidImage),
 }
 Ok(out)
}

fn thumbnail_size(width:u32,height:u32)->(u32,u32){
 let scale=(THUMB_MAX as f64/width as f64).min(THUMB_MAX as f64/height as f64).min(1.0);
 (((width as f64*scale).round().max(1.0) as u32).min(THUMB_MAX),((height as f64*scale).round().max(1.0) as u32).min(THUMB_MAX))
}

fn downscale(source:&[u8],width:usize,height:usize,target_width:usize,target_height:usize)->Vec<u8>{
 let mut output=vec![0u8;target_width*target_height*4];
 for y in 0..target_height{
  let top=y*height/target_height;
  let bottom=(((y+1)*height).div_ceil(target_height)).max(top+1).min(height);
  for x in 0..target_width{
   let left=x*width/target_width;
   let right=(((x+1)*width).div_ceil(target_width)).max(left+1).min(width);
   let mut sum=[0u64;4];let mut count=0u64;
   for source_y in top..bottom{
    for source_x in left..right{
     let index=(source_y*width+source_x)*4;
     for channel in 0..4{sum[channel]+=u64::from(source[index+channel]);}
     count+=1;
    }
   }
   let index=(y*target_width+x)*4;
   for channel in 0..4{output[index+channel]=(sum[channel]/count) as u8;}
  }
 }
 output
}

fn create_private_dir(path:&Path)->Result<(),Error>{
 let mut builder=fs::DirBuilder::new();builder.recursive(true);
 #[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
 builder.create(path)?;Ok(())
}

fn write_private(path:&Path,bytes:&[u8])->Result<(),Error>{
 static TEMP:AtomicU64=AtomicU64::new(0);
 let mut temporary=path.as_os_str().to_os_string();
 temporary.push(format!(".{}.{}.tmp",std::process::id(),TEMP.fetch_add(1,Ordering::Relaxed)));
 let temporary=PathBuf::from(temporary);
 let mut options=fs::OpenOptions::new();options.write(true).create(true).truncate(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
 let mut file=options.open(&temporary)?;file.write_all(bytes)?;file.sync_all()?;drop(file);
 fs::rename(&temporary,path)?;
 set_private(path)?;Ok(())
}

fn move_private(source:&Path,target:&Path)->Result<(),Error>{
 if fs::rename(source,target).is_err(){
  if let Err(error)=fs::copy(source,target){let _=fs::remove_file(target);return Err(error.into());}
  let _=fs::remove_file(source);
 }
 set_private(target)
}

fn set_private(path:&Path)->Result<(),Error>{
 #[cfg(unix)]{use std::os::unix::fs::PermissionsExt;fs::set_permissions(path,fs::Permissions::from_mode(0o600))?;}
 Ok(())
}

fn read_bounded(path:&Path,max:u64)->Result<Vec<u8>,Error>{
 let file=fs::File::open(path)?;
 if !file.metadata()?.is_file(){return Err(Error::AttachmentMissing);}
 let mut bytes=Vec::new();
 file.take(max.saturating_add(1)).read_to_end(&mut bytes)?;
 if bytes.len() as u64>max{return Err(Error::AttachmentMissing);}
 Ok(bytes)
}
