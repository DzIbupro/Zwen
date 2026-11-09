//! Bounded document-only tool loop. No retries, file paths or general agent runtime.
use crate::Error;
use serde_json::{Value,json};
use tokio::sync::watch;
use zwen_core::{completion::{Message,FinishReason},paper::{Document,TOOL_TEXT_BUDGET},provider::{ProviderConfig,ApiKey},tool_completion::ToolDecoder};
pub const MAX_REQUESTS:usize=6;
pub async fn read(client:&reqwest::Client,config:&ProviderConfig,key:&ApiKey,messages:&[Message],mut document:Document,cancel:watch::Receiver<bool>,mut text:impl FnMut(&str)->Result<(),()>)->Result<FinishReason,Error>{
 let mut used=0usize;
 let result=run(client,config,key,messages,&mut document,cancel,&mut text,&mut used).await;
 // Always preserve program coverage, including stopped/failed/protocol-error turns.
 let footer=format!("\n\n### 材料覆盖（程序记录）\n\n{}\n",document.coverage());
 let delivered=if footer.len()<=crate::MAX_RESPONSE_BYTES.saturating_sub(used){text(&footer).map_err(|_|Error::Delivery)}else{Err(Error::TooLarge)};
 match result{Ok(reason)=>{delivered?;Ok(reason)},Err(error)=>{let _=delivered;Err(error)}}
}
async fn run(client:&reqwest::Client,config:&ProviderConfig,key:&ApiKey,messages:&[Message],document:&mut Document,mut cancel:watch::Receiver<bool>,text:&mut impl FnMut(&str)->Result<(),()>,used:&mut usize)->Result<FinishReason,Error>{
 let mut history:Vec<Value>=messages.iter().map(serde_json::to_value).collect::<Result<_,_>>().map_err(|_|Error::InvalidInput)?;
 let mut budget=TOOL_TEXT_BUDGET;let tools=zwen_core::paper::tools();
 let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(300);
 emit(text,used,"> 长论文按需读取：最多6次模型请求；以下保留读取记录，覆盖范围以程序统计为准。\n\n")?;
 for round in 0..MAX_REQUESTS{
  if *cancel.borrow()||cancel.has_changed().is_err(){return Err(Error::Cancelled);}
  let allow=round+1<MAX_REQUESTS&&budget>2048;
  let body=json!({"model":config.model(),"messages":history,"stream":true,"tools":tools,"tool_choice":if allow{"auto"}else{"none"}});
  if serde_json::to_vec(&body).map_err(|_|Error::InvalidInput)?.len()>256*1024{return Err(Error::DocumentBudget);}
  let mut decoder=ToolDecoder::default();let mut shown=0usize;let mut output_started=false;let can_show=document.has_read();
  let request_cancel=cancel.clone();
  tokio::select!{biased;_=crate::cancelled(&mut cancel)=>return Err(Error::Cancelled),_=tokio::time::sleep_until(deadline)=>return Err(Error::Timeout),result=crate::stream_events(client,config,key,&body,request_cancel,|data|{
   let done=decoder.accept(data).map_err(|_|Error::Protocol)?;
   if can_show&&decoder.text().len()>shown{if !output_started{emit(text,used,"\n### 模型输出（可能继续取材，以最终状态为准）\n\n")?;output_started=true;}emit(text,used,&decoder.text()[shown..])?;shown=decoder.text().len();}Ok(done)
  })=>result?};
  if *cancel.borrow()||cancel.has_changed().is_err(){return Err(Error::Cancelled);}
  let turn=decoder.finish().map_err(|_|Error::Protocol)?;
  if turn.calls.is_empty(){
   if !document.has_read(){return Err(Error::DocumentTools);}
   if turn.content.trim().is_empty()&&turn.finish!=FinishReason::Filtered{return Err(Error::Empty);}
   return Ok(turn.finish);
  }
  if !allow{return Err(Error::DocumentTools);}
  history.push(json!({"role":"assistant","content":Value::Null,"tool_calls":turn.calls}));
  let count=turn.calls.len();
  for (index,call) in turn.calls.into_iter().enumerate(){
   if *cancel.borrow()||cancel.has_changed().is_err(){return Err(Error::Cancelled);}
   let result=document.execute(&call.function.name,&call.function.arguments,budget.saturating_sub((count-index-1)*512));
   // Even error messages consume the fixed tool-result budget.
   let cost=serde_json::to_string(&result.content).map_err(|_|Error::Protocol)?.len();if cost>budget{return Err(Error::DocumentBudget);}budget-=cost;
   emit(text,used,&format!("> {}\n\n",result.label))?;
   history.push(json!({"role":"tool","tool_call_id":call.id,"content":result.content}));
  }
 }
 Err(Error::DocumentTools)
}

fn emit(text:&mut impl FnMut(&str)->Result<(),()>,used:&mut usize,part:&str)->Result<(),Error>{
 if part.len()>(crate::MAX_RESPONSE_BYTES-8192).saturating_sub(*used){return Err(Error::TooLarge);}*used+=part.len();text(part).map_err(|_|Error::Delivery)
}
