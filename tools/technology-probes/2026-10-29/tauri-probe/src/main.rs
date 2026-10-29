use tauri::Manager;
use std::sync::atomic::{AtomicBool,AtomicUsize,Ordering};
static READY_COUNT:AtomicUsize=AtomicUsize::new(0);
static STARTED:AtomicBool=AtomicBool::new(false);
#[tauri::command]
fn probe_event(payload:String,app:tauri::AppHandle){
 println!("{}",serde_json::json!({"stage":"page","title":payload}));
 if payload.starts_with("SPIKE_READY:"){READY_COUNT.fetch_add(1,Ordering::SeqCst);}
 if payload.starts_with("SPIKE_READY:")&&!STARTED.swap(true,Ordering::SeqCst){
  let checks:serde_json::Value=serde_json::from_str(payload.split_once(':').unwrap().1).unwrap();
  assert!(["table","formula","image","graph","code"].iter().all(|k|checks[*k]==true));assert_eq!(checks["ticks"],50);
  std::thread::spawn(move||{
   let w=app.get_webview_window("main").unwrap();std::thread::sleep(std::time::Duration::from_secs(1));w.eval("window.runFaultSuite()").unwrap();println!("{}",serde_json::json!({"stage":"fault_requested"}));
   std::thread::sleep(std::time::Duration::from_secs(3));w.hide().unwrap();println!("{}",serde_json::json!({"stage":"hidden"}));
   std::thread::sleep(std::time::Duration::from_secs(16));w.show().unwrap();w.set_focus().unwrap();println!("{}",serde_json::json!({"stage":"restored"}));
   std::thread::sleep(std::time::Duration::from_secs(4));for i in 0..100 {if i%2==0{w.hide().unwrap()}else{w.show().unwrap()};std::thread::sleep(std::time::Duration::from_millis(20));}println!("{}",serde_json::json!({"stage":"cycles","count":100}));
   if std::env::var("SPIKE_RECOVERY_TEST").as_deref()==Ok("1") {
    println!("{}",serde_json::json!({"stage":"recovery_armed"}));
    std::thread::sleep(std::time::Duration::from_secs(2));w.reload().unwrap();println!("{}",serde_json::json!({"stage":"recovery_reload"}));
    std::thread::sleep(std::time::Duration::from_secs(9));let n=READY_COUNT.load(Ordering::SeqCst);println!("{}",serde_json::json!({"stage":"recovery_complete","ready_count":n}));
    if n<2{app.exit(3);return;}
    w.hide().unwrap();println!("{}",serde_json::json!({"stage":"long_hidden"}));
    std::thread::sleep(std::time::Duration::from_secs(60));w.show().unwrap();
    w.eval("notify('SPIKE_AFTER_IDLE:'+JSON.stringify({formula:document.querySelectorAll('.katex').length,image:document.querySelector('article img').naturalWidth,width:innerWidth,height:innerHeight,dpr:devicePixelRatio}))").unwrap();
    std::thread::sleep(std::time::Duration::from_secs(2));println!("{}",serde_json::json!({"stage":"done","ready":true}));app.exit(0);
   }else{std::thread::sleep(std::time::Duration::from_secs(3));println!("{}",serde_json::json!({"stage":"done","ready":true}));app.exit(0);}
  });
 }
}
fn main(){tauri::Builder::default().invoke_handler(tauri::generate_handler![probe_event]).setup(|app|{println!("{}",serde_json::json!({"stage":"started"}));let a=app.handle().clone();std::thread::spawn(move||{std::thread::sleep(std::time::Duration::from_secs(if std::env::var("SPIKE_RECOVERY_TEST").is_ok(){180}else{60}));println!("{}",serde_json::json!({"stage":"timeout"}));a.exit(2);});Ok(())}).run(tauri::generate_context!()).expect("spike runtime failure");}
