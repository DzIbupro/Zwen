//! Standard Chat Completions streaming function calls. Reports remain free text.
use crate::completion::{FinishReason,ProtocolError};
use serde::{Serialize,Deserialize};
use serde_json::Value;
use std::collections::{BTreeMap,BTreeSet};
#[derive(Clone,Default,Debug,Serialize,Deserialize)]pub struct Function{pub name:String,pub arguments:String}
#[derive(Clone,Default,Debug,Serialize,Deserialize)]pub struct ToolCall{pub id:String,#[serde(rename="type")]pub kind:String,pub function:Function}
pub struct ToolTurn{pub content:String,pub calls:Vec<ToolCall>,pub finish:FinishReason}
#[derive(Default)]pub struct ToolDecoder{text:String,calls:BTreeMap<usize,ToolCall>,finish:Option<String>,done:bool,failed:bool}
impl ToolDecoder{
 pub fn text(&self)->&str{&self.text}
 pub fn accept(&mut self,data:&str)->Result<bool,ProtocolError>{if self.failed||self.done{return Err(ProtocolError);}let result=self.parse(data);if result.is_err(){self.failed=true;}result}
 fn parse(&mut self,data:&str)->Result<bool,ProtocolError>{
  if data.len()>crate::stream::MAX_EVENT_BYTES{return Err(ProtocolError);}
  if data=="[DONE]"{if self.finish.is_none(){return Err(ProtocolError);}self.done=true;return Ok(true);}
  let v:Value=serde_json::from_str(data).map_err(|_|ProtocolError)?;if v.get("error").is_some(){return Err(ProtocolError);}
  let choices=v.get("choices").and_then(Value::as_array).ok_or(ProtocolError)?;if choices.is_empty(){return Ok(false);}if choices.len()!=1||self.finish.is_some(){return Err(ProtocolError);}
  let c=&choices[0];if c.get("index").is_some_and(|v|v.as_u64()!=Some(0)){return Err(ProtocolError);}let d=c.get("delta").and_then(Value::as_object).ok_or(ProtocolError)?;
  for key in ["function_call","audio"]{if d.get(key).is_some_and(|v|!v.is_null()){return Err(ProtocolError);}}
  for key in ["content","refusal"]{match d.get(key){None|Some(Value::Null)=>{},Some(Value::String(s))=>self.text.push_str(s),_=>return Err(ProtocolError)}}if self.text.len()>256*1024{return Err(ProtocolError);}
  if let Some(calls)=d.get("tool_calls").filter(|v|!v.is_null()){
   let calls=calls.as_array().ok_or(ProtocolError)?;if calls.len()>4{return Err(ProtocolError);}
   for part in calls{let index=part.get("index").and_then(Value::as_u64).filter(|n|*n<4).ok_or(ProtocolError)? as usize;let out=self.calls.entry(index).or_default();
    if let Some(id)=part.get("id").filter(|v|!v.is_null()&&v.as_str()!=Some("")){let id=id.as_str().ok_or(ProtocolError)?;if !out.id.is_empty()&&out.id!=id{return Err(ProtocolError);}out.id=id.into();}
    if part.get("type").filter(|v|!v.is_null()).is_some_and(|v|v.as_str()!=Some("function")){return Err(ProtocolError);}out.kind="function".into();
    if let Some(f)=part.get("function").filter(|v|!v.is_null()){let f=f.as_object().ok_or(ProtocolError)?;for (key,dest) in [("name",&mut out.function.name),("arguments",&mut out.function.arguments)]{if let Some(v)=f.get(key).filter(|v|!v.is_null()){dest.push_str(v.as_str().ok_or(ProtocolError)?);}}}
    if out.id.len()>128||out.function.name.len()>128||out.function.arguments.len()>8192{return Err(ProtocolError);}
   }
  }
  if let Some(reason)=c.get("finish_reason").filter(|v|!v.is_null()){let s=reason.as_str().filter(|s|["stop","length","content_filter","tool_calls"].contains(s)).ok_or(ProtocolError)?;self.finish=Some(s.into());}
  Ok(false)
 }
 pub fn finish(self)->Result<ToolTurn,ProtocolError>{
  if !self.done||self.failed{return Err(ProtocolError);}let reason=self.finish.ok_or(ProtocolError)?;
  if (reason=="tool_calls")!=!self.calls.is_empty(){return Err(ProtocolError);}
  if self.calls.values().map(|c|c.function.arguments.len()).sum::<usize>()>8192{return Err(ProtocolError);}
  let mut ids=BTreeSet::new();for (index,(n,call)) in self.calls.iter().enumerate(){if index!=*n||call.id.is_empty()||!ids.insert(call.id.clone())||call.function.name.is_empty()||call.function.arguments.is_empty(){return Err(ProtocolError);}}
  Ok(ToolTurn{content:self.text,calls:self.calls.into_values().collect(),finish:match reason.as_str(){"length"=>FinishReason::Length,"content_filter"=>FinishReason::Filtered,_=>FinishReason::Stop}})
 }
}
