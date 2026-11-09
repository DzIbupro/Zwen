//! Cancellable retries of one immutable request, only before any body text.
use std::time::{Duration,SystemTime};
use tokio::sync::watch;
use zwen_core::{completion::{FinishReason,Message},provider::{ApiKey,ProviderConfig}};
use crate::Error;
#[derive(Clone,Copy,Debug)]pub struct RetryPolicy{pub initial:Duration,pub maximum:Duration}
impl Default for RetryPolicy{fn default()->Self{Self{initial:Duration::from_secs(5),maximum:Duration::from_secs(60)}}}
fn transient_code(code:Option<u32>)->bool{matches!(code,None|Some(1302|1303|1305))}
impl RetryPolicy{
 pub fn delay(&self,error:&Error,attempt:u32)->Option<Duration>{
  let after=match error{Error::Timeout|Error::Http(429)=>Duration::ZERO,Error::Api{status:429,code}if transient_code(Some(*code))=>Duration::ZERO,Error::RateLimited{code,retry_after}if transient_code(*code)=>*retry_after,_=>return None};
  // Refuse unusable server hints instead of overflowing an Instant or retrying early.
  if after>Duration::from_secs(7*86400){return None;}
  let multiplier=1u32.checked_shl(attempt.saturating_sub(1).min(31)).unwrap_or(u32::MAX);
  Some(self.initial.saturating_mul(multiplier).min(self.maximum).max(Duration::from_millis(1)).max(after))
 }
}
pub fn parse_retry_after(value:&str,now:SystemTime)->Option<Duration>{
 if let Ok(seconds)=value.trim().parse::<u64>(){return Some(Duration::from_secs(seconds));}
 let date=chrono::DateTime::parse_from_rfc2822(value.trim()).ok()?;
 let seconds=u64::try_from(date.timestamp()).ok()?;
 Some((SystemTime::UNIX_EPOCH+Duration::from_secs(seconds)).duration_since(now).unwrap_or(Duration::ZERO))
}
#[derive(Clone,Copy,Debug)]pub struct RetryStatus{pub attempt:u32,pub delay:Duration,pub reason:&'static str}
pub enum StreamEvent<'a>{Text(&'a str),Retrying(RetryStatus),Attempt}
pub async fn stream_messages_with_retry(client:&reqwest::Client,config:&ProviderConfig,key:&ApiKey,messages:&[Message],mut cancel:watch::Receiver<bool>,policy:RetryPolicy,mut event:impl FnMut(StreamEvent<'_>)->Result<(),()>)->Result<FinishReason,Error>{
 let mut received=false;let mut attempt=0u32;
 loop{
  let result=crate::stream_messages(client,config,key,messages,cancel.clone(),|part|{received=true;event(StreamEvent::Text(part))}).await;
  let delay=match &result{Err(error)if !received=>policy.delay(error,attempt.saturating_add(1)),_=>None};
  let Some(delay)=delay else{return result;};
  attempt=attempt.saturating_add(1);
  let reason=if matches!(result,Err(Error::Timeout)){"timeout"}else{"rate-limit"};
  event(StreamEvent::Retrying(RetryStatus{attempt,delay,reason})).map_err(|_|Error::Delivery)?;
  tokio::select!{biased;_=crate::cancelled(&mut cancel)=>return Err(Error::Cancelled),_=tokio::time::sleep(delay)=>{}}
  event(StreamEvent::Attempt).map_err(|_|Error::Delivery)?;
 }
}
