use serde::{Serialize,Deserialize};
#[derive(Debug,Clone,Copy,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct CapsuleBounds { pub width:u32, pub initial_height:u32, pub max_height:u32 }
impl CapsuleBounds {
    pub fn new(width:f64,height:f64)->Self {
        let w=if width.is_finite()&&width>0.0 {width} else {1920.0};
        let h=if height.is_finite()&&height>0.0 {height} else {1080.0};
        let max_height=(h*0.55).floor().max(1.0).min((h-32.0).max(1.0)) as u32;
        Self {width:(w-32.0).max(1.0).min(480.0) as u32, initial_height:96.min(max_height),max_height}
    }
    pub fn height(&self,requested:f64)->u32 {
        if !requested.is_finite()||requested<0.0 {return self.initial_height;}
        requested.ceil().clamp(self.initial_height as f64,self.max_height as f64) as u32
    }
}
#[derive(Default)]
pub struct SurfaceState { generation:u64, capsule:bool }
impl SurfaceState {
    pub fn new()->Self {Self::default()}
    pub fn set_mode(&mut self,capsule:bool)->u64 {self.capsule=capsule;self.generation+=1;self.generation}
    pub fn accept_resize(&self,generation:u64)->bool {self.capsule&&self.generation==generation}
    pub fn is_capsule(&self)->bool {self.capsule}
    pub fn generation(&self)->u64 {self.generation}
}
