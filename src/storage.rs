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

/// Base path: std::env::temp_dir()/anchorscope
fn anchorscope_temp_dir() -> PathBuf {
    std::env::temp_dir().join("anchorscope")
}

fn ensure_dir(path: &PathBuf) -> Result<(), String> {
    match fs::create_dir_all(path) {
        Ok(()) => Ok(()),
        Err(e) => match e.kind() {
            std::io::ErrorKind::PermissionDenied => Err("IO_ERROR: permission denied".to_string()),
            _ => Err(format!("IO_ERROR: cannot create directory: {}", e)),
        },
    }
}

fn ensure_anchor_dir() -> Result<PathBuf, String> {
    let dir = anchorscope_temp_dir().join("anchors");
    ensure_dir(&dir).map(|_| dir)
}

fn ensure_label_dir() -> Result<PathBuf, String> {
    let dir = anchorscope_temp_dir().join("labels");
    ensure_dir(&dir).map(|_| dir)
}

fn io_error_to_spec(e: std::io::Error, context: &str) -> String {
    match e.kind() {
        std::io::ErrorKind::NotFound => "IO_ERROR: file not found".to_string(),
        std::io::ErrorKind::PermissionDenied => "IO_ERROR: permission denied".to_string(),
        _ => format!("IO_ERROR: {}", context),
    }
}

/// Save anchor metadata to {TEMP}/anchorscope/anchors/{hash}.json.
/// Errors use SPEC §4.5 format.
pub fn save_anchor_metadata(meta: &AnchorMeta) -> Result<(), String> {
    let dir = ensure_anchor_dir()?;
    let path = dir.join(format!("{}.json", meta.hash));
    let json = serde_json::to_string_pretty(meta)
        .map_err(|e| format!("IO_ERROR: JSON serialization failed: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Load anchor metadata from {TEMP}/anchorscope/anchors/{hash}.json.
/// Errors use SPEC §4.5 format.
pub fn load_anchor_metadata(hash: &str) -> Result<AnchorMeta, String> {
    let dir = ensure_anchor_dir()?;
    let path = dir.join(format!("{}.json", hash));
    let content = fs::read_to_string(&path)
        .map_err(|e| io_error_to_spec(e, "read failure"))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("IO_ERROR: anchor metadata corrupted: {}", e))
}

/// Save label mapping to {TEMP}/anchorscope/labels/{name}.json.
/// The label file contains: { "internal_label": "<hash>" }.
/// Errors use SPEC §4.5 format.
pub fn save_label_mapping(name: &str, internal_label: &str) -> Result<(), String> {
    let dir = ensure_label_dir()?;
    let path = dir.join(format!("{}.json", name));

    // Check for collision
    if path.exists() {
        let existing = fs::read_to_string(&path)
            .map_err(|e| io_error_to_spec(e, "read failure"))?;
        match serde_json::from_str::<LabelMeta>(&existing) {
            Ok(existing_meta) => {
                if existing_meta.internal_label != internal_label {
                    return Err(format!("LABEL_EXISTS: label '{}' already points to a different internal label", name));
                }
            }
            Err(_) => {
                return Err("IO_ERROR: existing label file corrupted".to_string());
            }
        }
    }

    let meta = LabelMeta { internal_label: internal_label.to_string() };
    let json = serde_json::to_string_pretty(&meta)
        .map_err(|e| format!("IO_ERROR: JSON serialization failed: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Load label target from {TEMP}/anchorscope/labels/{name}.json.
/// Returns the internal_label hash.
/// Errors use SPEC §4.5 format.
pub fn load_label_target(name: &str) -> Result<String, String> {
    let dir = ensure_label_dir()?;
    let path = dir.join(format!("{}.json", name));
    let content = fs::read_to_string(&path)
        .map_err(|e| io_error_to_spec(e, "read failure"))?;
    serde_json::from_str::<LabelMeta>(&content)
        .map_err(|e| format!("IO_ERROR: label mapping corrupted: {}", e))
        .map(|meta| meta.internal_label)
}

/// Delete ephemeral anchor metadata after successful write (SPEC §3.3).
pub fn invalidate_anchor(hash: &str) {
    if let Ok(dir) = ensure_anchor_dir() {
        let path = dir.join(format!("{}.json", hash));
        let _ = fs::remove_file(path);
    }
}

/// Delete ephemeral label mapping after successful write (SPEC §3.3).
pub fn invalidate_label(name: &str) {
    if let Ok(dir) = ensure_label_dir() {
        let path = dir.join(format!("{}.json", name));
        let _ = fs::remove_file(path);
    }
}
