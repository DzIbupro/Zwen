//! Native system tray: window actions reuse normal lifecycle/save protection.
use std::sync::Mutex;
use tauri::{Emitter,Manager,menu::{Menu,MenuItem},tray::{TrayIconBuilder,TrayIconEvent,MouseButton,MouseButtonState}};
use zwen_core::cli::LaunchAction;
const ID:&str="zwen-resident";
#[derive(Debug,PartialEq)]enum Action{Open,Capsule,Screenshot,Settings,Quit}
fn action(id:&str)->Option<Action>{Some(match id{"zwen.tray.open"=>Action::Open,"zwen.tray.capsule"=>Action::Capsule,"zwen.tray.screenshot"=>Action::Screenshot,"zwen.tray.settings"=>Action::Settings,"zwen.tray.quit"=>Action::Quit,_=>return None})}
struct TrayItems {
 open: MenuItem<tauri::Wry>,
 capsule: MenuItem<tauri::Wry>,
 screenshot: MenuItem<tauri::Wry>,
 settings: MenuItem<tauri::Wry>,
 quit: MenuItem<tauri::Wry>,
}
static TRAY_ITEMS: Mutex<Option<TrayItems>> = Mutex::new(None);
pub fn refresh_locale(_app: &tauri::AppHandle) {
 let s = crate::i18n::strings();
 if let Ok(guard) = TRAY_ITEMS.lock() {
  if let Some(items) = guard.as_ref() {
   let _ = items.open.set_text(s.tray_open);
   let _ = items.capsule.set_text(s.tray_capsule);
   let _ = items.screenshot.set_text(s.tray_screenshot);
   let _ = items.settings.set_text(s.tray_settings);
   let _ = items.quit.set_text(s.tray_quit);
  }
 }
}
pub fn dispatch(app:&tauri::AppHandle,id:&str){
 match action(id){
  Some(Action::Open)=>crate::launch(app,LaunchAction::Show,None),
  Some(Action::Capsule)=>crate::launch(app,LaunchAction::Capsule,None),
  Some(Action::Screenshot)=>crate::launch(app,LaunchAction::Screenshot,None),
  Some(Action::Settings)=>{
   let app=app.clone();let activate=crate::windows::activate(app.clone(),false);
   tauri::async_runtime::spawn(async move{if activate.await.is_ok(){if let Some(window)=app.get_webview_window("main"){let _=window.emit("request-settings",());}}});
  }
  Some(Action::Quit)=>{
   if let Some(window)=app.get_webview_window("main"){let _=window.emit("request-quit",());}
   else{app.exit(0);}
  }
  None=>{}
 }
}
pub fn initialize(app: &tauri::AppHandle) {
 if let Err(error) = setup(app) {
  eprintln!("Zwen 托盘暂不可用，仍可通过应用入口或快捷键唤出：{error}");
  #[cfg(target_os = "linux")]
  {
   let app = app.clone();
   // At login the compositor can start Zwen before the panel owns the tray service.
   tauri::async_runtime::spawn(async move {
    loop {
     tokio::time::sleep(std::time::Duration::from_secs(3)).await;
     let (finished, result) = tokio::sync::oneshot::channel();
     let handle = app.clone();
     if app.run_on_main_thread(move || {
      let _ = finished.send(setup(&handle).is_ok());
     }).is_err() { break; }
     if !matches!(result.await, Ok(false)) { break; }
    }
   });
  }
 }
}

fn setup(app:&tauri::AppHandle)->tauri::Result<()>{
 let s = crate::i18n::strings();
 let open=MenuItem::with_id(app,"zwen.tray.open",s.tray_open,true,None::<&str>)?;
 let capsule=MenuItem::with_id(app,"zwen.tray.capsule",s.tray_capsule,true,None::<&str>)?;
 let screenshot=MenuItem::with_id(app,"zwen.tray.screenshot",s.tray_screenshot,true,None::<&str>)?;
 let settings=MenuItem::with_id(app,"zwen.tray.settings",s.tray_settings,true,None::<&str>)?;
 let quit=MenuItem::with_id(app,"zwen.tray.quit",s.tray_quit,true,None::<&str>)?;
 let menu=Menu::with_items(app,&[&open,&capsule,&screenshot,&settings,&quit])?;
 let mut builder=TrayIconBuilder::with_id(ID).menu(&menu).title("Zwen").tooltip("Zwen").show_menu_on_left_click(false)
  .on_menu_event(|app,event|dispatch(app,event.id.as_ref()))
  .on_tray_icon_event(|tray,event|{
   if matches!(event,TrayIconEvent::Click{button:MouseButton::Left,button_state:MouseButtonState::Up,..}){
    dispatch(tray.app_handle(),"zwen.tray.open");
   }
  });
 if let Some(icon)=app.default_window_icon(){builder=builder.icon(icon.clone());}
 builder.build(app)?;
 if let Ok(mut guard) = TRAY_ITEMS.lock() {
  *guard = Some(TrayItems { open, capsule, screenshot, settings, quit });
 }
 Ok(())
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn tray_actions_use_explicit_ids_and_ignore_unrelated_menu_events(){
  assert_eq!(action("zwen.tray.open"),Some(Action::Open));assert_eq!(action("zwen.tray.capsule"),Some(Action::Capsule));
  assert_eq!(action("zwen.tray.screenshot"),Some(Action::Screenshot));assert_eq!(action("zwen.tray.settings"),Some(Action::Settings));
  assert_eq!(action("zwen.tray.quit"),Some(Action::Quit));assert_eq!(action("quit"),None);
 }
}
