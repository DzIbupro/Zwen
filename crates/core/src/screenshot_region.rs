//! Owned region selection. Waiting for the person has no production deadline.
use super::{CaptureError,cleanup};
use std::{fs,io::{Read,ErrorKind},path::Path,process::{Child,Command,Stdio},thread,time::{Duration,Instant}};
use std::{sync::Mutex,collections::BTreeSet};
static CHILDREN:Mutex<(bool,BTreeSet<u32>)>=Mutex::new((false,BTreeSet::new()));
pub fn shutdown_capture_processes(){if let Ok(mut children)=CHILDREN.lock(){
 children.0=true;for id in &children.1{#[cfg(unix)]unsafe{libc::kill(-(*id as libc::pid_t),libc::SIGKILL);}}
 #[cfg(target_os="linux")]{
  let began=Instant::now();loop{
   let ended=children.1.iter().all(|id|{let mut info=std::mem::MaybeUninit::<libc::siginfo_t>::zeroed();let result=unsafe{libc::waitid(libc::P_PID,*id,info.as_mut_ptr(),libc::WEXITED|libc::WNOHANG|libc::WNOWAIT)};result<0||unsafe{info.assume_init().si_pid()}!=0});
   if ended||began.elapsed()>Duration::from_millis(500){break;}thread::sleep(Duration::from_millis(5));
  }
 }
}}
const GRAB_TIMEOUT:Duration=Duration::from_secs(5);
struct Owned{child:Child,id:u32}
impl Owned{
 fn exited(&mut self)->Result<Option<i32>,CaptureError>{
  #[cfg(target_os="linux")]{
   // Observe without reaping: retaining the leader prevents its PID/PGID from
   // being reused before Drop terminates the owned process group.
   let mut info=std::mem::MaybeUninit::<libc::siginfo_t>::zeroed();
   let result=unsafe{libc::waitid(libc::P_PID,self.id,info.as_mut_ptr(),libc::WEXITED|libc::WNOHANG|libc::WNOWAIT)};
   if result<0{return Err(CaptureError::Io);}let info=unsafe{info.assume_init()};
   return Ok((unsafe{info.si_pid()}!=0).then(||if info.si_code==libc::CLD_EXITED{unsafe{info.si_status()}}else{-1}));
  }
  #[cfg(not(target_os="linux"))]{Ok(self.child.try_wait().map_err(|_|CaptureError::Io)?.map(|s|s.code().unwrap_or(-1)))}
 }
 fn spawn(command:&mut Command)->Result<Self,CaptureError>{
  let mut children=CHILDREN.lock().map_err(|_|CaptureError::Io)?;if children.0{return Err(CaptureError::Cancelled);}
  #[cfg(unix)]{use std::os::unix::process::CommandExt;
   #[cfg(target_os="linux")]let parent=std::process::id();
   // SAFETY: child setup only uses async-signal-safe syscalls before exec.
   unsafe{command.pre_exec(move||{
    if libc::setsid()<0{return Err(std::io::Error::last_os_error());}
    #[cfg(target_os="linux")]{if libc::prctl(libc::PR_SET_PDEATHSIG,libc::SIGKILL)!=0{return Err(std::io::Error::last_os_error());}if libc::getppid()!=parent as libc::pid_t{libc::_exit(127);}}
    Ok(())
   });}
  }
  let child=command.stdin(Stdio::null()).stderr(Stdio::null()).spawn().map_err(|_|CaptureError::Unavailable)?;let id=child.id();children.1.insert(id);Ok(Self{child,id})
 }
}
impl Drop for Owned{fn drop(&mut self){
 let mut children=CHILDREN.lock().ok();
 #[cfg(unix)]unsafe{libc::kill(-(self.id as libc::pid_t),libc::SIGKILL);}
 let _=self.child.kill();let _=self.child.wait();if let Some(children)=children.as_mut(){children.1.remove(&self.id);}
}}
fn geometry(raw:&[u8])->Option<String>{
 let raw=std::str::from_utf8(raw).ok()?.trim();let (position,size)=raw.split_once(' ')?;let (x,y)=position.split_once(',')?;let (w,h)=size.split_once('x')?;
 let x=x.parse::<i32>().ok()?;let y=y.parse::<i32>().ok()?;let w=w.parse::<u32>().ok()?;let h=h.parse::<u32>().ok()?;
 if w==0||h==0||w>32768||h>32768||u64::from(w)*u64::from(h)>64*1024*1024{return None;}
 Some(format!("{x},{y} {w}x{h}"))
}
#[cfg(unix)]
pub fn capture_region_with(selector:&Path,grabber:&Path,path:&Path,user_timeout:Option<Duration>,cancelled:impl Fn()->bool)->Result<(),CaptureError>{
 use std::os::fd::AsRawFd;
 if cancelled(){return Err(CaptureError::Cancelled);}if !path.is_absolute(){return Err(CaptureError::CommandFailed);}
 fs::create_dir_all(path.parent().ok_or(CaptureError::Io)?).map_err(|_|CaptureError::Io)?;
 let result=(||{
  let mut selector_cmd=Command::new(selector);selector_cmd.args(["-f","%x,%y %wx%h"]).stdout(Stdio::piped());
  let mut selected=Owned::spawn(&mut selector_cmd)?;let mut output=selected.child.stdout.take().ok_or(CaptureError::Io)?;let fd=output.as_raw_fd();
  let flags=unsafe{libc::fcntl(fd,libc::F_GETFL)};
  if flags<0||unsafe{libc::fcntl(fd,libc::F_SETFL,flags|libc::O_NONBLOCK)}<0{return Err(CaptureError::Io);}
  let began=Instant::now();let mut bytes=Vec::new();
  loop{
   if cancelled(){return Err(CaptureError::Cancelled);}
   if user_timeout.is_some_and(|limit|began.elapsed()>=limit){return Err(CaptureError::Timeout);}
   let mut drained=false;let mut buffer=[0;256];match output.read(&mut buffer){Ok(0)=>drained=true,Ok(n)=>{if bytes.len()+n>512{return Err(CaptureError::CommandFailed);}bytes.extend_from_slice(&buffer[..n]);},Err(e) if e.kind()==ErrorKind::WouldBlock=>drained=true,Err(_)=>return Err(CaptureError::Io)}
   if let Some(status)=selected.exited()?{
    if status!=0{return Err(if status==1{CaptureError::Cancelled}else{CaptureError::CommandFailed});}
    if drained{break;}
   }
   thread::sleep(Duration::from_millis(15));
  }
  let region=geometry(&bytes).ok_or(CaptureError::CommandFailed)?;drop(selected);
  if cancelled(){return Err(CaptureError::Cancelled);}
  let mut grab_cmd=Command::new(grabber);grab_cmd.arg("-g").arg(region).arg(path).stdout(Stdio::null());let mut grab=Owned::spawn(&mut grab_cmd)?;let began=Instant::now();
  loop{
   if cancelled(){return Err(CaptureError::Cancelled);}
   if let Some(status)=grab.exited()?{if status!=0{return Err(CaptureError::CommandFailed);}break;}
   if began.elapsed()>=GRAB_TIMEOUT{return Err(CaptureError::CommandFailed);}thread::sleep(Duration::from_millis(15));
  }
  if cancelled(){return Err(CaptureError::Cancelled);}
  if !fs::symlink_metadata(path).is_ok_and(|m|m.is_file()&&m.len()>0){return Err(CaptureError::Empty);}Ok(())
 })();
 if result.is_err(){cleanup(path);}result
}
#[cfg(not(unix))]
pub fn capture_region_with(_: &Path,_:&Path,_:&Path,_:Option<Duration>,_:impl Fn()->bool)->Result<(),CaptureError>{Err(CaptureError::Unavailable)}
