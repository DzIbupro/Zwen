//! Bounded, cancellable page rendering inside the existing request worker.
use std::{path::PathBuf,time::{Duration,Instant}};
use base64::{Engine as _,engine::general_purpose::STANDARD};
use tokio::sync::watch;
use crate::paper_process::{ProcessSpec,ProcessRunner};
use zwen_core::paper::{Result,invalid,MAX_PAGE_IMAGE_BYTES,MAX_VISUAL_PAGES,MAX_PAGE_IMAGE_EDGE,MAX_PAGE_IMAGE_TOTAL_BYTES,validate_page_png};
pub struct Request{pub pdf:PathBuf,pub pages:Vec<usize>}
pub fn render(request:&Request,cancel:&watch::Receiver<bool>)->Result<Vec<String>>{
 if request.pages.is_empty()||request.pages.len()>MAX_VISUAL_PAGES||request.pages.iter().any(|p|*p==0||*p>300){return Err(invalid("整页图范围无效或超过20页"));}
 let mut images=Vec::new();let mut total=0usize;let deadline=Instant::now()+Duration::from_secs(180);
 for page in &request.pages{
  let cancelled=||*cancel.borrow()||cancel.has_changed().is_err();if cancelled(){return Err(invalid("页图补读已取消"));}
  let output=ProcessRunner::run_cancellable(ProcessSpec{program:"pdftoppm".into(),args:vec!["-f".into(),page.to_string(),"-l".into(),page.to_string(),"-singlefile".into(),"-scale-to".into(),MAX_PAGE_IMAGE_EDGE.to_string(),"-png".into(),request.pdf.to_string_lossy().into_owned()],deadline:(Instant::now()+Duration::from_secs(20)).min(deadline),output_limit:MAX_PAGE_IMAGE_BYTES},cancelled)?;
  if !output.success{return Err(invalid("PDF页图无法读取，请检查原件或关闭页图补读后重试"));}
  validate_page_png(&output.stdout)?;let payload=format!("data:image/png;base64,{}",STANDARD.encode(output.stdout));total+=payload.len();if total>MAX_PAGE_IMAGE_TOTAL_BYTES{return Err(invalid("所选整页图总量超过12 MiB，未发送任何页；请缩小页范围后重试"));}images.push(payload);
 }
 Ok(images)
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn poppler_renders_page_to_valid_model_payload_without_files(){
  let request=Request{pdf:std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/paper-pages.pdf"),pages:vec![1]};
  let (_tx,rx)=watch::channel(false);let images=render(&request,&rx).unwrap();assert_eq!(images.len(),1);let bytes=STANDARD.decode(images[0].strip_prefix("data:image/png;base64,").unwrap()).unwrap();assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));validate_page_png(&bytes).unwrap();
 }
 #[test]fn cancelled_render_never_opens_a_document(){let (_tx,rx)=watch::channel(true);assert!(render(&Request{pdf:PathBuf::from("missing.pdf"),pages:vec![1]},&rx).is_err());}
 #[test]fn cancellation_stops_an_active_renderer_process(){
  let (tx,rx)=watch::channel(false);let canceller=std::thread::spawn(move||{std::thread::sleep(Duration::from_millis(50));let _=tx.send(true);std::thread::sleep(Duration::from_millis(100));});
  let began=Instant::now();let result=ProcessRunner::run_cancellable(ProcessSpec{program:"/bin/sh".into(),args:vec!["-c".into(),"sleep 5".into()],deadline:Instant::now()+Duration::from_secs(8),output_limit:1024},||*rx.borrow()||rx.has_changed().is_err());
  assert!(result.is_err());assert!(began.elapsed()<Duration::from_secs(2));canceller.join().unwrap();
 }

}
