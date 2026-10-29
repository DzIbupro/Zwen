use gtk::{glib, prelude::*};
use webkit6::prelude::*;
use std::{cell::Cell, rc::Rc, time::Duration};
fn main() -> glib::ExitCode {
 let app=gtk::Application::builder().application_id("org.example.StudyRustSpike").build();
 app.connect_activate(|app| {
  let win=gtk::ApplicationWindow::builder().application(app).title("Study Assistant · Rust GTK4 技术验证").default_width(900).default_height(850).build();
  let web=webkit6::WebView::new();win.set_child(Some(&web));let fired=Rc::new(Cell::new(false));
  let app_c=app.clone();let win_c=win.clone();
  web.connect_title_notify(move |web| {
   let title=web.title().unwrap_or_default();
   if title.starts_with("SPIKE_"){println!("{}",serde_json::json!({"stage":"page","title":title.as_str()}));}
   if title.starts_with("SPIKE_READY:") && !fired.replace(true) {
    let checks:serde_json::Value=serde_json::from_str(title.split_once(':').unwrap().1).unwrap();
    assert!(["table","formula","image","graph","code"].iter().all(|k|checks[*k]==true));assert_eq!(checks["ticks"],50);
    let w=web.clone();glib::timeout_add_local_once(Duration::from_secs(1),move||{w.evaluate_javascript("window.runFaultSuite()",None,None,gtk::gio::Cancellable::NONE,|_|{});println!("{}",serde_json::json!({"stage":"fault_requested"}));});
    let w=win_c.clone();glib::timeout_add_local_once(Duration::from_secs(4),move||{w.set_visible(false);println!("{}",serde_json::json!({"stage":"hidden"}));});
    let w=win_c.clone();glib::timeout_add_local_once(Duration::from_secs(20),move||{w.present();println!("{}",serde_json::json!({"stage":"restored"}));});
    let w=win_c.clone();glib::timeout_add_local_once(Duration::from_secs(24),move||{let n=Rc::new(Cell::new(0));glib::timeout_add_local(Duration::from_millis(20),move||{n.set(n.get()+1);w.set_visible(n.get()%2==0);if n.get()==100 {w.present();println!("{}",serde_json::json!({"stage":"cycles","count":100}));glib::ControlFlow::Break}else{glib::ControlFlow::Continue}});});
    let a=app_c.clone();glib::timeout_add_local_once(Duration::from_secs(29),move||{println!("{}",serde_json::json!({"stage":"done","ready":true}));a.quit();});
   }
  });
  web.connect_web_process_terminated(|_,r|{println!("{}",serde_json::json!({"stage":"web_process_terminated","reason":format!("{r:?}")}));});
  let root=std::env::var("STUDY_SPIKE_ROOT").expect("STUDY_SPIKE_ROOT");web.load_uri(&format!("file://{root}/web/index.html"));win.present();println!("{}",serde_json::json!({"stage":"started"}));
  let a=app.clone();glib::timeout_add_local_once(Duration::from_secs(60),move||{println!("{}",serde_json::json!({"stage":"timeout"}));a.quit();});
 });app.run()
}
