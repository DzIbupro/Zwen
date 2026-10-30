use std::{path::{Path,PathBuf},sync::atomic::{AtomicU64,Ordering}};
use zwen_core::attachment::{AttachmentStore,MAX_ATTACHMENTS,MAX_IMAGE_DIM};
use zwen_core::Error;

static NEXT:AtomicU64=AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch{
 fn new()->Self{let path=std::env::temp_dir().join(format!("zwen-attachment-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));std::fs::create_dir(&path).unwrap();Self(path)}
 fn source(&self,name:&str)->PathBuf{self.0.join(name)}
}
impl Drop for Scratch{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}

fn png(width:u32,height:u32)->Vec<u8>{
 let mut output=Vec::new();
 let mut encoder=png::Encoder::new(&mut output,width,height);
 encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);
 let mut writer=encoder.write_header().unwrap();
 let pixels=vec![0x6du8;(width as usize*height as usize*4) as usize];
 writer.write_image_data(&pixels).unwrap();writer.finish().unwrap();output
}

fn import_png(store:&mut AttachmentStore,scratch:&Scratch,id:&str,width:u32,height:u32)->zwen_core::attachment::Attachment{
 let source=scratch.source(&format!("{id}.png"));std::fs::write(&source,png(width,height)).unwrap();
 store.import(id,&source).unwrap()
}

#[test] fn imports_png_with_private_files_and_thumbnail_metadata(){
 let scratch=Scratch::new();let mut store=AttachmentStore::open(&scratch.0).unwrap();
 let attachment=import_png(&mut store,&scratch,&"a".repeat(32),320,200);
 assert_eq!((attachment.width,attachment.height), (320,200));
 assert_eq!(store.list().unwrap().len(),1);
 assert!(store.thumbnail(&attachment.id).unwrap().starts_with(&[137,80,78,71]));
 #[cfg(unix)]
 {
  use std::os::unix::fs::PermissionsExt;
  let image=scratch.0.join("attachments").join(format!("{}.png",attachment.id));
  assert_eq!(std::fs::metadata(image).unwrap().permissions().mode()&0o077,0);
 }
 store.remove(&attachment.id).unwrap();
 assert!(store.list().unwrap().is_empty());
 assert!(!scratch.0.join("attachments").join(format!("{}.png",attachment.id)).exists());
 store.remove(&attachment.id).unwrap();
}

#[test] fn reopening_keeps_metadata_and_regenerates_missing_thumbnail(){
 let scratch=Scratch::new();let id="b".repeat(32);
 {let mut store=AttachmentStore::open(&scratch.0).unwrap();import_png(&mut store,&scratch,&id,64,48);}
 let thumb=scratch.0.join("attachments").join(format!("{id}.thumb.png"));std::fs::remove_file(&thumb).unwrap();
 let store=AttachmentStore::open(&scratch.0).unwrap();let list=store.list().unwrap();
 assert_eq!(list.len(),1);assert_eq!(list[0].id,id);
 assert!(store.thumbnail(&id).unwrap().starts_with(&[137,80,78,71]));
}

#[test] fn rejects_invalid_images_dimensions_and_unknown_ids(){
 let scratch=Scratch::new();let mut store=AttachmentStore::open(&scratch.0).unwrap();
 let garbage=scratch.source("garbage.png");std::fs::write(&garbage,b"not a png").unwrap();
 assert!(matches!(store.import(&"c".repeat(32),&garbage),Err(Error::InvalidImage)));
 let empty=scratch.source("empty.png");std::fs::write(&empty,b"").unwrap();
 assert!(matches!(store.import(&"d".repeat(32),&empty),Err(Error::InvalidImage)));
 let huge=scratch.source("huge.png");std::fs::write(&huge,png(MAX_IMAGE_DIM+1,1)).unwrap();
 assert!(matches!(store.import(&"e".repeat(32),&huge),Err(Error::InvalidImage)));
 assert!(matches!(store.import("../escape",&garbage),Err(Error::InvalidImage)));
 assert!(matches!(store.thumbnail(&"f".repeat(32)),Err(Error::AttachmentMissing)));
}

#[test] fn enforces_attachment_count_without_touching_saved_items(){
 let scratch=Scratch::new();let mut store=AttachmentStore::open(&scratch.0).unwrap();
 for index in 0..MAX_ATTACHMENTS{import_png(&mut store,&scratch,&format!("{index:032x}"),32,32);}
 let source=scratch.source("extra.png");std::fs::write(&source,png(32,32)).unwrap();
 assert!(matches!(store.import(&"9".repeat(32),&source),Err(Error::AttachmentLimit)));
 assert_eq!(store.list().unwrap().len(),MAX_ATTACHMENTS);
}

#[test] fn replacing_at_the_limit_keeps_the_count_and_removes_the_old_files(){
 let scratch=Scratch::new();let mut store=AttachmentStore::open(&scratch.0).unwrap();
 for index in 0..MAX_ATTACHMENTS{import_png(&mut store,&scratch,&format!("{index:032x}"),32,32);}
 let old="0".repeat(32);let source=scratch.source("replacement.png");std::fs::write(&source,png(48,48)).unwrap();
 let replacement=store.replace(&"a".repeat(32),&source,&old).unwrap();
 assert_eq!(replacement.width,48);
 let list=store.list().unwrap();
 assert_eq!(list.len(),MAX_ATTACHMENTS);
 assert!(list.iter().all(|item|item.id!=old));
 assert!(!scratch.0.join("attachments").join(format!("{old}.png")).exists());
 assert!(!scratch.0.join("attachments").join(format!("{old}.thumb.png")).exists());
}

#[test] fn a_failed_replacement_preserves_the_old_attachment(){
 let scratch=Scratch::new();let mut store=AttachmentStore::open(&scratch.0).unwrap();
 let old=import_png(&mut store,&scratch,&"b".repeat(32),32,32);
 let invalid=scratch.source("invalid-replacement.png");std::fs::write(&invalid,b"not a png").unwrap();
 assert!(matches!(store.replace(&"c".repeat(32),&invalid,&old.id),Err(Error::InvalidImage)));
 let list=store.list().unwrap();
 assert_eq!(list.len(),1);assert_eq!(list[0].id,old.id);
 assert!(scratch.0.join("attachments").join(format!("{}.png",old.id)).exists());
}

#[test] fn deleted_image_does_not_return_a_preview_or_leak_other_files(){
 let scratch=Scratch::new();let mut store=AttachmentStore::open(&scratch.0).unwrap();
 let attachment=import_png(&mut store,&scratch,&"1".repeat(32),32,32);
 let directory=scratch.0.join("attachments");
 std::fs::remove_file(directory.join(format!("{}.png",attachment.id))).unwrap();
 std::fs::remove_file(directory.join(format!("{}.thumb.png",attachment.id))).unwrap();
 assert!(matches!(store.thumbnail(&attachment.id),Err(Error::AttachmentMissing)));
 let keep=Path::new(&scratch.0).join("attachments").join("keep.txt");std::fs::write(&keep,b"keep").unwrap();
 assert!(keep.exists());
}
