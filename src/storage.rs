use std::path::PathBuf;
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnchorMeta {
    pub file: String,
    pub anchor: String,
    pub hash: String,
    pub line_range: (usize, usize),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LabelMeta {
    pub internal_label: String,
}

fn anchorscope_temp_dir() -> Result<PathBuf, String> {
    let base = std::env::temp_dir().join("anchorscope");
    fs::create_dir_all(&base).map_err(|e| format!("IO_ERROR: cannot create temp dir: {}", e))?;
    Ok(base)
}

fn ensure_anchor_dir() -> Result<PathBuf, String> {
    let base = anchorscope_temp_dir()?;
    let dir = base.join("anchors");
    fs::create_dir_all(&dir).map_err(|e| format!("IO_ERROR: cannot create anchor dir: {}", e))?;
    Ok(dir)
}

fn ensure_label_dir() -> Result<PathBuf, String> {
    let base = anchorscope_temp_dir()?;
    let dir = base.join("labels");
    fs::create_dir_all(&dir).map_err(|e| format!("IO_ERROR: cannot create label dir: {}", e))?;
    Ok(dir)
}

pub fn save_anchor_metadata(meta: &AnchorMeta) -> Result<(), String> {
    let dir = ensure_anchor_dir()?;
    let path = dir.join(format!("{}.json", meta.hash));
    let json = serde_json::to_string_pretty(meta).map_err(|e| format!("IO_ERROR: JSON serialization failed: {}", e))?;
    fs::write(path, json).map_err(|e| format!("IO_ERROR: cannot write anchor metadata: {}", e))?;
    Ok(())
}

pub fn load_anchor_metadata(hash: &str) -> Result<AnchorMeta, String> {
    let dir = ensure_anchor_dir()?;
    let path = dir.join(format!("{}.json", hash));
    let content = fs::read_to_string(&path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => format!("IO_ERROR: anchor metadata not found"),
        _ => format!("IO_ERROR: cannot read anchor metadata: {}", e),
    })?;
    serde_json::from_str(&content).map_err(|e| format!("IO_ERROR: anchor metadata corrupted: {}", e))
}

pub fn save_label_mapping(name: &str, internal_label: &str) -> Result<(), String> {
    let dir = ensure_label_dir()?;
    let path = dir.join(format!("{}.json", name));
    if path.exists() {
        let existing = fs::read_to_string(&path)
            .map_err(|e| format!("IO_ERROR: cannot read existing label: {}", e))?;
        let existing_meta: LabelMeta = serde_json::from_str(&existing)
            .map_err(|e| format!("IO_ERROR: existing label corrupted: {}", e))?;
        if existing_meta.internal_label != internal_label {
            return Err(format!("LABEL_EXISTS: label '{}' already points to a different internal label", name));
        }
    }
    let meta = LabelMeta { internal_label: internal_label.to_string() };
    let json = serde_json::to_string_pretty(&meta).map_err(|e| format!("IO_ERROR: JSON serialization failed: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("IO_ERROR: cannot write label mapping: {}", e))?;
    Ok(())
}

pub fn load_label_target(name: &str) -> Result<String, String> {
    let dir = ensure_label_dir()?;
    let path = dir.join(format!("{}.json", name));
    let content = fs::read_to_string(&path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => format!("IO_ERROR: label not found"),
        _ => format!("IO_ERROR: cannot read label mapping: {}", e),
    })?;
    let meta: LabelMeta = serde_json::from_str(&content).map_err(|e| format!("IO_ERROR: label mapping corrupted: {}", e))?;
    Ok(meta.internal_label)
}

/// Delete ephemeral anchor metadata file after successful write (SPEC §3.3).
pub fn invalidate_anchor(hash: &str) {
    if let Ok(dir) = ensure_anchor_dir() {
        let path = dir.join(format!("{}.json", hash));
        let _ = fs::remove_file(path);
    }
}

/// Delete ephemeral label mapping file after successful write (SPEC §3.3).
pub fn invalidate_label(name: &str) {
    if let Ok(dir) = ensure_label_dir() {
        let path = dir.join(format!("{}.json", name));
        let _ = fs::remove_file(path);
    }
}
