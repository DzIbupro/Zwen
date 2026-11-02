#![cfg(unix)]
use std::{process::Command,time::{Duration,Instant}};
use zwen_core::selection::{read_from,SelectionError,Inbox,Mode,MAX_SELECTION_BYTES};
fn run(script:&str,ms:u64)->Result<String,SelectionError>{let mut c=Command::new("/usr/bin/python3");c.args(["-c",script]);read_from(c,Duration::from_millis(ms))}
#[test] fn exact_limit_and_unicode_are_preserved(){assert_eq!(run("import os;os.write(1,('中文\\n'+'x'*65529).encode())",750).unwrap().len(),65536);assert_eq!(MAX_SELECTION_BYTES,65536);}
#[test] fn oversized_output_is_rejected_not_truncated(){assert_eq!(run("import os;os.write(1,b'x'*65537)",750),Err(SelectionError::TooLarge));}
#[test] fn invalid_empty_and_failed_outputs_are_never_accepted(){for s in ["import os;os.write(1,b'\\xff')","print('   ')","import os;os.write(1,b'a\\x00b')"]{assert_eq!(run(s,750),Err(SelectionError::InvalidText));}assert_eq!(run("print('old');raise SystemExit(1)",750),Err(SelectionError::Unavailable));}
#[test] fn partial_output_and_closed_stdout_do_not_bypass_deadline(){for s in ["import os,time;os.write(1,b'partial');time.sleep(5)","import os,time;os.close(1);time.sleep(5)"]{let t=Instant::now();assert_eq!(run(s,80),Err(SelectionError::Timeout));assert!(t.elapsed()<Duration::from_secs(1));}}
#[test] fn inherited_pipe_does_not_leave_a_blocked_reader(){let t=Instant::now();assert_eq!(run("import os,time;pid=os.fork();time.sleep(.25) if pid==0 else None;os._exit(0)",80),Err(SelectionError::Timeout));assert!(t.elapsed()<Duration::from_millis(240));}
#[test] fn missing_helper_is_distinct_and_stderr_is_not_returned(){assert_eq!(read_from(Command::new("/zwen-nonexistent-helper"),Duration::from_millis(50)),Err(SelectionError::MissingHelper));let err=run("import sys;print('private fixture',file=sys.stderr);sys.exit(1)",750).unwrap_err();assert!(!err.to_string().contains("private"));}
#[test] fn inbox_never_overwrites_unacknowledged_material_and_claims_once(){let mut b=Inbox::default();let id=b.begin(Mode::Explain).unwrap();assert!(b.begin(Mode::Translate).is_none());assert!(b.claim().is_none());assert!(!b.finish(id+1,Ok("late".into())));assert!(b.finish(id,Ok("selected".into())));let first=b.claim().unwrap();assert!(!first.claimed);assert_eq!(first.text.as_deref(),Some("selected"));assert!(b.claim().unwrap().claimed);assert!(!b.acknowledge(id+1));assert!(b.begin(Mode::Translate).is_none());assert!(b.acknowledge(id));let next=b.begin(Mode::Translate).unwrap();assert!(next>id);assert!(!b.finish(id,Ok("late".into())));}
#[test] fn failed_capture_is_a_safe_claimable_notice(){let mut b=Inbox::default();let id=b.begin(Mode::Raw).unwrap();assert!(b.finish(id,Err(SelectionError::Timeout)));let s=b.claim().unwrap();assert!(s.text.is_none());assert!(s.error.unwrap().contains("超时"));assert!(b.acknowledge(id));}
#[test] fn selection_cli_actions_are_explicit(){use zwen_core::cli::{parse_action,LaunchAction};for (s,m) in [("select",Mode::Raw),("explain-selection",Mode::Explain),("translate-selection",Mode::Translate)]{assert_eq!(parse_action(&[s]),Ok(LaunchAction::Selection(m)));}assert!(parse_action(&["select","private text"]).is_err());}
#[test] fn pending_capture_must_block_exit_until_explicit_acknowledgement(){let mut b=Inbox::default();assert!(!b.is_pending());let id=b.begin(Mode::Raw).unwrap();assert!(b.is_pending());assert!(!b.acknowledge(id));b.finish(id,Ok("text".into()));assert!(b.is_pending());b.claim();b.acknowledge(id);assert!(!b.is_pending());}

#[test]fn revoked_read_cannot_deliver_or_clear_another_capture(){
 let mut inbox=Inbox::default();let old=inbox.begin(Mode::Candidate).unwrap();
 assert!(!inbox.cancel_read(old+1));assert!(inbox.reading());assert!(inbox.cancel_read(old));
 assert!(!inbox.is_pending());assert!(!inbox.finish(old,Ok("revoked private text".into())));
 let new=inbox.begin(Mode::Raw).unwrap();assert!(!inbox.cancel_read(old));assert!(inbox.finish(new,Ok("explicit text".into())));
 assert!(!inbox.cancel_read(new),"completed material requires explicit acknowledgement");
 assert_eq!(inbox.claim().unwrap().text.as_deref(),Some("explicit text"));
}
