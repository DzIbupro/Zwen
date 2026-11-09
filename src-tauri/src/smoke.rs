//! Synthetic desktop checks. Excluded from the normal executable.
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::{Emitter, Listener, Manager};
static LOADS:AtomicUsize=AtomicUsize::new(0);
pub fn setup(app:&tauri::App) {
    if std::env::var("ZWEN_SMOKE").is_err() {return;}
    if let Some(path)=std::env::var_os("ZWEN_SCREENSHOT_FIXTURE_MODE_FILE") {
        app.listen("screenshot-fixture-mode",move|e|{
            if let Ok(mode)=serde_json::from_str::<String>(e.payload()) {
                if ["normal","cancel","timeout","escape"].contains(&mode.as_str()){let _=std::fs::write(&path,mode);}
            }
        });
    }
    #[cfg(target_os="linux")]
    if std::env::var("ZWEN_SMOKE").as_deref()==Ok("system-theme") && std::env::var("GSETTINGS_BACKEND").as_deref()==Ok("memory") {
     let h=app.handle().clone();app.listen("system-theme-fixture",move|e|{if let Ok(dark)=serde_json::from_str::<bool>(e.payload()){let _=h.run_on_main_thread(move||{use gtk::gio::prelude::*;let s=gtk::gio::Settings::new("org.gnome.desktop.interface");let _=s.set_string("color-scheme",if dark{"prefer-dark"}else{"default"});});}});
    }
    let fault_app=app.handle().clone();
    app.listen("fault-stage",move|_|{
        println!("FAULT_STAGE:unsaved");let h=fault_app.clone();
        std::thread::spawn(move||{if let Some(path)=std::env::var_os("ZWEN_FIXTURE_ACK"){for _ in 0..100{if std::path::Path::new(&path).exists(){let _=h.emit("fault-ready",());return;}std::thread::sleep(std::time::Duration::from_millis(50));}}});
    });
    if let Some(path)=std::env::var_os("ZWEN_CAPSULE_FIXTURE_ACK") {
        let h=app.handle().clone();
        app.listen("capsule-progress",move|e|{
            let payload:serde_json::Value=serde_json::from_str(e.payload()).unwrap_or_default();
            if let Some(stage)=payload["stage"].as_str().filter(|s|s.starts_with("move-")) {
                let stage=stage.to_owned();let path=path.clone();let h=h.clone();
                std::thread::spawn(move||{for _ in 0..160{if std::fs::read_to_string(&path).ok().as_deref()==Some(stage.as_str()){let _=h.emit("capsule-position-continued",stage);return;}std::thread::sleep(std::time::Duration::from_millis(50));}});
            }
        });
    }
    app.listen("capsule-progress",move|e|println!("CAPSULE_PROGRESS:{}",e.payload()));
    app.listen("selection-progress",move|e|println!("SELECTION_PROGRESS:{}",e.payload()));
    let inspect_handle=app.handle().clone();app.listen("capture-frame-inspect",move|_|{let h=inspect_handle.clone();let a=h.clone();let _=h.run_on_main_thread(move||{let visible=a.get_webview_window("main").and_then(|w|w.gtk_window().ok()).map(|w|{use gtk::prelude::*;w.is_visible()}).unwrap_or(false);let _=a.emit("capture-frame-state",serde_json::json!({"visible":visible}));});});
    app.listen("capture-pixel-stage",|e|println!("CAPTURE_PIXEL_STAGE:{}",e.payload()));
    app.listen("capture-frame-proof",|e|println!("CAPTURE_FRAME_PROOF:{}",e.payload()));
    app.listen("screenshot-progress",move|e|println!("SCREENSHOT_PROGRESS:{}",e.payload()));
    if std::env::var("ZWEN_SMOKE").as_deref()==Ok("selection-recovery") {
        let h=app.handle().clone();
        app.listen("selection-ready",move|_|{if let Ok(mut inbox)=h.state::<crate::selection::Shared>().lock(){let _=inbox.claim();}});
    }
    if let Some(path)=std::env::var_os("ZWEN_ACTIONS_FIXTURE_SELECTION_FILE") {
        app.listen("actions-fixture-selection",move|e|{if let Ok(text)=serde_json::from_str::<String>(e.payload()){let _=std::fs::write(&path,text);}});
    }
    app.listen("bar-input-ready",|_|println!("BAR_INPUT_READY"));
    app.listen("bar-input-observed",|e|println!("BAR_INPUT_OBSERVED:{}",e.payload()));
    let tray_app=app.handle().clone();
    app.listen("selection-bar-test",move|e|{if let Ok(action)=serde_json::from_str::<String>(e.payload()){
        if action=="inspect"{crate::selection_bar::smoke_inspect(&tray_app);}else if let Some(id)=action.strip_prefix("click:"){crate::selection_bar::smoke_click(&tray_app,id.to_owned());}
    }});
    if let Some(path)=std::env::var_os("ZWEN_ACTIONS_FIXTURE_SOURCE_FILE") {app.listen("actions-fixture-source-window",move|e|{if let Ok(id)=serde_json::from_str::<u64>(e.payload()){let _=std::fs::write(&path,id.to_string());}});}
    if std::env::var("ZWEN_SMOKE").as_deref()==Ok("paper-v2"){
        let h=app.handle().clone();app.listen("paper-fixture-selection",move|_|{
            if let Ok(mut inbox)=h.state::<crate::selection::Shared>().lock(){if let Some(id)=inbox.begin(zwen_core::selection::Mode::Candidate){let _=inbox.finish(id,Ok("synthetic selected sentence".into()));}}
            let _=h.emit("selection-ready",());
        });
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
    let mode=if std::env::var("ZWEN_SMOKE").as_deref()==Ok("paper-long"){"paper-long"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("paper-v2"){"paper-v2"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("capture-exit"){"capture-exit"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("capture-pixels"){"capture-pixels"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("organize-round"){"organize-round"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("history-bulk-delete"){"history-bulk-delete"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("history-delete"){"history-delete"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("capture-hide"){"capture-hide"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("system-theme"){"system-theme"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("fresh-start"){"fresh-start"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("appearance"){"appearance"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("bar-input"){"bar-input"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("actions"){"actions"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("selection-lifecycle"){"selection-lifecycle"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("selection-recovery"){if first{"selection-recovery"}else{"selection-recovery-restored"}}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("selection"){"selection"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("screenshot"){"screenshot"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("knowledge"){"knowledge"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("stress"){"stress"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("chat-image"){"chat-image"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("chat-fault"){if first{"chat-fault"}else{"chat-fault-reload"}}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("chat-recover"){"chat-recover"}else if std::env::var("ZWEN_SMOKE").as_deref()==Ok("chat"){if first{"chat"}else{"chat-restore"}}else if !first {"restore"} else {match std::env::var("ZWEN_SMOKE").as_deref(){Ok("provider")=>"provider",Ok("capsule")=>"capsule",Ok("quit")=>"quit",Ok("restore")=>"restore",_=>"edit"}};
    let source=if mode=="paper-long"{include_str!("../tests/paper-long.js")}else if mode=="paper-v2"{include_str!("../tests/paper-v2.js")}else if mode=="capture-exit"{include_str!("../tests/capture-exit.js")}else if mode=="capture-pixels"{include_str!("../tests/capture-pixels.js")}else if mode=="organize-round"{include_str!("../tests/organize-round.js")}else if mode=="history-bulk-delete"{include_str!("../tests/history-bulk-delete.js")}else if mode=="history-delete"{include_str!("../tests/history-delete.js")}else if mode=="capture-hide"{include_str!("../tests/capture-hide.js")}else if mode=="system-theme"{include_str!("../tests/system-theme.js")}else if mode=="fresh-start"{include_str!("../tests/fresh-start.js")}else if mode=="appearance"{include_str!("../tests/appearance.js")}else if mode=="bar-input"{include_str!("../tests/bar-input.js")}else if mode=="actions"{include_str!("../tests/actions.js")}else if mode.starts_with("selection"){include_str!("../tests/selection.js")}else if mode=="screenshot"{include_str!("../tests/screenshot.js")}else if mode=="knowledge"{include_str!("../tests/knowledge.js")}else if mode=="stress"{include_str!("../tests/stress.js")}else if mode=="chat-image"{include_str!("../tests/chat-image.js")}else if mode.starts_with("chat-fault"){include_str!("../tests/chat-fault.js")}else if mode.starts_with("chat"){include_str!("../tests/chat.js")}else if mode=="provider"{include_str!("../tests/provider.js")}else if mode=="capsule"{include_str!("../tests/capsule.js")}else{include_str!("../tests/smoke.js")};
    let base=std::env::var("ZWEN_PROVIDER_TEST_BASE").unwrap_or_default();
    let script=format!("window.__ZWEN_CAPSULE_POSITION_TEST__={};window.__ZWEN_SMOKE_MODE__={};window.__ZWEN_FIXTURE_BASE__={};window.__ZWEN_ACTION_OUTPUT_DIR__={};window.__ZWEN_PAPER_FIXTURE_PDF__={};{}",std::env::var_os("ZWEN_CAPSULE_FIXTURE_ACK").is_some(),serde_json::to_string(mode).unwrap(),serde_json::to_string(&base).unwrap(),serde_json::to_string(&std::env::var("ZWEN_ACTIONS_FIXTURE_OUTPUT_DIR").unwrap_or_default()).unwrap(),serde_json::to_string(&std::env::var("ZWEN_PAPER_FIXTURE_PDF").unwrap_or_default()).unwrap(),source);
    let _=webview.eval(&script);
}
