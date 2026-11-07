//! Bounded SSE framing for provider transports. Feed chunks <=64 KiB; this is
//! not an HTTP client and never reconnects or interprets provider JSON/[DONE].
pub const MAX_CHUNK_BYTES: usize = 64 * 1024;
pub const MAX_EVENT_BYTES: usize = 256 * 1024;
#[derive(Debug, PartialEq, Eq)]
pub struct Event { pub event: String, pub data: String }
#[derive(Debug, PartialEq, Eq)]
pub enum StreamError { TooLarge, Closed }
#[derive(Default)]
pub struct Decoder {
 line: Vec<u8>, data: String, event: String,
 has_data: bool, skip_lf: bool, started: bool, closed: bool,
}
impl Decoder {
 pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<Event>, StreamError> {
  if self.closed { return Err(StreamError::Closed); }
  if bytes.len()>MAX_CHUNK_BYTES {return self.fail();}
  let mut result=Vec::new();
  for &b in bytes {
   if self.skip_lf {self.skip_lf=false;if b==b'\n' {continue;}}
   if b==b'\r'||b==b'\n' {
    self.skip_lf=b==b'\r';
    match self.line_event() {Ok(Some(e))=>result.push(e),Ok(None)=>{},Err(_)=>return self.fail()}
    self.line.clear();
   }else{if self.line.len()>=MAX_EVENT_BYTES{return self.fail();}self.line.push(b);}
  }
  Ok(result)
 }
 fn line_event(&mut self)->Result<Option<Event>,StreamError>{
  let text=String::from_utf8_lossy(&self.line);
  let text=if !self.started {self.started=true;text.strip_prefix('\u{feff}').unwrap_or(&text)}else{&text};
  if text.is_empty(){
   let event=std::mem::take(&mut self.event);
   if !self.has_data{return Ok(None);}
   self.has_data=false;self.data.pop();
   return Ok(Some(Event{event:if event.is_empty(){"message".into()}else{event},data:std::mem::take(&mut self.data)}));
  }
  if text.starts_with(':'){return Ok(None);}
  let (field,value)=text.split_once(':').map(|(k,v)|(k,v.strip_prefix(' ').unwrap_or(v))).unwrap_or((text,""));
  match field {
   "data"=>{if self.data.len()+value.len()+1>MAX_EVENT_BYTES{return Err(StreamError::TooLarge);}self.has_data=true;self.data.push_str(value);self.data.push('\n');},
   "event"=>self.event=value.to_owned(),
   _=>{},
  }
  Ok(None)
 }
 fn fail<T>(&mut self)->Result<T,StreamError>{self.closed=true;self.line=Vec::new();self.data=String::new();self.event=String::new();Err(StreamError::TooLarge)}
 /// EOF does not dispatch an event that lacks its terminating blank line.
 pub fn finish(self){}
}
