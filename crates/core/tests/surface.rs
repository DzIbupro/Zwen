use zwen_core::surface::{CapsuleBounds,SurfaceState};
#[test] fn compact_initial_size_and_fixed_width(){let b=CapsuleBounds::new(1920.0,1080.0);assert_eq!(b.width,480);assert_eq!(b.initial_height,96);assert_eq!(b.max_height,594);assert_eq!(b.height(300.0),300);assert_eq!(b.height(9000.0),594);}
#[test] fn malformed_heights_do_not_escape_bounds(){let b=CapsuleBounds::new(1920.0,1080.0);for h in [f64::NAN,f64::INFINITY,-50.0]{assert_eq!(b.height(h),96);}}
#[test] fn small_screens_fit(){let b=CapsuleBounds::new(360.0,240.0);assert!(b.width<=328);assert!(b.max_height<=208);assert!(b.initial_height<=b.max_height);}
#[test] fn stale_resize_cannot_change_full_surface(){let mut s=SurfaceState::new();let old=s.set_mode(true);assert!(s.accept_resize(old));s.set_mode(false);assert!(!s.accept_resize(old));let new=s.set_mode(true);assert!(new>old);assert!(!s.accept_resize(old));assert!(s.accept_resize(new));}
