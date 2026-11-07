//! Text-only provider HTTP transport. No body/key/URL is included in errors.
use std::{fmt,time::Duration};
use tokio::sync::watch;
use zwen_core::{provider::{ProviderConfig,ApiKey},completion::{CompletionDecoder,FinishReason},stream::{Decoder,MAX_CHUNK_BYTES}};
pub mod retry;
pub const MAX_RESPONSE_BYTES:usize=256*1024;
pub const MAX_WIRE_BYTES:usize=8*1024*1024;
pub const MAX_MESSAGE_IMAGES:usize=8;
pub const MAX_IMAGE_PAYLOAD_BYTES:usize=8*1024*1024;
#[derive(Debug,PartialEq)] pub enum Error {Quota,RateLimited{code:Option<u32>,retry_after:Duration},Cancelled,Http(u16),Api{status:u16,code:u32},Protocol,Network,Timeout,TooLarge,Delivery,Empty,InvalidInput}
impl fmt::Display for Error {
 fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{if let Self::RateLimited{code,..}=self{return write!(f,"服务返回HTTP429{}",code.map(|v|format!("（代码 {v}）")).unwrap_or_default());}if let Self::Api{status,code}=self{return write!(f,"{}（HTTP {}，代码 {}），请核对服务状态后手动重试",if *status==429{"服务拒绝本次请求"}else{"服务返回异常状态"},status,code);}f.write_str(match self{
  Self::Quota=>"服务账户或额度限制（HTTP429），问题已保留，请核对账户后再试",Self::RateLimited{..}=>"服务返回HTTP429",Self::Api{..}=>"服务返回异常状态",Self::Cancelled=>"已停止生成，已接收内容保留",Self::Http(401|403)=>"服务拒绝了凭据或模型权限，请检查设置",Self::Http(429)=>"服务返回HTTP429，可能是速率、配额或账户限制，请核对后手动重试",Self::Http(_)=>"模型服务返回异常状态，未自动重试",Self::Protocol=>"响应格式不支持或流式连接未完整结束，已接收内容保留",Self::Network=>"无法连接模型服务，未自动重试",Self::Timeout=>"模型请求超时，已接收内容保留",Self::TooLarge=>"回答达到本轮大小限制，已接收内容保留",Self::Delivery=>"回答暂无法保存或展示，请保留当前内容",Self::Empty=>"模型未返回文本，请检查模型与服务协议",Self::InvalidInput=>"问题为空或超过256 KiB，请调整后发送"})}
}
impl std::error::Error for Error {}
pub fn client()->Result<reqwest::Client,Error>{
 reqwest::Client::builder().no_proxy().redirect(reqwest::redirect::Policy::none()).retry(reqwest::retry::never())
 .connect_timeout(Duration::from_secs(10)).read_timeout(Duration::from_secs(30)).timeout(Duration::from_secs(120))
 .pool_idle_timeout(Duration::from_secs(15)).pool_max_idle_per_host(1).build().map_err(|_|Error::Network)
}
fn network(error:reqwest::Error)->Error{if error.is_timeout(){Error::Timeout}else{Error::Network}}
async fn cancelled(rx:&mut watch::Receiver<bool>){loop{if *rx.borrow_and_update(){return;}if rx.changed().await.is_err(){return;}}}
async fn http_error(response:&mut reqwest::Response,cancel:&mut watch::Receiver<bool>,status:u16)->Error{
 let retry_after=response.headers().get(reqwest::header::RETRY_AFTER).and_then(|v|v.to_str().ok()).and_then(|v|retry::parse_retry_after(v,std::time::SystemTime::now()));
 let deadline=tokio::time::Instant::now()+Duration::from_secs(2);let mut bytes=Vec::new();
 loop {
  let part=tokio::select!{biased;_=cancelled(cancel)=>return Error::Cancelled,_=tokio::time::sleep_until(deadline)=>break,r=response.chunk()=>match r{Ok(Some(v))=>v,_=>break}};
  let remaining=8192usize.saturating_sub(bytes.len());bytes.extend_from_slice(&part[..part.len().min(remaining)]);if part.len()>=remaining{break;}
 }
 let parsed=serde_json::from_slice::<serde_json::Value>(&bytes).ok();
 if status==429&&parsed.as_ref().and_then(|v|v.get("error")).is_some_and(|e|["code","type"].iter().filter_map(|k|e.get(k)?.as_str()).any(|s|matches!(s,"insufficient_quota"|"billing_hard_limit_reached"|"billing_not_active"|"credits_exhausted"|"quota_exceeded"|"account_deactivated"|"account_suspended"|"usage_limit_reached"|"monthly_limit_exceeded"|"daily_limit_exceeded"|"permission_denied"|"invalid_api_key"|"model_not_found"))){return Error::Quota;}
 let code=parsed.and_then(|v|{
  let code=v.get("error")?.get("code")?;
  match code{serde_json::Value::Number(n)=>n.as_u64().filter(|n|*n<=999999).map(|n|n as u32),serde_json::Value::String(s)if s.len()<=6&&!s.is_empty()&&s.bytes().all(|b|b.is_ascii_digit())=>s.parse::<u32>().ok(),_=>None}
 });
 if status==429{if let Some(retry_after)=retry_after{return Error::RateLimited{code,retry_after};}}
 code.map(|code|Error::Api{status,code}).unwrap_or(Error::Http(status))
}
/// Any true cancellation value or loss of the sender ends the request. The
/// consumer owns generation filtering and persistence; callback errors stop I/O.
pub async fn stream_messages(client:&reqwest::Client,config:&ProviderConfig,key:&ApiKey,messages:&[zwen_core::completion::Message],mut cancel:watch::Receiver<bool>,mut on_text:impl FnMut(&str)->Result<(),()>)->Result<FinishReason,Error>{
 use zwen_core::completion::Role;
 if messages.is_empty()||messages.len()>17||messages.len()%2==0||messages.last().is_none_or(|m|m.content.trim().is_empty())||messages.iter().enumerate().any(|(i,m)|m.role!=if i%2==0{Role::User}else{Role::Assistant}){return Err(Error::InvalidInput);}
 let total_images:usize=messages.iter().map(|m|m.images.len()).sum();
 if messages.iter().any(|m|m.images.len()>MAX_MESSAGE_IMAGES)||total_images>MAX_MESSAGE_IMAGES{return Err(Error::InvalidInput);}
 if messages.iter().any(|m|m.images.iter().any(|image|image.is_empty()||image.len()>MAX_IMAGE_PAYLOAD_BYTES||!image.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'+'||b==b'/'||b==b'='))){return Err(Error::InvalidInput);}
 let bytes=messages.iter().try_fold(0usize,|n,m|n.checked_add(m.content.len())).ok_or(Error::InvalidInput)?;
 if bytes>zwen_core::MAX_DRAFT_BYTES{return Err(Error::InvalidInput);}
 if *cancel.borrow(){return Err(Error::Cancelled);}
 let request=client.post(format!("{}/chat/completions",config.base_url())).bearer_auth(key.expose()).header(reqwest::header::ACCEPT,"text/event-stream")
 .json(&serde_json::json!({"model":config.model(),"messages":messages,"stream":true}));
 let mut response=tokio::select!{biased;_=cancelled(&mut cancel)=>return Err(Error::Cancelled),r=request.send()=>r.map_err(network)?};
 if response.status()!=reqwest::StatusCode::OK{let status=response.status().as_u16();return Err(http_error(&mut response,&mut cancel,status).await);}
 let kind=response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|h|h.to_str().ok()).unwrap_or("");
 if !kind.split(';').next().unwrap_or("").trim().eq_ignore_ascii_case("text/event-stream"){return Err(Error::Protocol);}
 let mut framing=Decoder::default();let mut completion=CompletionDecoder::default();let mut wire=0usize;let mut text_bytes=0usize;
 loop {
  let chunk=tokio::select!{biased;_=cancelled(&mut cancel)=>return Err(Error::Cancelled),r=response.chunk()=>r.map_err(network)?};
  let Some(chunk)=chunk else{framing.finish();return Err(Error::Protocol);};
  wire=wire.checked_add(chunk.len()).ok_or(Error::TooLarge)?;if wire>MAX_WIRE_BYTES{return Err(Error::TooLarge);}
  for bytes in chunk.chunks(MAX_CHUNK_BYTES){
   let events=framing.push(bytes).map_err(|_|Error::TooLarge)?;
   for event in events {
    if *cancel.borrow(){return Err(Error::Cancelled);}
    if event.event=="error"{return Err(Error::Protocol);}
    let delta=completion.accept(&event.data).map_err(|_|Error::Protocol)?;
    if !delta.text.is_empty(){text_bytes=text_bytes.checked_add(delta.text.len()).ok_or(Error::TooLarge)?;if text_bytes>MAX_RESPONSE_BYTES{return Err(Error::TooLarge);}on_text(&delta.text).map_err(|_|Error::Delivery)?;}
    if delta.done{let reason=completion.finish().map_err(|_|Error::Protocol)?;if text_bytes==0&&reason!=FinishReason::Filtered{return Err(Error::Empty);}return Ok(reason);}
   }
  }
 }
}
pub async fn stream_text(client:&reqwest::Client,config:&ProviderConfig,key:&ApiKey,prompt:&str,cancel:watch::Receiver<bool>,on_text:impl FnMut(&str)->Result<(),()>)->Result<FinishReason,Error>{
 let messages=[zwen_core::completion::Message{role:zwen_core::completion::Role::User,content:prompt.into(),images:Vec::new()}];stream_messages(client,config,key,&messages,cancel,on_text).await
}
