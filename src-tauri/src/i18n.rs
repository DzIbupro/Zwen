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
    pub fn parse(raw: &str) -> Self {
        match raw.trim() {
            "en" => Self::En,
            "ja" => Self::Ja,
            "ko" => Self::Ko,
            "es" => Self::Es,
            _ => Self::ZhCn,
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

static CURRENT_LOCALE: RwLock<Locale> = RwLock::new(Locale::ZhCn);

pub fn current_locale() -> Locale {
    CURRENT_LOCALE.read().map(|g| *g).unwrap_or(Locale::ZhCn)
}

pub fn strings() -> &'static NativeStrings {
    strings_for(current_locale())
}

pub fn strings_for(locale: Locale) -> &'static NativeStrings {
    match locale {
        Locale::ZhCn => &NativeStrings {
            tray_open: "打开 Zwen",
            tray_capsule: "快捷胶囊",
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
            tray_capsule: "Quick Capsule",
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
            tray_capsule: "クイックカプセル",
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
            tray_capsule: "빠른 캡슐",
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
            tray_capsule: "Cápsula rápida",
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

fn ensure_settings_db(dir: &Path) -> Result<rusqlite::Connection, ClientError> {
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

pub fn restore_from_dir(dir: &Result<PathBufAlias, ClientError>) {
    let Ok(dir) = dir else { return; };
    let path = dir.join("settings.sqlite3");
    if !path.exists() {
        return;
    }
    if let Ok(conn) = rusqlite::Connection::open(&path) {
        if let Ok(Some(val)) = conn
            .query_row("SELECT value FROM settings WHERE key='locale'", [], |r| r.get::<_, String>(0))
            .optional()
        {
            if let Ok(mut guard) = CURRENT_LOCALE.write() {
                *guard = Locale::parse(&val);
            }
        }
    }
}
type PathBufAlias = std::path::PathBuf;

#[tauri::command]
pub fn set_app_locale(
    app: tauri::AppHandle,
    storage: tauri::State<'_, SharedStorage>,
    locale: String,
) -> Result<(), ClientError> {
    let parsed = Locale::parse(&locale);
    if let Ok(mut guard) = CURRENT_LOCALE.write() {
        *guard = parsed;
    }
    if let Ok(dir) = storage.directory.as_ref() {
        let conn = ensure_settings_db(dir)?;
        conn.execute(
            "INSERT INTO settings(key,value) VALUES('locale',?1) ON CONFLICT(key) DO UPDATE SET value=?1",
            [parsed.code()],
        )
        .map_err(zwen_core::Error::Database)?;
    }
    crate::tray::refresh_locale(&app);
    Ok(())
}
