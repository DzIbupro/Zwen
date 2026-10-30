//! Inbox access only. A WebView cannot initiate system selection reads.
use std::sync::{Arc,Mutex};
use tauri::{Emitter,Manager};
use zwen_core::selection::{Inbox,Item,Mode,read_primary_text};
use crate::ClientError;
pub type Shared=Arc<Mutex<Inbox>>;
#[derive(serde::Serialize)]pub struct Snapshot{reading:bool,item:Option<Item>}
#[tauri::command]
pub fn claim_selection(state:tauri::State<'_,Shared>)->Result<Snapshot,ClientError>{
 let mut inbox=state.lock().map_err(|_|ClientError::window())?;
 Ok(Snapshot{reading:inbox.reading(),item:inbox.claim()})
}
#[tauri::command]
pub fn acknowledge_selection(state:tauri::State<'_,Shared>,id:u32)->Result<(),ClientError>{
 if state.lock().map_err(|_|ClientError::window())?.acknowledge(id){Ok(())}else{Err(ClientError{code:"selection_changed",message:"待处理选区状态已改变，请重新读取".into()})}
}
pub fn capture(app:tauri::AppHandle,mode:Mode){
 let shared=app.state::<Shared>().inner().clone();
 let (id,reading)=match shared.lock(){Ok(mut inbox)=>{let id=inbox.begin(mode);(id,inbox.reading())},Err(_)=>(None,false)};
 if id.is_none()&&reading{return;}
 let intent=crate::windows::new_intent();
 let _=app.emit("selection-ready",());
 tauri::async_runtime::spawn(async move{
  if let Some(id)=id{
   let worker=tauri::async_runtime::spawn_blocking(read_primary_text).await;
   let result=worker.unwrap_or(Err(zwen_core::selection::SelectionError::Unavailable));
   if let Ok(mut inbox)=shared.lock(){inbox.finish(id,result);}
   let _=app.emit("selection-ready",());
  }
  // Capture ends before any focus/activation request.
  let _=crate::windows::activate_if_current(app,true,intent).await;
 });
}
