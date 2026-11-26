//! Native UI localization for tray menu and Linux selection bar.
use std::{path::Path, sync::RwLock};
use rusqlite::OptionalExtension;
use crate::{ClientError, SharedStorage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    ZhCn,
    En,
    Ja,
    Ko,
    Es,
}

impl Locale {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "zh-CN" => Some(Self::ZhCn),
            "en" => Some(Self::En),
            "ja" => Some(Self::Ja),
            "ko" => Some(Self::Ko),
            "es" => Some(Self::Es),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::En => "en",
            Self::Ja => "ja",
            Self::Ko => "ko",
            Self::Es => "es",
        }
    }
}

pub struct NativeStrings {
    pub tray_open: &'static str,
    pub tray_capsule: &'static str,
    pub tray_screenshot: &'static str,
    pub tray_settings: &'static str,
    pub tray_quit: &'static str,
    pub bar_ask: &'static str,
    pub bar_translate: &'static str,
    pub bar_explain: &'static str,
    pub bar_other: &'static str,
    pub bar_manage: &'static str,
}

#[derive(Clone, Copy)]
struct LocalePreference {
    manual: Option<Locale>,
    saved: bool,
}

static PREFERENCE: RwLock<LocalePreference> = RwLock::new(LocalePreference {manual: None, saved: false});

fn detect_system_locale(raw: &str) -> Option<Locale> {
    raw.split(':').find_map(|candidate| {
        let language = candidate.trim().split(['-', '_', '.', '@']).next()?;
        match language.to_ascii_lowercase().as_str() {
            "zh" => Some(Locale::ZhCn),
            "en" => Some(Locale::En),
            "ja" => Some(Locale::Ja),
            "ko" => Some(Locale::Ko),
            "es" => Some(Locale::Es),
            "c" | "posix" => Some(Locale::En),
            _ => None,
        }
    })
}

fn system_locale() -> Locale {
    static RESOLVED: std::sync::OnceLock<Locale> = std::sync::OnceLock::new();
    *RESOLVED.get_or_init(|| {
        let value = ["LC_ALL", "LC_MESSAGES", "LANGUAGE", "LANG"].into_iter()
            .filter_map(|name| std::env::var(name).ok())
            .find(|value| !value.trim().is_empty());
        value.as_deref().and_then(detect_system_locale).unwrap_or(Locale::En)
    })
}

pub fn current_locale() -> Locale {
    PREFERENCE.read().ok().and_then(|pref| pref.manual).unwrap_or_else(system_locale)
}

pub fn initialization_script() -> String {
    let (choice, saved) = PREFERENCE.read().map(|pref| {
        (pref.manual.map(Locale::code).unwrap_or("system"), pref.saved)
    }).unwrap_or(("system", false));
    let snapshot = serde_json::json!({
        "choice": choice,
        "systemLocale": system_locale().code(),
        "saved": saved,
    });
    format!("window.__ZWEN_LOCALE__={snapshot};")
}

pub fn strings() -> &'static NativeStrings {
    strings_for(current_locale())
}

pub fn strings_for(locale: Locale) -> &'static NativeStrings {
    match locale {
        Locale::ZhCn => &NativeStrings {
            tray_open: "打开 Zwen",
            tray_capsule: "快捷唤出",
            tray_screenshot: "截图（Shift+Alt+S）",
            tray_settings: "设置",
            tray_quit: "保存并退出",
            bar_ask: "问一问 Zwen",
            bar_translate: "翻译",
            bar_explain: "详细解释",
            bar_other: "其他",
            bar_manage: "管理快捷提示词",
        },
        Locale::En => &NativeStrings {
            tray_open: "Open Zwen",
            tray_capsule: "Quick Launch",
            tray_screenshot: "Screenshot (Shift+Alt+S)",
            tray_settings: "Settings",
            tray_quit: "Save & Quit",
            bar_ask: "Ask Zwen",
            bar_translate: "Translate",
            bar_explain: "Explain",
            bar_other: "More",
            bar_manage: "Manage Quick Prompts",
        },
        Locale::Ja => &NativeStrings {
            tray_open: "Zwen を開く",
            tray_capsule: "すばやく開く",
            tray_screenshot: "スクリーンショット (Shift+Alt+S)",
            tray_settings: "設定",
            tray_quit: "保存して終了",
            bar_ask: "Zwen に質問",
            bar_translate: "翻訳",
            bar_explain: "詳しく解説",
            bar_other: "その他",
            bar_manage: "プロンプト管理",
        },
        Locale::Ko => &NativeStrings {
            tray_open: "Zwen 열기",
            tray_capsule: "빠른 열기",
            tray_screenshot: "스크린샷 (Shift+Alt+S)",
            tray_settings: "설정",
            tray_quit: "저장 후 종료",
            bar_ask: "Zwen에게 묻기",
            bar_translate: "번역",
            bar_explain: "상세 설명",
            bar_other: "더보기",
            bar_manage: "프롬프트 관리",
        },
        Locale::Es => &NativeStrings {
            tray_open: "Abrir Zwen",
            tray_capsule: "Apertura rápida",
            tray_screenshot: "Captura (Shift+Alt+S)",
            tray_settings: "Ajustes",
            tray_quit: "Guardar y salir",
            bar_ask: "Preguntar a Zwen",
            bar_translate: "Traducir",
            bar_explain: "Explicar",
            bar_other: "Más",
            bar_manage: "Gestionar prompts",
        },
    }
}

pub(crate) fn ensure_settings_db(dir: &Path) -> Result<rusqlite::Connection, ClientError> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(dir).map_err(|_| ClientError::storage())?;
    let conn = zwen_core::open_private_database(&dir.join("settings.sqlite3"), |c| {
        let version: i64 = c.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version != 0 && version != 1 {
            return Err(zwen_core::Error::UnsupportedSchema);
        }
        if version == 0 {
            let tx = c.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            tx.execute_batch("CREATE TABLE settings(key TEXT PRIMARY KEY,value TEXT NOT NULL);PRAGMA user_version = 1;")?;
            tx.commit()?;
        }
        Ok(())
    })?;
    Ok(conn)
}

pub fn restore_from_dir(dir: &Result<std::path::PathBuf, ClientError>) {
    let Ok(dir) = dir else { return; };
    if !dir.join("settings.sqlite3").is_file() { return; }
    let Ok(conn) = ensure_settings_db(dir) else { return; };
    let pref = conn.query_row("SELECT value FROM settings WHERE key='locale'", [], |r| r.get::<_, String>(0)).optional();
    if let Ok(Some(value)) = pref {
        let manual = Locale::parse(&value);
        if (value == "system" || manual.is_some()) && let Ok(mut guard) = PREFERENCE.write() {
            *guard = LocalePreference {manual, saved: true};
        }
    }
}

#[tauri::command]
pub fn set_app_locale(
    app: tauri::AppHandle,
    storage: tauri::State<'_, SharedStorage>,
    locale: String,
) -> Result<(), ClientError> {
    let manual = Locale::parse(&locale);
    if locale != "system" && manual.is_none() {
        return Err(ClientError {code: "invalid_locale", message: "应用语言无效".into()});
    }
    let dir = storage.directory.as_ref().map_err(Clone::clone)?;
    let conn = ensure_settings_db(dir)?;
    conn.execute(
        "INSERT INTO settings(key,value) VALUES('locale',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [locale],
    ).map_err(zwen_core::Error::Database)?;
    *PREFERENCE.write().map_err(|_| ClientError::storage())? = LocalePreference {manual, saved: true};
    crate::tray::refresh_locale(&app);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn system_locale_accepts_posix_and_bcp47_locale_names() {
        assert_eq!(detect_system_locale("zh_CN.UTF-8"), Some(Locale::ZhCn));
        assert_eq!(detect_system_locale("fr:ja-JP"), Some(Locale::Ja));
        assert_eq!(detect_system_locale("C.UTF-8"), Some(Locale::En));
        assert_eq!(detect_system_locale("fr_CA"), None);
    }
}
