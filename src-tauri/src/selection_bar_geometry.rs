//! Logical monitor-local geometry and cancellation tokens, independent of GTK.
pub fn accepts(active:u64,token:u64)->bool { active==token }
pub fn position(x: f64, y: f64, mw: i32, mh: i32, w: i32, h: i32) -> (i32, i32) {
    let above=(y as i32).saturating_sub(h+10);
    let top=if above>=0{above}else{(y as i32).saturating_add(10)};
    (((x as i32).saturating_add(10)).clamp(0, (mw-w).max(0)),top.clamp(0,(mh-h).max(0)))
}
pub fn expanded_position(left:i32,top:i32,mw:i32,mh:i32,w:i32,h:i32)->(i32,i32){(left.clamp(0,(mw-w).max(0)),top.clamp(0,(mh-h).max(0)))}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn anchor_is_near_pointer_above_and_clamped_in_logical_monitor() {
        assert_eq!(position(100., 200., 1920,1080,320,44),(110,146));
        assert_eq!(position(1919.,1079.,1920,1080,320,44),(1600,1025));
        assert_eq!(position(2.,2.,1920,1080,320,44),(12,12));
        assert_eq!(position(500.,500.,200,30,320,44),(0,0));
    }
    #[test] fn expanding_menu_preserves_primary_position_unless_screen_bounds_require_movement(){assert_eq!(expanded_position(100,200,1920,1080,320,180),(100,200));assert_eq!(expanded_position(100,1000,1920,1080,320,180),(100,900));}
    #[test] fn top_right_popup_does_not_cover_the_pointer(){let (x,y)=position(1919.,2.,1920,1080,320,44);assert!(!(1919>=x&&1919<x+320&&2>=y&&2<y+44));}
    #[test] fn dismiss_and_replacement_invalidate_late_callbacks() {
        let mut active=1; let old=active; assert!(accepts(active,old));
        active+=1; let replacement=active; assert!(!accepts(active,old)); assert!(accepts(active,replacement));
        active+=1; assert!(!accepts(active,replacement));
    }
}

#[derive(Default)]
pub struct ProbeGate { active:bool }
impl ProbeGate {
    pub fn start(&mut self) {self.active=true;}
    pub fn active(&self)->bool {self.active}
    pub fn claim(&mut self)->bool {std::mem::replace(&mut self.active,false)}
    pub fn cancel(&mut self) {self.active=false;}
}
#[cfg(test)] mod probe_tests {
    use super::*;
    #[test] fn first_enter_closes_gate_for_later_monitor_maps_and_duplicate_enters() {
        let mut gate=ProbeGate::default();gate.start();assert!(gate.active());
        assert!(gate.claim());assert!(!gate.active());assert!(!gate.claim());
        gate.start();gate.cancel();assert!(!gate.active());assert!(!gate.claim());
    }
}

pub struct PrimedAnchor<P> { id:u32,pid:u32,point:Option<P>,expired:bool }
impl<P> PrimedAnchor<P> {
    pub fn new(id:u32,pid:u32)->Self {Self{id,pid,point:None,expired:false}}
    pub fn matches(&self,id:u32,pid:u32)->bool {self.id==id&&self.pid==pid}
    pub fn capture(&mut self,point:P)->bool {if self.expired||self.point.is_some(){return false;}self.point=Some(point);true}
    #[cfg(test)]
    pub fn point_for(&self,id:u32,pid:u32)->Option<&P> {if self.matches(id,pid){self.point.as_ref()}else{None}}
    pub fn expire(&mut self) {if self.point.is_none(){self.expired=true;}}
    #[cfg(test)]
    pub fn expired(&self)->bool {self.expired}
}
#[cfg(test)] mod priming_tests {
    use super::*;
    #[test] fn delayed_show_uses_early_point_and_cannot_resample_later_pointer() {
        let mut anchor=PrimedAnchor::new(7,42);assert!(anchor.capture((100,200)));
        assert!(!anchor.capture((800,900)));assert_eq!(anchor.point_for(7,42),Some(&(100,200)));
        assert_eq!(anchor.point_for(8,42),None);assert_eq!(anchor.point_for(7,43),None);
    }
    #[test] fn expired_probe_never_falls_back_to_current_pointer() {
        let mut anchor=PrimedAnchor::new(7,42);anchor.expire();assert!(anchor.expired());
        assert!(!anchor.capture((800,900)));assert_eq!(anchor.point_for(7,42),None);
    }
}

#[cfg(test)] mod fallback_tests {
    use super::*;
    #[test] fn expired_probe_presents_current_candidate_at_fixed_position_without_late_pointer() {
        let mut anchor=PrimedAnchor::<(i32,i32)>::new(7,42);
        assert_eq!(anchor.placement_for(7,42),None);
        anchor.expire();
        assert_eq!(anchor.placement_for(7,42),Some(Placement::TopCenter));
        assert!(!anchor.capture((800,900)));
        assert_eq!(anchor.placement_for(7,42),Some(Placement::TopCenter));
        assert_eq!(anchor.placement_for(8,42),None);
        assert_eq!(anchor.placement_for(7,43),None);
    }
    #[test] fn captured_anchor_keeps_near_placement_after_deadline() {
        let mut anchor=PrimedAnchor::new(7,42);anchor.capture((100,200));anchor.expire();
        assert_eq!(anchor.placement_for(7,42),Some(Placement::Near(&(100,200))));
    }
    #[test] fn fixed_bar_is_centered_and_clamped_on_small_outputs() {
        assert_eq!(top_center(1920,1080,320,44),(800,12));
        assert_eq!(top_center(200,30,320,44),(0,0));
    }
    #[test] fn reading_output_matches_logical_monitor_without_guessing_primary() {
        let left=OutputRect{x:0,y:0,width:1920,height:1080};
        let right=OutputRect{x:1920,y:0,width:1280,height:720};
        assert_eq!(reading_monitor(Some(right),&[left,right]),Some(1));
        assert_eq!(reading_monitor(None,&[left,right]),None);
        assert_eq!(reading_monitor(Some(OutputRect{x:0,y:0,width:2560,height:1440}),&[left,right]),None);
        assert_eq!(reading_monitor(Some(right),&[right,right]),None,"ambiguous mapping must use explicit fallback");
    }
}

#[derive(Debug,PartialEq)]
pub enum Placement<P> { Near(P),TopCenter }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct OutputRect { pub x:i32,pub y:i32,pub width:i32,pub height:i32 }
pub fn reading_monitor(output:Option<OutputRect>,monitors:&[OutputRect])->Option<usize> {
    let output=output.filter(|o|o.width>0&&o.height>0)?;
    let mut matches=monitors.iter().enumerate().filter(|(_,m)|**m==output);
    let first=matches.next()?.0;if matches.next().is_some(){None}else{Some(first)}
}
pub fn top_center(mw:i32,mh:i32,w:i32,h:i32)->(i32,i32) {((mw-w).max(0)/2,12.min((mh-h).max(0)))}
impl<P> PrimedAnchor<P> {
    pub fn placement_for(&self,id:u32,pid:u32)->Option<Placement<&P>> {
        if !self.matches(id,pid){return None;}
        if let Some(point)=self.point.as_ref(){Some(Placement::Near(point))}else if self.expired{Some(Placement::TopCenter)}else{None}
    }
}
