//! Explicit Niri screenshot capture. Every request owns one unique absolute path.
//! A saved file is attributed only to the request that named it; cancel, expiry
//! and late results discard that exact path instead of attaching it elsewhere.
use std::{fmt,fs,io::{BufRead,BufReader},path::{Path,PathBuf},process::{Child,Command,Stdio},sync::mpsc,thread,time::{Duration,Instant,SystemTime}};

pub const DEFAULT_CAPTURE_TIMEOUT:Duration=Duration::from_secs(60);
const ACTION_TIMEOUT:Duration=Duration::from_secs(3);
const SUBSCRIBE_TIMEOUT:Duration=Duration::from_millis(1500);
const POLL_INTERVAL:Duration=Duration::from_millis(100);
const STABLE_FOR:Duration=Duration::from_millis(250);
const MAX_EVENT_LINE:usize=4*1024*1024;
pub const INCOMING_MAX_AGE:Duration=Duration::from_secs(10*60);

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum CaptureError{Unavailable,CommandFailed,Timeout,Cancelled,Empty,Io}
impl fmt::Display for CaptureError{
 fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{f.write_str(match self{
  Self::Unavailable=>"当前桌面不支持 Niri 截图；未附加图片，原草稿已保留",
  Self::CommandFailed=>"Niri 未能完成截图请求；未附加图片，原草稿已保留",
  Self::Timeout=>"等待截图超时（可能已取消）；未附加图片，原草稿已保留",
  Self::Cancelled=>"已取消等待截图；若 Niri 截图界面仍打开，可按 Esc 关闭",
  Self::Empty=>"截图文件为空或尚未写入；未附加图片，原草稿已保留",
  Self::Io=>"截图文件无法读取；未附加图片，原草稿已保留",
 })}}
impl std::error::Error for CaptureError{}

enum Event{Ready,Captured}
struct Guard(Child);
impl Guard{
 fn stop(&mut self){let _=self.0.kill();let _=self.0.wait();}
}
impl Drop for Guard{fn drop(&mut self){self.stop();}}

pub fn capture(path:&Path,timeout:Duration,cancelled:impl Fn()->bool)->Result<(),CaptureError>{
 if std::env::var_os("NIRI_SOCKET").is_none(){return Err(CaptureError::Unavailable);}
 capture_with(Path::new("niri"),path,timeout,cancelled)
}

/// Test seam: the production path always calls the fixed `niri` binary.
#[doc(hidden)]
pub fn capture_with(program:&Path,path:&Path,timeout:Duration,cancelled:impl Fn()->bool)->Result<(),CaptureError>{
 if cancelled(){return Err(CaptureError::Cancelled);}
 if !path.is_absolute(){return Err(CaptureError::CommandFailed);}
 let parent=path.parent().ok_or(CaptureError::Io)?;
 fs::create_dir_all(parent).map_err(|_|CaptureError::Io)?;
 // Subscribe before requesting the capture so an immediate save cannot be missed.
 let mut stream=Command::new(program).args(["msg","event-stream"]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().map_err(|_|CaptureError::Unavailable)?;
 let stdout=stream.stdout.take().ok_or(CaptureError::Io)?;
 let mut guard=Guard(stream);
 let (sender,receiver)=mpsc::channel();
 let target=path.to_string_lossy().into_owned();
 let _reader=thread::spawn(move||{
  let mut reader=BufReader::new(stdout);let mut line=Vec::new();let mut announced=false;
  loop{
   match read_line_bounded(&mut reader,&mut line,MAX_EVENT_LINE){
    Ok(0)=>break,
    Ok(_)=>{
     if !announced{announced=true;let _=sender.send(Event::Ready);}
     if screenshot_path(&String::from_utf8_lossy(&line)).as_deref()==Some(target.as_str()){let _=sender.send(Event::Captured);}
    }
    Err(_)=>break,
   }
  }
 });
 // Wait briefly for the subscription; file polling below still covers a missed event.
 let subscribe_deadline=Instant::now()+SUBSCRIBE_TIMEOUT;
 loop{
  if cancelled(){cleanup(path);return Err(CaptureError::Cancelled);}
  if Instant::now()>=subscribe_deadline{break;}
  match receiver.recv_timeout(Duration::from_millis(25)){
   Ok(Event::Ready)|Ok(Event::Captured)|Err(mpsc::RecvTimeoutError::Disconnected)=>break,
   Err(mpsc::RecvTimeoutError::Timeout)=>{}
  }
 }
 if cancelled(){cleanup(path);return Err(CaptureError::Cancelled);}
 let mut action=Command::new(program).args(["msg","action","screenshot","--show-pointer","false","--path"]).arg(path).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().map_err(|_|CaptureError::Unavailable)?;
 if let Err(error)=wait_action(&mut action,ACTION_TIMEOUT,&cancelled){cleanup(path);guard.stop();return Err(error);}
 let deadline=Instant::now()+timeout;
 let mut stable:Option<(u64,Instant)>=None;
 loop{
  if cancelled(){cleanup(path);guard.stop();return Err(CaptureError::Cancelled);}
  while receiver.try_recv().is_ok(){}
  match fs::symlink_metadata(path){
   Ok(meta) if meta.is_file()&&meta.len()>0=>{
    let length=meta.len();
    match stable{
     Some((last,since)) if last==length=>{if since.elapsed()>=STABLE_FOR{guard.stop();return Ok(());}}
     _=>stable=Some((length,Instant::now())),
    }
   }
   _=>stable=None,
  }
  if Instant::now()>=deadline{cleanup(path);guard.stop();return Err(CaptureError::Timeout);}
  thread::sleep(POLL_INTERVAL);
 }
}

fn wait_action(child:&mut Child,timeout:Duration,cancelled:impl Fn()->bool)->Result<(),CaptureError>{
 let deadline=Instant::now()+timeout;
 loop{
  if cancelled(){let _=child.kill();let _=child.wait();return Err(CaptureError::Cancelled);}
  match child.try_wait(){
   Ok(Some(status))=>return if status.success(){Ok(())}else{Err(CaptureError::CommandFailed)},
   Ok(None) if Instant::now()<deadline=>thread::sleep(Duration::from_millis(10)),
   _=>{let _=child.kill();let _=child.wait();return Err(CaptureError::CommandFailed);}
  }
 }
}

fn read_line_bounded(reader:&mut impl BufRead,out:&mut Vec<u8>,max:usize)->std::io::Result<usize>{
 out.clear();let mut total=0;let mut over=false;
 loop{
  let (consume,newline,bytes)={
   let buffer=reader.fill_buf()?;
   if buffer.is_empty(){return Ok(total);}
   let newline=buffer.iter().position(|&b|b==b'\n');
   let consume=newline.map_or(buffer.len(),|i|i+1);
   (consume,newline.is_some(),buffer[..consume].to_vec())
  };
  if !over{
   if out.len()+bytes.len()>max{over=true;out.clear();}else{out.extend_from_slice(&bytes);}
  }
  total+=consume;reader.consume(consume);
  if newline{return Ok(total);}
 }
}

/// Only a path-bearing ScreenshotCaptured event is considered. A cancel event is
/// not guaranteed by Niri, so it is never inferred from silence.
pub fn screenshot_path(line:&str)->Option<String>{
 let value:serde_json::Value=serde_json::from_str(line).ok()?;
 value.get("ScreenshotCaptured")?.get("path")?.as_str().map(str::to_owned)
}

fn cleanup(path:&Path){
 if fs::symlink_metadata(path).is_ok_and(|meta|meta.is_file()){let _=fs::remove_file(path);}
}

/// Remove only abandoned request files that this application named. Never touches
/// saved attachments or arbitrary files.
pub fn cleanup_incoming(directory:&Path,max_age:Duration)->usize{
 let Ok(entries)=fs::read_dir(directory)else{return 0;};
 let now=SystemTime::now();let mut removed=0;
 for entry in entries.flatten(){
  let name=entry.file_name();let Some(name)=name.to_str()else{continue};
  if !is_request_file(name){continue;}
  let Ok(meta)=entry.metadata()else{continue};
  if !meta.is_file(){continue;}
  let Ok(modified)=meta.modified()else{continue};
  if now.duration_since(modified).is_ok_and(|age|age>=max_age)&&fs::remove_file(entry.path()).is_ok(){removed+=1;}
 }
 removed
}

fn is_request_file(name:&str)->bool{
 name.len()==36&&name.ends_with(".png")&&name[..32].bytes().all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_uppercase())
}

pub fn default_incoming(directory:&Path)->PathBuf{directory.join("attachments").join("incoming")}
