use zwen_core::provider::{ProviderConfig,ProviderStore,ApiKey};
fn config()->ProviderConfig{ProviderConfig::new("https://models.example/v1/","model-a","ZWEN_TEST_KEY").unwrap()}
fn path()->std::path::PathBuf{let p=std::env::temp_dir().join(format!("zwen-provider-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));std::fs::create_dir(&p).unwrap();p.join("provider.sqlite3")}
#[test] fn validates_endpoint_model_and_environment(){for url in ["http://remote.example/v1","https://u:password@models.example/v1","https://models.example/v1?key=secret","https://models.example/#fragment","file:///tmp/a"]{assert!(ProviderConfig::new(url,"model","KEY").is_err());}for url in ["https://models.example/v1","http://127.0.0.1:12345/v1","http://[::1]:12345/v1"]{assert!(ProviderConfig::new(url,"model","KEY").is_ok());}assert!(ProviderConfig::new("https://example.com","bad\nmodel","KEY").is_err());assert!(ProviderConfig::new("https://example.com","model","bad-name").is_err());}
#[test] fn rejects_unknown_secret_field(){assert!(serde_json::from_str::<ProviderConfig>(r#"{"baseUrl":"https://example.com/v1","model":"m","keyEnv":"KEY","apiKey":"secret"}"#).is_err());}
#[test] fn key_is_not_debugged_or_empty(){let k=ApiKey::new("synthetic-token").unwrap();assert!(!format!("{k:?}").contains("synthetic"));for key in ["","a\rb","a\nb","  "]{assert!(ApiKey::new(key).is_err());}}
#[test] fn config_roundtrip_cas_and_restart(){let p=path();let mut s=ProviderStore::open(&p).unwrap();assert_eq!(s.load().unwrap().revision,0);assert!(s.load().unwrap().config.is_none());let saved=s.save(0,config()).unwrap();assert_eq!(saved.revision,1);assert!(s.save(0,config()).is_err());drop(s);let s=ProviderStore::open(&p).unwrap();assert_eq!(s.load().unwrap(),saved);drop(s);std::fs::remove_dir_all(p.parent().unwrap()).unwrap();}
#[test] fn malformed_config_is_preserved(){let p=path();drop(ProviderStore::open(&p).unwrap());let c=rusqlite::Connection::open(&p).unwrap();c.execute("UPDATE provider_config SET config='broken'",[]).unwrap();drop(c);assert!(ProviderStore::open(&p).is_err());let c=rusqlite::Connection::open(&p).unwrap();assert_eq!(c.query_row("SELECT config FROM provider_config",[],|r|r.get::<_,String>(0)).unwrap(),"broken");drop(c);std::fs::remove_dir_all(p.parent().unwrap()).unwrap();}
#[test] fn provider_db_does_not_modify_draft_schema(){let p=path();let draft_path=p.with_file_name("state.sqlite3");{let mut s=zwen_core::Store::open(&draft_path).unwrap();s.save(0,"保留草稿").unwrap();}{let mut s=ProviderStore::open(&p).unwrap();s.save(0,config()).unwrap();}let s=zwen_core::Store::open(&draft_path).unwrap();assert_eq!(s.load().unwrap().text,"保留草稿");drop(s);let c=rusqlite::Connection::open(&draft_path).unwrap();assert_eq!(c.pragma_query_value(None,"user_version",|r|r.get::<_,i64>(0)).unwrap(),1);drop(c);std::fs::remove_dir_all(p.parent().unwrap()).unwrap();}
#[cfg(unix)] #[test] fn provider_database_is_private(){use std::os::unix::fs::PermissionsExt;let p=path();let mut s=ProviderStore::open(&p).unwrap();s.save(0,config()).unwrap();assert_eq!(std::fs::metadata(&p).unwrap().permissions().mode()&0o077,0);drop(s);std::fs::remove_dir_all(p.parent().unwrap()).unwrap();}
#[test] fn future_provider_schema_is_preserved(){let p=path();let c=rusqlite::Connection::open(&p).unwrap();c.pragma_update(None,"user_version",99).unwrap();drop(c);assert!(ProviderStore::open(&p).is_err());let c=rusqlite::Connection::open(&p).unwrap();assert_eq!(c.pragma_query_value(None,"user_version",|r|r.get::<_,i64>(0)).unwrap(),99);drop(c);std::fs::remove_dir_all(p.parent().unwrap()).unwrap();}
#[test] fn normalized_url_cannot_exceed_roundtrip_limit(){let raw=format!("https://example.com/{}","中".repeat(500));assert!(raw.len()<2048);assert!(ProviderConfig::new(&raw,"m","KEY").is_err());}
#[test] fn image_model_is_optional_and_legacy_profiles_stay_bound(){
 let base=config();
 assert_eq!(base.image_model(),None);
 // Without an image model the serialized profile is byte-identical to the
 // pre-vision format, so existing conversation bindings keep matching.
 assert_eq!(serde_json::to_string(&base).unwrap(),format!("{{\"baseUrl\":\"{}\",\"model\":\"model-a\",\"keyEnv\":\"ZWEN_TEST_KEY\"}}",base.base_url()));
 let legacy:ProviderConfig=serde_json::from_str(r#"{"baseUrl":"https://models.example/v1","model":"model-a","keyEnv":"ZWEN_TEST_KEY"}"#).unwrap();
 assert_eq!(legacy,base);
 for blank in ["","   "]{assert_eq!(ProviderConfig::build("https://example.com","model","KEY",Some(blank)).unwrap().image_model(),None);}
}
#[test] fn image_model_is_validated_and_routed_without_touching_the_stored_config(){
 for bad in ["bad\nmodel",&"m".repeat(201)]{assert!(ProviderConfig::build("https://example.com","model","KEY",Some(bad)).is_err());}
 let good=ProviderConfig::build("https://example.com","model","KEY",Some(" glm-4.6v-flash ")).unwrap();
 assert_eq!(good.image_model(),Some("glm-4.6v-flash"));
 assert_eq!(serde_json::to_string(&good).unwrap(),r#"{"baseUrl":"https://example.com","model":"model","keyEnv":"KEY","imageModel":"glm-4.6v-flash"}"#);
 let routed=good.with_model("text-model").unwrap();
 assert_eq!(routed.model(),"text-model");assert_eq!(routed.image_model(),Some("glm-4.6v-flash"));
 assert!(good.with_model("bad\nmodel").is_err());
 let roundtrip:ProviderConfig=serde_json::from_str(&serde_json::to_string(&good).unwrap()).unwrap();
 assert_eq!(roundtrip,good);
}
