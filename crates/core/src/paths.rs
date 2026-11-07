use std::path::{Path,PathBuf};
/// Keep the existing database in place; do not rename a SQLite file with live WAL.
pub fn choose_data_directory(preferred:&Path,legacy:&Path,new_exists:bool,old_exists:bool)->PathBuf {
 if !new_exists&&old_exists {legacy.to_path_buf()}else{preferred.to_path_buf()}
}
