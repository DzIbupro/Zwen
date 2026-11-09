//! File adaptation only. Analysis uses the existing chat lifecycle.
use crate::{ClientError,SharedStorage};
use zwen_core::paper::{Library,Metadata,ReadingOptions,MAX_PDF_BYTES,MAX_TEXT_BYTES};
use std::{fs,io::{Read,Write},path::PathBuf,sync::atomic::AtomicBool,time::{Duration,Instant}};
use crate::paper_process::{ProcessRunner,ProcessSpec};
pub fn error(e:zwen_core::paper::PaperError)->ClientError{ClientError{code:"paper_unavailable",message:e.to_string()}}
pub fn library(storage:&SharedStorage)->Result<Library,ClientError>{Library::open(storage.directory.as_ref().map_err(Clone::clone)?).map_err(error)}
fn random_id()->Result<String,ClientError>{let mut bytes=[0u8;16];fs::File::open("/dev/urandom").and_then(|mut f|f.read_exact(&mut bytes)).map_err(|_|ClientError::storage())?;Ok(bytes.iter().map(|b|format!("{b:02x}")).collect())}
#[tauri::command]
pub fn paper_defaults()->ReadingOptions{ReadingOptions::default()}
#[tauri::command]
pub async fn paper_list(storage:tauri::State<'_,SharedStorage>)->Result<Vec<Metadata>,ClientError>{let s=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||library(&s)?.list().map_err(error)).await.map_err(|_|ClientError::storage())?}
#[tauri::command]
pub async fn paper_list_page(storage:tauri::State<'_,SharedStorage>,cursor:Option<i64>)->Result<zwen_core::paper::MetadataPage,ClientError>{let s=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||library(&s)?.list_page(cursor,50).map_err(error)).await.map_err(|_|ClientError::storage())?}
#[tauri::command]
pub async fn paper_metadata(storage:tauri::State<'_,SharedStorage>,paper_id:String)->Result<Metadata,ClientError>{let s=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||library(&s)?.metadata(&paper_id).map_err(error)).await.map_err(|_|ClientError::storage())?}
#[tauri::command]
pub async fn paper_choose(app:tauri::AppHandle,storage:tauri::State<'_,SharedStorage>)->Result<Option<Metadata>,ClientError>{
 let (tx,rx)=tokio::sync::oneshot::channel();
 app.run_on_main_thread(move||{
  use gtk::prelude::*;
  let dialog=gtk::FileChooserNative::new(Some("选择论文PDF"),None::<&gtk::Window>,gtk::FileChooserAction::Open,Some("导入"),Some("取消"));
  let filter=gtk::FileFilter::new();filter.set_name(Some("PDF论文"));filter.add_pattern("*.pdf");filter.add_pattern("*.PDF");dialog.add_filter(filter);
  let keeper=std::rc::Rc::new(std::cell::RefCell::new(Some(dialog.clone())));
  let tx=std::cell::RefCell::new(Some(tx));dialog.connect_response(move|d,response|{if let Some(tx)=tx.borrow_mut().take(){let _=tx.send(if response==gtk::ResponseType::Accept{d.filename()}else{None});}d.destroy();keeper.borrow_mut().take();});dialog.show();
  #[cfg(feature="desktop-smoke")]
  if let Some(path)=std::env::var_os("ZWEN_PAPER_FIXTURE_PDF"){
   use gtk::glib::translate::IntoGlib;
   static CHOOSERS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
   let cancel=CHOOSERS.fetch_add(1,std::sync::atomic::Ordering::SeqCst)==0;
   gtk::glib::timeout_add_local_once(Duration::from_millis(700),move||{
    if cancel{dialog.emit_by_name::<()>("response",&[&gtk::ResponseType::Cancel.into_glib()]);}else{dialog.set_filename(PathBuf::from(path));gtk::glib::timeout_add_local_once(Duration::from_millis(300),move||dialog.emit_by_name::<()>("response",&[&gtk::ResponseType::Accept.into_glib()]));}
   });
  }
 }).map_err(|_|ClientError::window())?;
 let Some(path)=rx.await.map_err(|_|ClientError::window())? else{return Ok(None);};let s=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||import(&s,path).map(Some)).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn paper_import_pdf(storage:tauri::State<'_,SharedStorage>,path:String)->Result<Metadata,ClientError>{
 let path=PathBuf::from(path);validate_drop_path(&path)?;let s=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||import(&s,path)).await.map_err(|_|ClientError::storage())?
}
fn validate_drop_path(path:&std::path::Path)->Result<(),ClientError>{
 if !path.is_absolute()||!path.extension().and_then(|v|v.to_str()).is_some_and(|v|v.eq_ignore_ascii_case("pdf")){return Err(ClientError{code:"paper_import",message:"请拖入本机单篇PDF文件".into()});}Ok(())
}
#[tauri::command]
pub async fn paper_open(app:tauri::AppHandle,storage:tauri::State<'_,SharedStorage>,paper_id:String)->Result<(),ClientError>{
 let s=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||{
  let lib=library(&s)?;lib.metadata(&paper_id).map_err(error)?;let path=lib.pdf_path(&paper_id).map_err(error)?;
  read_pdf(&path)?;spawn_document(document_opener(&path),move||{use tauri::Emitter;let _=app.emit("paper-open-failed",serde_json::json!({"paperId":paper_id}));})
 }).await.map_err(|_|ClientError::storage())?
}
fn open_document(path:&std::path::Path)->Result<(),ClientError>{
 spawn_document(document_opener(path),||{})
}
fn spawn_document(mut command:std::process::Command,failed:impl FnOnce()+Send+'static)->Result<(),ClientError>{
 let mut child=command.spawn().map_err(|_|ClientError{code:"paper_open",message:"无法打开原件，请检查系统PDF阅读器".into()})?;
 std::thread::spawn(move||{if !child.wait().is_ok_and(|status|status.success()){failed();}});Ok(())
}
fn document_opener(path:&std::path::Path)->std::process::Command{
 let mut command=std::process::Command::new("xdg-open");command.arg(path).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());command
}
fn read_pdf(path:&std::path::Path)->Result<Vec<u8>,ClientError>{
 let fail=|s:&str|ClientError{code:"paper_import",message:s.into()};
 let mut input=fs::OpenOptions::new();input.read(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;input.custom_flags(libc::O_NONBLOCK);}
 let file=input.open(path).map_err(|_|fail("PDF无法读取"))?;
 let meta=file.metadata().map_err(|_|fail("PDF无法读取"))?;
 if !meta.is_file()||meta.len()>MAX_PDF_BYTES as u64{return Err(fail("请选择不超过50 MiB的普通PDF文件"));}
 let mut bytes=Vec::new();file.take(MAX_PDF_BYTES as u64+1).read_to_end(&mut bytes).map_err(|_|fail("PDF无法读取"))?;
 if bytes.len()>MAX_PDF_BYTES||!bytes.starts_with(b"%PDF-"){return Err(fail("请选择不超过50 MiB的PDF文件"));}
 Ok(bytes)
}
fn import(storage:&SharedStorage,path:PathBuf)->Result<Metadata,ClientError>{
 let fail=|s:&str|ClientError{code:"paper_import",message:s.into()};
 let bytes=read_pdf(&path)?;
 let mut lib=library(storage)?;let id=random_id()?;let tmp=lib.pdf_path(&id).map_err(error)?.with_extension("incoming");
 let mut options=fs::OpenOptions::new();options.write(true).create_new(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
 let mut file=options.open(&tmp).map_err(|_|ClientError::storage())?;
 struct Cleanup(PathBuf);impl Drop for Cleanup{fn drop(&mut self){let _=fs::remove_file(&self.0);}}
 let _cleanup=Cleanup(tmp.clone());file.write_all(&bytes).map_err(|_|ClientError::storage())?;file.sync_all().map_err(|_|ClientError::storage())?;drop(file);
 let cancel=AtomicBool::new(false);
 let info=ProcessRunner::run(ProcessSpec{program:"pdfinfo".into(),args:vec![tmp.to_string_lossy().into_owned()],deadline:Instant::now()+Duration::from_secs(15),output_limit:65536},&cancel).map_err(error)?;
 if !info.success{return Err(fail("PDF损坏、加密或无法读取"));}
 let count=String::from_utf8_lossy(&info.stdout).lines().find_map(|l|l.strip_prefix("Pages:").and_then(|s|s.trim().parse::<usize>().ok())).filter(|n|*n>0&&*n<=300).ok_or_else(||fail("PDF页数无效或超过300页"))?;
 let text=ProcessRunner::run(ProcessSpec{program:"pdftotext".into(),args:vec!["-enc".into(),"UTF-8".into(),"-eol".into(),"unix".into(),tmp.to_string_lossy().into_owned(),"-".into()],deadline:Instant::now()+Duration::from_secs(60),output_limit:MAX_TEXT_BYTES},&cancel).map_err(error)?;
 if !text.success{return Err(fail("PDF文字提取失败，未发送模型"));}
 let text=String::from_utf8(text.stdout).map_err(|_|fail("提取文字不是有效UTF-8"))?;
 let mut pages=text.split('\x0c').map(|s|s.trim().to_owned()).collect::<Vec<_>>();if pages.last().is_some_and(|s|s.is_empty()){pages.pop();}
 if pages.len()!=count{return Err(fail("提取页数与PDF不一致，请检查原文；未将部分内容视为全文"));}
 let name=path.file_name().and_then(|s|s.to_str()).unwrap_or("论文.pdf");lib.add(&id,name,&bytes,pages).map_err(error)
}
#[tauri::command]
pub async fn paper_sessions(storage:tauri::State<'_,SharedStorage>,paper_id:String)->Result<Vec<zwen_core::journal::Turn>,ClientError>{
 let s=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||{
  let lib=library(&s)?;let ids=lib.sessions(&paper_id).map_err(error)?;
  let journal=zwen_core::journal::Journal::open(&s.directory.as_ref().map_err(Clone::clone)?.join("chat.sqlite3"))?;
  ids.iter().filter_map(|id|journal.latest_in(id).transpose()).collect::<Result<Vec<_>,zwen_core::Error>>().map_err(Into::into)
 }).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn paper_source(storage:tauri::State<'_,SharedStorage>,paper_id:String)->Result<Vec<String>,ClientError>{let s=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||Ok(library(&s)?.get(&paper_id).map_err(error)?.pages)).await.map_err(|_|ClientError::storage())?}
#[tauri::command]
pub async fn paper_start(app:tauri::AppHandle,storage:tauri::State<'_,SharedStorage>,chat:tauri::State<'_,crate::chat::Shared>,request_id:String,paper_id:String,options:ReadingOptions,question:Option<String>)->Result<crate::chat::Reply,ClientError>{
 crate::chat::start_paper(app,storage.inner().clone(),chat.inner().clone(),request_id,paper_id,options,question).await
}
#[tauri::command]
pub async fn paper_legacy(storage:tauri::State<'_,SharedStorage>)->Result<Vec<zwen_core::paper::LegacyRun>,ClientError>{let s=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||zwen_core::paper::legacy_list(s.directory.as_ref().map_err(Clone::clone)?).map_err(error)).await.map_err(|_|ClientError::storage())?}
#[tauri::command]
pub async fn paper_legacy_report(storage:tauri::State<'_,SharedStorage>,run_id:String)->Result<String,ClientError>{let s=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||zwen_core::paper::legacy_report(s.directory.as_ref().map_err(Clone::clone)?,&run_id).map_err(error)).await.map_err(|_|ClientError::storage())?}
#[tauri::command]
pub async fn paper_export(storage:tauri::State<'_,SharedStorage>,request_id:String)->Result<String,ClientError>{
 let s=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||{
  let dir=s.directory.as_ref().map_err(Clone::clone)?;let journal=zwen_core::journal::Journal::open(&dir.join("chat.sqlite3"))?;
  let turn=journal.find(&request_id)?.ok_or_else(ClientError::storage)?;
  let (source,options)=library(&s)?.session(&turn.conversation_id).map_err(error)?.ok_or_else(ClientError::storage)?;
  if turn.state==zwen_core::journal::TurnState::Streaming||turn.answer.is_empty(){return Err(ClientError{code:"paper_export",message:"请等待请求结束并保存后导出".into()});}
  let home=std::env::var("HOME").map_err(|_|ClientError::storage())?;
  let target=crate::notes::export_target(&source.meta.name,&random_id()?,std::path::Path::new(&home))?;
  let mut opt=fs::OpenOptions::new();opt.write(true).create_new(true);
  #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;opt.mode(0o600);}
  let mut f=opt.open(&target).map_err(|_|ClientError::storage())?;
  let media=if options.read_images{"已提交所选页图补读，图表、公式与视觉结论仍需原文核对。"}else{"仅提交提取文字，图表图像未读取，公式与引用需核对。"};
  let markdown=format!("# {}\n\n模型：{}；状态：{:?}；PDF物理页：{}–{} / {}\n\n{media}\n\n## 本轮问题\n\n{}\n\n{}",source.meta.name,turn.model,turn.state,options.start_page,if options.end_page==0{source.meta.page_count}else{options.end_page},source.meta.page_count,turn.prompt,turn.answer);
  f.write_all(markdown.as_bytes()).and_then(|_|f.sync_all()).map_err(|_|ClientError::storage())?;Ok(target.to_string_lossy().into_owned())
 }).await.map_err(|_|ClientError::storage())?
}

#[derive(serde::Serialize)]#[serde(rename_all="camelCase")]
pub struct SessionInfo{source:Metadata,options:ReadingOptions,endpoint:String,model:String}
#[tauri::command]
pub async fn paper_session_info(storage:tauri::State<'_,SharedStorage>,conversation_id:String)->Result<SessionInfo,ClientError>{
 let s=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||{
  let (source,options)=library(&s)?.session(&conversation_id).map_err(error)?.ok_or_else(||ClientError{code:"paper_source_missing",message:"论文来源丢失，请重新导入并新建分析".into()})?;
  let journal=zwen_core::journal::Journal::open(&s.directory.as_ref().map_err(Clone::clone)?.join("chat.sqlite3"))?;
  let turn=journal.latest_in(&conversation_id)?.ok_or_else(ClientError::storage)?;
  let profile:zwen_core::provider::ProviderConfig=serde_json::from_str(&turn.provider_profile).map_err(|_|ClientError::storage())?;
  Ok(SessionInfo{source:source.meta,options,endpoint:profile.base_url().into(),model:turn.model.clone()})
 }).await.map_err(|_|ClientError::storage())?
}

#[tauri::command]
pub async fn paper_legacy_papers(storage:tauri::State<'_,SharedStorage>)->Result<Vec<Metadata>,ClientError>{let s=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||zwen_core::paper::legacy_papers(s.directory.as_ref().map_err(Clone::clone)?).map_err(error)).await.map_err(|_|ClientError::storage())?}
#[tauri::command]
pub async fn paper_legacy_open(storage:tauri::State<'_,SharedStorage>,paper_id:String)->Result<(),ClientError>{let s=storage.inner().clone();tauri::async_runtime::spawn_blocking(move||{
 let path=zwen_core::paper::legacy_pdf(s.directory.as_ref().map_err(Clone::clone)?,&paper_id).map_err(error)?;
 open_document(&path)
 }).await.map_err(|_|ClientError::storage())?}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn opener_failure_is_reported_without_waiting_on_the_ui(){
  for exit in [0,3]{let (tx,rx)=std::sync::mpsc::channel();let mut c=std::process::Command::new("/bin/sh");c.args(["-c",&format!("exit {exit}")]);spawn_document(c,move||{let _=tx.send(());}).unwrap();let result=rx.recv_timeout(Duration::from_secs(2));if exit==0{assert!(matches!(result,Err(std::sync::mpsc::RecvTimeoutError::Disconnected)));}else{assert!(result.is_ok());}}
 }
 #[test]fn dropped_paths_require_absolute_pdf_names(){assert!(validate_drop_path(std::path::Path::new("/tmp/paper.PDF")).is_ok());for p in ["relative.pdf","/tmp/private.txt","https://example.test/paper.pdf"]{assert!(validate_drop_path(std::path::Path::new(p)).is_err());}}
 #[test]fn original_opener_keeps_the_path_as_one_argument(){let path=std::path::Path::new("/tmp/name with spaces ; $(x).pdf");let c=document_opener(path);assert_eq!(c.get_program(),"xdg-open");assert_eq!(c.get_args().collect::<Vec<_>>(),vec![path.as_os_str()]);}
 #[test]fn named_pipe_is_rejected_without_waiting_for_writer(){
  use std::ffi::CString;
  let path=std::env::temp_dir().join(format!("zwen-fifo-{}.pdf",std::process::id()));let c=CString::new(path.to_string_lossy().as_bytes()).unwrap();
  assert_eq!(unsafe{libc::mkfifo(c.as_ptr(),0o600)},0);let start=Instant::now();let result=read_pdf(&path);let _=fs::remove_file(path);assert!(result.is_err());assert!(start.elapsed()<Duration::from_secs(1));
 }
}
