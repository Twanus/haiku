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

fn acquire_lock(dir: &Path) -> Result<File, StoreError> {
    fs::create_dir_all(dir).map_err(|e| io_err(dir, e))?;
    let lock_path = dir.join("haikus.lock");
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
    list_in(&dir()?)
}

pub fn save(haiku: Haiku) -> Result<(), StoreError> {
    save_in(&dir()?, haiku)
}

pub fn random() -> Result<Haiku, StoreError> {
    random_in(&dir()?)
}

fn list_in(dir: &Path) -> Result<Vec<Haiku>, StoreError> {
    let path = dir.join("haikus.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&path).map_err(|e| io_err(&path, e))?;
    serde_json::from_str(&data).map_err(|source| StoreError::Json { path, source })
}

fn save_in(dir: &Path, haiku: Haiku) -> Result<(), StoreError> {
    let _lock = acquire_lock(dir)?;
    let mut all = list_in(dir)?;
    all.push(haiku);
    let path = dir.join("haikus.json");
    let body = serde_json::to_string_pretty(&all).map_err(|source| StoreError::Json {
        path: path.clone(),
        source,
    })?;
    atomic_write(&path, &body)
}

fn random_in(dir: &Path) -> Result<Haiku, StoreError> {
    let all = list_in(dir)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn haiku(n: u32) -> Haiku {
        match n {
            1 => Haiku::new(
                "an old silent pond",
                "a frog jumps into the pond",
                "splash silence again",
            ),
            _ => Haiku::new(
                "i walked to the store",
                "wanted a little table",
                "whole apple loved much",
            ),
        }
        .expect("fixture haiku should be valid 5-7-5")
    }

    #[test]
    fn list_on_empty_dir_returns_empty_vec() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(list_in(tmp.path()).unwrap(), Vec::new());
    }

    #[test]
    fn save_then_list_roundtrips() {
        let tmp = tempfile::tempdir().unwrap();
        save_in(tmp.path(), haiku(1)).unwrap();
        save_in(tmp.path(), haiku(2)).unwrap();

        let all = list_in(tmp.path()).unwrap();
        assert_eq!(all, vec![haiku(1), haiku(2)]);
    }

    #[test]
    fn save_creates_missing_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("nested").join("haiku");
        assert!(!nested.exists());

        save_in(&nested, haiku(1)).unwrap();
        assert_eq!(list_in(&nested).unwrap(), vec![haiku(1)]);
    }

    #[test]
    fn random_on_empty_dir_is_err_empty() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(matches!(random_in(tmp.path()), Err(StoreError::Empty)));
    }

    #[test]
    fn random_returns_a_saved_haiku() {
        let tmp = tempfile::tempdir().unwrap();
        save_in(tmp.path(), haiku(1)).unwrap();

        let picked = random_in(tmp.path()).unwrap();
        assert_eq!(picked, haiku(1));
    }

    #[test]
    fn list_on_corrupt_json_is_err_json() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("haikus.json"), "not json").unwrap();
        assert!(matches!(
            list_in(tmp.path()),
            Err(StoreError::Json { .. })
        ));
    }
}
