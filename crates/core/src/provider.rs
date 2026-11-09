//! Non-secret configuration. Secret values never implement Serialize or plain Debug.
use crate::{Error,open_private_database};
use rusqlite::{Connection,TransactionBehavior,params};
use serde::{Deserialize,Serialize};
use std::{fmt,path::Path};
use url::{Url,Host};

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",try_from="Input")]
pub struct ProviderConfig {base_url:String,model:String,key_env:String,#[serde(skip_serializing_if="Option::is_none")] image_model:Option<String>}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Input {base_url:String,model:String,key_env:String,#[serde(default)] image_model:Option<String>}
impl TryFrom<Input> for ProviderConfig {
 type Error=Error;
 fn try_from(v:Input)->Result<Self,Error>{Self::build(&v.base_url,&v.model,&v.key_env,v.image_model.as_deref())}
}
fn clean_model_name(value:&str)->Result<Option<String>,Error>{
 let model=value.trim();
 if model.is_empty(){return Ok(None);}
 if model.len()>200||model.chars().any(char::is_control){return Err(Error::InvalidProvider);}
 Ok(Some(model.into()))
}
impl ProviderConfig {
 pub fn new(base_url:&str,model:&str,key_env:&str)->Result<Self,Error>{Self::build(base_url,model,key_env,None)}
 /// `image_model` routes screenshot questions; `None` keeps the serialized
 /// profile byte-identical to pre-vision configs so conversations stay bound.
 pub fn build(base_url:&str,model:&str,key_env:&str,image_model:Option<&str>)->Result<Self,Error>{
  if base_url.len()>2048||base_url.chars().any(char::is_control){return Err(Error::InvalidProvider);}
  let url=Url::parse(base_url.trim()).map_err(|_|Error::InvalidProvider)?;
  if url.as_str().len()>2048{return Err(Error::InvalidProvider);}
  let loopback=match url.host(){Some(Host::Ipv4(ip))=>ip.is_loopback(),Some(Host::Ipv6(ip))=>ip.is_loopback(),Some(Host::Domain(name))=>name=="localhost",None=>false};
  if !(url.scheme()=="https"||(url.scheme()=="http"&&loopback))||url.host().is_none()||!url.username().is_empty()||url.password().is_some()||url.query().is_some()||url.fragment().is_some(){return Err(Error::InvalidProvider);}
  let model=model.trim();let key_env=key_env.trim();
  if model.is_empty()||model.len()>200||model.chars().any(char::is_control)||key_env.is_empty()||key_env.len()>128||!key_env.bytes().all(|b|b.is_ascii_uppercase()||b.is_ascii_digit()||b==b'_')||key_env.as_bytes()[0].is_ascii_digit(){return Err(Error::InvalidProvider);}
  Ok(Self{base_url:url.as_str().trim_end_matches('/').into(),model:model.into(),key_env:key_env.into(),image_model:clean_model_name(image_model.unwrap_or(""))?})
 }
 pub fn base_url(&self)->&str{&self.base_url}
 pub fn model(&self)->&str{&self.model}
 pub fn image_model(&self)->Option<&str>{self.image_model.as_deref()}
 pub fn key_env(&self)->&str{&self.key_env}
 /// Same service and key, a different request model. Used to route screenshot
 /// questions to the configured vision model without touching the stored config.
 pub fn with_model(&self,model:&str)->Result<Self,Error>{Self::build(&self.base_url,model,&self.key_env,self.image_model.as_deref())}
 pub fn read_key(&self)->Result<ApiKey,Error>{let value=std::env::var(&self.key_env).map_err(|_|Error::InvalidProvider)?;ApiKey::new(&value)}
}
pub struct ApiKey(String);
impl fmt::Debug for ApiKey {fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{f.write_str("ApiKey([REDACTED])")}}
impl ApiKey {
 pub fn new(value:&str)->Result<Self,Error>{if value.is_empty()||value.len()>4096||!value.bytes().all(|b|b.is_ascii_graphic()){Err(Error::InvalidProvider)}else{Ok(Self(value.into()))}}
 pub fn expose(&self)->&str{&self.0}
}
#[derive(Clone,Debug,PartialEq,Serialize)]
pub struct ProviderSnapshot {pub config:Option<ProviderConfig>,pub revision:u64}
pub struct ProviderStore {connection:Connection}
fn read(connection:&Connection)->Result<ProviderSnapshot,Error>{
 let (raw,revision):(Option<String>,i64)=connection.query_row("SELECT config,revision FROM provider_config WHERE id=1",[],|r|Ok((r.get(0)?,r.get(1)?)))?;
 let revision=u64::try_from(revision).map_err(|_|Error::InvalidProvider)?;
 let config=raw.map(|s|{if s.len()>8192{return Err(Error::InvalidProvider);}serde_json::from_str(&s).map_err(|_|Error::InvalidProvider)}).transpose()?;
 Ok(ProviderSnapshot{config,revision})
}
impl ProviderStore {
 pub fn open(path:&Path)->Result<Self,Error>{
  let connection=open_private_database(path,|c|{
   let version:i64=c.pragma_query_value(None,"user_version",|r|r.get(0))?;
   if version!=0&&version!=1{return Err(Error::UnsupportedSchema);}
   if version==0{let tx=c.transaction_with_behavior(TransactionBehavior::Immediate)?;tx.execute_batch("CREATE TABLE provider_config(id INTEGER PRIMARY KEY CHECK(id=1),config TEXT,revision INTEGER NOT NULL CHECK(revision>=0));INSERT INTO provider_config VALUES(1,NULL,0);PRAGMA user_version=1;")?;tx.commit()?;}
   read(c)?;Ok(())
  })?;Ok(Self{connection})
 }
 pub fn load(&self)->Result<ProviderSnapshot,Error>{read(&self.connection)}
 pub fn save(&mut self,expected_revision:u64,config:ProviderConfig)->Result<ProviderSnapshot,Error>{
  let revision=i64::try_from(expected_revision).map_err(|_|Error::ProviderConflict)?;
  let next=revision.checked_add(1).ok_or(Error::ProviderConflict)?;
  let raw=serde_json::to_string(&config).map_err(|_|Error::InvalidProvider)?;
  let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
  if tx.execute("UPDATE provider_config SET config=?1,revision=?2 WHERE id=1 AND revision=?3",params![raw,next,revision])?!=1{return Err(Error::ProviderConflict);}
  tx.commit()?;Ok(ProviderSnapshot{config:Some(config),revision:next as u64})
 }
}
