//! Text-only Chat Completions protocol; framing and HTTP live outside this decoder.
use serde_json::Value;
#[derive(Debug,Clone,Copy,PartialEq)] pub enum FinishReason {Stop,Length,Filtered}
#[derive(Debug)] pub struct Delta {pub text:String,pub finish:Option<FinishReason>,pub done:bool}
#[derive(Debug)] pub struct ProtocolError;
#[derive(Default)] pub struct CompletionDecoder {finish:Option<FinishReason>,done:bool,failed:bool}
impl CompletionDecoder {
 pub fn accept(&mut self,data:&str)->Result<Delta,ProtocolError>{
  if self.failed||self.done{return Err(ProtocolError);}
  match self.parse(data){Ok(delta)=>Ok(delta),Err(error)=>{self.failed=true;Err(error)}}
 }
 fn parse(&mut self,data:&str)->Result<Delta,ProtocolError>{
  if data.len()>crate::stream::MAX_EVENT_BYTES{return Err(ProtocolError);}
  if data=="[DONE]"{if self.finish.is_none(){return Err(ProtocolError);}self.done=true;return Ok(Delta{text:String::new(),finish:None,done:true});}
  let v:Value=serde_json::from_str(data).map_err(|_|ProtocolError)?;
  if v.get("error").is_some(){return Err(ProtocolError);}
  let choices=v.get("choices").and_then(Value::as_array).ok_or(ProtocolError)?;
  if choices.is_empty(){return Ok(Delta{text:String::new(),finish:None,done:false});}
  if choices.len()!=1||self.finish.is_some(){return Err(ProtocolError);}
  let choice=&choices[0];
  if choice.get("index").is_some_and(|i|i.as_u64()!=Some(0)){return Err(ProtocolError);}
  let delta=choice.get("delta").and_then(Value::as_object).ok_or(ProtocolError)?;
  if ["tool_calls","function_call","audio"].iter().any(|k|delta.get(*k).is_some_and(|v|!v.is_null())){return Err(ProtocolError);}
  let string=|key:&str|->Result<&str,ProtocolError>{match delta.get(key){None|Some(Value::Null)=>Ok(""),Some(Value::String(s))=>Ok(s),_=>Err(ProtocolError)}};
  let mut text=string("content")?.to_owned();let refusal=string("refusal")?;
  if !refusal.is_empty(){if !text.is_empty(){text.push('\n');}text.push_str(refusal);}
  let finish=match choice.get("finish_reason"){
   None|Some(Value::Null)=>None,
   Some(Value::String(s))=>Some(match s.as_str(){"stop"=>FinishReason::Stop,"length"=>FinishReason::Length,"content_filter"=>FinishReason::Filtered,_=>return Err(ProtocolError)}),
   _=>return Err(ProtocolError),
  };
  self.finish=finish;Ok(Delta{text,finish,done:false})
 }
 pub fn finish(self)->Result<FinishReason,ProtocolError>{if self.done&&!self.failed{self.finish.ok_or(ProtocolError)}else{Err(ProtocolError)}}
}

#[derive(Clone,Debug,PartialEq,serde::Serialize)]
#[serde(rename_all="lowercase")]
pub enum Role {User,Assistant}
/// `images` carries legacy bare base64 payloads or PNG data URLs for PDF pages.
/// Empty renders the string form text-only services expect; images render
/// the compatible content-parts array.
#[derive(Clone,Debug,PartialEq)]
pub struct Message {pub role:Role,pub content:String,pub images:Vec<String>}
impl serde::Serialize for Message {
 fn serialize<S:serde::Serializer>(&self,serializer:S)->Result<S::Ok,S::Error>{
  use serde::ser::SerializeStruct;
  let mut out=serializer.serialize_struct("Message",2)?;
  out.serialize_field("role",&self.role)?;
  if self.images.is_empty(){
   out.serialize_field("content",&self.content)?;
  }else{
   let mut parts=Vec::with_capacity(self.images.len()+1);
   for image in &self.images{parts.push(serde_json::json!({"type":"image_url","image_url":{"url":image}}));}
   parts.push(serde_json::json!({"type":"text","text":self.content}));
   out.serialize_field("content",&parts)?;
  }
  out.end()
 }
}
