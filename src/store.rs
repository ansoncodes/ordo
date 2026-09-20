//! Persistence: the whole database lives in memory and is written atomically
//! to a single JSON file after every mutation.

use crate::model::Database;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{RwLock, RwLockReadGuard};

pub struct Store {
    path: PathBuf,
    db: RwLock<Database>,
}

impl Store {
    /// Loads the database at `path`, or creates it from `seed` when it does not exist yet.
    pub fn open(path: PathBuf, seed: impl FnOnce() -> Database) -> io::Result<Store> {
        let db = if path.exists() {
            let text = fs::read_to_string(&path)?;
            serde_json::from_str(&text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        } else {
            let db = seed();
            persist(&path, &db)?;
            db
        };
        Ok(Store { path, db: RwLock::new(db) })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn read(&self) -> RwLockReadGuard<'_, Database> {
        self.db.read().unwrap_or_else(|e| e.into_inner())
    }

    /// Applies `f` under the write lock and persists the result when it succeeds.
    pub fn write<T, E>(&self, f: impl FnOnce(&mut Database) -> Result<T, E>) -> Result<T, E>
    where
        E: From<io::Error>,
    {
        let mut guard = self.db.write().unwrap_or_else(|e| e.into_inner());
        let out = f(&mut guard)?;
        persist(&self.path, &guard)?;
        Ok(out)
    }
}

fn persist(path: &Path, db: &Database) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let json = serde_json::to_string_pretty(db).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
