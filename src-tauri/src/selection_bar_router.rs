//! The bar's lifetime and explicit action routing. No candidate activates the WebView.
use std::{sync::Mutex,time::{Duration,Instant}};
use tauri::{Emitter,Manager};
use super::Shared;
#[derive(Clone)]struct Session{epoch:u64,id:u32,pid:u32,source:super::FocusSource,revision:u64,actions:Vec<String>}
#[derive(Clone,Copy)]struct AnchorLease{id:u32,pid:u32,generation:u64}
#[derive(Default)]struct Ledger{next:u64,generation:u64,active:Option<Session>,anchor:Option<AnchorLease>}
impl Ledger{
 fn start(&mut self,id:u32,source:super::FocusSource,revision:u64,actions:Vec<String>)->Session{self.next+=1;let s=Session{epoch:self.next,id,pid:source.pid,source,revision,actions};self.active=Some(s.clone());s}
 fn dismiss(&mut self){self.generation+=1;self.active=None;self.anchor=None;}
 fn start_if_current(&mut self,generation:u64,id:u32,source:super::FocusSource,revision:u64,actions:Vec<String>)->Option<Session>{if self.generation!=generation{return None;}Some(self.start(id,source,revision,actions))}
 fn reserve_prime(&mut self,inbox:&zwen_core::selection::Inbox,id:u32,pid:u32,generation:u64)->bool{if self.generation!=generation||!inbox.is_current_read(id){return false;}self.anchor=Some(AnchorLease{id,pid,generation});true}
 fn start_for_candidate(&mut self,inbox:&zwen_core::selection::Inbox,generation:u64,id:u32,source:super::FocusSource,revision:u64,actions:Vec<String>)->Option<Session>{if inbox.reading()||!inbox.candidate_exists(id){return None;}self.start_if_current(generation,id,source,revision,actions)}
 fn revoke(&mut self,epoch:u64)->bool{if self.active.as_ref().is_some_and(|s|s.epoch==epoch){if self.anchor.as_ref().is_some_and(|a|self.active.as_ref().is_some_and(|s|a.id==s.id)){self.anchor=None;}self.active=None;true}else{false}}
}
fn bind_action(ledger:&mut Ledger,inbox:&mut zwen_core::selection::Inbox,epoch:u64,id:u32,settings:bool)->bool{
 if !ledger.active.as_ref().is_some_and(|s|s.epoch==epoch&&s.id==id){return false;}
 if settings{if !inbox.acknowledge(id){return false;}}else if inbox.freeze(id).is_none(){return false;}
 ledger.revoke(epoch)
}
static LEDGER:Mutex<Ledger>=Mutex::new(Ledger{next:0,generation:0,active:None,anchor:None});
pub(super) fn generation()->Option<u64>{LEDGER.lock().ok().map(|l|l.generation)}
pub(super) fn prime(app:&tauri::AppHandle,id:u32,pid:u32,generation:u64){
 let shared=app.state::<Shared>();let Ok(inbox)=shared.lock() else{return;};
 let accepted=LEDGER.lock().is_ok_and(|mut l|l.reserve_prime(&inbox,id,pid,generation));
 // Keep the read identity locked through native REQUEST reservation. A newer
 // inbox read cannot reserve its UI and then be overwritten by this worker.
 if accepted{crate::selection_bar::prime(app,id,pid);}
}
pub fn anchor_is_current(app:&tauri::AppHandle,id:u32,pid:u32)->bool{
 let lease=LEDGER.lock().is_ok_and(|l|l.anchor.as_ref().is_some_and(|a|a.id==id&&a.pid==pid&&a.generation==l.generation));
 lease&&app.state::<Shared>().lock().is_ok_and(|inbox|inbox.is_current_read(id)||(!inbox.reading()&&inbox.candidate_exists(id)))
}
pub(super) fn discard_prime(app:&tauri::AppHandle,id:u32){if let Ok(mut l)=LEDGER.lock(){if l.anchor.as_ref().is_some_and(|a|a.id==id){l.anchor=None;}}crate::selection_bar::discard_prime(app,id);}
pub(super) fn current_epoch()->Option<u64>{LEDGER.lock().ok().and_then(|l|l.active.as_ref().map(|s|s.epoch))}
pub fn dismiss(app:&tauri::AppHandle){if let Ok(mut l)=LEDGER.lock(){l.dismiss();}crate::selection_bar::dismiss(app);}
pub fn dismiss_id(app:&tauri::AppHandle,id:u32){let removed=LEDGER.lock().is_ok_and(|mut l|{if let Some(epoch)=l.active.as_ref().filter(|s|s.id==id).map(|s|s.epoch){l.revoke(epoch)}else{false}});if removed{crate::selection_bar::discard_prime(app,id);}}
pub(super) fn revoke_epoch(app:&tauri::AppHandle,epoch:u64){
 let removed=LEDGER.lock().ok().and_then(|mut l|{let id=l.active.as_ref().filter(|s|s.epoch==epoch).map(|s|s.id)?;if l.revoke(epoch){Some(id)}else{None}});
 if let Some(id)=removed{crate::selection_bar::discard_prime(app,id);}
}
pub fn bar_is_current(app:&tauri::AppHandle,id:u32,pid:u32)->bool{
 let valid=LEDGER.lock().is_ok_and(|l|l.active.as_ref().is_some_and(|s|s.id==id&&s.pid==pid));
 valid&&app.state::<Shared>().lock().is_ok_and(|inbox|!inbox.reading()&&inbox.candidate_exists(id))
}
// This is an output identity, never a late pointer sample. Unknown/malformed
// compositor metadata deliberately falls back to the primary monitor.
fn reading_output()->Option<crate::selection_bar::OutputRect>{
 let mut command=std::process::Command::new("niri");command.args(["msg","-j","focused-output"]);
 let raw=zwen_core::selection::read_from(command,Duration::from_millis(150)).ok()?;
 parse_reading_output(&raw)
}
fn parse_reading_output(raw:&str)->Option<crate::selection_bar::OutputRect>{
 let value:serde_json::Value=serde_json::from_str(raw).ok()?;let logical=value.get("logical")?;
 let number=|key|i32::try_from(logical.get(key)?.as_i64()?).ok();
 let output=crate::selection_bar::OutputRect{x:number("x")?,y:number("y")?,width:number("width")?,height:number("height")?};
 (output.width>0&&output.height>0).then_some(output)
}
pub async fn present(app:tauri::AppHandle,id:u32,watch_token:Option<String>,expected_source:super::FocusSource,generation:u64){
 let a=app.clone();
 let prepared=tauri::async_runtime::spawn_blocking(move||{
  let source=super::external_source()?;if source!=expected_source||source.pid==std::process::id(){return None;}
  let storage=a.state::<crate::SharedStorage>();
  let config=crate::quick_actions::config_for_bar(storage.inner()).ok()?;
  let actions=config.actions.into_iter().filter(|a|a.enabled).map(|a|crate::selection_bar::BarAction{id:a.id,name:a.name}).collect::<Vec<_>>();
  let output=reading_output();
  if super::external_source()!=Some(source){return None;}
  Some((source,config.revision,actions,output))
 }).await.ok().flatten();
 let Some((source,revision,actions,output))=prepared else{return;};
 let pid=source.pid;
 if !super::candidate_allowed(&watch_token){return;}
 let mut keys=actions.iter().map(|a|a.id.clone()).collect::<Vec<_>>();keys.extend(["ask".into(),"__settings".into(),"__dismiss".into()]);
 let session={
  let shared=app.state::<Shared>();let Ok(inbox)=shared.lock() else{return;};
  let session=match LEDGER.lock().ok().and_then(|mut l|l.start_for_candidate(&inbox,generation,id,source,revision,keys)){Some(s)=>s,None=>return};
  crate::selection_bar::show(&app,id,revision,actions,pid,output);session
 };
 // GTK only reads the cheap ledger. Compositor IPC stays off its event thread.
 tauri::async_runtime::spawn_blocking(move||{
  let deadline=Instant::now()+Duration::from_secs(20);
  loop{
   std::thread::sleep(Duration::from_millis(250));
   if !LEDGER.lock().is_ok_and(|l|l.active.as_ref().is_some_and(|s|s.epoch==session.epoch)){return;}
   if Instant::now()>=deadline||super::external_source()!=Some(source)||!bar_is_current(&app,id,pid){revoke_epoch(&app,session.epoch);return;}
  }
 });
}
#[derive(Clone,serde::Serialize)]#[serde(rename_all="camelCase")]
struct Command{selection_id:u32,action_id:String,config_revision:u64,command_id:String}
pub fn toolbar_action(app:&tauri::AppHandle,id:u32,action:&str,revision:u64){
 let session=LEDGER.lock().ok().and_then(|l|l.active.clone());
 let Some(session)=session.filter(|s|s.id==id&&s.revision==revision&&s.actions.iter().any(|a|a==action)) else{return;};
 if !bar_is_current(app,id,session.pid){return;}
 if action=="__dismiss"{
  revoke_epoch(app,session.epoch);
  if let Ok(mut inbox)=app.state::<Shared>().lock(){let _=inbox.acknowledge(id);}
  let _=app.emit("selection-ready",());return;
 }
 // Bind the action before compositor IPC; a PRIMARY clear caused by this
 // press must not replace the material while the lookup is in flight.
 {
  let shared=app.state::<Shared>();let Ok(mut inbox)=shared.lock()else{return;};
  let Ok(mut ledger)=LEDGER.lock()else{return;};
  if !bind_action(&mut ledger,&mut inbox,session.epoch,id,action=="__settings"){return;}
 }
 let app=app.clone();let action=action.to_owned();
 tauri::async_runtime::spawn_blocking(move||{
  if super::external_source()!=Some(session.source){
   if let Ok(mut inbox)=app.state::<Shared>().lock(){let _=inbox.acknowledge(id);}
   let _=app.emit("selection-ready",());return;
  }
  if action!="__settings"&&app.state::<Shared>().lock().ok().and_then(|mut inbox|inbox.freeze(id)).is_none(){return;}
  crate::selection_bar::dismiss(&app);
  let activation=crate::windows::activate(app.clone(),action!="__settings");
  tauri::async_runtime::spawn(async move{
   if activation.await.is_err(){return;}
   if action=="__settings"{let _=app.emit("request-settings",());return;}
   let _=app.emit("selection-ready",());
   let _=app.emit("selection-command",Command{selection_id:id,action_id:action,config_revision:revision,command_id:format!("bar-{}-{id}",session.epoch)});
  });
 });
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn reading_output_uses_logical_coordinates_and_rejects_untrusted_metadata(){
  let output=parse_reading_output(r#"{"name":"DP-2","logical":{"x":1920,"y":-120,"width":1280,"height":720,"scale":1.5}}"#).unwrap();
  assert_eq!(output,crate::selection_bar::OutputRect{x:1920,y:-120,width:1280,height:720});
  for raw in ["null", "{}", r#"{"logical":null}"#, r#"{"logical":{"x":0,"y":0,"width":0,"height":720}}"#, r#"{"logical":{"x":0,"y":0,"width":1920}}"#, r#"{"logical":{"x":0,"y":0,"width":2147483648,"height":720}}"#, r#"{"logical":{"x":0.5,"y":0,"width":1920,"height":720}}"#] {
   assert_eq!(parse_reading_output(raw),None,"invalid output must use explicit primary-monitor fallback: {raw}");
  }
 }

 #[test]fn stale_read_cannot_overwrite_a_new_anchor_or_presentation(){let mut l=Ledger::default();let mut inbox=zwen_core::selection::Inbox::default();let old=inbox.begin(zwen_core::selection::Mode::Candidate).unwrap();assert!(inbox.is_current_read(old));let new=inbox.begin(zwen_core::selection::Mode::Candidate).unwrap();assert!(l.reserve_prime(&inbox,new,2,0));assert!(!l.reserve_prime(&inbox,old,2,0));assert_eq!(l.anchor.as_ref().unwrap().id,new);inbox.finish(new,Ok("fixture".into()));let source=super::super::FocusSource{pid:2,window_id:10};assert!(l.start_for_candidate(&inbox,0,old,source,1,vec![]).is_none());assert!(l.start_for_candidate(&inbox,0,new,source,1,vec![]).is_some());}
 #[test]fn cleaning_old_bar_does_not_cancel_a_new_selection_lookup(){let mut l=Ledger::default();let source=super::super::FocusSource{pid:2,window_id:10};let old=l.start(1,source,3,vec![]);let lookup=l.generation;assert!(l.revoke(old.epoch));assert!(l.start_if_current(lookup,2,source,3,vec![]).is_some());}
 #[test]fn disabled_or_dismissed_bar_cannot_be_recreated_by_an_old_lookup(){let mut l=Ledger::default();let g=l.generation;l.dismiss();let source=super::super::FocusSource{pid:2,window_id:10};assert!(l.start_if_current(g,1,source,3,vec![]).is_none());let g=l.generation;let session=l.start_if_current(g,1,source,3,vec![]).unwrap();let pending=l.generation;assert!(l.revoke(session.epoch));l.dismiss();assert!(l.start_if_current(pending,2,source,3,vec![]).is_none());}
 #[test]fn same_process_other_window_is_a_different_source(){let first=super::super::FocusSource{pid:2,window_id:10};let other=super::super::FocusSource{pid:2,window_id:11};assert!(first!=other);}
 #[test]fn a_stale_focus_worker_cannot_revoke_a_new_bar(){let mut l=Ledger::default();let old=l.start(1,super::super::FocusSource{pid:2,window_id:10},3,vec![]);let current=l.start(2,super::super::FocusSource{pid:2,window_id:11},3,vec![]);assert!(!l.revoke(old.epoch));assert_eq!(l.active.as_ref().unwrap().id,2);assert!(l.revoke(current.epoch));assert!(!l.revoke(current.epoch));}
}

#[cfg(test)]mod clear_tests{
 use super::*;
 #[test]fn action_binding_precedes_empty_callbacks_and_dispatches_once(){
  let mut inbox=zwen_core::selection::Inbox::default();let id=inbox.begin(zwen_core::selection::Mode::Candidate).unwrap();inbox.finish(id,Ok("material".into()));
  let mut ledger=Ledger::default();let s=ledger.start(id,super::super::FocusSource{pid:9,window_id:10},1,vec!["ask".into()]);
  assert!(bind_action(&mut ledger,&mut inbox,s.epoch,id,false));assert!(inbox.begin(zwen_core::selection::Mode::Candidate).is_none());assert!(!bind_action(&mut ledger,&mut inbox,s.epoch,id,false));assert_eq!(inbox.frozen_text(),Some("material"));
 }
 #[test]fn stale_epoch_cannot_claim_another_window_and_settings_drops_preview(){
  let mut inbox=zwen_core::selection::Inbox::default();let id=inbox.begin(zwen_core::selection::Mode::Candidate).unwrap();inbox.finish(id,Ok("material".into()));
  let mut ledger=Ledger::default();let old=ledger.start(id,super::super::FocusSource{pid:9,window_id:10},1,vec![]);let new=ledger.start(id,super::super::FocusSource{pid:9,window_id:11},1,vec![]);
  assert!(!bind_action(&mut ledger,&mut inbox,old.epoch,id,false));assert!(inbox.frozen_text().is_none());assert!(bind_action(&mut ledger,&mut inbox,new.epoch,id,true));assert!(inbox.claim().is_none());
 }
}
