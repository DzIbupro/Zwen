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
}
impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self { Self::Database(value) }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Database(_) | Self::Io(_) => "本地数据库无法读取或写入；原有文件已保留",
            Self::Conflict => "草稿已在别处更新；当前输入已保留，请复制后重新打开",
            Self::TooLarge => "草稿超过 256 KiB，请拆分内容后保存",
            Self::UnsupportedSchema => "数据库来自更新版本；请使用对应版本打开",
        })
    }
}
impl std::error::Error for Error {}

pub struct Store { connection: Connection }
impl Store {
    pub fn open(path: &Path) -> Result<Self, Error> {
        #[cfg(unix)] {
            use std::os::unix::fs::OpenOptionsExt;
            match std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path) {
                Ok(_) => (),
                Err(error) if error.kind()==std::io::ErrorKind::AlreadyExists => (),
                Err(error) => return Err(Error::Io(error)),
            }
        }
        let mut connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(2))?;
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
        let store = Self { connection };
        store.load()?; // Fail closed on an incompatible/damaged schema, without repairing it.
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
        store.connection.pragma_update(None, "journal_mode", "WAL")?;
        store.connection.pragma_update(None, "synchronous", "FULL")?;
        Ok(store)
    }
    pub fn load(&self) -> Result<Draft, Error> {
        Ok(self.connection.query_row("SELECT text, revision FROM drafts WHERE id = 1", [], |r| {
            let revision: i64 = r.get(1)?;
            let revision = u64::try_from(revision).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(1, revision))?;
            Ok(Draft { text: r.get(0)?, revision })
        })?)
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
