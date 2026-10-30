#![cfg(unix)]
use std::{fs,path::{Path,PathBuf},sync::{Arc,atomic::{AtomicBool,Ordering}},thread,time::{Duration,Instant}};
use zwen_core::screenshot::{capture_with,cleanup_incoming,screenshot_path,CaptureError};

static NEXT:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch{
 fn new()->Self{let path=std::env::temp_dir().join(format!("zwen-capture-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));fs::create_dir(&path).unwrap();Self(path)}
}
impl Drop for Scratch{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}

const SCRIPT:&str=r#"#!/usr/bin/python3
import pathlib,shutil,subprocess,sys,time
root=pathlib.Path(__file__).resolve().parent.parent
mode=(root/'mode').read_text().strip() if (root/'mode').exists() else 'success'
args=sys.argv[1:]
if args[:2]==['msg','event-stream']:
    print('{"ConfigLoaded":{}}',flush=True)
    time.sleep(30)
    sys.exit(0)
if args[:3]==['msg','action','screenshot']:
    if mode=='fail':sys.exit(1)
    path=args[-1]
    if mode=='late':
        subprocess.Popen([sys.executable,'-c','import shutil,sys,time;time.sleep(1.2);shutil.copyfile(sys.argv[1],sys.argv[2])',str(root/'image.png'),path],start_new_session=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        sys.exit(0)
    if mode=='slow':time.sleep(0.8)
    shutil.copyfile(root/'image.png',path)
    sys.exit(0)
sys.exit(2)
"#;

fn fixture(root:&Path,mode:&str)->PathBuf{
 let bin=root.join("bin");fs::create_dir_all(&bin).unwrap();
 let program=bin.join("niri");fs::write(&program,SCRIPT).unwrap();
 #[cfg(unix)]{use std::os::unix::fs::PermissionsExt;fs::set_permissions(&program,fs::Permissions::from_mode(0o700)).unwrap();}
 fs::write(root.join("mode"),mode).unwrap();
 let mut image=Vec::new();
 {let mut encoder=png::Encoder::new(&mut image,64,48);encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);let mut writer=encoder.write_header().unwrap();writer.write_image_data(&vec![0x4eu8;64*48*4]).unwrap();writer.finish().unwrap();}
 fs::write(root.join("image.png"),image).unwrap();
 program
}

fn incoming(root:&Path,id:&str)->PathBuf{
 let directory=root.join("attachments").join("incoming");fs::create_dir_all(&directory).unwrap();
 directory.join(format!("{id}.png"))
}

#[test] fn successful_capture_uses_the_exact_request_path(){
 let scratch=Scratch::new();let program=fixture(&scratch.0,"success");let path=incoming(&scratch.0,&"a".repeat(32));
 assert_eq!(capture_with(&program,&path,Duration::from_millis(2000),||false),Ok(()));
 assert!(path.is_file());assert!(fs::metadata(&path).unwrap().len()>0);
}

#[test] fn cancel_discards_its_own_file_and_reports_cancel(){
 let scratch=Scratch::new();let program=fixture(&scratch.0,"slow");let path=incoming(&scratch.0,&"b".repeat(32));
 let cancelled=Arc::new(AtomicBool::new(false));let flag=cancelled.clone();
 let worker=thread::spawn(move||{thread::sleep(Duration::from_millis(120));flag.store(true,Ordering::SeqCst);});
 let began=Instant::now();
 let result=capture_with(&program,&path,Duration::from_millis(2000),||cancelled.load(Ordering::SeqCst));
 worker.join().unwrap();
 assert_eq!(result,Err(CaptureError::Cancelled));
 assert!(began.elapsed()<Duration::from_secs(2));
 assert!(!path.exists());
}

#[test] fn late_result_after_timeout_is_cleaned_and_never_returned(){
 let scratch=Scratch::new();let program=fixture(&scratch.0,"late");let path=incoming(&scratch.0,&"c".repeat(32));
 assert_eq!(capture_with(&program,&path,Duration::from_millis(250),||false),Err(CaptureError::Timeout));
 assert!(!path.exists());
 thread::sleep(Duration::from_millis(1500));
 assert!(path.exists(),"fixture should write late");
 let directory=scratch.0.join("attachments").join("incoming");
 assert_eq!(cleanup_incoming(&directory,Duration::ZERO),1);
 assert!(!path.exists());
}

#[test] fn failed_capture_command_is_not_treated_as_success(){
 let scratch=Scratch::new();let program=fixture(&scratch.0,"fail");let path=incoming(&scratch.0,&"d".repeat(32));
 assert_eq!(capture_with(&program,&path,Duration::from_millis(500),||false),Err(CaptureError::CommandFailed));
 assert!(!path.exists());
}

#[test] fn missing_helper_is_distinct_from_a_failed_capture(){
 let scratch=Scratch::new();let path=incoming(&scratch.0,&"e".repeat(32));
 assert_eq!(capture_with(Path::new("/zwen-nonexistent-niri"),&path,Duration::from_millis(200),||false),Err(CaptureError::Unavailable));
}

#[test] fn event_parser_requires_a_string_path(){
 assert_eq!(screenshot_path(r#"{"ScreenshotCaptured":{"path":"/tmp/a.png"}}"#).as_deref(),Some("/tmp/a.png"));
 assert_eq!(screenshot_path(r#"{"ScreenshotCaptured":{"path":null}}"#),None);
 assert_eq!(screenshot_path(r#"{"ConfigLoaded":{"failed":false}}"#),None);
 assert_eq!(screenshot_path("not json"),None);
}

#[test] fn cleanup_only_removes_our_request_names(){
 let scratch=Scratch::new();let directory=scratch.0.join("incoming");fs::create_dir_all(&directory).unwrap();
 let ours=directory.join(format!("{}.png","f".repeat(32)));fs::write(&ours,b"x").unwrap();
 let other=directory.join("keep.png");fs::write(&other,b"x").unwrap();
 let short=directory.join("abc.png");fs::write(&short,b"x").unwrap();
 assert_eq!(cleanup_incoming(&directory,Duration::ZERO),1);
 assert!(!ours.exists());assert!(other.exists());assert!(short.exists());
}
