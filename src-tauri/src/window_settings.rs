//! Desktop preferences shared by the shortcut, tray and the main window.
use crate::{ClientError, SharedStorage};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::atomic::{AtomicBool, Ordering}};
use tauri::{AppHandle, Manager};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuickSurface {
    Capsule,
    Full,
}

pub struct State(AtomicBool);

impl State {
    pub fn new(directory: Option<&Path>) -> Self {
        let saved = directory.and_then(|path| load_surface(path).ok()).unwrap_or(QuickSurface::Capsule);
        Self(AtomicBool::new(saved == QuickSurface::Capsule))
    }

    fn get(&self) -> QuickSurface {
        if self.0.load(Ordering::Acquire) { QuickSurface::Capsule } else { QuickSurface::Full }
    }
}

pub fn quick_capsule(app: &AppHandle) -> bool {
    app.try_state::<State>().is_none_or(|state| state.get() == QuickSurface::Capsule)
}

fn load_surface(directory: &Path) -> Result<QuickSurface, ClientError> {
    if !directory.join("settings.sqlite3").is_file() { return Ok(QuickSurface::Capsule); }
    let db = crate::i18n::ensure_settings_db(directory)?;
    let value: Option<String> = db.query_row(
        "SELECT value FROM settings WHERE key='quick_surface'", [], |row| row.get(0),
    ).optional().map_err(zwen_core::Error::Database)?;
    Ok(if value.as_deref() == Some("full") { QuickSurface::Full } else { QuickSurface::Capsule })
}

#[tauri::command]
pub fn load_quick_surface(state: tauri::State<'_, State>) -> QuickSurface {
    state.get()
}

#[tauri::command]
pub fn save_quick_surface(
    storage: tauri::State<'_, SharedStorage>,
    state: tauri::State<'_, State>,
    surface: QuickSurface,
) -> Result<QuickSurface, ClientError> {
    let directory = storage.directory.as_ref().map_err(Clone::clone)?;
    let db = crate::i18n::ensure_settings_db(directory)?;
    let capsule = surface == QuickSurface::Capsule;
    db.execute(
        "INSERT INTO settings(key,value) VALUES('quick_surface',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [if capsule { "capsule" } else { "full" }],
    ).map_err(zwen_core::Error::Database)?;
    state.0.store(capsule, Ordering::Release);
    Ok(surface)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveTasks {
    chat_active: bool,
    organize_active: bool,
}

#[tauri::command]
pub async fn get_active_tasks(app: AppHandle) -> Result<ActiveTasks, ClientError> {
    tauri::async_runtime::spawn_blocking(move || {
        let chat_active = crate::chat::has_active_request(&app.state::<crate::chat::Shared>())?;
        let organize_active = *app.state::<crate::organize_job::JobState>().0.lock()
            .map_err(|_| ClientError::storage())?;
        Ok(ActiveTasks {chat_active, organize_active})
    }).await.map_err(|_| ClientError::window())?
}

#[tauri::command]
pub async fn confirm_app_leave(
    app: AppHandle,
    title: String,
    detail: String,
    accept: String,
    cancel: String,
) -> Result<bool, ClientError> {
    if title.len() > 256 || detail.len() > 1024 || accept.len() > 128 || cancel.len() > 128 {
        return Err(ClientError::window());
    }
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let handle = app.clone();
    app.run_on_main_thread(move || {
        use gtk::prelude::*;
        let parent = handle.get_webview_window("main").and_then(|window| window.gtk_window().ok())
            .filter(|window| window.is_visible());
        let dialog = gtk::MessageDialog::new(
            parent.as_ref(), gtk::DialogFlags::MODAL, gtk::MessageType::Warning, gtk::ButtonsType::None, &title,
        );
        dialog.set_title("Zwen");
        dialog.set_position(gtk::WindowPosition::Center);
        gtk::prelude::MessageDialogExt::set_secondary_text(&dialog, Some(&detail));
        dialog.add_button(&cancel, gtk::ResponseType::Cancel);
        dialog.add_button(&accept, gtk::ResponseType::Accept);
        dialog.set_default_response(gtk::ResponseType::Cancel);
        let sender = std::cell::RefCell::new(Some(sender));
        let keeper = std::rc::Rc::new(std::cell::RefCell::new(Some(dialog.clone())));
        dialog.connect_response(move |dialog, response| {
            if let Some(sender) = sender.borrow_mut().take() {
                dialog.hide();
                keeper.borrow_mut().take();
                // The response is final and the dialog is never used again.
                unsafe { dialog.destroy(); }
                let _ = sender.send(response == gtk::ResponseType::Accept);
            }
        });
        dialog.show();
    }).map_err(|_| ClientError::window())?;
    receiver.await.map_err(|_| ClientError::window())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_surface_defaults_to_capsule_and_restores_a_saved_full_window() {
        let dir = std::env::temp_dir().join(zwen_core::notes::generate_id());
        assert_eq!(State::new(Some(&dir)).get(), QuickSurface::Capsule);
        let db = crate::i18n::ensure_settings_db(&dir).unwrap();
        db.execute("INSERT INTO settings(key,value) VALUES('quick_surface','full')", []).unwrap();
        assert_eq!(State::new(Some(&dir)).get(), QuickSurface::Full);
        drop(db);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
