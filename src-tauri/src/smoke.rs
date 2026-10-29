//! Synthetic desktop checks. Excluded from the normal executable.
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::{Listener, Manager};
static LOADS:AtomicUsize=AtomicUsize::new(0);
pub fn setup(app:&tauri::App) {
    if std::env::var("STUDY_ASSISTANT_SMOKE").is_err() {return;}
    let handle=app.handle().clone();
    app.listen("foundation-smoke", move |event| {
        let payload:serde_json::Value=serde_json::from_str(event.payload()).unwrap_or_default();
        println!("FOUNDATION_SMOKE:{payload}");
        let h=handle.clone();
        if payload["ok"]!=true {h.exit(2);return;}
        std::thread::spawn(move || {
            if payload["mode"]=="edit" {
                if let Some(w)=h.get_webview_window("main") {
                    let began=std::time::Instant::now();
                    let _=w.close();
                    let mut hidden=false;
                    for _ in 0..30 {
                        if matches!(w.is_visible(),Ok(false)) { hidden=true;break; }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    if !hidden { println!("FOUNDATION_SMOKE:close-failed");h.exit(2);return; }
                    println!("FOUNDATION_SMOKE:hidden:{}ms",began.elapsed().as_millis());
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    let _=w.show();let _=w.reload();
                }
            } else if payload["mode"]=="quit" {
                let exe=std::env::current_exe().unwrap();
                let _=std::process::Command::new(exe).arg("quit").env_remove("STUDY_ASSISTANT_SMOKE").status();
            } else {
                std::thread::sleep(std::time::Duration::from_millis(500));h.exit(0);
            }
        });
    });
    let h=app.handle().clone();
    std::thread::spawn(move||{std::thread::sleep(std::time::Duration::from_secs(30));h.exit(3);});
}
pub fn page_loaded(webview:&tauri::Webview, payload:&tauri::webview::PageLoadPayload<'_>) {
    if std::env::var("STUDY_ASSISTANT_SMOKE").is_err() || payload.event()!=tauri::webview::PageLoadEvent::Finished {return;}
    let first=LOADS.fetch_add(1,Ordering::SeqCst)==0;
    let mode=if !first {"restore"} else {match std::env::var("STUDY_ASSISTANT_SMOKE").as_deref(){Ok("quit")=>"quit",Ok("restore")=>"restore",_=>"edit"}};
    let script=format!("window.__STUDY_SMOKE_MODE__={};{}",serde_json::to_string(mode).unwrap(),include_str!("../tests/smoke.js"));
    let _=webview.eval(&script);
}
