//! Model API key storage in the user's Secret Service keyring (gnome-keyring).
//! The key never enters SQLite, IPC responses, or logs; only presence and the
//! storage source are reported. The plain session keeps the secret on the
//! user's private bus, matching how local tools read this keyring.
use crate::ClientError;
use std::{collections::HashMap,fmt};
use zbus::{blocking::Connection,zvariant::{ObjectPath,OwnedObjectPath,Value}};

const SERVICE:&str="org.freedesktop.secrets";
const SERVICE_PATH:&str="/org/freedesktop/secrets";
const SERVICE_IFACE:&str="org.freedesktop.Secret.Service";
const COLLECTION_IFACE:&str="org.freedesktop.Secret.Collection";
const ITEM_IFACE:&str="org.freedesktop.Secret.Item";
const SCHEMA:&str="xdg:schema";
#[cfg(not(any(test,feature="desktop-smoke")))]
const SCHEMA_VALUE:&str="io.github.dzibupro.zwen";
// Every test binary, including ignored integration tests, has a separate
// identity. CreateItem(replace=true) can never match the production credential.
#[cfg(any(test,feature="desktop-smoke"))]
const SCHEMA_VALUE:&str="io.github.dzibupro.zwen.test-only";
const ROLE:&str="zwen-role";
const ROLE_VALUE:&str="model-key";
const LABEL:&str="Zwen 模型密钥";

#[derive(Debug)]
pub enum SecretError{Unavailable(#[allow(dead_code)] String),Locked,ManualCleanup}
impl fmt::Display for SecretError{
 fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{f.write_str(match self{
  Self::Unavailable(_)=>"系统钥匙环不可用或已锁定；请解锁后重试，密钥未被保存或读取",
  Self::Locked=>"系统钥匙环需要解锁或确认；请在桌面钥匙环提示中解锁后重试",
  Self::ManualCleanup=>"请在系统钥匙环管理器中手动删除「Zwen 模型密钥」",
 })}}
impl std::error::Error for SecretError{}
impl From<zbus::Error> for SecretError{
 fn from(error:zbus::Error)->Self{Self::Unavailable(error.to_string())}
}

pub fn valid_secret(value:&str)->bool{!value.is_empty()&&value.len()<=4096&&value.bytes().all(|b|b.is_ascii_graphic())}

fn attributes()->HashMap<String,String>{
 HashMap::from([(SCHEMA.to_owned(),SCHEMA_VALUE.to_owned()),(ROLE.to_owned(),ROLE_VALUE.to_owned())])
}



fn service<'a>(connection:&'a Connection)->Result<zbus::blocking::Proxy<'a>,SecretError>{
 Ok(zbus::blocking::Proxy::new(connection,SERVICE,SERVICE_PATH,SERVICE_IFACE)?)
}

fn open_session(connection:&Connection)->Result<OwnedObjectPath,SecretError>{
 let (_,session):(zbus::zvariant::OwnedValue,OwnedObjectPath)=
  service(connection)?.call("OpenSession",&("plain",Value::from("")))?;
 Ok(session)
}

fn default_collection(connection:&Connection)->Result<OwnedObjectPath,SecretError>{
 let (collection,):(OwnedObjectPath,)=service(connection)?.call("ReadAlias",&("default",))?;
 Ok(collection)
}

fn search_attributes(connection:&Connection,attrs:HashMap<String,String>)->Result<Vec<OwnedObjectPath>,SecretError>{
 let collection=default_collection(connection)?;
 let (items,):(Vec<OwnedObjectPath>,)=
  zbus::blocking::Proxy::new(connection,SERVICE,collection,COLLECTION_IFACE)?.call("SearchItems",&(attrs,))?;
 Ok(items)
}

fn item_proxy<'a>(connection:&'a Connection,item:&'a OwnedObjectPath)->Result<zbus::blocking::Proxy<'a>,SecretError>{
 Ok(zbus::blocking::Proxy::new(connection,SERVICE,item,ITEM_IFACE)?)
}

fn item_secret(connection:&Connection,item:&OwnedObjectPath,session:&ObjectPath)->Result<String,SecretError>{
 let secret:(OwnedObjectPath,Vec<u8>,Vec<u8>,String)=item_proxy(connection,item)?.call("GetSecret",&(session,))?;
 let value=String::from_utf8(secret.2).map_err(|_|SecretError::Unavailable("keyring item is not valid UTF-8".into()))?;
 if !valid_secret(&value){return Err(SecretError::Unavailable("keyring item does not contain a usable key".into()));}
 Ok(value)
}

pub fn read_model_key()->Result<Option<String>,SecretError>{read_key_attributes(attributes())}
fn read_key_attributes(attrs:HashMap<String,String>)->Result<Option<String>,SecretError>{
 let connection=Connection::session()?;
 let session=open_session(&connection)?;
 for item in search_attributes(&connection,attrs)?{
  let value=item_secret(&connection,&item,&session)?;
  if !value.is_empty(){return Ok(Some(value));}
 }
 Ok(None)
}

pub fn store_model_key(value:&str)->Result<(),SecretError>{store_key_attributes(value,attributes())}
fn store_key_attributes(value:&str,attrs:HashMap<String,String>)->Result<(),SecretError>{
 if !valid_secret(value){return Err(SecretError::Unavailable("invalid key value".into()));}
 let connection=Connection::session()?;
 let session=open_session(&connection)?;
 let properties:HashMap<String,Value>=HashMap::from([
  ("org.freedesktop.Secret.Item.Label".to_owned(),Value::from(LABEL)),
  ("org.freedesktop.Secret.Item.Attributes".to_owned(),Value::new(attrs)),
 ]);
 let secret=(session.clone(),Vec::<u8>::new(),value.as_bytes().to_vec(),String::from("text/plain"));
 let (_item,prompt):(OwnedObjectPath,OwnedObjectPath)=
  zbus::blocking::Proxy::new(&connection,SERVICE,default_collection(&connection)?,COLLECTION_IFACE)?
  .call("CreateItem",&(properties,secret,true))?;
 if prompt.as_str()!="/"{return Err(SecretError::Locked);}
 Ok(())
}

pub fn remove_model_key()->Result<(),SecretError>{
 let connection=Connection::session()?;
 let session=open_session(&connection)?;
 for item in search_attributes(&connection,HashMap::from([(SCHEMA.into(),SCHEMA_VALUE.into())]))?{
  if item_secret(&connection,&item,&session).is_err(){continue;}
  let prompt:OwnedObjectPath=item_proxy(&connection,&item)?.call("Delete",&())?;
  if prompt.as_str()!="/"{return Err(SecretError::ManualCleanup);}
 }
 Ok(())
}

pub fn has_model_key()->Result<bool,SecretError>{Ok(read_model_key()?.is_some())}

fn client_error(error:SecretError)->ClientError{
 ClientError{code:"keyring_unavailable",message:error.to_string()}
}

/// Stores the key once, from the settings page. The value is never echoed back.
#[tauri::command]
pub async fn save_model_secret(storage:tauri::State<'_,crate::SharedStorage>,secret:String,expected_revision:u64)->Result<(),ClientError>{
 if !valid_secret(&secret){return Err(ClientError{code:"invalid_secret",message:"密钥为空、过长或包含无效字符；未保存".into()});}
 let storage=storage.inner().clone();
 tauri::async_runtime::spawn_blocking(move||{
   // Serialize config edits and credential binding in the same provider lock.
   let _=crate::provider::configured(&storage)?;
   let guard=storage.provider.lock().map_err(|_|ClientError::storage())?;
   let snapshot=guard.as_ref().ok_or_else(ClientError::storage)?.load()?;
   if snapshot.revision!=expected_revision{return Err(ClientError{code:"provider_conflict",message:"模型服务设置已变化，请重新读取并核对后保存密钥".into()});}
   let config=snapshot.config.ok_or_else(ClientError::storage)?;
   store_key_attributes(&secret,profile_attributes(&config)).map_err(client_error)?;
   store_model_key(&secret).map_err(client_error)
  }).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn clear_model_secret()->Result<(),ClientError>{
 tauri::async_runtime::spawn_blocking(remove_model_key)
  .await.map_err(|_|ClientError::storage())?
  .map_err(client_error)
}

#[cfg(test)]
mod tests{
 use super::*;
 #[test] fn validation_rejects_empty_long_and_control_bytes(){
  assert!(!valid_secret(""));
  assert!(!valid_secret(&"k".repeat(4097)));
  assert!(!valid_secret("bad\nkey"));
  assert!(!valid_secret("中文密钥"));
  assert!(valid_secret("ascii-graphic-key-123"));
 }
 #[test] fn test_identity_cannot_match_production(){assert_ne!(SCHEMA_VALUE,"io.github.dzibupro.zwen");}
 #[test] fn attributes_are_stable_for_lookup(){
  let attrs=attributes();
  assert_eq!(attrs.get(SCHEMA).map(String::as_str),Some(SCHEMA_VALUE));
  assert_eq!(attrs.get(ROLE).map(String::as_str),Some(ROLE_VALUE));
  assert_eq!(attrs.len(),2);
 }
}

#[cfg(test)]
mod ring_tests{
 use super::*;
 /// Uses only the isolated test identity in the real user keyring: stores a synthetic value, reads it
 /// back and removes it. Run explicitly: cargo test -p zwen-desktop -- --ignored
 #[test]
 #[ignore]
 fn real_keyring_roundtrip(){
  store_model_key("zwen-ring-check-123").unwrap();
  assert_eq!(read_model_key().unwrap().as_deref(),Some("zwen-ring-check-123"));
  remove_model_key().unwrap();
  assert_eq!(read_model_key().unwrap(),None);
 }
}

fn profile_attributes(config:&zwen_core::provider::ProviderConfig)->HashMap<String,String>{
 let mut a=attributes();a.insert(ROLE.into(),"paper-key-v2".into());a.insert("zwen-service".into(),config.base_url().into());a.insert("zwen-key-env".into(),config.key_env().into());a
}
fn legacy_paper_attributes(config:&zwen_core::provider::ProviderConfig)->HashMap<String,String>{
 use sha2::{Digest,Sha256};let mut a=attributes();
 let identity=serde_json::to_vec(&(config.base_url(),config.key_env())).expect("valid config identity");
 a.insert(ROLE.into(),format!("paper-key-{:x}",Sha256::digest(identity)));a
}
pub fn read_paper_key(config:&zwen_core::provider::ProviderConfig)->Result<Option<String>,SecretError>{
 if let Some(key)=read_key_attributes(profile_attributes(config))?{return Ok(Some(key));}
 // Earlier paper releases already bound this credential to the same endpoint.
 // Read that exact identity; never fall back to the generic unbound model key.
 read_key_attributes(legacy_paper_attributes(config))
}

#[cfg(test)]mod paper_binding_tests{
 use super::*;
 #[test]fn paper_key_binding_changes_with_service_not_model(){
  let a=zwen_core::provider::ProviderConfig::new("https://a.example/v1","a","ZWEN_TEST_KEY").unwrap();
  let b=zwen_core::provider::ProviderConfig::new("https://b.example/v1","a","ZWEN_TEST_KEY").unwrap();
  assert_ne!(legacy_paper_attributes(&a),legacy_paper_attributes(&b));assert_ne!(profile_attributes(&a),profile_attributes(&b));assert_ne!(profile_attributes(&a),attributes());
  assert_eq!(profile_attributes(&a),profile_attributes(&a.with_model("other-model").unwrap()));
 }
}
