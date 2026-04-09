use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};
use crate::buffer_path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnchorMeta {
    pub file: String,
    pub anchor: String,
    pub hash: String,
    pub line_range: (usize, usize),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BufferMeta {
    pub true_id: String,
    pub parent_true_id: Option<String>,
    pub region_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LabelMeta {
    pub true_id: String,
}

/// Helper to ensure directory exists
fn ensure_dir(path: &Path) -> Result<(), String> {
    match fs::create_dir_all(path) {
        Ok(()) => Ok(()),
        Err(e) => match e.kind() {
            std::io::ErrorKind::PermissionDenied => Err("IO_ERROR: permission denied".to_string()),
            _ => Err(format!("IO_ERROR: cannot create directory: {}", e)),
        },
    }
}

/// Helper to convert io error to spec format
fn io_error_to_spec(e: std::io::Error, context: &str) -> String {
    match e.kind() {
        std::io::ErrorKind::NotFound => "IO_ERROR: file not found".to_string(),
        std::io::ErrorKind::PermissionDenied => "IO_ERROR: permission denied".to_string(),
        _ => format!("IO_ERROR: {}", context),
    }
}

/// Save anchor metadata to {TMPDIR}/anchorscope/anchors/{hash}.json.
/// Errors use SPEC §4.5 format.
pub fn save_anchor_metadata(meta: &AnchorMeta) -> Result<(), String> {
    let dir = buffer_path::anchors_dir();
    let path = dir.join(format!("{}.json", meta.hash));
    let json = serde_json::to_string_pretty(meta)
        .map_err(|e| format!("IO_ERROR: JSON serialization failed: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Load anchor metadata from {TMPDIR}/anchorscope/anchors/{hash}.json.
/// Errors use SPEC §4.5 format.
pub fn load_anchor_metadata(hash: &str) -> Result<AnchorMeta, String> {
    let dir = buffer_path::anchors_dir();
    let path = dir.join(format!("{}.json", hash));
    let content = fs::read_to_string(&path)
        .map_err(|e| io_error_to_spec(e, "read failure"))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("IO_ERROR: anchor metadata corrupted: {}", e))
}

/// Save label mapping to {TMPDIR}/anchorscope/labels/{name}.json.
/// The label file contains: { "true_id": "<hash>" }.
/// Errors use SPEC §4.5 format.
pub fn save_label_mapping(name: &str, true_id: &str) -> Result<(), String> {
    let dir = buffer_path::labels_dir();
    let path = dir.join(format!("{}.json", name));

    // Check for collision
    if path.exists() {
        let existing = fs::read_to_string(&path)
            .map_err(|e| io_error_to_spec(e, "read failure"))?;
        match serde_json::from_str::<LabelMeta>(&existing) {
            Ok(existing_meta) => {
                if existing_meta.true_id != true_id {
                    return Err(format!("LABEL_EXISTS: label '{}' already points to a different true_id", name));
                }
            }
            Err(_) => {
                return Err("IO_ERROR: existing label file corrupted".to_string());
            }
        }
    }

    let meta = LabelMeta { true_id: true_id.to_string() };
    let json = serde_json::to_string_pretty(&meta)
        .map_err(|e| format!("IO_ERROR: JSON serialization failed: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Load label target from {TMPDIR}/anchorscope/labels/{name}.json.
/// Returns the true_id.
/// Errors use SPEC §4.5 format.
pub fn load_label_target(name: &str) -> Result<String, String> {
    let dir = buffer_path::labels_dir();
    let path = dir.join(format!("{}.json", name));
    let content = fs::read_to_string(&path)
        .map_err(|e| io_error_to_spec(e, "read failure"))?;
    serde_json::from_str::<LabelMeta>(&content)
        .map_err(|e| format!("IO_ERROR: label mapping corrupted: {}", e))
        .map(|meta| meta.true_id)
}

/// Save buffer content to {TMPDIR}/anchorscope/{file_hash}/{true_id}/content.
pub fn save_buffer_content(file_hash: &str, true_id: &str, content: &[u8]) -> Result<(), String> {
    let dir = buffer_path::true_id_dir(file_hash, true_id);
    ensure_dir(&dir).map_err(|e| e)?;
    let path = dir.join("content");
    fs::write(&path, content)
        .map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Save buffer metadata to {TMPDIR}/anchorscope/{file_hash}/{true_id}/metadata.json.
pub fn save_buffer_metadata(file_hash: &str, true_id: &str, meta: &BufferMeta) -> Result<(), String> {
    let dir = buffer_path::true_id_dir(file_hash, true_id);
    ensure_dir(&dir).map_err(|e| e)?;
    let path = dir.join("metadata.json");
    let json = serde_json::to_string_pretty(meta)
        .map_err(|e| format!("IO_ERROR: JSON serialization failed: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Load buffer content from {TMPDIR}/anchorscope/{file_hash}/{true_id}/content.
pub fn load_buffer_content(file_hash: &str, true_id: &str) -> Result<Vec<u8>, String> {
    let path = buffer_path::true_id_dir(file_hash, true_id).join("content");
    fs::read(&path)
        .map_err(|e| io_error_to_spec(e, "read failure"))
}

/// Load buffer metadata from {TMPDIR}/anchorscope/{file_hash}/{true_id}/metadata.json.
pub fn load_buffer_metadata(file_hash: &str, true_id: &str) -> Result<BufferMeta, String> {
    let path = buffer_path::true_id_dir(file_hash, true_id).join("metadata.json");
    let content = fs::read_to_string(&path)
        .map_err(|e| io_error_to_spec(e, "read failure"))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("IO_ERROR: buffer metadata corrupted: {}", e))
}

/// Load source path from {TMPDIR}/anchorscope/{file_hash}/source_path.
pub fn load_source_path(file_hash: &str) -> Result<String, String> {
    let path = buffer_path::file_dir(file_hash).join("source_path");
    fs::read_to_string(&path)
        .map_err(|e| io_error_to_spec(e, "read failure"))
}

/// Save anchor metadata with True ID to {TMPDIR}/anchorscope/{file_hash}/{true_id}/content.
pub fn save_anchor_metadata_with_true_id(meta: &AnchorMeta, true_id: &str, parent_true_id: Option<&str>) -> Result<(), String> {
    let file = Path::new(&meta.file);
    let raw = fs::read(file).map_err(|e| io_error_to_spec(e, "read failure"))?;
    let normalized = crate::matcher::normalize_line_endings(&raw);
    let file_hash = crate::hash::compute(&normalized);
    
    // Save source path
    let source_path = file
        .canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| meta.file.clone());
    let source_path_dir = buffer_path::file_dir(&file_hash);
    ensure_dir(&source_path_dir).map_err(|e| e)?;
    let source_path_path = source_path_dir.join("source_path");
    fs::write(&source_path_path, &source_path)
        .map_err(|e| io_error_to_spec(e, "write failure"))?;
    
    // Save content
    save_buffer_content(&file_hash, true_id, &normalized)?;
    
    // Save metadata
    let buffer_meta = BufferMeta {
        true_id: true_id.to_string(),
        parent_true_id: parent_true_id.map(|s| s.to_string()),
        region_hash: meta.hash.clone(),
    };
    save_buffer_metadata(&file_hash, true_id, &buffer_meta)?;
    
    Ok(())
}

/// Delete anchor metadata from anchors directory.
pub fn invalidate_anchor(hash: &str) {
    let path = buffer_path::anchors_dir().join(format!("{}.json", hash));
    let _ = fs::remove_file(path);
}

/// Delete buffer directory and all descendants for a True ID.
pub fn invalidate_true_id(file_hash: &str, true_id: &str) {
    let path = buffer_path::true_id_dir(file_hash, true_id);
    let _ = fs::remove_dir_all(path);
}

/// Delete buffer directory and all descendants for a nested True ID.
pub fn invalidate_nested_true_id(file_hash: &str, parent_true_id: &str, true_id: &str) {
    let path = buffer_path::nested_true_id_dir(file_hash, parent_true_id, true_id);
    let _ = fs::remove_dir_all(path);
}

/// Delete ephemeral label mapping after successful write (SPEC §4.4).
pub fn invalidate_label(name: &str) {
    let path = buffer_path::labels_dir().join(format!("{}.json", name));
    let _ = fs::remove_file(path);
}
