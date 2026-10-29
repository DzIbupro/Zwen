use crate::{SharedStorage,ClientError};
use zwen_core::{Error,provider::{ProviderConfig,ProviderSnapshot,ProviderStore}};
use serde::Serialize;
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct View {#[serde(flatten)] snapshot:ProviderSnapshot,key_available:bool}
async fn work(storage:SharedStorage,op:impl FnOnce(&mut ProviderStore)->Result<ProviderSnapshot,Error>+Send+'static)->Result<View,ClientError>{
 tauri::async_runtime::spawn_blocking(move||{
  let dir=storage.directory.as_ref().map_err(Clone::clone)?;
  let mut connection=storage.provider.lock().map_err(|_|ClientError::storage())?;
  if connection.is_none(){
   let mut builder=std::fs::DirBuilder::new();builder.recursive(true);
   #[cfg(unix)] {use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
   builder.create(dir).map_err(|_|ClientError::storage())?;
   *connection=Some(ProviderStore::open(&dir.join("provider.sqlite3"))?);
  }
  let snapshot=op(connection.as_mut().ok_or_else(ClientError::storage)?)?;
  let key_available=snapshot.config.as_ref().is_some_and(|c|c.read_key().is_ok());
  Ok(View{snapshot,key_available})
 }).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
pub async fn load_provider(storage:tauri::State<'_,SharedStorage>)->Result<View,ClientError>{work(storage.inner().clone(),|s|s.load()).await}
#[tauri::command]
pub async fn save_provider(storage:tauri::State<'_,SharedStorage>,expected_revision:u64,config:ProviderConfig)->Result<View,ClientError>{work(storage.inner().clone(),move|s|s.save(expected_revision,config)).await}

pub fn configured(storage:&SharedStorage)->Result<ProviderConfig,ClientError>{
 let dir=storage.directory.as_ref().map_err(Clone::clone)?;
 let mut builder=std::fs::DirBuilder::new();builder.recursive(true);
 #[cfg(unix)] {use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
 builder.create(dir).map_err(|_|ClientError::storage())?;
 let mut connection=storage.provider.lock().map_err(|_|ClientError::storage())?;
 if connection.is_none(){*connection=Some(ProviderStore::open(&dir.join("provider.sqlite3"))?);}
 connection.as_ref().ok_or_else(ClientError::storage)?.load()?.config.ok_or_else(||ClientError{code:"provider_missing",message:"请先在完整窗口的模型设置中配置服务".into()})
}
