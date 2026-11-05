//! Native system tray: window actions reuse normal lifecycle/save protection.
use tauri::{Emitter,Manager,menu::{Menu,MenuItem},tray::{TrayIconBuilder,TrayIconEvent,MouseButton,MouseButtonState}};
use zwen_core::cli::LaunchAction;
const ID:&str="zwen-resident";
#[derive(Debug,PartialEq)]enum Action{Open,Capsule,Screenshot,Settings,Quit}
fn action(id:&str)->Option<Action>{Some(match id{"zwen.tray.open"=>Action::Open,"zwen.tray.capsule"=>Action::Capsule,"zwen.tray.screenshot"=>Action::Screenshot,"zwen.tray.settings"=>Action::Settings,"zwen.tray.quit"=>Action::Quit,_=>return None})}
pub fn dispatch(app:&tauri::AppHandle,id:&str){
 match action(id){
  Some(Action::Open)=>crate::launch(app,LaunchAction::Show,None),
  Some(Action::Capsule)=>crate::launch(app,LaunchAction::Capsule,None),
  Some(Action::Screenshot)=>crate::launch(app,LaunchAction::Screenshot,None),
  Some(Action::Settings)|Some(Action::Quit)=>{
   let event=if action(id)==Some(Action::Settings){"request-settings"}else{"request-quit"};
   let app=app.clone();let activate=crate::windows::activate(app.clone(),false);
   tauri::async_runtime::spawn(async move{if activate.await.is_ok(){if let Some(window)=app.get_webview_window("main"){let _=window.emit(event,());}}});
  }
  None=>{}
 }
}
pub fn setup(app:&tauri::App)->tauri::Result<()>{
 let open=MenuItem::with_id(app,"zwen.tray.open","打开 Zwen",true,None::<&str>)?;
 let capsule=MenuItem::with_id(app,"zwen.tray.capsule","快捷胶囊",true,None::<&str>)?;
 let screenshot=MenuItem::with_id(app,"zwen.tray.screenshot","截图（Shift+Alt+S）",true,None::<&str>)?;
 let settings=MenuItem::with_id(app,"zwen.tray.settings","设置",true,None::<&str>)?;
 let quit=MenuItem::with_id(app,"zwen.tray.quit","保存并退出",true,None::<&str>)?;
 let menu=Menu::with_items(app,&[&open,&capsule,&screenshot,&settings,&quit])?;
 let mut builder=TrayIconBuilder::with_id(ID).menu(&menu).title("Zwen").tooltip("Zwen").show_menu_on_left_click(false)
  .on_menu_event(|app,event|dispatch(app,event.id.as_ref()))
  .on_tray_icon_event(|tray,event|{
   if matches!(event,TrayIconEvent::Click{button:MouseButton::Left,button_state:MouseButtonState::Up,..}){
    dispatch(tray.app_handle(),"zwen.tray.open");
   }
  });
 if let Some(icon)=app.default_window_icon(){builder=builder.icon(icon.clone());}
 builder.build(app)?;Ok(())
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn tray_actions_use_explicit_ids_and_ignore_unrelated_menu_events(){
  assert_eq!(action("zwen.tray.open"),Some(Action::Open));assert_eq!(action("zwen.tray.capsule"),Some(Action::Capsule));
  assert_eq!(action("zwen.tray.screenshot"),Some(Action::Screenshot));assert_eq!(action("zwen.tray.settings"),Some(Action::Settings));
  assert_eq!(action("zwen.tray.quit"),Some(Action::Quit));assert_eq!(action("quit"),None);
 }
}
