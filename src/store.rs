use std::fs;
use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::haiku::Haiku;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("could not find a data directory")]
    NoDataDir,
    #[error("no haikus saved yet")]
    Empty,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

fn dir() -> Result<PathBuf, StoreError> {
    let mut dir = dirs::data_dir().ok_or(StoreError::NoDataDir)?;
    dir.push("haiku");
    Ok(dir)
}

fn path() -> Result<PathBuf, StoreError> {
    Ok(dir()?.join("haikus.json"))
}

pub fn list() -> Result<Vec<Haiku>, StoreError> {
    let path = path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(path)?;
    if data.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(serde_json::from_str(&data)?)
}

pub fn save(haiku: &Haiku) -> Result<(), StoreError> {
    fs::create_dir_all(dir()?)?;
    let mut all = list()?;
    all.push(haiku.clone());
    fs::write(path()?, serde_json::to_string_pretty(&all)?)?;
    Ok(())
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
    Ok(all[idx].clone())
}
