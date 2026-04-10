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
    pub anchor: String,  // The anchor text that was matched
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
    ensure_dir(&dir)?;
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
    ensure_dir(&dir).map_err(|e| e)?;
    let path = dir.join(format!("{}.json", name));

    // Allow overwriting labels
    // No collision check needed

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
    ensure_dir(&dir).map_err(|e| e)?;
    let path = dir.join(format!("{}.json", name));
    let content = fs::read_to_string(&path)
        .map_err(|e| io_error_to_spec(e, "read failure"))?;
    serde_json::from_str::<LabelMeta>(&content)
        .map_err(|e| format!("IO_ERROR: label mapping corrupted: {}", e))
        .map(|meta| meta.true_id)
}

/// Save normalized file content to {TMPDIR}/anchorscope/{file_hash}/content.
pub fn save_file_content(file_hash: &str, content: &[u8]) -> Result<(), String> {
    let dir = buffer_path::file_dir(file_hash);
    ensure_dir(&dir)?;
    let path = dir.join("content");
    fs::write(&path, content)
        .map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Save source path to {TMPDIR}/anchorscope/{file_hash}/source_path.
pub fn save_source_path(file_hash: &str, path: &str) -> Result<(), String> {
    let dir = buffer_path::file_dir(file_hash);
    ensure_dir(&dir)?;
    let path_file = dir.join("source_path");
    fs::write(&path_file, path)
        .map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Save buffer content to {TMPDIR}/anchorscope/{file_hash}/{true_id}/content.
pub fn save_buffer_content(file_hash: &str, true_id: &str, content: &[u8]) -> Result<(), String> {
    let dir = buffer_path::true_id_dir(file_hash, true_id);
    ensure_dir(&dir).map_err(|e| e)?;
    let path = dir.join("content");
    fs::write(&path, content)
        .map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Save matched region content to {TMPDIR}/anchorscope/{file_hash}/{true_id}/content.
/// This is an alias for save_buffer_content for clarity.
pub fn save_region_content(file_hash: &str, true_id: &str, content: &[u8]) -> Result<(), String> {
    save_buffer_content(file_hash, true_id, content)
}

/// Save nested buffer content to {TMPDIR}/anchorscope/{file_hash}/{parent_true_id}/{true_id}/content.
pub fn save_nested_buffer_content(file_hash: &str, parent_true_id: &str, true_id: &str, content: &[u8]) -> Result<(), String> {
    let dir = buffer_path::nested_true_id_dir(file_hash, parent_true_id, true_id);
    ensure_dir(&dir)?;
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

/// Find file_hash containing a given true_id by searching all buffer directories
pub fn find_file_hash_for_true_id(true_id: &str) -> Option<String> {
    let temp_dir = std::env::temp_dir();
    let anchorscope_dir = temp_dir.join("anchorscope");
    
    if let Ok(entries) = std::fs::read_dir(&anchorscope_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let file_hash = entry.file_name();
                let file_hash_str = file_hash.to_string_lossy();
                
                // Check if {file_hash}/{true_id}/content exists
                let content_path = buffer_path::true_id_dir(&file_hash_str, true_id).join("content");
                if content_path.exists() {
                    return Some(file_hash_str.to_string());
                }
                
                // Check nested: {file_hash}/{parent_true_id}/{true_id}/content
                // We need to search within the file_hash directory
                if let Ok(file_dir_entries) = std::fs::read_dir(buffer_path::file_dir(&file_hash_str)) {
                    for parent_entry in file_dir_entries.flatten() {
                        if parent_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                            let parent_true_id = parent_entry.file_name();
                            let nested_content_path = buffer_path::nested_true_id_dir(
                                &file_hash_str,
                                &parent_true_id.to_string_lossy(),
                                true_id
                            ).join("content");
                            
                            if nested_content_path.exists() {
                                return Some(file_hash_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    None
}

/// Return the file hash for a given True ID, or error if not found.
pub fn file_hash_for_true_id(true_id: &str) -> Result<String, String> {
    find_file_hash_for_true_id(true_id)
        .ok_or_else(|| format!("IO_ERROR: file hash for True ID '{}' not found", true_id))
}

/// Load buffer content from {TMPDIR}/anchorscope/{file_hash}/{true_id}/content.

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolves_file_hash_for_true_id_test() {
        let content = b"temporary test content";
        let file_hash = crate::hash::compute(content);
        let true_id = "test_true_id_123";
        save_buffer_content(&file_hash, true_id, content).expect("save buffer content");
        let buffer_meta = BufferMeta {
            true_id: true_id.to_string(),
            parent_true_id: None,
            region_hash: "region_hash_dummy".to_string(),
            anchor: "test_anchor".to_string(),
        };
        save_buffer_metadata(&file_hash, true_id, &buffer_meta).expect("save buffer metadata");
        let resolved = file_hash_for_true_id(true_id).expect("resolve file hash");
        assert_eq!(resolved, file_hash);
        invalidate_true_id(&file_hash, true_id);
    }
}

pub fn load_buffer_content(file_hash: &str, true_id: &str) -> Result<Vec<u8>, String> {
    let path = buffer_path::true_id_dir(file_hash, true_id).join("content");
    fs::read(&path)
        .map_err(|e| io_error_to_spec(e, "read failure"))
}

/// Load file content from {TMPDIR}/anchorscope/{file_hash}/content.
pub fn load_file_content(file_hash: &str) -> Result<Vec<u8>, String> {
    let path = buffer_path::file_dir(file_hash).join("content");
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

/// Load anchor metadata with True ID from {TMPDIR}/anchorscope/{file_hash}/{true_id}/metadata.json.
pub fn load_anchor_metadata_by_true_id(true_id: &str) -> Result<AnchorMeta, String> {
    // First check old location (v1.1.0 compatibility)
    let temp_dir = std::env::temp_dir();
    let anchors_dir = temp_dir.join("anchors");
    let path = anchors_dir.join(format!("{}.json", true_id));
    if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| io_error_to_spec(e, "read failure"))?;
        return serde_json::from_str(&content)
            .map_err(|e| format!("IO_ERROR: anchor metadata corrupted: {}", e));
    }
    
    // Check new buffer locations - only iterate through file_hash directories, not special subdirs
    let anchorscope_dir = temp_dir.join("anchorscope");
    if let Ok(entries) = std::fs::read_dir(&anchorscope_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let file_hash = entry.file_name();
                let file_hash_str = file_hash.to_string_lossy();
                // Skip special subdirectories like "anchors" and "labels"
                if file_hash_str == "anchors" || file_hash_str == "labels" {
                    continue;
                }
                
                // First check {file_hash}/{true_id}/ for a direct match
                let true_id_dir = buffer_path::true_id_dir(&file_hash_str, true_id);
                let content_path = true_id_dir.join("content");
                let metadata_path = true_id_dir.join("metadata.json");
                
                if content_path.exists() || metadata_path.exists() {
                    // Check if this is the buffer we're looking for
                    // The true_id could be the buffer's true_id or its region_hash
                    let is_match = {
                        if metadata_path.exists() {
                            let content = fs::read_to_string(&metadata_path)
                                .map_err(|e| io_error_to_spec(e, "read failure"))?;
                            let buffer_meta: BufferMeta = serde_json::from_str(&content)
                                .map_err(|e| format!("IO_ERROR: buffer metadata corrupted: {}", e))?;
                            let match_result = buffer_meta.true_id == true_id || buffer_meta.region_hash == true_id;
                            match_result
                        } else {
                            false
                        }
                    };
                    
                    if is_match {
                        // Load metadata if exists
                        let content = fs::read_to_string(&metadata_path)
                            .map_err(|e| io_error_to_spec(e, "read failure"))?;
                        let buffer_meta: BufferMeta = serde_json::from_str(&content)
                            .map_err(|e| format!("IO_ERROR: buffer metadata corrupted: {}", e))?;
                        
                        // Load source path
                        let source_path = buffer_path::file_dir(&file_hash_str).join("source_path");
                        let file = fs::read_to_string(&source_path)
                            .map_err(|e| io_error_to_spec(e, "read failure"))?;
                        
                        let region_hash = buffer_meta.region_hash.clone();
                        let anchor = buffer_meta.anchor.clone();
                        return Ok(AnchorMeta {
                            file,
                            anchor,
                            hash: region_hash,
                            line_range: (0, 0),  // Line range not stored in buffer metadata
                        });
                    }
                }
                
                // Also check nested directories under {file_hash}/
                if let Ok(dir_entries) = std::fs::read_dir(buffer_path::file_dir(&file_hash_str)) {
                    for dir_entry in dir_entries.flatten() {
                        if dir_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                            let subdir_true_id = dir_entry.file_name();
                            let subdir_true_id_str = subdir_true_id.to_string_lossy();
                            
                            let subdir_content_path = buffer_path::true_id_dir(&file_hash_str, &subdir_true_id_str).join("content");
                            let subdir_metadata_path = buffer_path::true_id_dir(&file_hash_str, &subdir_true_id_str).join("metadata.json");
                            
                            if subdir_metadata_path.exists() {
                                let content = fs::read_to_string(&subdir_metadata_path)
                                    .map_err(|e| io_error_to_spec(e, "read failure"))?;
                                let buffer_meta: BufferMeta = serde_json::from_str(&content)
                                    .map_err(|e| format!("IO_ERROR: buffer metadata corrupted: {}", e))?;
                                
                                // Check if this buffer's true_id or region_hash matches
                                let match_result = buffer_meta.true_id == true_id || buffer_meta.region_hash == true_id;
                                if match_result {
                                    
                                    // Load source path
                                    let source_path = buffer_path::file_dir(&file_hash_str).join("source_path");
                                    let file = fs::read_to_string(&source_path)
                                        .map_err(|e| io_error_to_spec(e, "read failure"))?;
                                    
                                    let region_hash = buffer_meta.region_hash.clone();
                                    let anchor = buffer_meta.anchor.clone();
                                    return Ok(AnchorMeta {
                                        file,
                                        anchor,
                                        hash: region_hash,
                                        line_range: (0, 0),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Err(format!("IO_ERROR: anchor metadata for true_id '{}' not found", true_id))
}

/// Debug: print all buffer contents
pub fn print_all_buffers() {
    let temp_dir = std::env::temp_dir().join("anchorscope");
    
    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let file_hash = entry.file_name();
                let file_hash_str = file_hash.to_string_lossy();
                
                // Check {file_hash}/{true_id}/
                if let Ok(dir_entries) = std::fs::read_dir(buffer_path::file_dir(&file_hash_str)) {
                    for dir_entry in dir_entries.flatten() {
                        if dir_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                            let true_id = dir_entry.file_name();
                            let true_id_str = true_id.to_string_lossy();
                            
                            let content_path = buffer_path::true_id_dir(&file_hash_str, &true_id_str).join("content");
                            let metadata_path = buffer_path::true_id_dir(&file_hash_str, &true_id_str).join("metadata.json");
                            if content_path.exists() || metadata_path.exists() {
                                if metadata_path.exists() {
                                    if let Ok(content) = fs::read_to_string(&metadata_path) {
                                        if let Ok(buffer_meta) = serde_json::from_str::<BufferMeta>(&content) {
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
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
        anchor: meta.anchor.clone(),
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

/// Delete buffer directory and all descendants for a True ID hierarchy.
/// This recursively removes the directory {file_hash}/{true_id} and all nested children.
/// SPEC §4.3 requires that write operations delete the anchor's directory "and all its descendants".
pub fn invalidate_true_id_hierarchy(file_hash: &str, true_id: &str) -> Result<(), String> {
    let base_path = buffer_path::true_id_dir(file_hash, true_id);
    
    // Delete the immediate directory
    if base_path.exists() {
        std::fs::remove_dir_all(&base_path)
            .map_err(|e| format!("IO_ERROR: cannot delete buffer {}: {}", base_path.display(), e))?;
    }
    
    // Search for nested children and delete them too
    let file_dir = buffer_path::file_dir(file_hash);
    if let Ok(entries) = std::fs::read_dir(&file_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let parent_id = entry.file_name();
                let parent_id_str = parent_id.to_string_lossy();
                let nested_path = buffer_path::nested_true_id_dir(file_hash, &parent_id_str, true_id);
                
                if nested_path.exists() {
                    std::fs::remove_dir_all(&nested_path)
                        .map_err(|e| format!("IO_ERROR: cannot delete nested buffer {}: {}", nested_path.display(), e))?;
                }
            }
        }
    }
    
    Ok(())
}

/// Delete ephemeral label mapping after successful write (SPEC §4.4).
pub fn invalidate_label(name: &str) {
    let path = buffer_path::labels_dir().join(format!("{}.json", name));
    let _ = fs::remove_file(path);
}
