use rusqlite::{Connection, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::{fmt, path::Path, time::Duration};

pub const MAX_DRAFT_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Draft {
    pub text: String,
    pub revision: u64,
}

#[derive(Debug)]
pub enum Error {
    Database(rusqlite::Error),
    Io(std::io::Error),
    Conflict,
    TooLarge,
    UnsupportedSchema,
    InvalidProvider,
    ProviderConflict,
    InvalidChat,
    ChatConflict,
    ConversationChanged,
    ContextTooLarge,
    AttachmentLimit,
    InvalidImage,
    AttachmentMissing,
    InvalidNote,
    NoteMissing,
}
impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self { Self::Database(value) }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self { Self::Io(value) }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Database(_) | Self::Io(_) => "本地数据库无法读取或写入；原有文件已保留",
            Self::Conflict => "草稿已在别处更新；当前输入已保留，请复制后重新打开",
            Self::TooLarge => "草稿超过 256 KiB，请拆分内容后保存",
            Self::ConversationChanged => "模型配置已改变，请新建对话；旧内容不会发送到新服务",
            Self::ContextTooLarge => "最近一轮内容过长，请新建对话并带入需要的片段",
            Self::InvalidChat => "对话内容无效或超过大小限制；本机记录已保留",
            Self::ChatConflict => "对话已被更新或结束；原有回答已保留",
            Self::InvalidProvider => "模型配置无效或密钥不可用；请检查设置，原内容已保留",
            Self::ProviderConflict => "模型配置已被更新，请重新读取后再保存",
            Self::UnsupportedSchema => "数据库版本不受支持；请使用对应版本打开",
            Self::AttachmentLimit => "截图附件已达上限（最多 3 张、合计 48 MiB）；请先删除旧截图",
            Self::InvalidImage => "截图无效或超出大小限制；请删除该截图后重新截图",
            Self::AttachmentMissing => "截图文件不可读；可删除后重新截图",
            Self::InvalidNote => "笔记内容无效或超过大小限制；未保存，原文件已保留",
            Self::NoteMissing => "笔记不存在或已被删除",
        })
    }
}
impl std::error::Error for Error {}

pub struct Store { connection: Connection }
impl Store {
    pub fn open(path: &Path) -> Result<Self, Error> {
        let connection=open_private_database(path,|connection| {
        let version: i64 = connection.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version != 0 && version != 1 { return Err(Error::UnsupportedSchema); }
        if version == 0 {
            let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            tx.execute_batch("CREATE TABLE drafts (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                text TEXT NOT NULL,
                revision INTEGER NOT NULL CHECK (revision >= 0)
            ); INSERT INTO drafts VALUES (1, '', 0);
            PRAGMA user_version = 1;")?;
            tx.commit()?;
        }
            read_draft(connection)?;
            Ok(())
        })?;
        Ok(Self {connection})
    }
    pub fn load(&self) -> Result<Draft, Error> {
        read_draft(&self.connection)
    }
    pub fn save(&mut self, expected_revision: u64, text: &str) -> Result<Draft, Error> {
        if text.len() > MAX_DRAFT_BYTES { return Err(Error::TooLarge); }
        let revision = i64::try_from(expected_revision).map_err(|_| Error::Conflict)?;
        let next = revision.checked_add(1).ok_or(Error::Conflict)?;
        let tx = self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = tx.execute("UPDATE drafts SET text = ?1, revision = ?2 WHERE id = 1 AND revision = ?3", params![text, next, revision])?;
        if changed != 1 { return Err(Error::Conflict); }
        tx.commit()?;
        Ok(Draft { text: text.to_owned(), revision: next as u64 })
    }
}

pub mod cli;

pub mod surface;

pub mod paths;

pub mod stream;

pub mod provider;

fn read_draft(connection:&Connection)->Result<Draft,Error> {
        Ok(connection.query_row("SELECT text, revision FROM drafts WHERE id = 1", [], |r| {
            let revision: i64 = r.get(1)?;
            let revision = u64::try_from(revision).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(1, revision))?;
            Ok(Draft { text: r.get(0)?, revision })
        })?)
    }

pub fn open_private_database(path:&Path,initialize:impl FnOnce(&mut Connection)->Result<(),Error>)->Result<Connection,Error> {
        // Cross-thread/process first-open serialization. Schema callbacks still
        // use their own SQLite transactions; normal connection use is unlocked.
        let mut options=std::fs::OpenOptions::new();options.read(true).write(true).create(true);
        #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
        let initialization_lock=options.open(path).map_err(Error::Io)?;
        initialization_lock.lock().map_err(Error::Io)?;
        let mut connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(2))?;
        connection.set_limit(rusqlite::limits::Limit::SQLITE_LIMIT_LENGTH, 1024*1024)?;

        initialize(&mut connection)?;
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            for suffix in ["", "-wal", "-shm"] {
                let mut name=path.as_os_str().to_os_string();name.push(suffix);
                let candidate=std::path::PathBuf::from(name);
                match std::fs::metadata(&candidate) {
                    Ok(_) => std::fs::set_permissions(candidate,std::fs::Permissions::from_mode(0o600)).map_err(Error::Io)?,
                    Err(error) if error.kind()==std::io::ErrorKind::NotFound => (),
                    Err(error) => return Err(Error::Io(error)),
                }
            }
        }
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "FULL")?;
        Ok(connection)
}

pub mod completion;
pub mod tool_completion;

pub mod journal;

pub mod selection;

pub mod screenshot;

pub mod attachment;

pub mod notes;

pub mod organize;

pub mod paper;
