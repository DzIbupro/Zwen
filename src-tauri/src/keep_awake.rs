//! Task-scoped Linux sleep inhibition for explicitly started long work.
use crate::{ClientError, SharedStorage};
use rusqlite::OptionalExtension;
use std::{
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};
use tauri::{AppHandle, Manager};

#[cfg(target_os = "linux")]
type Inhibitor = zbus::zvariant::OwnedFd;
#[cfg(not(target_os = "linux"))]
type Inhibitor = ();

struct Pending {
    #[cfg(target_os = "linux")]
    request: Arc<()>,
    handle: tauri::async_runtime::JoinHandle<()>,
}

#[derive(Default)]
struct Inner {
    enabled: bool,
    active: usize,
    exiting: bool,
    pending: Option<Pending>,
    inhibitor: Option<Inhibitor>,
}

impl Inner {
    fn release(&mut self) {
        if let Some(pending) = self.pending.take() {
            pending.handle.abort();
        }
        self.inhibitor = None;
    }
}

#[derive(Clone, Default)]
pub struct State {
    inner: Arc<Mutex<Inner>>,
    preference_writes: Arc<Mutex<()>>,
}

impl State {
    pub fn new(dir: Option<&Path>) -> Self {
        let state = Self::default();
        state.lock().enabled = dir.and_then(|path| enabled(path).ok()).unwrap_or(false);
        state
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn begin(&self) -> Task {
        let mut inner = self.lock();
        inner.active += 1;
        self.maybe_inhibit(&mut inner);
        Task(self.clone())
    }

    fn set_preference(&self, dir: &Path, value: bool) -> Result<bool, ClientError> {
        // Apply concurrent saves to the runtime in the same order as the commits.
        let _write = self.preference_writes.lock().map_err(|_| ClientError::storage())?;
        set_enabled(dir, value)?;
        let mut inner = self.lock();
        inner.enabled = value;
        if value {
            self.maybe_inhibit(&mut inner);
        } else {
            inner.release();
        }
        Ok(value)
    }

    fn shutdown(&self) {
        let mut inner = self.lock();
        inner.exiting = true;
        inner.release();
    }

    #[cfg(target_os = "linux")]
    fn maybe_inhibit(&self, inner: &mut Inner) {
        if !inner.enabled || inner.active == 0 || inner.exiting || inner.inhibitor.is_some() || inner.pending.is_some() {
            return;
        }
        let request = Arc::new(());
        let expected = request.clone();
        let state = Arc::downgrade(&self.inner);
        let handle = tauri::async_runtime::spawn(async move {
            let result = tokio::time::timeout(std::time::Duration::from_secs(1), inhibit()).await;
            let Some(state) = state.upgrade() else { return };
            let mut inner = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            // A task ending, preference change or exit can invalidate the call before it returns.
            if !inner.pending.as_ref().is_some_and(|pending| Arc::ptr_eq(&pending.request, &expected)) {
                return;
            }
            inner.pending = None;
            match result {
                Ok(Ok(fd)) => inner.inhibitor = Some(fd),
                Ok(Err(error)) => eprintln!("Zwen sleep inhibitor unavailable: {error}"),
                Err(_) => eprintln!("Zwen sleep inhibitor request timed out"),
            }
        });
        inner.pending = Some(Pending { request, handle });
    }

    #[cfg(not(target_os = "linux"))]
    fn maybe_inhibit(&self, _inner: &mut Inner) {}
}

pub struct Task(State);

impl Drop for Task {
    fn drop(&mut self) {
        let mut inner = self.0.lock();
        inner.active -= 1;
        if inner.active == 0 {
            inner.release();
        }
    }
}

pub fn begin(app: &AppHandle) -> Option<Task> {
    app.try_state::<State>().map(|state| state.begin())
}

pub fn shutdown(app: &AppHandle) {
    if let Some(state) = app.try_state::<State>() {
        state.shutdown();
    }
}

#[cfg(target_os = "linux")]
async fn inhibit() -> zbus::Result<Inhibitor> {
    let connection = zbus::Connection::system().await?;
    // D-Bus method calls default to noninteractive authorization.
    let reply = connection.call_method(
        Some("org.freedesktop.login1"),
        "/org/freedesktop/login1",
        Some("org.freedesktop.login1.Manager"),
        "Inhibit",
        &("sleep", "Zwen", "Paper analysis or knowledge organization", "block"),
    ).await?;
    reply.body().deserialize()
}

fn enabled(dir: &Path) -> Result<bool, ClientError> {
    if !dir.join("settings.sqlite3").exists() {
        return Ok(false);
    }
    let db = crate::i18n::ensure_settings_db(dir)?;
    let value: Option<String> = db.query_row(
        "SELECT value FROM settings WHERE key='keep_awake'", [], |row| row.get(0),
    ).optional().map_err(zwen_core::Error::Database)?;
    Ok(value.as_deref() == Some("on"))
}

fn set_enabled(dir: &Path, value: bool) -> Result<(), ClientError> {
    let db = crate::i18n::ensure_settings_db(dir)?;
    db.execute(
        "INSERT INTO settings(key,value) VALUES('keep_awake',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [if value { "on" } else { "off" }],
    ).map_err(zwen_core::Error::Database)?;
    Ok(())
}

#[tauri::command]
pub async fn load_keep_awake(storage: tauri::State<'_, SharedStorage>) -> Result<bool, ClientError> {
    let storage = storage.inner().clone();
    tauri::async_runtime::spawn_blocking(move || enabled(storage.directory.as_ref().map_err(Clone::clone)?))
        .await.map_err(|_| ClientError::storage())?
}

#[tauri::command]
pub async fn save_keep_awake(storage: tauri::State<'_, SharedStorage>, state: tauri::State<'_, State>, enabled: bool) -> Result<bool, ClientError> {
    let storage = storage.inner().clone();
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.set_preference(storage.directory.as_ref().map_err(Clone::clone)?, enabled))
        .await.map_err(|_| ClientError::storage())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preference_is_opt_in_and_persists() {
        let dir = std::env::temp_dir().join(zwen_core::notes::generate_id());
        assert!(!enabled(&dir).unwrap());
        let state = State::new(Some(&dir));
        let _task = state.begin();
        assert!(state.lock().pending.is_none());
        set_enabled(&dir, true).unwrap();
        assert!(State::new(Some(&dir)).lock().enabled);
        set_enabled(&dir, false).unwrap();
        assert!(!State::new(Some(&dir)).lock().enabled);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(target_os = "linux")]
    fn simulated_inhibitor(state: &State) -> std::os::unix::net::UnixStream {
        let (fd, observer) = std::os::unix::net::UnixStream::pair().unwrap();
        observer.set_nonblocking(true).unwrap();
        let mut state = state.lock();
        state.enabled = true;
        state.inhibitor = Some(std::os::fd::OwnedFd::from(fd).into());
        observer
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn overlapping_tasks_release_the_fd_only_when_the_last_task_finishes() {
        use std::io::{ErrorKind, Read};
        let state = State::new(None);
        let paper = state.begin();
        let round = state.begin();
        let mut observer = simulated_inhibitor(&state);
        drop(paper);
        let mut buffer = [0];
        assert_eq!(observer.read(&mut buffer).unwrap_err().kind(), ErrorKind::WouldBlock);
        drop(round);
        assert_eq!(observer.read(&mut buffer).unwrap(), 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn quitting_releases_the_fd_while_a_task_is_running() {
        use std::io::Read;
        let state = State::new(None);
        let _task = state.begin();
        let mut observer = simulated_inhibitor(&state);
        state.shutdown();
        assert_eq!(observer.read(&mut [0]).unwrap(), 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn turning_off_the_preference_releases_the_fd_while_a_task_is_running() {
        use std::io::Read;
        let dir = std::env::temp_dir().join(zwen_core::notes::generate_id());
        let state = State::new(None);
        let _task = state.begin();
        let mut observer = simulated_inhibitor(&state);
        assert!(!state.set_preference(&dir, false).unwrap());
        assert_eq!(observer.read(&mut [0]).unwrap(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
