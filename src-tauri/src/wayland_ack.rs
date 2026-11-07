//! Async wl_display.sync on GTK's own connection, dispatched by GTK's event loop.
//! Objects and listener data never leave the GTK thread. No roundtrip blocks UI.
use gtk::{glib::translate::ToGlibPtr,prelude::*};
use std::{cell::Cell,ffi::{c_char,c_int,c_void},marker::PhantomData,rc::Rc};
#[repr(C)]struct Interface{name:*const c_char,version:c_int,method_count:c_int,methods:*const c_void,event_count:c_int,events:*const c_void}
#[repr(C)]struct Listener{done:unsafe extern "C" fn(*mut c_void,*mut c_void,u32)}
#[link(name="wayland-client")]
unsafe extern "C"{
 static wl_callback_interface:Interface;
 fn wl_proxy_marshal_flags(proxy:*mut c_void,opcode:u32,interface:*const Interface,version:u32,flags:u32,...)->*mut c_void;
 fn wl_proxy_get_version(proxy:*mut c_void)->u32;
 fn wl_proxy_add_listener(proxy:*mut c_void,listener:*const Listener,data:*mut c_void)->c_int;
 fn wl_proxy_destroy(proxy:*mut c_void);
}
#[link(name="gdk-3")]
unsafe extern "C"{fn gdk_wayland_display_get_wl_display(display:*mut gtk::gdk::ffi::GdkDisplay)->*mut c_void;}
struct Data{done:Cell<bool>,proxy:Cell<*mut c_void>}
unsafe extern "C" fn done(data:*mut c_void,callback:*mut c_void,_serial:u32){
 // Listener data remains boxed in Ack until this one-shot proxy is destroyed.
 let state=unsafe{&*(data as *const Data)};
 state.proxy.set(std::ptr::null_mut());state.done.set(true);unsafe{wl_proxy_destroy(callback);}
}
static LISTENER:Listener=Listener{done};
pub struct Ack{data:Box<Data>,_gtk_thread:PhantomData<Rc<()>>}
impl Ack{
 pub fn new(display:&gtk::gdk::Display)->Option<Self>{
  if !display.type_().name().contains("Wayland"){return None;}
  let raw=unsafe{gdk_wayland_display_get_wl_display(display.to_glib_none().0)};if raw.is_null(){return None;}
  let proxy=unsafe{wl_proxy_marshal_flags(raw,0,&raw const wl_callback_interface,wl_proxy_get_version(raw),0,std::ptr::null_mut::<c_void>())};if proxy.is_null(){return None;}
  let mut data=Box::new(Data{done:Cell::new(false),proxy:Cell::new(proxy)});
  if unsafe{wl_proxy_add_listener(proxy,&LISTENER,(&mut *data as *mut Data).cast())}<0{unsafe{wl_proxy_destroy(proxy);}return None;}
  display.flush();Some(Self{data,_gtk_thread:PhantomData})
 }
 pub fn done(&self)->bool{self.data.done.get()}
}
impl Drop for Ack{fn drop(&mut self){let proxy=self.data.proxy.replace(std::ptr::null_mut());if !proxy.is_null(){unsafe{wl_proxy_destroy(proxy);}}}}
