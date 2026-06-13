// This file contains the refactored storage.rs with AnchorScopeError
// It will replace src/storage.rs

use crate::buffer_path;
use crate::error::AnchorScopeError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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
    pub scope_hash: String,
    pub anchor: String, // The anchor text that was matched
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LabelMeta {
    pub true_id: String,
}

/// Helper to ensure directory exists
fn ensure_dir(path: &Path) -> Result<(), AnchorScopeError> {
    fs::create_dir_all(path).map_err(AnchorScopeError::from)
}

/// Helper to convert io error to AnchorScopeError
fn io_error_to_spec(e: std::io::Error, _context: &str) -> AnchorScopeError {
    match e.kind() {
        std::io::ErrorKind::NotFound => AnchorScopeError::FileNotFound,
        std::io::ErrorKind::PermissionDenied => AnchorScopeError::PermissionDenied,
        _ => AnchorScopeError::WriteFailure(e),
    }
}

/// Save anchor metadata to {TMPDIR}/anchorscope/anchors/{hash}.json.
/// Errors use SPEC §4.5 format.
pub fn save_anchor_metadata(meta: &AnchorMeta) -> Result<(), AnchorScopeError> {
    let dir = buffer_path::anchors_dir();
    ensure_dir(&dir)?;
    let path = dir.join(format!("{}.json", meta.hash));
    let json = serde_json::to_string_pretty(meta)
        .map_err(|_| AnchorScopeError::JsonSerializationFailed("metadata".to_string()))?;
    fs::write(&path, json).map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Save label mapping to {TMPDIR}/anchorscope/labels/{name}.json.
/// The label file contains: { "true_id": "<hash>" }.
/// Errors use SPEC §4.5 format.
pub fn save_label_mapping(name: &str, true_id: &str) -> Result<(), AnchorScopeError> {
    let dir = buffer_path::labels_dir();
    ensure_dir(&dir)?;
    let path = dir.join(format!("{}.json", name));

    // Check if label already exists per SPEC §6.4
    let label_exists = path.exists();

    if label_exists {
        // Load existing label and compare true_id
        let existing_content = fs::read_to_string(&path)
            .map_err(|e| io_error_to_spec(e, "read failure"))?;
        let existing_meta: LabelMeta = serde_json::from_str(&existing_content)
            .map_err(|_| AnchorScopeError::LabelMappingCorrupted("label".to_string()))?;

        // If true_id is different, return LABEL_EXISTS error
        if existing_meta.true_id != true_id {
            return Err(AnchorScopeError::LabelExists);
        }
        // Same true_id, allow idempotent overwrite
    }

    let meta = LabelMeta {
        true_id: true_id.to_string(),
    };
    let json = serde_json::to_string_pretty(&meta)
        .map_err(|_| AnchorScopeError::JsonSerializationFailed("label".to_string()))?;
    fs::write(&path, json).map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Load label target from {TMPDIR}/anchorscope/labels/{name}.json.
/// Returns the true_id.
/// Errors use SPEC §4.5 format.
pub fn load_label_target(name: &str) -> Result<String, AnchorScopeError> {
    let dir = buffer_path::labels_dir();
    ensure_dir(&dir)?;
    let path = dir.join(format!("{}.json", name));
    let content = fs::read_to_string(&path).map_err(|e| io_error_to_spec(e, "read failure"))?;
    serde_json::from_str::<LabelMeta>(&content)
        .map_err(|_| AnchorScopeError::LabelMappingCorrupted("label".to_string()))
        .map(|meta| meta.true_id)
}

/// Save normalized file content to {TMPDIR}/anchorscope/{file_hash}/content.
pub fn save_file_content(file_hash: &str, content: &[u8]) -> Result<(), AnchorScopeError> {
    let dir = buffer_path::file_dir(file_hash);
    ensure_dir(&dir)?;
    let path = dir.join("content");
    fs::write(&path, content).map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Save source path to {TMPDIR}/anchorscope/{file_hash}/source_path.
pub fn save_source_path(file_hash: &str, path: &str) -> Result<(), AnchorScopeError> {
    let dir = buffer_path::file_dir(file_hash);
    ensure_dir(&dir)?;
    let path_file = dir.join("source_path");
    fs::write(&path_file, path).map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Save buffer content to {TMPDIR}/anchorscope/{file_hash}/{true_id}/content.
pub fn save_buffer_content(
    file_hash: &str,
    true_id: &str,
    content: &[u8],
) -> Result<(), AnchorScopeError> {
    let dir = buffer_path::true_id_dir(file_hash, true_id);
    ensure_dir(&dir)?;
    let path = dir.join("content");
    fs::write(&path, content).map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Save matched scope content to {TMPDIR}/anchorscope/{file_hash}/{true_id}/content.
/// This is an alias for save_buffer_content for clarity.
pub fn save_scope_content(
    file_hash: &str,
    true_id: &str,
    content: &[u8],
) -> Result<(), AnchorScopeError> {
    save_buffer_content(file_hash, true_id, content)
}

/// Save buffer metadata to {TMPDIR}/anchorscope/{file_hash}/{true_id}/metadata.json.
pub fn save_buffer_metadata(
    file_hash: &str,
    true_id: &str,
    meta: &BufferMeta,
) -> Result<(), AnchorScopeError> {
    let dir = buffer_path::true_id_dir(file_hash, true_id);
    ensure_dir(&dir)?;
    let path = dir.join("metadata.json");
    let json = serde_json::to_string_pretty(meta)
        .map_err(|_| AnchorScopeError::JsonSerializationFailed("buffer metadata".to_string()))?;
    fs::write(&path, json).map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Find the directory path for a given true_id (could be flat or nested).
/// Returns the path to the directory containing the true_id's content.
/// Returns Err(AmbiguousAnchorError) if the same true_id exists in multiple locations.
pub fn find_true_id_dir(
    file_hash: &str,
    true_id: &str,
) -> Result<Option<PathBuf>, AmbiguousAnchorError> {
    use std::collections::VecDeque;

    let mut found_paths: Vec<PathBuf> = Vec::new();
    let file_dir = buffer_path::file_dir(file_hash);

    // BFS search to find ALL locations of this true_id
    let mut queue = VecDeque::new();
    queue.push_back(file_dir.clone());

    while let Some(current_dir) = queue.pop_front() {
        // Check if {current_dir}/{true_id}/ exists
        let target_dir = current_dir.join(true_id);

        if target_dir.join("content").exists() || target_dir.join("metadata.json").exists() {
            found_paths.push(target_dir.clone());

            // SPEC §3.2: If the same True ID is found at multiple locations within the 
            // same {file_hash} directory, the system MUST terminate immediately with DUPLICATE_TRUE_ID
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

    Ok(if found_paths.is_empty() {
        None
    } else {
        Some(found_paths[0].clone())
    })
}

/// Find buffer content for a true_id by searching all directory levels.
/// Returns content if found, Err(AnchorScopeError::CannotLoadBufferContent) if not found.
/// This function eliminates duplicate traversal logic that was in commands/read.rs.
///
/// Currently unused but kept for potential future use.
#[allow(dead_code)]
pub fn find_buffer_content(file_hash: &str, true_id: &str) -> Result<Vec<u8>, AnchorScopeError> {
    // Check flat location first
    let flat_path = buffer_path::true_id_dir(file_hash, true_id).join("content");
    if flat_path.exists() {
        return fs::read(&flat_path).map_err(|e| io_error_to_spec(e, "read failure"));
    }

    // Check nested locations using BFS
    let file_dir = buffer_path::file_dir(file_hash);
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(file_dir);

    while let Some(current_dir) = queue.pop_front() {
        // Check all subdirectories
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let child_dir = entry.path();
                    let content_path = child_dir.join(true_id).join("content");

                    if content_path.exists() {
                        return fs::read(&content_path)
                            .map_err(|e| io_error_to_spec(e, "read failure"));
                    }

                    // Add to queue for deeper search
                    queue.push_back(child_dir);
                }
            }
        }
    }

    Err(AnchorScopeError::CannotLoadBufferContent)
}

/// Find file_hash containing a given true_id by searching all buffer directories.
/// Returns Ok(Some(file_hash)) if found, Ok(None) if not found.
/// Returns Err(AmbiguousAnchorError) if true_id exists in multiple locations.
/// This function eliminates duplicate code that was in commands/read.rs and commands/label.rs.
///
/// Currently unused but kept for potential future use.
#[allow(dead_code)]
pub fn file_hash_for_true_id_opt(true_id: &str) -> Result<Option<String>, AmbiguousAnchorError> {
    find_file_hash_for_true_id_with_dup_check(true_id)
}

/// Check if true_id exists in the directory tree.
/// Returns (found, count) where count is the number of matching directories.
pub fn file_hash_exists_in_dir_with_count(dir: &Path, true_id: &str) -> (bool, usize) {
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
fn find_file_hash_for_true_id_with_dup_check(
    true_id: &str,
) -> Result<Option<String>, AmbiguousAnchorError> {
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
                            locations: found_hashes
                                .iter()
                                .map(|h| buffer_path::file_dir(h))
                                .collect(),
                        });
                    }
                }
            }
        }
    }

    Ok(if found_hashes.is_empty() {
        None
    } else {
        Some(found_hashes[0].clone())
    })
}

/**
 * Return the file hash for a given True ID, or error if not found.
 * Returns AmbiguousAnchorError if the same true_id exists in multiple file_hash directories.
 */
pub fn file_hash_for_true_id(true_id: &str) -> Result<String, AnchorScopeError> {
    match find_file_hash_for_true_id_with_dup_check(true_id) {
        Ok(Some(hash)) => Ok(hash),
        Ok(None) => Err(AnchorScopeError::BufferNotFound),
        Err(AmbiguousAnchorError {
            true_id: _tid,
            locations: _locations,
        }) => {
            // SPEC \u00a73.2: Return DUPLICATE_TRUE_ID for ambiguous anchors
            Err(AnchorScopeError::DuplicateTrueId)
        }
    }
}

/// Load buffer content from {TMPDIR}/anchorscope/{file_hash}/{true_id}/content or nested location.
pub fn load_buffer_content(file_hash: &str, true_id: &str) -> Result<Vec<u8>, AnchorScopeError> {
    // First try flat location
    let flat_path = buffer_path::true_id_dir(file_hash, true_id).join("content");
    if flat_path.exists() {
        return fs::read(&flat_path).map_err(|e| io_error_to_spec(e, "read failure"));
    }

    // Search all nested locations recursively using BFS
    let file_dir = buffer_path::file_dir(file_hash);
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(file_dir.clone());

    while let Some(current_dir) = queue.pop_front() {
        // Check all subdirectories for {true_id}/content
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let child_dir = entry.path();
                    let content_path = child_dir.join(true_id).join("content");

                    if content_path.exists() {
                        return fs::read(&content_path).map_err(|e| {
                            io_error_to_spec(e, "read failure")
                        });
                    }

                    // Continue searching in nested directories
                    queue.push_back(child_dir);
                }
            }
        }
    }

    Err(AnchorScopeError::CannotLoadBufferContent)
}

/// Load buffer metadata from {TMPDIR}/anchorscope/{file_hash}/{true_id}/metadata.json or nested location.
pub fn load_buffer_metadata(
    file_hash: &str,
    true_id: &str,
) -> Result<BufferMeta, AnchorScopeError> {
    // Use find_true_id_dir which also detects duplicates
    match find_true_id_dir(file_hash, true_id) {
        Ok(Some(dir_path)) => {
            let nested_metadata_path = dir_path.join("metadata.json");
            let content = fs::read_to_string(&nested_metadata_path)
                .map_err(|e| io_error_to_spec(e, "read failure"))?;
            let meta: BufferMeta = serde_json::from_str(&content).map_err(|_| {
                AnchorScopeError::ParentBufferMetadataCorrupted("metadata".to_string())
            })?;

            // Verify the true_id matches
            if meta.true_id == true_id || meta.scope_hash == true_id {
                return Ok(meta);
            }

            Err(AnchorScopeError::FileNotFound)
        }
        Ok(None) => Err(AnchorScopeError::FileNotFound),
        Err(AmbiguousAnchorError {
            true_id: _tid,
            locations,
        }) => {
            let _locations_str: Vec<String> =
                locations.iter().map(|p| p.display().to_string()).collect();
            Err(AnchorScopeError::DuplicateTrueId)
        }
    }
}

/// Load source path from {TMPDIR}/anchorscope/{file_hash}/source_path.
pub fn load_source_path(file_hash: &str) -> Result<String, AnchorScopeError> {
    let path = buffer_path::file_dir(file_hash).join("source_path");
    fs::read_to_string(&path).map_err(|e| io_error_to_spec(e, "read failure"))
}

/// Load anchor metadata with True ID from {TMPDIR}/anchorscope/{file_hash}/{true_id}/metadata.json.
pub fn load_anchor_metadata_by_true_id(true_id: &str) -> Result<AnchorMeta, AnchorScopeError> {
    let temp_dir = std::env::temp_dir();
    let anchorscope_dir = temp_dir.join("anchorscope");

    // First, count how many times this true_id appears across all file_hash directories
    let mut found_locations: Vec<PathBuf> = Vec::new();

    // Check old location (v1.1.0 compatibility)
    let anchors_dir = anchorscope_dir.join("anchors");
    let old_path = anchors_dir.join(format!("{}.json", true_id));
    if old_path.exists() {
        found_locations.push(anchors_dir.clone());
    }

    // Search all buffer locations recursively
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
                if find_true_id_in_dir_recursive(&buffer_path::file_dir(&file_hash_str), true_id) {
                    found_locations.push(buffer_path::file_dir(&file_hash_str));
                }
            }
        }
    }

    // Check for duplicates (ambiguous anchor)
    if found_locations.len() > 1 {
        let _locations_str: Vec<String> = found_locations
            .iter()
            .map(|p| p.display().to_string())
            .collect();
        return Err(AnchorScopeError::DuplicateTrueId);
    }

    // Find the location and return metadata
    if found_locations.is_empty() {
        return Err(AnchorScopeError::BufferNotFound);
    }

    let location = &found_locations[0];

    // Check if it's the old location (v1.1.0 compatibility)
    if location.ends_with("anchors") {
        let content = fs::read_to_string(location.join(format!("{}.json", true_id)))
            .map_err(|e| io_error_to_spec(e, "read failure"))?;
        return serde_json::from_str(&content).map_err(|_| {
            AnchorScopeError::ParentBufferMetadataCorrupted("anchor metadata".to_string())
        });
    }

    // Search in the buffer location
    let file_hash = location.file_name().unwrap().to_string_lossy().to_string();
    if let Some(meta) = search_true_id_in_dir(&file_hash, true_id) {
        return meta;
    }

    Err(AnchorScopeError::BufferNotFound)
}

/// Recursively search for a true_id in the directory tree using BFS.
/// Returns true if found (at least one match).
fn find_true_id_in_dir_recursive(dir: &Path, target_true_id: &str) -> bool {
    use std::collections::VecDeque;

    let mut queue = VecDeque::new();
    queue.push_back(dir.to_path_buf());

    while let Some(current_dir) = queue.pop_front() {
        // Check if {current_dir}/{target_true_id}/content exists
        let content_path = current_dir.join(target_true_id).join("content");
        if content_path.exists() {
            return true;
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

    false
}

/// Search for a true_id in the buffer directory tree using BFS.
fn search_true_id_in_dir(
    file_hash: &str,
    target_true_id: &str,
) -> Option<Result<AnchorMeta, AnchorScopeError>> {
    use std::collections::VecDeque;

    let mut queue = VecDeque::new();
    queue.push_back(buffer_path::file_dir(file_hash));

    while let Some(current_dir) = queue.pop_front() {
        // Check if {current_dir}/{target_true_id}/metadata.json exists
        let metadata_path = current_dir.join(target_true_id).join("metadata.json");
        if metadata_path.exists() {
            match load_buffer_metadata(file_hash, target_true_id) {
                Ok(meta) => {
                    if meta.true_id == target_true_id || meta.scope_hash == target_true_id {
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
fn load_anchor_meta_from_buffer(
    file_hash: &str,
    buffer_meta: &BufferMeta,
) -> Result<AnchorMeta, AnchorScopeError> {
    let source_path = buffer_path::file_dir(file_hash).join("source_path");
    let file = fs::read_to_string(&source_path).map_err(|e| io_error_to_spec(e, "read failure"))?;

    let scope_hash = buffer_meta.scope_hash.clone();
    let anchor = buffer_meta.anchor.clone();

    Ok(AnchorMeta {
        file,
        anchor,
        hash: scope_hash,
        line_range: (0, 0),
    })
}

/// Delete anchor metadata from anchors directory.
pub fn invalidate_anchor(hash: &str) {
    let path = buffer_path::anchors_dir().join(format!("{}.json", hash));
    let _ = fs::remove_file(path);
}

/// Delete buffer directory and all descendants for a True ID hierarchy.
/// This recursively removes the directory {file_hash}/{true_id} and all nested children.
/// SPEC §4.3 requires that write operations delete the anchor's directory "and all its descendants".
pub fn invalidate_true_id_hierarchy(
    file_hash: &str,
    true_id: &str,
) -> Result<(), AnchorScopeError> {
    use std::collections::VecDeque;

    let file_dir = buffer_path::file_dir(file_hash);

    // Use BFS to find and delete all directories named {true_id}
    let mut queue = VecDeque::new();
    queue.push_back(file_dir);

    while let Some(current_dir) = queue.pop_front() {
        // Check if {current_dir}/{true_id}/ exists
        let target_dir = current_dir.join(true_id);

        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir).map_err(|e| AnchorScopeError::WriteFailure(e))?;
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

/// Check if a true_id has duplicates within a specific file_hash directory.
/// Returns Err("DUPLICATE_TRUE_ID") if found, Ok(()) otherwise.
/// SPEC §3.2: Duplicate True IDs within the same file_hash directory must be detected.
pub fn check_duplicate_true_id_in_file_hash(
    file_hash: &str,
    true_id: &str,
) -> Result<(), AnchorScopeError> {
    let file_dir = buffer_path::file_dir(file_hash);
    let mut count = 0;

    // Check flat location: {file_hash}/{true_id}/
    let flat_dir = buffer_path::true_id_dir(file_hash, true_id);
    if flat_dir.join("content").exists() || flat_dir.join("metadata.json").exists() {
        count += 1;
        if count > 1 {
            return Err(AnchorScopeError::DuplicateTrueId);
        }
    }

    // Check nested locations: search all subdirectories for {true_id}/content or {true_id}/metadata.json
    if let Ok(entries) = std::fs::read_dir(&file_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let nested_dir = entry.path().join(true_id);
                if nested_dir.join("content").exists() || nested_dir.join("metadata.json").exists() {
                    count += 1;
                    if count > 1 {
                        return Err(AnchorScopeError::DuplicateTrueId);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Load replacement content from {file_hash}/{true_id}/replacement.
pub fn load_replacement_content(
    file_hash: &str,
    true_id: &str,
) -> Result<Vec<u8>, AnchorScopeError> {
    let replacement_path = buffer_path::true_id_dir(file_hash, true_id).join("replacement");
    fs::read(&replacement_path).map_err(|e| io_error_to_spec(e, "replacement not found"))
}

/// Check if a true_id exists in the buffer (flat or nested locations).
/// Returns true if found, false otherwise.
/// This function eliminates duplicate code that was in commands/read.rs.
///
/// Currently unused but kept for potential future use.
#[allow(dead_code)]
pub fn true_id_exists(file_hash: &str, true_id: &str) -> bool {
    // Check flat location
    let flat_path = buffer_path::true_id_dir(file_hash, true_id).join("content");
    if flat_path.exists() {
        return true;
    }

    // Check nested locations
    if let Ok(entries) = std::fs::read_dir(buffer_path::file_dir(file_hash)) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let content_path = entry.path().join(true_id).join("content");
                if content_path.exists() {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_hash_for_true_id_returns_duplicate_true_id_on_ambiguity() {
        // Setup: Create same true_id in two different file_hash directories
        let content1 = b"test content 1";
        let content2 = b"test content 2";
        let file_hash1 = crate::hash::compute(content1);
        let file_hash2 = crate::hash::compute(content2);
        let true_id = "duplicate_test";

        // Save in first location
        save_file_content(&file_hash1, content1).unwrap();
        save_buffer_content(&file_hash1, &true_id, content1).unwrap();
        save_buffer_metadata(
            &file_hash1,
            &true_id,
            &BufferMeta {
                true_id: true_id.to_string(),
                parent_true_id: None,
                scope_hash: crate::hash::compute(content1),
                anchor: "test".to_string(),
            },
        )
        .unwrap();

        // Save in second location (same true_id, different file_hash)
        save_file_content(&file_hash2, content2).unwrap();
        save_buffer_content(&file_hash2, &true_id, content2).unwrap();
        save_buffer_metadata(
            &file_hash2,
            &true_id,
            &BufferMeta {
                true_id: true_id.to_string(),
                parent_true_id: None,
                scope_hash: crate::hash::compute(content2),
                anchor: "test".to_string(),
            },
        )
        .unwrap();

        // Should return DuplicateTrueId error
        let result = file_hash_for_true_id(&true_id);
        assert!(result.is_err(), "should return error for duplicate true_id");
        let err = result.unwrap_err();
        assert_eq!(
            err.to_spec_string(),
            "DUPLICATE_TRUE_ID",
            "should return DuplicateTrueId, not BufferNotFound"
        );

        // Cleanup
        invalidate_true_id_hierarchy(&file_hash1, &true_id).unwrap();
        invalidate_true_id_hierarchy(&file_hash2, &true_id).unwrap();
    }
}
