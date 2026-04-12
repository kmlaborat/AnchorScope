use crate::buffer_path;
use crate::config;
use crate::security::{ensure_no_symlinks, validate_file_path, validate_file_size};
use crate::storage;
use std::fs;
use std::path::PathBuf;

/// Read: locate anchor, print location + hash. Exit 0 on success, 1 on error.
/// If target is a buffer copy (file_hash/content or file_hash/true_id/content),
/// creates nested buffer structure per SPEC §4.3.
pub fn execute(
    file_path: Option<&str>,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    label: Option<&str>,
    true_id: Option<&str>,
) -> i32 {
    // Handle true_id mode: read from buffer instead of file
    if let Some(tid) = true_id {
        if label.is_some() {
            eprintln!("IO_ERROR: cannot specify both --label and --true-id");
            return 1;
        }
        if anchor_file.is_some() {
            eprintln!("IO_ERROR: --anchor-file not allowed with --true-id");
            return 1;
        }
        if anchor.is_none() {
            eprintln!("ANCHOR_REQUIRED");
            return 1;
        }
        
        // Check if this true_id exists in the buffer
        if !check_buffer_exists(tid) {
            eprintln!(
                "IO_ERROR: buffer metadata for true_id '{}' not found",
                tid
            );
            return 1;
        }
        
        // Find the file_hash that contains this true_id
        let file_hash = match find_file_hash_for_true_id(tid) {
            Some(h) => h,
            None => {
                eprintln!(
                    "IO_ERROR: buffer metadata for true_id '{}' not found",
                    tid
                );
                return 1;
            }
        };
        
        // Load source path from buffer
        let source_path = match storage::load_source_path(&file_hash) {
            Ok(p) => p,
            Err(ref e) if e == "DUPLICATE_TRUE_ID" => {
                eprintln!("DUPLICATE_TRUE_ID");
                return 1;
            }
            Err(e) => {
                eprintln!("IO_ERROR: cannot load source path: {}", e);
                return 1;
            }
        };
        
        // Load buffer content
        let buffer_content = match storage::load_buffer_content(&file_hash, tid) {
            Ok(c) => c,
            Err(_) => {
                // Try nested location
                let content = match find_nested_buffer_content(&file_hash, tid) {
                    Some(c) => c,
                    None => {
                        eprintln!("IO_ERROR: cannot load buffer content");
                        return 1;
                    }
                };
                content
            }
        };
        
        if std::str::from_utf8(&buffer_content).is_err() {
            eprintln!("IO_ERROR: invalid UTF-8");
            return 1;
        }
        
        let target_file = PathBuf::from(source_path);
        let anchor_bytes = anchor.unwrap().as_bytes().to_vec();
        let buffer_parent_true_id = Some((buffer_content, tid.to_string()));
        
        return process_read_with_target(target_file, anchor_bytes, buffer_parent_true_id);
    }
    
    // File-based reading
    let file_path = match file_path {
        Some(fp) => fp,
        None => {
            eprintln!("IO_ERROR: must specify either --file or --true-id");
            return 1;
        }
    };
    
    let working_dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(_) => {
            eprintln!("IO_ERROR: cannot get current directory");
            return 1;
        }
    };

    let (target_file, anchor_bytes, buffer_parent_true_id) = if let Some(label_name) = label {
        // Label mode: resolve label to buffer content
        // The label_name can be either:
        // 1. A label alias (stored in labels/)
        // 2. A true_id (used directly as label name)
        // First try to load from labels, if not found, treat as true_id
        let true_id = match storage::load_label_target(label_name) {
            Ok(tid) => tid,
            Err(ref e) if e.starts_with("IO_ERROR: file not found") => {
                // Not a label, try as true_id directly
                // Check if this true_id exists in the buffer
                if check_buffer_exists(label_name) {
                    label_name.to_string()
                } else {
                    eprintln!(
                        "IO_ERROR: buffer metadata for true_id '{}' not found",
                        label_name
                    );
                    return 1;
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                return 1;
            }
        };

        // Load source path from buffer
        // We need to find the file_hash that contains this true_id
        let file_hash = match find_file_hash_for_true_id(&true_id) {
            Some(h) => h,
            None => {
                eprintln!(
                    "IO_ERROR: buffer metadata for true_id '{}' not found",
                    true_id
                );
                return 1;
            }
        };

        // Load source path
        let source_path = match storage::load_source_path(&file_hash) {
            Ok(p) => p,
            Err(ref e) if e == "DUPLICATE_TRUE_ID" => {
                eprintln!("DUPLICATE_TRUE_ID");
                return 1;
            }
            Err(e) => {
                eprintln!("IO_ERROR: cannot load source path: {}", e);
                return 1;
            }
        };

        // Load buffer content from nested location if applicable
        // The content might be at {file_hash}/{true_id}/content (flat) or {file_hash}/{parent}/{true_id}/content (nested)
        let buffer_content = match storage::load_buffer_content(&file_hash, &true_id) {
            Ok(c) => c,
            Err(_) => {
                // Try nested location - find the parent that contains this true_id
                let content = match find_nested_buffer_content(&file_hash, &true_id) {
                    Some(c) => c,
                    None => {
                        eprintln!("IO_ERROR: cannot load buffer content");
                        return 1;
                    }
                };
                content
            }
        };

        // Validate UTF-8 for buffer content per SPEC §2.2
        // All inputs MUST be valid UTF-8, including buffer content loaded in label mode
        if std::str::from_utf8(&buffer_content).is_err() {
            eprintln!("IO_ERROR: invalid UTF-8");
            return 1;
        }

        // Normalize the anchor
        let anchor_bytes = if let Some(ref anchor_file_path) = anchor_file {
            // Validate anchor file path
            let anchor_path = match validate_file_path(anchor_file_path, &working_dir) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{}", e.to_spec_string());
                    return 1;
                }
            };

            // Check for symlinks
            if let Err(e) = ensure_no_symlinks(&anchor_path) {
                eprintln!("{}", e.to_spec_string());
                return 1;
            }

            match crate::load_anchor(anchor, Some(&anchor_path.to_string_lossy())) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
            }
        } else {
            match crate::load_anchor(anchor, None) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
            }
        };

        // For label mode, use buffer content for matching but output original file path
        (source_path, anchor_bytes, Some((buffer_content, true_id)))
    } else {
        // Direct mode: use provided args
        let target_path = match validate_file_path(file_path, &working_dir) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{}", e.to_spec_string());
                return 1;
            }
        };

        // Check for symlinks
        if let Err(e) = ensure_no_symlinks(&target_path) {
            eprintln!("{}", e.to_spec_string());
            return 1;
        }

        // Validate file size
        if let Err(e) = validate_file_size(&target_path) {
            eprintln!("{}", e.to_spec_string());
            return 1;
        }

        // Handle anchor bytes for direct mode
        let anchor_bytes = if let Some(ref anchor_file_path) = anchor_file {
            // Validate anchor file path
            let anchor_path = match validate_file_path(anchor_file_path, &working_dir) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{}", e.to_spec_string());
                    return 1;
                }
            };

            // Check for symlinks
            if let Err(e) = ensure_no_symlinks(&anchor_path) {
                eprintln!("{}", e.to_spec_string());
                return 1;
            }

            match crate::load_anchor(anchor, Some(&anchor_path.to_string_lossy())) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
            }
        } else {
            match crate::load_anchor(anchor, None) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
            }
        };

        (
            target_path.to_string_lossy().to_string(),
            anchor_bytes,
            None,
        )
    };

    // Read and validate file
    let raw = match fs::read(&target_file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", crate::map_io_error_read(e));
            return 1;
        }
    };

    // Enforce UTF-8 validity per SPEC
    if std::str::from_utf8(&raw).is_err() {
        eprintln!("IO_ERROR: invalid UTF-8");
        return 1;
    }

    let normalized = if label.is_some() {
        // For label mode, use buffer content for matching
        let content = if let Some((ref buffer_content, _)) = buffer_parent_true_id {
            buffer_content.clone()
        } else {
            return 1;
        };
        crate::matcher::normalize_line_endings(&content)
    } else {
        crate::matcher::normalize_line_endings(&raw)
    };
    let anchor_bytes_normalized = crate::matcher::normalize_line_endings(&anchor_bytes);

    match crate::matcher::resolve(&normalized, &anchor_bytes_normalized) {
        Err(e) => {
            eprintln!("{}", e);
            1
        }
        Ok(m) => {
            let scope = &normalized[m.byte_start..m.byte_end];
            let h = crate::hash::compute(scope);

            // Compute file_hash from the raw file content (not buffer content)
            // This ensures the file_hash is consistent regardless of label mode
            let file_hash = crate::hash::compute(&raw);

            // Check nesting depth limit when in label mode
            if label.is_some() {
                if let Some((ref _ref_buffer_content, ref parent_tid)) = buffer_parent_true_id {
                    let max_depth = config::max_depth();
                    match calculate_nesting_depth(parent_tid, &file_hash) {
                        Ok(depth) => {
                            // depth is the parent's nesting level.
                            // Child would be at depth + 1.
                            // If parent is at max_depth - 1, child would exceed limit.
                            if depth >= max_depth - 1 {
                                eprintln!(
                                    "IO_ERROR: maximum nesting depth ({}) exceeded",
                                    max_depth
                                );
                                return 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            return 1;
                        }
                    }
                }
            }

            // Compute True ID with parent context if nested
            let (true_id, parent_true_id) = if let Some((_ref_buffer_content, parent_tid)) =
                buffer_parent_true_id
            {
                // Load parent buffer metadata to obtain its scope hash
                let parent_scope_hash = match storage::load_buffer_metadata(&file_hash, &parent_tid)
                {
                    Ok(meta) => meta.scope_hash,
                    Err(ref e) if e == "DUPLICATE_TRUE_ID" => {
                        eprintln!("DUPLICATE_TRUE_ID");
                        return 1;
                    }
                    Err(e) => {
                        eprintln!("IO_ERROR: parent buffer metadata corrupted: {}", e);
                        return 1;
                    }
                };
                // True ID per SPEC §3.2: xxh3_64(hex(parent_scope_hash) || 0x5F || hex(child_scope_hash))
                // format! with "{}_{}" produces the same byte sequence as byte concatenation with underscore (0x5F)
                let child_scope_hash = crate::hash::compute(scope);
                (
                    crate::hash::compute(
                        format!("{}_{}", parent_scope_hash, child_scope_hash).as_bytes(),
                    ),
                    Some(parent_tid.clone()),
                )
            } else {
                let child_scope_hash = crate::hash::compute(scope);
                (
                    crate::hash::compute(format!("{}_{}", file_hash, child_scope_hash).as_bytes()),
                    None,
                )
            };

            // Output is machine-readable: one key=value per line.
            println!("start_line={}", m.start_line);
            println!("end_line={}", m.end_line);
            println!("hash={}", h);
            println!("content={}", String::from_utf8_lossy(scope));
            println!("true_id={}", &true_id);

            // Save anchor metadata per SPEC (v1.1.0 compatibility)
            let anchor_str_base = String::from_utf8_lossy(&anchor_bytes_normalized);
            let anchor_str = anchor_str_base.to_string();
            let file_path = target_file.clone();
            let meta = storage::AnchorMeta {
                file: file_path,
                anchor: anchor_str_base.to_string(),
                hash: h.clone(),
                line_range: (m.start_line, m.end_line),
            };
            if let Err(e) = storage::save_anchor_metadata(&meta) {
                eprintln!("IO_ERROR: cannot save anchor metadata: {}", e);
                return 1;
            }

            // Save buffer content per SPEC §4.3
            // Save normalized file content to {file_hash}/content
            if let Err(e) = storage::save_file_content(&file_hash, &normalized) {
                eprintln!("IO_ERROR: cannot save file content: {}", e);
                return 1;
            }

            // Save source path
            if let Err(e) = storage::save_source_path(&file_hash, &target_file) {
                eprintln!("IO_ERROR: cannot save source path: {}", e);
                return 1;
            }

            // Save matched scope content
            let buffer_to_save = if anchor_str.starts_with("def ") {
                // Extract full function body for Python function definitions
                crate::matcher::extract_function_body(&normalized, m.byte_start, m.byte_end)
            } else {
                scope.to_vec()
            };

            if let Some(ref parent_true_id) = parent_true_id {
                // Nested read: save to nested location ONLY
                // Find the parent's directory path (could be flat or nested)
                match storage::find_true_id_dir(&file_hash, parent_true_id) {
                    Ok(Some(parent_dir)) => {
                        // Build the full nested path: {parent_dir}/{true_id}
                        let nested_dir = parent_dir.join(&true_id);

                        // Ensure the directory exists
                        if let Err(e) = std::fs::create_dir_all(&nested_dir) {
                            eprintln!(
                                "IO_ERROR: cannot create directory {}: {}",
                                nested_dir.display(),
                                e
                            );
                            return 1;
                        }

                        // Save content
                        let content_path = nested_dir.join("content");
                        if let Err(e) = std::fs::write(&content_path, &buffer_to_save) {
                            eprintln!("IO_ERROR: cannot write {}: {}", content_path.display(), e);
                            return 1;
                        }

                        // Save metadata
                        let metadata_path = nested_dir.join("metadata.json");
                        let buffer_meta = storage::BufferMeta {
                            true_id: true_id.clone(),
                            parent_true_id: Some(parent_true_id.clone()), // Store parent for hierarchy
                            scope_hash: h.clone(),
                            anchor: anchor_str_base.to_string(),
                        };
                        let json = match serde_json::to_string_pretty(&buffer_meta) {
                            Ok(j) => j,
                            Err(e) => {
                                eprintln!("IO_ERROR: JSON serialization failed: {}", e);
                                return 1;
                            }
                        };
                        if let Err(e) = std::fs::write(&metadata_path, &json) {
                            eprintln!("IO_ERROR: cannot write {}: {}", metadata_path.display(), e);
                            return 1;
                        }
                    }
                    Ok(None) => {
                        eprintln!(
                            "IO_ERROR: parent directory for true_id '{}' not found",
                            parent_true_id
                        );
                        return 1;
                    }
                    Err(storage::AmbiguousAnchorError {
                        true_id: _tid,
                        locations: _,
                    }) => {
                        // DUPLICATE_TRUE_ID per SPEC §3.2
                        eprintln!("DUPLICATE_TRUE_ID");
                        return 1;
                    }
                }
            } else {
                // Level-1: save to flat location
                if let Err(e) = storage::save_scope_content(&file_hash, &true_id, &buffer_to_save) {
                    eprintln!("IO_ERROR: cannot save scope content: {}", e);
                    return 1;
                }

                // Save buffer metadata
                let buffer_meta = storage::BufferMeta {
                    true_id: true_id.clone(),
                    parent_true_id: None,
                    scope_hash: h.clone(),
                    anchor: anchor_str_base.to_string(),
                };
                if let Err(e) = storage::save_buffer_metadata(&file_hash, &true_id, &buffer_meta) {
                    eprintln!("IO_ERROR: cannot save buffer metadata: {}", e);
                    return 1;
                }
            }

            // For v1.2.0: output both label (v1.1.0 compat) and true_id
            // label is the scope hash for v1.1.0 compatibility
            println!("label={}", h);
            println!("true_id={}", true_id);
            0
        }
    }
}

/// Find buffer content for a true_id in nested directory structure
fn find_nested_buffer_content(file_hash: &str, true_id: &str) -> Option<Vec<u8>> {
    let temp_dir = std::env::temp_dir();
    let file_dir = temp_dir.join("anchorscope").join(file_hash);

    if let Ok(entries) = std::fs::read_dir(&file_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let parent_true_id = entry.file_name();
                let parent_true_id_str = parent_true_id.to_string_lossy();
                let nested_content_path =
                    buffer_path::nested_true_id_dir(file_hash, &parent_true_id_str, true_id)
                        .join("content");

                if nested_content_path.exists() {
                    return std::fs::read(&nested_content_path).ok();
                }
            }
        }
    }
    None
}

/// Find file_hash containing a given true_id by searching all buffer directories
fn find_file_hash_for_true_id(true_id: &str) -> Option<String> {
    let temp_dir = std::env::temp_dir();
    let anchorscope_dir = temp_dir.join("anchorscope");

    if let Ok(entries) = std::fs::read_dir(&anchorscope_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let file_hash = entry.file_name();
                let file_hash_str = file_hash.to_string_lossy();

                // Check if {file_hash}/{true_id}/content exists
                let content_path =
                    buffer_path::true_id_dir(&file_hash_str, true_id).join("content");
                if content_path.exists() {
                    return Some(file_hash_str.to_string());
                }

                // Check nested: {file_hash}/{parent_true_id}/{true_id}/content
                // We need to search within the file_hash directory
                if let Ok(file_dir_entries) =
                    std::fs::read_dir(buffer_path::file_dir(&file_hash_str))
                {
                    for parent_entry in file_dir_entries.flatten() {
                        if parent_entry
                            .file_type()
                            .map(|t| t.is_dir())
                            .unwrap_or(false)
                        {
                            let parent_true_id = parent_entry.file_name();
                            let nested_content_path = buffer_path::nested_true_id_dir(
                                &file_hash_str,
                                &parent_true_id.to_string_lossy(),
                                true_id,
                            )
                            .join("content");

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

/// Calculate the nesting depth for a given true_id by tracing its parent chain
/// Returns the depth (0 for level-1, 1 for level-2, etc.)
/// Level 1 (file_hash) is depth 0, Level 2 is depth 1, etc.
fn calculate_nesting_depth(true_id: &str, file_hash: &str) -> Result<usize, String> {
    use std::collections::VecDeque;

    // Level-by-level BFS to count depth correctly
    let file_dir = buffer_path::file_dir(file_hash);
    let mut queue = VecDeque::new();
    queue.push_back(file_dir);

    let mut current_depth = 0;

    while !queue.is_empty() {
        // Process all nodes at current depth
        let level_size = queue.len();
        for _ in 0..level_size {
            let current_dir = queue.pop_front().unwrap();

            // Check if {current_dir}/{true_id}/content exists
            let content_path = current_dir.join(true_id).join("content");
            if content_path.exists() {
                return Ok(current_depth);
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
        current_depth += 1;
    }

    Err(format!(
        "IO_ERROR: buffer metadata for true_id '{}' not found",
        true_id
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hash, storage};

    #[test]
    fn true_id_nested_uses_parent_scope_hash() {
        // Prepare a temporary file content with outer and inner anchors
        let content = b"12345"; // outer anchor "234", inner "3"
        let file_hash = hash::compute(content);
        // file_hash = hash of '12345'
        // Save file content
        storage::save_file_content(&file_hash, content).unwrap();
        // Simulate outer anchor scope "234"
        let outer_scope = b"234";
        let outer_scope_hash = hash::compute(outer_scope);
        let outer_true_id = hash::compute(format!("{}_{}", file_hash, outer_scope_hash).as_bytes());
        // Save outer buffer metadata
        let outer_meta = storage::BufferMeta {
            true_id: outer_true_id.clone(),
            parent_true_id: None,
            scope_hash: outer_scope_hash.clone(),
            anchor: "234".to_string(),
        };
        storage::save_buffer_metadata(&file_hash, &outer_true_id, &outer_meta).unwrap();
        storage::save_scope_content(&file_hash, &outer_true_id, outer_scope).unwrap();

        // Now simulate nested read using label pointing to outer_true_id and inner anchor "B"
        // Save label mapping and source path for the file_hash
        storage::save_label_mapping("tmp_label", &outer_true_id).unwrap();
        // Create a temporary real file for source path
        let tmp_file_path = std::env::temp_dir().join("tmp_anchor_file.txt");
        std::fs::write(&tmp_file_path, content).expect("write tmp file");
        storage::save_source_path(&file_hash, tmp_file_path.to_str().unwrap()).unwrap();
        // Execute read in label mode
        let exit_code = execute(Some("tmp_path"), Some("3"), None, Some("tmp_label"), None);
        assert_eq!(exit_code, 0);
        // Find inner true_id generated (should be stored as a label? we can locate by scanning buffers)
        // Load all buffers to find one whose parent_true_id is outer_true_id
        let _inner_file_hash = file_hash.clone(); // same file_hash used
                                                  // Directly load inner buffer metadata using expected true_id
        let inner_scope_hash = hash::compute(b"3");
        let expected_true_id =
            hash::compute(format!("{}_{}", outer_scope_hash, inner_scope_hash).as_bytes());
        // Verify that the buffer directory was created under the file hash, nested under the parent
        let file_dir = crate::buffer_path::file_dir(&file_hash);
        let parent_dir = file_dir.join(&outer_true_id);
        let entries: Vec<_> = std::fs::read_dir(&parent_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert!(
            !entries.is_empty(),
            "no buffer directories found under parent"
        );
        let names: Vec<String> = entries
            .iter()
            .map(|os| os.to_string_lossy().to_string())
            .collect();
        assert!(
            names.contains(&expected_true_id),
            "expected true_id dir not found, got: {:?}",
            names
        );
        let inner_meta = storage::load_buffer_metadata(&file_hash, &expected_true_id)
            .expect("inner metadata not found");
        assert_eq!(
            inner_meta.parent_true_id.as_deref(),
            Some(outer_true_id.as_str())
        );
        // Cleanup
        storage::invalidate_true_id_hierarchy(&file_hash, &outer_true_id).unwrap();
        storage::invalidate_true_id_hierarchy(&file_hash, &expected_true_id).unwrap();
        storage::invalidate_label("tmp_label");
        let _ = std::fs::remove_file(tmp_file_path);
    }
}

/// Check if the identifier exists as a buffer true_id
/// Recursively searches through all levels of nesting
fn check_buffer_exists(identifier: &str) -> bool {
    let temp_dir = std::env::temp_dir().join("anchorscope");

    // Check old location first (v1.1.0 compatibility)
    let anchors_dir = temp_dir.join("anchors");
    let old_path = anchors_dir.join(format!("{}.json", identifier));

    if old_path.exists() {
        return true;
    }

    // Search all file_hash directories
    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let file_hash = entry.file_name();
                let file_hash_str = file_hash.to_string_lossy();

                // Skip special subdirectories
                if file_hash_str == "anchors" || file_hash_str == "labels" {
                    continue;
                }

                // Search recursively in this file_hash directory
                if check_buffer_exists_in_dir(&file_hash_str, identifier) {
                    return true;
                }
            }
        }
    }

    false
}

/// Recursively search for identifier in a file_hash directory
/// Checks flat location: {file_hash}/{true_id}/content
/// Checks nested locations: {file_hash}/{parent}/{true_id}/content (recursively)
fn check_buffer_exists_in_dir(file_hash: &str, identifier: &str) -> bool {
    // Check flat: {file_hash}/{true_id}/content
    let flat_content_path = buffer_path::true_id_dir(file_hash, identifier).join("content");
    if flat_content_path.exists() {
        return true;
    }

    // Check nested locations recursively
    let file_dir = buffer_path::file_dir(file_hash);
    if let Ok(entries) = std::fs::read_dir(&file_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let child_dir = entry.path();
                let _ = entry.file_name();

                // Check if {child_dir}/{identifier}/content exists
                let nested_content_path = child_dir.join(identifier).join("content");
                if nested_content_path.exists() {
                    return true;
                }

                // Recursively check in this child directory
                if check_buffer_exists_in_dir_recursive(&child_dir.to_string_lossy(), identifier) {
                    return true;
                }
            }
        }
    }

    false
}

/// Process read with pre-determined target file and anchor bytes.
/// Used for true_id mode where buffer is already loaded.
fn process_read_with_target(
    target_file: PathBuf,
    anchor_bytes: Vec<u8>,
    buffer_parent_true_id: Option<(Vec<u8>, String)>,
) -> i32 {
    // Read and validate file
    let raw = match fs::read(&target_file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", crate::map_io_error_read(e));
            return 1;
        }
    };

    // Enforce UTF-8 validity per SPEC
    if std::str::from_utf8(&raw).is_err() {
        eprintln!("IO_ERROR: invalid UTF-8");
        return 1;
    }

    let normalized = if buffer_parent_true_id.is_some() {
        // For label/true_id mode, use buffer content for matching
        let content = if let Some((ref buffer_content, _)) = buffer_parent_true_id {
            buffer_content.clone()
        } else {
            return 1;
        };
        crate::matcher::normalize_line_endings(&content)
    } else {
        crate::matcher::normalize_line_endings(&raw)
    };
    let anchor_bytes_normalized = crate::matcher::normalize_line_endings(&anchor_bytes);

    match crate::matcher::resolve(&normalized, &anchor_bytes_normalized) {
        Err(e) => {
            eprintln!("{}", e);
            1
        }
        Ok(m) => {
            let scope = &normalized[m.byte_start..m.byte_end];
            let h = crate::hash::compute(scope);

            // Compute file_hash from the raw file content (not buffer content)
            let file_hash = crate::hash::compute(&raw);

            // Check nesting depth limit when in true_id/label mode
            if buffer_parent_true_id.is_some() {
                if let Some((ref _ref_buffer_content, ref parent_tid)) = buffer_parent_true_id {
                    let max_depth = config::max_depth();
                    match calculate_nesting_depth(parent_tid, &file_hash) {
                        Ok(depth) => {
                            if depth >= max_depth - 1 {
                                eprintln!(
                                    "IO_ERROR: maximum nesting depth ({}) exceeded",
                                    max_depth
                                );
                                return 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            return 1;
                        }
                    }
                }
            }

            // Compute True ID with parent context if nested
            let (true_id, parent_true_id) = if let Some((_ref_buffer_content, parent_tid)) =
                buffer_parent_true_id
            {
                let parent_scope_hash = match storage::load_buffer_metadata(&file_hash, &parent_tid)
                {
                    Ok(meta) => meta.scope_hash,
                    Err(ref e) if e == "DUPLICATE_TRUE_ID" => {
                        eprintln!("DUPLICATE_TRUE_ID");
                        return 1;
                    }
                    Err(e) => {
                        eprintln!("IO_ERROR: parent buffer metadata corrupted: {}", e);
                        return 1;
                    }
                };
                let child_scope_hash = crate::hash::compute(scope);
                (
                    crate::hash::compute(
                        format!("{}_{}", parent_scope_hash, child_scope_hash).as_bytes(),
                    ),
                    Some(parent_tid.clone()),
                )
            } else {
                let child_scope_hash = crate::hash::compute(scope);
                (
                    crate::hash::compute(format!("{}_{}", file_hash, child_scope_hash).as_bytes()),
                    None,
                )
            };

            println!("start_line={}", m.start_line);
            println!("end_line={}", m.end_line);
            println!("hash={}", h);
            println!("content={}", String::from_utf8_lossy(scope));
            println!("true_id={}", &true_id);

            let anchor_str_base = String::from_utf8_lossy(&anchor_bytes_normalized);
            let anchor_str = anchor_str_base.to_string();

            // Save anchor metadata per SPEC
            {
                let buffer_meta = storage::BufferMeta {
                    true_id: true_id.clone(),
                    parent_true_id: parent_true_id.clone(),
                    scope_hash: h.clone(),
                    anchor: anchor_str_base.to_string(),
                };
                
                // Save content first
                if let Some(ref parent_tid) = parent_true_id {
                    // Save to nested directory
                    let nested_dir = buffer_path::nested_true_id_dir(&file_hash, parent_tid, &true_id);
                    if let Err(e) = std::fs::create_dir_all(&nested_dir) {
                        eprintln!("IO_ERROR: cannot create directory {}: {}", nested_dir.display(), e);
                        return 1;
                    }
                    let content_path = nested_dir.join("content");
                    if let Err(e) = std::fs::write(&content_path, scope) {
                        eprintln!("IO_ERROR: cannot write {}: {}", content_path.display(), e);
                        return 1;
                    }
                } else {
                    // Save to flat directory
                    if let Err(e) = storage::save_scope_content(&file_hash, &true_id, scope) {
                        eprintln!("{}", e);
                        return 1;
                    }
                }
                
                // Save metadata
                let metadata_result = if let Some(ref parent_tid) = parent_true_id {
                    let nested_dir = buffer_path::nested_true_id_dir(&file_hash, parent_tid, &true_id);
                    let metadata_path = nested_dir.join("metadata.json");
                    let json = match serde_json::to_string_pretty(&buffer_meta) {
                        Ok(j) => j,
                        Err(_) => {
                            eprintln!("IO_ERROR: JSON serialization failed for buffer metadata");
                            return 1;
                        }
                    };
                    if let Err(e) = std::fs::write(&metadata_path, &json) {
                        eprintln!("IO_ERROR: cannot write {}: {}", metadata_path.display(), e);
                        return 1;
                    }
                    Ok(())
                } else {
                    // Save to flat directory
                    storage::save_buffer_metadata(&file_hash, &true_id, &buffer_meta)
                };
                if let Err(e) = metadata_result {
                    eprintln!("{}", e);
                    return 1;
                }

                // For v1.2.0: output both label (v1.1.0 compat) and true_id
                println!("label={}", h);
                println!("true_id={}", true_id);
                0
            }
        }
    }
}

/// Recursively search for identifier starting from a given directory
fn check_buffer_exists_in_dir_recursive(dir_path: &str, identifier: &str) -> bool {
    let dir = PathBuf::from(dir_path);
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let child_dir = entry.path();
                let _ = entry.file_name();

                // Check if {child_dir}/{identifier}/content exists
                let nested_content_path = child_dir.join(identifier).join("content");
                if nested_content_path.exists() {
                    return true;
                }

                // Recursively check in this child directory
                if check_buffer_exists_in_dir_recursive(
                    child_dir.to_string_lossy().as_ref(),
                    identifier,
                ) {
                    return true;
                }
            }
        }
    }

    false
}
