mod appearance;
mod niri;
mod selection;
mod chat;
mod provider;
mod windows;
mod attachment;
mod secret;
mod notes;
mod organize_job;
mod quick_actions;
mod tray;
mod selection_bar;
use serde::Serialize;
use std::{path::PathBuf, sync::{Arc, Mutex}};
use zwen_core::{Draft, Error as CoreError, Store, cli::{LaunchAction, parse_action}};
use tauri::{Emitter, Manager};

#[derive(Debug, Clone, Serialize)]
struct ClientError { code: &'static str, message: String }
impl ClientError {
    fn storage() -> Self { Self { code: "storage_unavailable", message: "本地存储暂不可用；原有文件与当前输入已保留".into() } }
    fn window() -> Self { Self { code: "window_unavailable", message: "窗口操作未完成，请重试".into() } }
}
impl From<CoreError> for ClientError {
    fn from(error: CoreError) -> Self {
        let code = match error { CoreError::ConversationChanged=>"conversation_changed",CoreError::ContextTooLarge=>"context_too_large",CoreError::InvalidChat=>"invalid_chat",CoreError::ChatConflict=>"chat_conflict",CoreError::InvalidProvider=>"invalid_provider",CoreError::ProviderConflict=>"provider_conflict",CoreError::Conflict=>"conflict", CoreError::TooLarge=>"too_large", CoreError::UnsupportedSchema=>"unsupported_schema", CoreError::AttachmentLimit=>"attachment_limit", CoreError::InvalidImage=>"invalid_image", CoreError::AttachmentMissing=>"attachment_missing", CoreError::InvalidNote=>"invalid_note", CoreError::NoteMissing=>"note_missing", CoreError::Database(_) | CoreError::Io(_)=>"storage_unavailable" };
        Self { code, message: error.to_string() }
    }
}
struct Storage { directory: Result<PathBuf, ClientError>, connection: Mutex<Option<Store>>, provider:Mutex<Option<zwen_core::provider::ProviderStore>> }
type SharedStorage = Arc<Storage>;
async fn database<T: Send+'static>(storage: SharedStorage, work: impl FnOnce(&mut Store)->Result<T,CoreError> + Send+'static) -> Result<T,ClientError> {
    tauri::async_runtime::spawn_blocking(move || {
        let dir = storage.directory.as_ref().map_err(Clone::clone)?;
        let mut connection = storage.connection.lock().map_err(|_|ClientError::storage())?;
        if connection.is_none() {
            let mut directory=std::fs::DirBuilder::new();directory.recursive(true);
            #[cfg(unix)] {
                use std::os::unix::fs::DirBuilderExt;
                directory.mode(0o700);
            }
            directory.create(dir).map_err(|_|ClientError::storage())?;
            *connection = Some(Store::open(&dir.join("state.sqlite3"))?);
        }
        work(connection.as_mut().ok_or_else(ClientError::storage)?).map_err(Into::into)
    }).await.map_err(|_|ClientError::storage())?
}
#[tauri::command]
async fn load_draft(storage: tauri::State<'_,SharedStorage>) -> Result<Draft,ClientError> {
    database(Arc::clone(storage.inner()), |store|store.load()).await
}
#[tauri::command]
async fn save_draft(storage: tauri::State<'_,SharedStorage>, expected_revision:u64, text:String) -> Result<Draft,ClientError> {
    database(Arc::clone(storage.inner()), move |store|store.save(expected_revision,&text)).await
}
#[tauri::command]
async fn hide_window(app:tauri::AppHandle) -> Result<(),ClientError> { windows::hide(app).await }
#[tauri::command]
fn quit_app(app:tauri::AppHandle) -> Result<(),ClientError> {
    if app.state::<selection::Shared>().lock().map_err(|_|ClientError::window())?.is_pending() {
        return Err(ClientError{code:"selection_pending",message:"有待处理选区；请先检查草稿并确认或丢弃，再退出".into()});
    }
    selection::stop_candidate_watcher();app.exit(0);Ok(())
}
#[tauri::command]
async fn get_surface(app:tauri::AppHandle)->Result<windows::Snapshot,ClientError>{tauri::async_runtime::spawn_blocking(move||windows::snapshot(&app)).await.map_err(|_|ClientError::window())?}
#[tauri::command]
async fn set_surface(app:tauri::AppHandle,mode:String)->Result<windows::Snapshot,ClientError>{
    match mode.as_str(){"full"=>windows::activate(app,false).await,"capsule"=>windows::activate(app,true).await,_=>Err(ClientError::window())}
}
#[tauri::command]
async fn resize_capsule(app:tauri::AppHandle,height:f64,generation:u64)->Result<(),ClientError>{windows::resize(app,height,generation).await}
fn launch(app:&tauri::AppHandle, action:LaunchAction,token:Option<&str>) {
    if let LaunchAction::Selection(mode)=action {selection::capture(app.clone(),mode);return;}
    if let LaunchAction::CandidateCallback=action {selection::candidate_callback(app.clone(),token);return;}
    if action==LaunchAction::Screenshot {selection::dismiss_bar(app);attachment::start_from_app(app,None);return;}
    if action == LaunchAction::Quit {
        if let Some(window)=app.get_webview_window("main") {
            let _=window.show();
            let _=window.emit("request-quit",());
        } else { app.exit(0); }
        return;
    }
    if action==LaunchAction::Recover {
        let recover=windows::recover(app.clone());tauri::async_runtime::spawn(async move{let _=recover.await;});
    } else {
        let activate=windows::activate(app.clone(),capsule_surface(action));tauri::async_runtime::spawn(async move{let _=activate.await;});
    }
}

fn capsule_surface(action:LaunchAction)->bool{matches!(action,LaunchAction::Capsule|LaunchAction::Selection(_)|LaunchAction::Screenshot)}

#[cfg(feature="desktop-smoke")]
mod smoke;

fn startup_script(preferences:appearance::Preferences)->String{
 let run=format!("{:x}-{:x}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
 format!("window.__ZWEN_RUN_ID__={};{}",serde_json::to_string(&run).expect("launch identifier"),appearance::initialization_script(preferences))
}
fn main() {
    let args:Vec<String>=std::env::args().skip(1).collect();
    let action=match parse_action(&args.iter().map(String::as_str).collect::<Vec<_>>()) {
        Ok(action)=>action, Err(message)=>{eprintln!("{message}");std::process::exit(2);}
    };
    let builder=tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app,args,_| {
            let refs:Vec<&str>=args.iter().skip(1).map(String::as_str).collect();
            if let Ok(action)=parse_action(&refs) { launch(app,action,refs.get(1).copied()); }
        }))
        .invoke_handler(tauri::generate_handler![appearance::load_appearance,appearance::save_appearance,load_draft,save_draft,hide_window,quit_app,get_surface,set_surface,resize_capsule,provider::load_provider,provider::save_provider,chat::start_chat,chat::get_chat,chat::stop_chat,chat::settle_chat,chat::get_chat_history,chat::get_chat_turn,selection::claim_selection,selection::acknowledge_selection,attachment::get_attachments,attachment::capture_screenshot,attachment::cancel_screenshot,attachment::remove_attachment,secret::save_model_secret,secret::clear_model_secret,notes::notes_save,notes::notes_list,notes::notes_read,notes::notes_delete,notes::notes_update,notes::notes_update_source,notes::notes_rebuild,organize_job::organize_run,notes::organize_topics,notes::organize_topic_notes,notes::organize_undo,notes::note_export,notes::notes_export_all,notes::open_external,notes::organize_rename,notes::organize_set_column,notes::organize_merge,notes::organize_split,notes::organize_set_locked,notes::organize_delete_topic,notes::organize_add_relation,notes::organize_remove_relation,notes::organize_list_relations,notes::organize_set_auto,notes::organize_get_auto,selection::candidate_set_enabled,selection::candidate_get_enabled,selection::freeze_selection,chat::start_selection_chat,quick_actions::load_quick_actions,quick_actions::save_quick_actions,quick_actions::save_action_document])
        .on_window_event(|window,event| {
            if let tauri::WindowEvent::ScaleFactorChanged{..}=event {windows::refresh_scale(window.app_handle().clone());}
            if let tauri::WindowEvent::CloseRequested{api,..}=event {
                api.prevent_close(); let _=window.emit("request-hide",());
            }
        })
        .setup(move |app| {
            // A stale watcher callback must never cold-start collection.
            if action==LaunchAction::CandidateCallback {app.handle().exit(0);return Ok(());}
            let directory=std::env::var_os("ZWEN_DATA_DIR").or_else(||std::env::var_os("STUDY_ASSISTANT_DATA_DIR")).map(PathBuf::from)
                .map(Ok).unwrap_or_else(|| app.path().app_data_dir().map(|preferred|{
                    let legacy=preferred.parent().unwrap_or(&preferred).join("io.github.dzibupro.studyassistant");
                    zwen_core::paths::choose_data_directory(&preferred,&legacy,preferred.join("state.sqlite3").exists(),legacy.join("state.sqlite3").exists())
                }).map_err(|_|ClientError::storage()));
            let appearance=appearance::initialize(&directory);
            app.manage(chat::Shared::default());
            app.manage(selection::Shared::default());
            app.manage(attachment::Shared::default());
            app.manage(organize_job::JobState(Mutex::new(false)));
            app.manage(Arc::new(Storage { directory, connection:Mutex::new(None),provider:Mutex::new(None) }));
            selection::restore_candidate_watcher_if_enabled(app.handle());
            app.manage(Arc::new(Mutex::new(windows::Controller::new(capsule_surface(action)))));
            #[cfg(feature="desktop-smoke")]
            smoke::setup(app);
            if action != LaunchAction::Quit {
                let window=tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
                    .title("Zwen").transparent(true)
                    .theme(Some(appearance.theme.native())).initialization_script(startup_script(appearance))
                    .inner_size(if capsule_surface(action){480.0}else{900.0},if capsule_surface(action){96.0}else{850.0})
                    .resizable(true).min_inner_size(480.0,if capsule_surface(action){96.0}else{560.0}).decorations(!capsule_surface(action)).visible(false);
                let window=if capsule_surface(action){window.max_inner_size(480.0,96.0)}else{window};
                window.on_navigation(|url| url.scheme()=="tauri" || url.host_str()==Some("tauri.localhost") || (cfg!(debug_assertions) && url.host_str()==Some("127.0.0.1") && url.port()==Some(1420)))
                    .build()?;
                if let Err(error)=tray::setup(app){eprintln!("Zwen 托盘暂不可用，仍可通过应用入口或快捷键唤出：{error}");}
            }
            appearance::watch_system(app.handle());
            launch(app.handle(), action,args.get(1).map(String::as_str));
            Ok(())
        });
    #[cfg(feature="desktop-smoke")]
    let builder=builder.on_page_load(smoke::page_loaded);
    let result=builder.build(tauri::generate_context!());
    if let Ok(app)=result {app.run(|_,event|{if matches!(event,tauri::RunEvent::Exit|tauri::RunEvent::ExitRequested{..}){selection::stop_candidate_watcher();}});} else {
        eprintln!("Zwen 启动失败；本地数据未被删除。请检查显示会话及系统依赖。");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod wiring_tests{
 /// Guards the three lists that must stay in sync: build.rs COMMANDS, the
 /// capabilities allow-* entries, and invoke_handler registration.
 #[test] fn commands_permissions_and_handler_agree(){
  fn quoted(source:&str)->std::collections::BTreeSet<String>{
   source.split('"').skip(1).step_by(2).map(str::to_owned).collect()
  }
  let build=include_str!("../build.rs");
  let commands_block=build.split("const COMMANDS").nth(1).unwrap_or("")
   .split("];").next().unwrap_or("");
  let cmds:std::collections::BTreeSet<String>=quoted(commands_block).into_iter()
   .filter(|name|!name.is_empty()&&name.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_')).collect();
  let capabilities=include_str!("../capabilities/main.json");
  let perms:std::collections::BTreeSet<String>=capabilities
   .split("\"allow-").skip(1)
   .map(|chunk|chunk.split('"').next().unwrap_or("").replace('-',"_"))
   .filter(|name|!name.is_empty())
   .collect();
  let handler=include_str!("main.rs");
  let handler_region=handler.split("tauri::generate_handler![").nth(1).unwrap_or("")
   .split(']').next().unwrap_or("");
  let handled:std::collections::BTreeSet<String>=handler_region.split(',')
   .map(|entry|entry.trim().rsplit("::").next().unwrap_or("").to_string())
   .filter(|name|!name.is_empty())
   .collect();
  let missing_perms:Vec<_>=cmds.difference(&perms).cloned().collect();
  let extra_perms:Vec<_>=perms.difference(&cmds).cloned().collect();
  let missing_handler:Vec<_>=cmds.difference(&handled).cloned().collect();
  let extra_handler:Vec<_>=handled.difference(&cmds).cloned().collect();
  assert!(missing_perms.is_empty(),"commands without permission: {missing_perms:?}");
  assert!(extra_perms.is_empty(),"permissions without command: {extra_perms:?}");
  assert!(missing_handler.is_empty(),"declared but not registered: {missing_handler:?}");
  assert!(extra_handler.is_empty(),"registered but not declared: {extra_handler:?}");
 }
}
