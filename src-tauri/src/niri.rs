//! Optional Niri geometry adapter. Only acts on this process's application window.
use serde::Deserialize;
use std::{process::{Command,Stdio},time::{Duration,Instant}};
fn request(args:&[&str])->Option<Vec<u8>> {
    if std::env::var_os("NIRI_SOCKET").is_none(){return None;}
    let mut child=Command::new("niri").args(args).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().ok()?;
    let deadline=Instant::now()+Duration::from_millis(400);
    loop {
        match child.try_wait(){
            Ok(Some(status))=>{let output=child.wait_with_output().ok()?;return status.success().then_some(output.stdout);}
            Ok(None) if Instant::now()<deadline=>std::thread::sleep(Duration::from_millis(8)),
            _=>{let _=child.kill();let _=child.wait();return None;}
        }
    }
}
#[derive(Deserialize)]struct Logical{width:f64,height:f64}
#[derive(Deserialize)]struct Output{name:String,logical:Option<Logical>}
#[derive(Clone)]pub struct Placement{pub output:String,pub width:f64,pub height:f64,pub id:Option<u64>}
pub fn focused()->Option<Placement>{let o:Output=serde_json::from_slice(&request(&["msg","-j","focused-output"])?).ok()?;let l=o.logical?;Some(Placement{output:o.name,width:l.width,height:l.height,id:None})}
#[derive(Deserialize)]struct Window{id:u64,pid:Option<u32>}
fn own_id()->Option<u64>{
    let windows:Vec<Window>=serde_json::from_slice(&request(&["msg","-j","windows"])?).ok()?;
    windows.into_iter().find(|w|w.pid==Some(std::process::id())).map(|w|w.id)
}
fn action(args:&[&str]){let mut argv=vec!["msg","action"];argv.extend_from_slice(args);let _=request(&argv);}
impl Placement {
    pub fn place(&mut self,width:u32,initial:bool){
        if self.id.is_none(){for _ in 0..8 {self.id=own_id();if self.id.is_some(){break;}std::thread::sleep(Duration::from_millis(20));}}
        let Some(id)=self.id else{return;};let id=id.to_string();
        if initial {
            action(&["move-window-to-floating","--id",&id]);
            action(&["move-window-to-monitor","--id",&id,&self.output]);
        }
        let x=((self.width-width as f64)/2.0).max(0.0).round().to_string();
        action(&["move-floating-window","--id",&id,"--x",&x,"--y","48"]);
    }
    pub fn tile(&self){if let Some(id)=self.id{action(&["move-window-to-tiling","--id",&id.to_string()]);}}
}
