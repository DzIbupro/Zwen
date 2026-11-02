//! Owned watcher process. No clipboard access occurs in its synthetic tests.
use std::process::{Child,Command};
#[derive(Default)]
pub struct Watcher { child:Option<Child>, generation:Option<String> }
impl Watcher {
 pub const fn new()->Self{Self{child:None,generation:None}}
 pub fn enabled(&mut self)->bool{
  if let Some(child)=self.child.as_mut(){if matches!(child.try_wait(),Ok(None)){return true;}}
  self.stop();false
 }
 pub fn accepts(&mut self,token:Option<&str>)->bool{self.enabled()&&token.is_some()&&token==self.generation.as_deref()}
 pub fn start(&mut self,command:&mut Command,token:String)->std::io::Result<()>{
  if self.enabled(){return Ok(());}
  #[cfg(target_os="linux")]{
   use std::os::unix::process::CommandExt;
   let parent=std::process::id() as libc::pid_t;
   // SAFETY: the child closure only makes async-signal-safe syscalls; it
   // allocates nothing and acquires no locks between fork and exec. The
   // post-registration check closes the race where the parent already died.
   unsafe{command.pre_exec(move||{
    if libc::prctl(libc::PR_SET_PDEATHSIG,libc::SIGKILL)!=0{libc::_exit(127);}
    if libc::getppid()!=parent{libc::_exit(127);}
    Ok(())
   });}
  }
  let child=command.spawn()?;self.child=Some(child);self.generation=Some(token);Ok(())
 }
 pub fn stop(&mut self){self.generation=None;if let Some(mut child)=self.child.take(){let _=child.kill();let _=child.wait();}}
}
impl Drop for Watcher{fn drop(&mut self){self.stop();}}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn single_child_and_generation_revocation(){
  let mut watcher=Watcher::new();let mut command=Command::new("sleep");command.arg("30");
  watcher.start(&mut command,"first".into()).unwrap();let pid=watcher.child.as_ref().unwrap().id();
  watcher.start(&mut command,"second".into()).unwrap();assert_eq!(watcher.child.as_ref().unwrap().id(),pid);
  assert!(watcher.accepts(Some("first")));assert!(!watcher.accepts(Some("second")));assert!(!watcher.accepts(None));
  watcher.stop();
  #[cfg(target_os="linux")] assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists(),"child must be reaped");
  assert!(!watcher.accepts(Some("first")));assert!(watcher.child.is_none());
  watcher.start(&mut command,"third".into()).unwrap();assert!(!watcher.accepts(Some("first")));assert!(watcher.accepts(Some("third")));
 }
 #[test]fn failed_spawn_never_enables(){let mut watcher=Watcher::new();assert!(watcher.start(&mut Command::new("/zwen-no-such-helper"),"x".into()).is_err());assert!(!watcher.enabled());}
 #[cfg(target_os="linux")]
 #[test]fn parent_death_fixture(){
  if std::env::var_os("ZWEN_SYNTHETIC_PARENT_FIXTURE").is_none(){return;}
  let mut watcher=Watcher::new();watcher.start(Command::new("sleep").arg("30"),"fixture".into()).unwrap();
  println!("WATCHER_PID={}",watcher.child.as_ref().unwrap().id());
  use std::io::Write;std::io::stdout().flush().unwrap();
  std::thread::sleep(std::time::Duration::from_secs(30));
 }
 #[cfg(target_os="linux")]
 #[test]fn abnormal_parent_exit_stops_child(){
  use std::{io::{BufRead,BufReader},process::Stdio,time::{Duration,Instant}};
  let module=module_path!().split_once("::").unwrap().1;
  let fixture=format!("{module}::parent_death_fixture");
  let mut parent=Command::new(std::env::current_exe().unwrap()).args(["--exact",&fixture,"--nocapture"])
   .env("ZWEN_SYNTHETIC_PARENT_FIXTURE","1").stdout(Stdio::piped()).spawn().unwrap();
  let output=parent.stdout.take().unwrap();let mut reader=BufReader::new(output);let mut line=String::new();
  let pid=loop{line.clear();assert!(reader.read_line(&mut line).unwrap()>0);if let Some(value)=line.split("WATCHER_PID=").nth(1){break value.trim().parse::<u32>().unwrap();}};
  parent.kill().unwrap();parent.wait().unwrap();let deadline=Instant::now()+Duration::from_secs(2);
  let stopped=loop{
   let state=std::fs::read_to_string(format!("/proc/{pid}/stat"));
   if state.as_ref().is_err()||state.is_ok_and(|text|text.split(')').nth(1).is_some_and(|tail|tail.trim_start().starts_with('Z'))){break true;}
   if Instant::now()>=deadline{break false;}std::thread::sleep(Duration::from_millis(10));
  };
  if !stopped{let _=Command::new("kill").args(["-KILL",&pid.to_string()]).status();}
  assert!(stopped,"watcher survived abrupt parent death");
 }

}
