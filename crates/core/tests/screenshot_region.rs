#![cfg(unix)]
use std::{fs,path::PathBuf,time::{Duration,Instant}};
use zwen_core::screenshot::{capture_region_with,CaptureError};
struct Scratch(PathBuf);impl Scratch{fn new()->Self{let p=std::env::temp_dir().join(format!("zwen-region-{}-{}",std::process::id(),zwen_core::notes::generate_id()));fs::create_dir(&p).unwrap();Self(p)}fn script(&self,name:&str,body:&str)->PathBuf{use std::os::unix::fs::PermissionsExt;let p=self.0.join(name);fs::write(&p,format!("#!/bin/sh\n{body}\n")).unwrap();fs::set_permissions(&p,fs::Permissions::from_mode(0o700)).unwrap();p}}impl Drop for Scratch{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
#[test]fn selection_wait_has_no_user_timeout_and_save_uses_validated_geometry(){let s=Scratch::new();let select=s.script("select","sleep .35; printf -- '-20,10 40x30\\n'");let grab=s.script("grab","test \"$1\" = '-g' && test \"$2\" = '-20,10 40x30' || exit 2; printf 'image' > \"$3\"");let target=s.0.join("image.png");assert_eq!(capture_region_with(&select,&grab,&target,None,||false),Ok(()));assert_eq!(fs::read(&target).unwrap(),b"image");}
#[test]fn escape_or_bad_geometry_never_launches_grabber(){let s=Scratch::new();let grab=s.script("grab","exit 99");let target=s.0.join("image.png");let cancel=s.script("cancel","exit 1");assert_eq!(capture_region_with(&cancel,&grab,&target,None,||false),Err(CaptureError::Cancelled));for text in ["0,0 0x3","0,0 3x-1","--help","0,0 100000x100000","0,0 3x3 extra"]{let select=s.script("select",&format!("printf '%s' '{text}'"));assert_eq!(capture_region_with(&select,&grab,&target,None,||false),Err(CaptureError::CommandFailed));}}
#[test]fn cancelling_selector_reaps_its_inherited_pipe_descendants(){let s=Scratch::new();let select=s.script("select","sleep 30 & echo $! > \"$0.pid\"; wait");let grab=s.script("grab","exit 99");let began=Instant::now();assert_eq!(capture_region_with(&select,&grab,&s.0.join("image.png"),None,||began.elapsed()>Duration::from_millis(180)),Err(CaptureError::Cancelled));assert!(began.elapsed()<Duration::from_secs(2));let pid=fs::read_to_string(select.with_extension("pid")).unwrap();let proc=PathBuf::from(format!("/proc/{}/stat",pid.trim()));assert!(fs::read_to_string(proc).map_or(true,|s|s.split(')').nth(1).unwrap_or("").trim_start().starts_with('Z')));}

#[cfg(target_os="linux")]
#[test]fn shutdown_fixture(){
 let Ok(root)=std::env::var("ZWEN_SHUTDOWN_TEST_ROOT")else{return;};let root=PathBuf::from(root);let select=root.join("select");let target=root.join("capture.png");
 let worker=std::thread::spawn(move||capture_region_with(&select,&root.join("grab"),&target,None,||false));
 let path=PathBuf::from(std::env::var("ZWEN_SHUTDOWN_TEST_ROOT").unwrap()).join("select.pid");let began=Instant::now();while !path.exists()&&began.elapsed()<Duration::from_secs(2){std::thread::sleep(Duration::from_millis(10));}assert!(path.exists());
 zwen_core::screenshot::shutdown_capture_processes();assert!(worker.join().unwrap().is_err());
 let root=PathBuf::from(std::env::var("ZWEN_SHUTDOWN_TEST_ROOT").unwrap());assert_eq!(capture_region_with(&root.join("select"),&root.join("grab"),&root.join("again.png"),None,||false),Err(CaptureError::Cancelled));
}
#[cfg(target_os="linux")]
#[test]fn app_shutdown_reaps_selector_group_and_prevents_new_work(){
 let s=Scratch::new();s.script("select","echo $$ > \"$0.leader\"; sleep 30 & echo $! > \"$0.pid\"; wait");s.script("grab","exit 99");
 let result=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","shutdown_fixture"]).env("ZWEN_SHUTDOWN_TEST_ROOT",&s.0).output().unwrap();assert!(result.status.success(),"{}",String::from_utf8_lossy(&result.stderr));
 for file in ["select.pid","select.leader"]{let pid=fs::read_to_string(s.0.join(file)).unwrap();let path=PathBuf::from(format!("/proc/{}/stat",pid.trim()));assert!(fs::read_to_string(path).map_or(true,|s|s.split(')').nth(1).unwrap_or("").trim_start().starts_with('Z')));}
}
#[test]fn cancelling_writer_discards_partial_file_and_reaps_group(){let s=Scratch::new();let select=s.script("select","printf '0,0 40x30'");let grab=s.script("grab","echo started > \"$0.started\"; printf partial > \"$3\"; sleep 30 & echo $! > \"$0.pid\"; wait");let path=s.0.join("image.png");let began=Instant::now();assert_eq!(capture_region_with(&select,&grab,&path,None,||began.elapsed()>Duration::from_millis(200)),Err(CaptureError::Cancelled));assert!(grab.with_extension("started").exists());assert!(!path.exists());assert!(began.elapsed()<Duration::from_secs(2));}
#[test]
#[ignore = "Explicit 61-second regression for indefinite human selection"]
fn human_selection_can_exceed_sixty_seconds(){let s=Scratch::new();let select=s.script("select","sleep 61.2; printf '0,0 40x30'");let grab=s.script("grab","printf image > \"$3\"");let began=Instant::now();assert_eq!(capture_region_with(&select,&grab,&s.0.join("image.png"),None,||false),Ok(()));assert!(began.elapsed()>Duration::from_secs(60));}
