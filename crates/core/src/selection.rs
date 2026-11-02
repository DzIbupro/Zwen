//! Explicit PRIMARY capture. Never reads CLIPBOARD or echoes helper diagnostics.
use serde::Serialize;
use std::{fmt,process::{Command,Stdio},time::{Duration,Instant}};
pub const MAX_SELECTION_BYTES:usize=64*1024;
const TIMEOUT:Duration=Duration::from_millis(750);
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum SelectionError {MissingHelper,Unavailable,Timeout,TooLarge,InvalidText}
impl fmt::Display for SelectionError {
 fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{f.write_str(match self{
  Self::MissingHelper=>"未找到wl-paste，请安装wl-clipboard后再试",
  Self::Unavailable=>"未能读取PRIMARY选区；可手动复制并粘贴，原草稿已保留",
  Self::Timeout=>"读取选区超时，原草稿已保留，请重新选择后再试",
  Self::TooLarge=>"选区超过64KiB，请缩小范围，原草稿已保留",
  Self::InvalidText=>"没有可用的UTF-8文字选区，原草稿已保留",
 })}
}
impl std::error::Error for SelectionError{}
fn validate(bytes:Vec<u8>)->Result<String,SelectionError>{
 if bytes.len()>MAX_SELECTION_BYTES{return Err(SelectionError::TooLarge);}
 let text=String::from_utf8(bytes).map_err(|_|SelectionError::InvalidText)?;
 if text.trim().is_empty()||text.contains('\0'){return Err(SelectionError::InvalidText);}
 Ok(text)
}
pub fn read_primary_text()->Result<String,SelectionError>{
 let mut command=Command::new("wl-paste");command.args(["--primary","--no-newline","--type","text"]);read_from(command,TIMEOUT)
}
/// Read one trusted helper invocation, with the same byte/time ceiling as PRIMARY.
/// The desktop exposes no command/arguments input to its WebView.
#[cfg(unix)]
pub fn read_from(mut command:Command,timeout:Duration)->Result<String,SelectionError>{
 use std::{io::{Read,ErrorKind},os::fd::AsRawFd};
 struct Guard(std::process::Child);
 impl Drop for Guard{fn drop(&mut self){let _=self.0.kill();let _=self.0.wait();}}
 let began=Instant::now();let timeout=timeout.min(TIMEOUT);
 let child=command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().map_err(|e|if e.kind()==ErrorKind::NotFound{SelectionError::MissingHelper}else{SelectionError::Unavailable})?;
 let mut child=Guard(child);let mut pipe=child.0.stdout.take().ok_or(SelectionError::Unavailable)?;
 let fd=pipe.as_raw_fd();
 // SAFETY: fd remains owned by pipe for both fcntl calls; flags affect only this pipe.
 let flags=unsafe{libc::fcntl(fd,libc::F_GETFL)};
 if flags<0||unsafe{libc::fcntl(fd,libc::F_SETFL,flags|libc::O_NONBLOCK)}<0{return Err(SelectionError::Unavailable);}
 let mut bytes=Vec::new();let mut eof=false;
 loop {
  if began.elapsed()>=timeout{return Err(SelectionError::Timeout);}
  let mut buffer=[0u8;8192];
  match pipe.read(&mut buffer){
   Ok(0)=>eof=true,
   Ok(n)=>{if bytes.len()+n>MAX_SELECTION_BYTES{return Err(SelectionError::TooLarge);}bytes.extend_from_slice(&buffer[..n]);continue;}
   Err(e) if e.kind()==ErrorKind::WouldBlock=>(),
   Err(e) if e.kind()==ErrorKind::Interrupted=>continue,
   Err(_)=>return Err(SelectionError::Unavailable),
  }
  match child.0.try_wait().map_err(|_|SelectionError::Unavailable)?{
   Some(status) if !status.success()=>return Err(SelectionError::Unavailable),
   Some(_) if eof=>return validate(bytes),
   _=>std::thread::sleep(Duration::from_millis(4)),
  }
 }
}
#[cfg(not(unix))]
pub fn read_from(_:Command,_:Duration)->Result<String,SelectionError>{Err(SelectionError::Unavailable)}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize)]
#[serde(rename_all="lowercase")]
pub enum Mode{Raw,Explain,Translate,Candidate}
#[derive(Clone,Serialize)]
pub struct Item{pub id:u32,pub mode:Mode,pub text:Option<String>,pub error:Option<String>,pub claimed:bool}
#[derive(Default)]
pub struct Inbox{next:u32,reading:bool,item:Option<Item>}
impl Inbox{
 pub fn is_pending(&self)->bool{self.item.is_some()}
 pub fn reading(&self)->bool{self.reading}
 pub fn begin(&mut self,mode:Mode)->Option<u32>{
  if self.item.is_some(){return None;}
  self.next=self.next.checked_add(1)?;self.reading=true;
  self.item=Some(Item{id:self.next,mode,text:None,error:None,claimed:false});Some(self.next)
 }
 /// Revoke only an unfinished matching capture; never discard another item.
 pub fn cancel_read(&mut self,id:u32)->bool{
  if !self.reading||!self.item.as_ref().is_some_and(|item|item.id==id){return false;}
  self.item=None;self.reading=false;true
 }
 pub fn finish(&mut self,id:u32,result:Result<String,SelectionError>)->bool{
  let Some(item)=self.item.as_mut().filter(|s|s.id==id&&self.reading)else{return false;};
  match result.and_then(|text|validate(text.into_bytes())){Ok(text)=>item.text=Some(text),Err(e)=>item.error=Some(e.to_string())};
  self.reading=false;true
 }
 pub fn claim(&mut self)->Option<Item>{if self.reading{return None;}let item=self.item.as_mut()?;let result=item.clone();item.claimed=true;Some(result)}
 pub fn acknowledge(&mut self,id:u32)->bool{if self.reading||!self.item.as_ref().is_some_and(|s|s.id==id&&s.claimed){return false;}self.item=None;true}
}
