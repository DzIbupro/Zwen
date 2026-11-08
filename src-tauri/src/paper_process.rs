//! Own only the process group created for this operation. Bound both pipes.
use std::{io::Read,process::{Command,Stdio},sync::{atomic::{AtomicBool,Ordering},mpsc},time::{Duration,Instant}};
use zwen_core::paper::{invalid,Result};
pub struct ProcessSpec {pub program:String,pub args:Vec<String>,pub deadline:Instant,pub output_limit:usize}
pub struct ProcessOutput {pub stdout:Vec<u8>,pub _stderr:Vec<u8>,pub success:bool}
pub struct ProcessRunner;
impl ProcessRunner {
 pub fn run(spec:ProcessSpec,cancel:&AtomicBool)->Result<ProcessOutput> {
  if cancel.load(Ordering::Acquire){return Err(invalid("PDF操作已取消"));}
  let mut command=Command::new(&spec.program);
  command.args(&spec.args).env("LC_ALL","C").stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
  #[cfg(unix)] {use std::os::unix::process::CommandExt;command.process_group(0);}
  let mut child=command.spawn().map_err(|_|invalid("PDF工具缺失或无法启动，请检查Poppler安装"))?;
  let pid=child.id();let (tx,rx)=mpsc::channel();
  let stdout=child.stdout.take().expect("piped stdout");let stderr=child.stderr.take().expect("piped stderr");
  let limit=spec.output_limit;let tx2=tx.clone();
  let a=std::thread::spawn(move||{let _=tx.send((0,read_pipe(stdout,limit)));});
  let b=std::thread::spawn(move||{let _=tx2.send((1,read_pipe(stderr,65536.min(limit))));});
  let result=(|| {
   let mut status=None;let mut out=None;let mut err=None;
   loop {
    if cancel.load(Ordering::Acquire){return Err(invalid("PDF操作已取消"));}
    if Instant::now()>=spec.deadline{return Err(invalid("PDF工具超时，已停止本次操作"));}
    while let Ok((pipe,value))=rx.try_recv(){let value=value?;if pipe==0{out=Some(value)}else{err=Some(value)}}
    if status.is_none(){status=child.try_wait()?;}
    if let Some(exit)=status.as_ref(){
     // A child that exited cannot leave inherited pipes alive in descendants.
     kill_group(pid);
     if out.is_some()&&err.is_some(){return Ok(ProcessOutput{stdout:out.take().unwrap(),_stderr:err.take().unwrap(),success:exit.success()});}
    }
    std::thread::sleep(Duration::from_millis(10));
   }
  })();
  kill_group(pid);let _=child.kill();let _=child.wait();let _=a.join();let _=b.join();result
 }
}
fn read_pipe(mut pipe:impl Read,limit:usize)->Result<Vec<u8>>{let mut bytes=Vec::new();pipe.by_ref().take(limit as u64+1).read_to_end(&mut bytes)?;if bytes.len()>limit{return Err(invalid("PDF工具输出超限，未截断为完整材料"));}Ok(bytes)}
fn kill_group(pid:u32){#[cfg(unix)]unsafe{libc::kill(-(pid as i32),libc::SIGKILL);}}
