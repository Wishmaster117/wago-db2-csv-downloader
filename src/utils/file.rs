use std::path::Path;
use anyhow::Result;

pub fn file_exists_with_size(path: &Path) -> bool {
    if let Ok(metadata) = path.metadata() {
        metadata.len() > 0 
    } else {
        false
    }
}

pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}