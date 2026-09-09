use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::haiku::Haiku;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("could not find a data directory")]
    NoDataDir,
    #[error("no haikus saved yet")]
    Empty,
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

fn io_err(path: impl Into<PathBuf>, source: io::Error) -> StoreError {
    StoreError::Io {
        path: path.into(),
        source,
    }
}

fn dir() -> Result<PathBuf, StoreError> {
    let mut dir = dirs::data_dir().ok_or(StoreError::NoDataDir)?;
    dir.push("haiku");
    Ok(dir)
}

fn path() -> Result<PathBuf, StoreError> {
    Ok(dir()?.join("haikus.json"))
}

fn lock_path() -> Result<PathBuf, StoreError> {
    Ok(dir()?.join("haikus.lock"))
}

fn acquire_lock() -> Result<File, StoreError> {
    let dir = dir()?;
    fs::create_dir_all(&dir).map_err(|e| io_err(&dir, e))?;
    let lock_path = lock_path()?;
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&lock_path)
        .map_err(|e| io_err(&lock_path, e))?;
    file.lock().map_err(|e| io_err(&lock_path, e))?;
    Ok(file)
}

fn atomic_write(path: &Path, contents: &str) -> Result<(), StoreError> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, contents).map_err(|e| io_err(&tmp, e))?;
    fs::rename(&tmp, path).map_err(|e| io_err(path, e))?;
    Ok(())
}

pub fn list() -> Result<Vec<Haiku>, StoreError> {
    let path = path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&path).map_err(|e| io_err(&path, e))?;
    serde_json::from_str(&data).map_err(|source| StoreError::Json { path, source })
}

pub fn save(haiku: Haiku) -> Result<(), StoreError> {
    let _lock = acquire_lock()?;
    let mut all = list()?;
    all.push(haiku);
    let path = path()?;
    let body = serde_json::to_string_pretty(&all).map_err(|source| StoreError::Json {
        path: path.clone(),
        source,
    })?;
    atomic_write(&path, &body)
}

pub fn random() -> Result<Haiku, StoreError> {
    let all = list()?;
    if all.is_empty() {
        return Err(StoreError::Empty);
    }
    let idx = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as usize)
        .unwrap_or(0))
        % all.len();
    Ok(all.into_iter().nth(idx).expect("idx < len"))
}
