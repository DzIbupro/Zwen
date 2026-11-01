//! Synthetic desktop checks. Excluded from the normal executable.
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::{Emitter, Listener, Manager};
static LOADS:AtomicUsize=AtomicUsize::new(0);
pub fn setup(app:&tauri::App) {
    if std::env::var("ZWEN_SMOKE").is_err() {return;}
    let fault_app=app.handle().clone();
    app.listen("fault-stage",move|_|{
        println!("FAULT_STAGE:unsaved");let h=fault_app.clone();
        std::thread::spawn(move||{if let Some(path)=std::env::var_os("ZWEN_FIXTURE_ACK"){for _ in 0..100{if std::path::Path::new(&path).exists(){let _=h.emit("fault-ready",());return;}std::thread::sleep(std::time::Duration::from_millis(50));}}});
    });
    app.listen("capsule-progress",move|e|println!("CAPSULE_PROGRESS:{}",e.payload()));
    app.listen("selection-progress",move|e|println!("SELECTION_PROGRESS:{}",e.payload()));
    app.listen("screenshot-progress",move|e|println!("SCREENSHOT_PROGRESS:{}",e.payload()));
    if std::env::var("ZWEN_SMOKE").as_deref()==Ok("selection-recovery") {
        let h=app.handle().clone();
        app.listen("selection-ready",move|_|{if let Ok(mut inbox)=h.state::<crate::selection::Shared>().lock(){let _=inbox.claim();}});
    }
    let handle=app.handle().clone();
    app.listen("foundation-smoke", move |event| {
        let payload:serde_json::Value=serde_json::from_str(event.payload()).unwrap_or_default();
        println!("FOUNDATION_SMOKE:{payload}");
        let h=handle.clone();
        if payload["ok"]!=true {h.exit(2);return;}
        std::thread::spawn(move || {
            if payload["mode"]=="chat-reload" {
                if let Some(w)=h.get_webview_window("main"){let _=w.reload();}
            } else if payload["mode"]=="edit" {
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
                let _=std::process::Command::new(exe).arg("quit").env_remove("ZWEN_SMOKE").status();
            } else {
                std::thread::sleep(std::time::Duration::from_millis(500));h.exit(0);
            }
        });
    });
    let h=app.handle().clone();
    std::thread::spawn(move||{
     let secs:u64=std::env::var("ZWEN_SMOKE_TIMEOUT_SECS").ok().and_then(|value|value.parse().ok()).unwrap_or(30);
     std::thread::sleep(std::time::Duration::from_secs(secs));h.exit(3);
    });
}
pub fn page_loaded(webview:&tauri::Webview, payload:&tauri::webview::PageLoadPayload<'_>) {
    if std::env::var("ZWEN_SMOKE").is_err() || payload.event()!=tauri::webview::PageLoadEvent::Finished {return;}
    let first=LOADS.fetch_add(1,Ordering::SeqCst)==0;
    let mode=if std::env::var("ZWEN_SMOKE").as_deref()==Ok("selection-lifecycle"){"selection-lifecycle"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("selection-recovery"){if first{"selection-recovery"}else{"selection-recovery-restored"}}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("selection"){"selection"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("screenshot"){"screenshot"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("knowledge"){"knowledge"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("stress"){"stress"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("chat-image"){"chat-image"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("chat-fault"){if first{"chat-fault"}else{"chat-fault-reload"}}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("chat-recover"){"chat-recover"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("chat"){if first{"chat"}else{"chat-restore"}}else if !first {"restore"} else {match std::env::var("ZWEN_SMOKE").as_deref(){Ok("provider")=>"provider",Ok("capsule")=>"capsule",Ok("quit")=>"quit",Ok("restore")=>"restore",_=>"edit"}};
    let source=if mode.starts_with("selection"){include_str!("../tests/selection.js")}else if mode=="screenshot"{include_str!("../tests/screenshot.js")}else if mode=="knowledge"{include_str!("../tests/knowledge.js")}else if mode=="stress"{include_str!("../tests/stress.js")}else if mode=="chat-image"{include_str!("../tests/chat-image.js")}else if mode.starts_with("chat-fault"){include_str!("../tests/chat-fault.js")}else if mode.starts_with("chat"){include_str!("../tests/chat.js")}else if mode=="provider"{include_str!("../tests/provider.js")}else if mode=="capsule"{include_str!("../tests/capsule.js")}else{include_str!("../tests/smoke.js")};
    let base=std::env::var("ZWEN_PROVIDER_TEST_BASE").unwrap_or_default();
    let script=format!("window.__ZWEN_SMOKE_MODE__={};window.__ZWEN_FIXTURE_BASE__={};{}",serde_json::to_string(mode).unwrap(),serde_json::to_string(&base).unwrap(),source);
    let _=webview.eval(&script);
}
