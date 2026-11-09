use std::path::Path;
use zwen_core::paths::choose_data_directory;
#[test] fn existing_zwen_wins(){assert_eq!(choose_data_directory(Path::new("new"),Path::new("old"),true,true),Path::new("new"));}
#[test] fn old_draft_directory_is_reused_without_moving_files(){assert_eq!(choose_data_directory(Path::new("new"),Path::new("old"),false,true),Path::new("old"));}
#[test] fn new_install_uses_zwen(){assert_eq!(choose_data_directory(Path::new("new"),Path::new("old"),false,false),Path::new("new"));}
