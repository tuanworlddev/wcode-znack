//! Multi-store metadata. Each store has its own token (keychain) and its own
//! SQLite database file, so switching stores swaps the active DB connection.

use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Default)]
pub struct StoresMeta {
    pub active: Option<String>,
    #[serde(default)]
    pub stores: Vec<StoreMeta>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StoreMeta {
    pub id: String,
    pub name: String,
}

fn meta_path(dir: &Path) -> PathBuf {
    dir.join("stores.json")
}

pub fn load(dir: &Path) -> StoresMeta {
    std::fs::read_to_string(meta_path(dir))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(dir: &Path, meta: &StoresMeta) -> AppResult<()> {
    std::fs::write(meta_path(dir), serde_json::to_string_pretty(meta)?)?;
    Ok(())
}

/// Time-based unique id (Rust std time is fine here).
pub fn new_id() -> String {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("s{ms}")
}

pub fn db_path(dir: &Path, id: &str) -> PathBuf {
    dir.join("stores").join(format!("{id}.db"))
}
