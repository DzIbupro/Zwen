use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use tauri::Manager;

use crate::ClientError;

const ENTRY_NAME: &str = "io.github.dzibupro.zwen.desktop";

fn entry_path(app: &tauri::AppHandle) -> tauri::Result<PathBuf> {
    Ok(app.path().config_dir()?.join("autostart").join(ENTRY_NAME))
}

fn is_enabled(path: &Path) -> io::Result<bool> {
    let entry = match fs::read_to_string(path) {
        Ok(entry) => entry,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    let fields = entry
        .lines()
        .map(str::trim)
        .skip_while(|line| *line != "[Desktop Entry]")
        .skip(1)
        .take_while(|line| !line.starts_with('['));
    let mut has_command = false;
    for (key, value) in fields.filter_map(|line| line.split_once('=')) {
        match (key.trim(), value.trim()) {
            ("Hidden", "true") | ("X-GNOME-Autostart-enabled", "false") => return Ok(false),
            ("Exec", command) => has_command = !command.is_empty(),
            _ => {}
        }
    }
    Ok(has_command)
}

fn desktop_entry(executable: &Path) -> io::Result<String> {
    let path = executable
        .to_str()
        .filter(|path| !path.chars().any(char::is_control))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid launcher path"))?;
    // Exec quoting and desktop-entry string escaping are two separate layers.
    let mut quoted = String::new();
    for character in path.chars() {
        match character {
            '\\' => quoted.push_str(r"\\\\"),
            '"' | '`' | '$' => {
                quoted.push_str(r"\\");
                quoted.push(character);
            }
            '%' => quoted.push_str("%%"),
            _ => quoted.push(character),
        }
    }
    Ok(format!(
        "[Desktop Entry]\nType=Application\nName=Zwen\nExec=/usr/bin/env -- \"{quoted}\" background\nIcon=io.github.dzibupro.zwen\nTerminal=false\n"
    ))
}

fn enable(path: &Path, executable: &Path) -> io::Result<()> {
    static NEXT_WRITE: AtomicU64 = AtomicU64::new(0);
    let entry = desktop_entry(executable)?;
    let directory = path.parent().expect("autostart entry has a parent directory");
    fs::create_dir_all(directory)?;
    let nonce = NEXT_WRITE.fetch_add(1, Ordering::Relaxed);
    let temporary = directory.join(format!(".{ENTRY_NAME}.{}.{nonce}.tmp", std::process::id()));
    let mut file = fs::File::create_new(&temporary)?;
    let result = file.write_all(entry.as_bytes()).and_then(|_| file.sync_all());
    drop(file);
    let result = result.and_then(|_| fs::rename(&temporary, path));
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

fn disable(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

#[tauri::command]
pub fn load_autostart(app: tauri::AppHandle) -> Result<bool, ClientError> {
    entry_path(&app)
        .ok()
        .and_then(|path| is_enabled(&path).ok())
        .ok_or_else(|| ClientError {
            code: "autostart_unavailable",
            message: "无法读取开机自启动设置，请重试".into(),
        })
}

#[tauri::command]
pub fn save_autostart(app: tauri::AppHandle, enabled: bool) -> Result<bool, ClientError> {
    let error = || ClientError {
        code: "autostart_unavailable",
        message: "无法保存开机自启动设置，请检查用户配置目录的权限后重试".into(),
    };
    let path = entry_path(&app).map_err(|_| error())?;
    if enabled {
        let executable = tauri::utils::platform::current_exe().map_err(|_| error())?;
        enable(&path, &executable).map_err(|_| error())?;
    } else {
        disable(&path).map_err(|_| error())?;
    }
    Ok(enabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_config(test: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("zwen-autostart-{test}-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn enabling_and_disabling_updates_the_desktop_entry() {
        let config = temp_config("toggle");
        let entry = config.join("autostart").join(ENTRY_NAME);
        assert!(!is_enabled(&entry).unwrap());

        enable(&entry, Path::new("/opt/Zwen Test/zwen")).unwrap();
        assert!(is_enabled(&entry).unwrap());
        assert!(fs::read_to_string(&entry)
            .unwrap()
            .lines()
            .any(|line| line == r#"Exec=/usr/bin/env -- "/opt/Zwen Test/zwen" background"#));

        disable(&entry).unwrap();
        disable(&entry).unwrap();
        assert!(!entry.exists());
        assert!(!is_enabled(&entry).unwrap());
        fs::remove_dir_all(config).unwrap();
    }

    #[test]
    fn reads_an_existing_entry_and_honors_hidden() {
        let config = temp_config("existing");
        let entry = config.join(ENTRY_NAME);
        let previous = "[Desktop Entry]\nType=Application\nName=Zwen\nExec=/usr/bin/env -- \"/opt/zwen\" background\n";
        fs::write(&entry, format!("{previous}Hidden=false\n")).unwrap();
        assert!(is_enabled(&entry).unwrap());
        fs::write(&entry, format!("{previous}Hidden=true\n")).unwrap();
        assert!(!is_enabled(&entry).unwrap());
        fs::remove_dir_all(config).unwrap();
    }
}
