use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};
use crate::buffer_path;

/// Error type for ambiguous anchor detection.
/// Raised when the same true_id exists in multiple locations, violating determinism.
pub struct AmbiguousAnchorError {
    pub true_id: String,
    pub locations: Vec<PathBuf>,
}

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

/// Save nested buffer metadata to {TMPDIR}/anchorscope/{file_hash}/{parent_true_id}/{true_id}/metadata.json.
pub fn save_nested_buffer_metadata(file_hash: &str, parent_true_id: &str, true_id: &str, meta: &BufferMeta) -> Result<(), String> {
    let dir = buffer_path::nested_true_id_dir(file_hash, parent_true_id, true_id);
    ensure_dir(&dir)?;
    let path = dir.join("metadata.json");
    let json = serde_json::to_string_pretty(meta)
        .map_err(|e| format!("IO_ERROR: JSON serialization failed: {}", e))?;
    fs::write(&path, json)
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

/// Find the directory path for a given true_id (could be flat or nested).
/// Returns the path to the directory containing the true_id's content.
/// Returns Err(AmbiguousAnchorError) if the same true_id exists in multiple locations.
pub fn find_true_id_dir(file_hash: &str, true_id: &str) -> Result<Option<PathBuf>, AmbiguousAnchorError> {
    use std::collections::VecDeque;
    
    let mut found_paths: Vec<PathBuf> = Vec::new();
    let file_dir = buffer_path::file_dir(file_hash);
    
    // BFS search to find all locations of this true_id
    let mut queue = VecDeque::new();
    queue.push_back(file_dir.clone());
    
    while let Some(current_dir) = queue.pop_front() {
        // Check if {current_dir}/{true_id}/ exists
        let target_dir = current_dir.join(true_id);
        
        if target_dir.join("content").exists() || target_dir.join("metadata.json").exists() {
            found_paths.push(target_dir.clone());
            
            // If we found more than one, it's ambiguous
            if found_paths.len() > 1 {
                return Err(AmbiguousAnchorError {
                    true_id: true_id.to_string(),
                    locations: found_paths,
                });
            }
        }
        
        // Add all subdirectories to the queue
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    queue.push_back(entry.path());
                }
            }
        }
    }
    
    Ok(if found_paths.is_empty() { None } else { Some(found_paths[0].clone()) })
}

/// Find file_hash containing a given true_id by searching all buffer directories
pub fn find_file_hash_for_true_id(true_id: &str) -> Option<String> {
    let temp_dir = std::env::temp_dir();
    let anchorscope_dir = temp_dir.join("anchorscope");
    
    // Search all file_hash directories
    if let Ok(entries) = std::fs::read_dir(&anchorscope_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let file_hash = entry.file_name();
                let file_hash_str = file_hash.to_string_lossy();
                
                // Use BFS to search all nested locations for this true_id
                let file_dir = buffer_path::file_dir(&file_hash_str);
                let (found, _count) = file_hash_exists_in_dir_with_count(&file_dir, true_id);
                if found {
                    return Some(file_hash_str.to_string());
                }
            }
        }
    }
    
    None
}

/// Check if true_id exists in the directory tree.
/// Returns (found, count) where count is the number of matching directories.
fn file_hash_exists_in_dir_with_count(dir: &Path, true_id: &str) -> (bool, usize) {
    use std::collections::VecDeque;
    
    let mut count = 0;
    let mut queue = VecDeque::new();
    queue.push_back(dir.to_path_buf());
    
    while let Some(current_dir) = queue.pop_front() {
        // Check if {current_dir}/{true_id}/content exists
        let content_path = current_dir.join(true_id).join("content");
        if content_path.exists() {
            count += 1;
            if count > 1 {
                return (true, count); // Ambiguous
            }
        }
        
        // Add all subdirectories to the queue
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    queue.push_back(entry.path());
                }
            }
        }
    }
    
    (count > 0, count)
}

/// Find file_hash containing a given true_id by searching all buffer directories.
/// Returns error if true_id exists in multiple file_hash directories (ambiguous).
fn find_file_hash_for_true_id_with_dup_check(true_id: &str) -> Result<Option<String>, AmbiguousAnchorError> {
    let temp_dir = std::env::temp_dir();
    let anchorscope_dir = temp_dir.join("anchorscope");
    
    let mut found_hashes: Vec<String> = Vec::new();
    
    // Search all file_hash directories
    if let Ok(entries) = std::fs::read_dir(&anchorscope_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let file_hash = entry.file_name();
                let file_hash_str = file_hash.to_string_lossy();
                
                let file_dir = buffer_path::file_dir(&file_hash_str);
                let (found, _count) = file_hash_exists_in_dir_with_count(&file_dir, true_id);
                
                if found {
                    found_hashes.push(file_hash_str.to_string());
                    
                    // If we found the same true_id in multiple file_hash directories, it's ambiguous
                    if found_hashes.len() > 1 {
                        return Err(AmbiguousAnchorError {
                            true_id: true_id.to_string(),
                            locations: found_hashes.iter().map(|h| buffer_path::file_dir(h)).collect(),
                        });
                    }
                }
            }
        }
    }
    
    Ok(if found_hashes.is_empty() { None } else { Some(found_hashes[0].clone()) })
}

/**
 * Return the file hash for a given True ID, or error if not found.
 * Returns AmbiguousAnchorError if the same true_id exists in multiple file_hash directories.
 */
pub fn file_hash_for_true_id(true_id: &str) -> Result<String, String> {
    match find_file_hash_for_true_id_with_dup_check(true_id) {
        Ok(Some(hash)) => Ok(hash),
        Ok(None) => Err(format!("IO_ERROR: file hash for True ID '{}' not found", true_id)),
        Err(AmbiguousAnchorError { true_id: tid, locations }) => {
            let locations_str: Vec<String> = locations.iter().map(|p| p.display().to_string()).collect();
            Err(format!("ERROR: Ambiguous anchor detection - same true_id '{}' found in multiple file_hash directories: {}", tid, locations_str.join(", ")))
        }
    }
}

/// Load buffer content from {TMPDIR}/anchorscope/{file_hash}/{true_id}/content or nested location.
pub fn load_buffer_content(file_hash: &str, true_id: &str) -> Result<Vec<u8>, String> {
    // First try flat location
    let flat_path = buffer_path::true_id_dir(file_hash, true_id).join("content");
    if flat_path.exists() {
        return fs::read(&flat_path)
            .map_err(|e| io_error_to_spec(e, "read failure"));
    }
    
    // Try nested location
    let nested_path = buffer_path::nested_true_id_dir(file_hash, "", true_id).join("content");
    if nested_path.exists() {
        return fs::read(&nested_path)
            .map_err(|e| io_error_to_spec(e, "read failure"));
    }
    
    Err(io_error_to_spec(std::io::Error::new(std::io::ErrorKind::NotFound, "content"), "file not found"))
}

/// Load buffer metadata from {TMPDIR}/anchorscope/{file_hash}/{true_id}/metadata.json or nested location.
pub fn load_buffer_metadata(file_hash: &str, true_id: &str) -> Result<BufferMeta, String> {
    // Use find_true_id_dir which also detects duplicates
    match find_true_id_dir(file_hash, true_id) {
        Ok(Some(dir_path)) => {
            let nested_metadata_path = dir_path.join("metadata.json");
            let content = fs::read_to_string(&nested_metadata_path)
                .map_err(|e| io_error_to_spec(e, "read failure"))?;
            let meta: BufferMeta = serde_json::from_str(&content)
                .map_err(|e| format!("IO_ERROR: buffer metadata corrupted: {}", e))?;
            
            // Verify the true_id matches
            if meta.true_id == true_id || meta.region_hash == true_id {
                return Ok(meta);
            }
            
            Err(io_error_to_spec(std::io::Error::new(std::io::ErrorKind::NotFound, "metadata.json"), "file not found"))
        }
        Ok(None) => {
            Err(io_error_to_spec(std::io::Error::new(std::io::ErrorKind::NotFound, "metadata.json"), "file not found"))
        }
        Err(AmbiguousAnchorError { true_id: tid, locations }) => {
            let locations_str: Vec<String> = locations.iter().map(|p| p.display().to_string()).collect();
            Err(format!("ERROR: Ambiguous anchor detection - same true_id '{}' found in multiple locations: {}", tid, locations_str.join(", ")))
        }
    }
}

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

/// Load file content from {TMPDIR}/anchorscope/{file_hash}/content.
pub fn load_file_content(file_hash: &str) -> Result<Vec<u8>, String> {
    let path = buffer_path::file_dir(file_hash).join("content");
    fs::read(&path)
        .map_err(|e| io_error_to_spec(e, "read failure"))
}

/// Load source path from {TMPDIR}/anchorscope/{file_hash}/source_path.
pub fn load_source_path(file_hash: &str) -> Result<String, String> {
    let path = buffer_path::file_dir(file_hash).join("source_path");
    fs::read_to_string(&path)
        .map_err(|e| io_error_to_spec(e, "read failure"))
}

/// Load anchor metadata with True ID from {TMPDIR}/anchorscope/{file_hash}/{true_id}/metadata.json.
pub fn load_anchor_metadata_by_true_id(true_id: &str) -> Result<AnchorMeta, String> {
    // First check old location (v1.1.0 compatibility) - at {anchorscope_dir}/anchors/
    let temp_dir = std::env::temp_dir();
    let anchors_dir = temp_dir.join("anchorscope").join("anchors");
    let path = anchors_dir.join(format!("{}.json", true_id));
    if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| io_error_to_spec(e, "read failure"))?;
        return serde_json::from_str(&content)
            .map_err(|e| format!("IO_ERROR: anchor metadata corrupted: {}", e));
    }
    
    // Search all buffer locations recursively
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
                
                // Search for true_id recursively in this file_hash
                if let Some(meta) = search_true_id_in_dir(&file_hash_str, true_id) {
                    return meta;
                }
            }
        }
    }
    
    Err(format!("IO_ERROR: anchor metadata for true_id '{}' not found", true_id))
}

/// Search for a true_id in the buffer directory tree using BFS.
fn search_true_id_in_dir(file_hash: &str, target_true_id: &str) -> Option<Result<AnchorMeta, String>> {
    use std::collections::VecDeque;
    
    let mut queue = VecDeque::new();
    queue.push_back(buffer_path::file_dir(file_hash));
    
    while let Some(current_dir) = queue.pop_front() {
        // Check if {current_dir}/{target_true_id}/metadata.json exists
        let metadata_path = current_dir.join(target_true_id).join("metadata.json");
        if metadata_path.exists() {
            match load_buffer_metadata(file_hash, target_true_id) {
                Ok(meta) => {
                    if meta.true_id == target_true_id || meta.region_hash == target_true_id {
                        return Some(load_anchor_meta_from_buffer(file_hash, &meta));
                    }
                }
                Err(_) => {}
            }
        }
        
        // Add all subdirectories to the queue
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    queue.push_back(entry.path());
                }
            }
        }
    }
    
    None
}

/// Load AnchorMeta from BufferMeta.
fn load_anchor_meta_from_buffer(file_hash: &str, buffer_meta: &BufferMeta) -> Result<AnchorMeta, String> {
    let source_path = buffer_path::file_dir(file_hash).join("source_path");
    let file = fs::read_to_string(&source_path)
        .map_err(|e| io_error_to_spec(e, "read failure"))?;
    
    let region_hash = buffer_meta.region_hash.clone();
    let anchor = buffer_meta.anchor.clone();
    
    Ok(AnchorMeta {
        file,
        anchor,
        hash: region_hash,
        line_range: (0, 0),
    })
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
                                        if let Ok(_buffer_meta) = serde_json::from_str::<BufferMeta>(&content) {
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
    use std::collections::VecDeque;
    
    let file_dir = buffer_path::file_dir(file_hash);
    
    // Use BFS to find and delete all directories named {true_id}
    let mut queue = VecDeque::new();
    queue.push_back(file_dir);
    
    while let Some(current_dir) = queue.pop_front() {
        // Check if {current_dir}/{true_id}/ exists
        let target_dir = current_dir.join(true_id);
        
        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)
                .map_err(|e| format!("IO_ERROR: cannot delete buffer {}: {}", target_dir.display(), e))?;
        }
        
        // Add all subdirectories to the queue
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    queue.push_back(entry.path());
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
