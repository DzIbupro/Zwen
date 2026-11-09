//! First-time non-secret setup, using the same validation and private storage as the app.
use std::path::PathBuf;
use zwen_core::provider::{ProviderConfig,ProviderStore};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<String>=std::env::args().skip(1).collect();
 if args.len()!=4{return Err("usage: configure_provider DATA_DIR API_ROOT MODEL KEY_ENV".into());}
 let config=ProviderConfig::new(&args[1],&args[2],&args[3])?;
 let directory=PathBuf::from(&args[0]);let mut builder=std::fs::DirBuilder::new();builder.recursive(true);
 #[cfg(unix)] {use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
 builder.create(&directory)?;
 let mut store=ProviderStore::open(&directory.join("provider.sqlite3"))?;let snapshot=store.load()?;
 if snapshot.config.is_some(){println!("Existing provider configuration preserved");return Ok(());}
 store.save(snapshot.revision,config)?;println!("Non-secret provider configuration saved; no network request sent");Ok(())
}
