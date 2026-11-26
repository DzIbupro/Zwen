//! Background task notifications through the Linux desktop notification service.
use crate::{ClientError, SharedStorage};
use rusqlite::OptionalExtension;
use serde::Serialize;
use std::{collections::HashMap, path::Path, time::Duration};
use tauri::{AppHandle, Emitter, Manager};
use zbus::{Connection, Proxy, export::futures_core::Stream, zvariant::Value};
use zwen_core::journal::TurnState;

fn enabled(dir: &Path) -> Result<bool, ClientError> {
    if !dir.join("settings.sqlite3").exists() {
        return Ok(false);
    }
    let db = crate::i18n::ensure_settings_db(dir)?;
    let value: Option<String> = db.query_row(
        "SELECT value FROM settings WHERE key='background_notifications'", [], |row| row.get(0),
    ).optional().map_err(zwen_core::Error::Database)?;
    Ok(value.as_deref() == Some("on"))
}

fn set_enabled(dir: &Path, value: bool) -> Result<bool, ClientError> {
    let db = crate::i18n::ensure_settings_db(dir)?;
    db.execute(
        "INSERT INTO settings(key,value) VALUES('background_notifications',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [if value {"on"} else {"off"}],
    ).map_err(zwen_core::Error::Database)?;
    Ok(value)
}

#[tauri::command]
pub async fn load_background_notifications(storage: tauri::State<'_, SharedStorage>) -> Result<bool, ClientError> {
    let storage = storage.inner().clone();
    tauri::async_runtime::spawn_blocking(move || enabled(storage.directory.as_ref().map_err(Clone::clone)?))
        .await.map_err(|_| ClientError::storage())?
}

#[tauri::command]
pub async fn save_background_notifications(storage: tauri::State<'_, SharedStorage>, enabled: bool) -> Result<bool, ClientError> {
    let storage = storage.inner().clone();
    tauri::async_runtime::spawn_blocking(move || set_enabled(storage.directory.as_ref().map_err(Clone::clone)?, enabled))
        .await.map_err(|_| ClientError::storage())?
}

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResultTarget {
    Paper { id: String },
    Organize { id: String, completed: usize, total: usize, success: bool },
}

pub fn paper_outcome(conversation: &str, state: &TurnState, saved: bool) -> Option<bool> {
    if !conversation.starts_with("paper-") {
        return None;
    }
    match state {
        TurnState::Complete => Some(saved),
        TurnState::Failed | TurnState::Limited | TurnState::Filtered => Some(false),
        _ => None,
    }
}

fn notification_text(target: &ResultTarget, success: bool) -> (&'static str, &'static str) {
    use crate::i18n::{current_locale, Locale};
    let paper = matches!(target, ResultTarget::Paper { .. });
    let (paper_ok, paper_failed, organize_ok, organize_failed, open) = match current_locale() {
        Locale::ZhCn => ("论文分析完成", "论文分析未完成", "知识整理完成", "知识整理未完成", "打开结果"),
        Locale::En => ("Paper analysis complete", "Paper analysis did not complete", "Knowledge organization complete", "Knowledge organization did not complete", "Open result"),
        Locale::Ja => ("論文分析が完了しました", "論文分析が完了しませんでした", "ナレッジ整理が完了しました", "ナレッジ整理が完了しませんでした", "結果を開く"),
        Locale::Ko => ("논문 분석 완료", "논문 분석 미완료", "지식 정리 완료", "지식 정리 미완료", "결과 열기"),
        Locale::Es => ("Análisis del artículo completado", "Análisis del artículo incompleto", "Organización de notas completada", "Organización de notas incompleta", "Abrir resultado"),
    };
    let title = match (paper, success) {
        (true, true) => paper_ok,
        (true, false) => paper_failed,
        (false, true) => organize_ok,
        (false, false) => organize_failed,
    };
    (title, open)
}

pub fn send(app: &AppHandle, dir: &Path, target: ResultTarget, success: bool) {
    let Some(window) = app.get_webview_window("main") else { return; };
    if window.is_visible().ok() != Some(false) || !enabled(dir).unwrap_or(false) {
        return;
    }
    let (title, open) = notification_text(&target, success);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Ok(Ok(true)) = tokio::time::timeout(Duration::from_secs(3600), show(title, open)).await {
            if crate::windows::activate(app.clone(), false).await.is_ok() {
                let _ = app.emit("open-task-result", target);
            }
        }
    });
}

async fn show(title: &str, open: &str) -> zbus::Result<bool> {
    let connection = Connection::session().await?;
    let proxy = Proxy::new(
        &connection, "org.freedesktop.Notifications", "/org/freedesktop/Notifications", "org.freedesktop.Notifications",
    ).await?;
    let mut signals = std::pin::pin!(proxy.receive_all_signals().await?);
    let id: u32 = proxy.call("Notify", &notification_args(title, open)).await?;
    while let Some(signal) = std::future::poll_fn(|cx| signals.as_mut().poll_next(cx)).await {
        match signal.header().member().map(|name| name.as_str()) {
            Some("ActionInvoked") => {
                let body = signal.body();
                let (received, action): (u32, &str) = body.deserialize()?;
                if received == id && matches!(action, "default" | "open") { return Ok(true); }
            }
            Some("NotificationClosed") => {
                let (received, _): (u32, u32) = signal.body().deserialize()?;
                if received == id { return Ok(false); }
            }
            _ => {}
        }
    }
    Ok(false)
}
type NotifyArgs<'a> = (&'a str, u32, &'a str, &'a str, &'a str, Vec<&'a str>, HashMap<&'a str, Value<'a>>, i32);

fn notification_args<'a>(title: &'a str, open: &'a str) -> NotifyArgs<'a> {
    ("Zwen", 0, "io.github.dzibupro.zwen", title, "", vec!["default", open, "open", open], HashMap::new(), -1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zwen_core::journal::TurnState;

    #[test]
    fn notify_call_matches_the_freedesktop_wire_signature() {
        let message = zbus::Message::method_call("/org/freedesktop/Notifications", "Notify")
            .unwrap().build(&notification_args("Result", "Open")).unwrap();
        assert_eq!(message.header().signature().to_string_no_parens(), "susssasa{sv}i");
    }

    #[test]
    fn only_background_paper_terminal_outcomes_notify() {
        assert_eq!(paper_outcome("paper-test", &TurnState::Complete, true), Some(true));
        assert_eq!(paper_outcome("paper-test", &TurnState::Failed, true), Some(false));
        assert_eq!(paper_outcome("paper-test", &TurnState::Complete, false), Some(false));
        assert_eq!(paper_outcome("paper-test", &TurnState::Stopped, true), None);
        assert_eq!(paper_outcome("regular-test", &TurnState::Complete, true), None);
    }

    #[test]
    fn preference_is_opt_in_and_persists() {
        let dir = std::env::temp_dir().join(zwen_core::notes::generate_id());
        assert!(!enabled(&dir).unwrap());
        set_enabled(&dir, true).unwrap();
        assert!(enabled(&dir).unwrap());
        set_enabled(&dir, false).unwrap();
        assert!(!enabled(&dir).unwrap());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
