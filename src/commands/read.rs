use std::fs;
use crate::storage;
use crate::buffer_path;
use crate::matcher;

/// Read: locate anchor, print location + hash. Exit 0 on success, 1 on error.
/// If target is a buffer copy (file_hash/content or file_hash/true_id/content),
/// creates nested buffer structure per SPEC §4.3.
pub fn execute(
    file_path: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    label: Option<&str>,
) -> i32 {
    // Resolve target file and anchor bytes
    // If label is provided, read from buffer content instead of file
    let (target_file, anchor_bytes, buffer_parent_true_id) = if let Some(label_name) = label {
        // Label mode: resolve label to buffer content
        let true_id = match storage::load_label_target(label_name) {
            Ok(tid) => tid,
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
                eprintln!("IO_ERROR: buffer metadata for true_id '{}' not found", true_id);
                return 1;
            }
        };
        
        // Load source path
        let source_path = match storage::load_source_path(&file_hash) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("IO_ERROR: cannot load source path: {}", e);
                return 1;
            }
        };
        
        // Load file content from {file_hash}/content (the full file for nested anchors)
        let buffer_content = match storage::load_file_content(&file_hash) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("IO_ERROR: cannot load file content: {}", e);
                return 1;
            }
        };
        
        // Normalize the anchor
        let anchor_bytes = match crate::load_anchor(anchor, anchor_file) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{}", e);
                return 1;
            }
        };
        
        // For label mode, use buffer content for matching but output original file path
        (source_path, anchor_bytes, Some((buffer_content, true_id)))
    } else {
        // Direct mode: use provided args
        let anchor_bytes = match crate::load_anchor(anchor, anchor_file) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{}", e);
                return 1;
            }
        };
        
        (file_path.to_string(), anchor_bytes, None)
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

    let is_label_mode = label.is_some();
    let normalized = if is_label_mode {
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
            let region = &normalized[m.byte_start..m.byte_end];
            let h = crate::hash::compute(region);
            
            // Compute file_hash and true_id first (needed for buffer save)
            let file_hash = crate::hash::compute(&normalized);
            
            // Compute True ID with parent context if nested
            let (true_id, parent_true_id) = if let Some((ref buffer_content, parent_tid)) = buffer_parent_true_id {
                let region_hash = crate::hash::compute(buffer_content);
                (crate::hash::compute(format!("{}_{}", parent_tid, region_hash).as_bytes()), Some(parent_tid.to_string()))
            } else {
                let region_hash = crate::hash::compute(region);
                (crate::hash::compute(format!("{}_{}", file_hash, region_hash).as_bytes()), None)
            };
            
            // Output is machine-readable: one key=value per line.
            println!("start_line={}", m.start_line);
            println!("end_line={}", m.end_line);
            println!("hash={}", h);
            println!("content={}", String::from_utf8_lossy(region));
            
            // Save anchor metadata per SPEC (v1.1.0 compatibility)
            let anchor_str = String::from_utf8_lossy(&anchor_bytes_normalized).to_string();
            let file_path = target_file.clone();
            let meta = storage::AnchorMeta {
                file: file_path,
                anchor: anchor_str,
                hash: h.clone(),
                line_range: (m.start_line, m.end_line),
            };
            if let Err(e) = storage::save_anchor_metadata(&meta) {
                eprintln!("IO_ERROR: cannot save anchor metadata: {}", e);
                return 1;
            }
            
            // Save buffer content per SPEC §4.3
            // Save normalized file content to {file_hash}/content
            // For function anchors, save only the function body for nested anchor support
            let buffer_to_save = if !is_label_mode {
                // Direct mode - extract function body if it's a function definition
                if String::from_utf8_lossy(region).starts_with("def ") {
                    if let Some(func_range) = matcher::extract_function_body(&normalized, m.start_line) {
                        normalized[func_range].to_vec()
                    } else {
                        normalized.clone()
                    }
                } else {
                    normalized.clone()
                }
            } else {
                // Label mode - use file content from the parent buffer
                normalized.clone()
            };
            
            if let Err(e) = storage::save_file_content(&file_hash, &buffer_to_save) {
                eprintln!("IO_ERROR: cannot save file content: {}", e);
                return 1;
            }
            
            // Save source path
            if let Err(e) = storage::save_source_path(&file_hash, &target_file) {
                eprintln!("IO_ERROR: cannot save source path: {}", e);
                return 1;
            }
            
            // Save matched region content to {file_hash}/{true_id}/content
            if let Err(e) = storage::save_region_content(&file_hash, &true_id, region) {
                eprintln!("IO_ERROR: cannot save region content: {}", e);
                return 1;
            }
            
            // For nested reads, save to nested location
            if let Some(ref parent_true_id) = parent_true_id {
                if let Err(e) = storage::save_nested_buffer_content(&file_hash, parent_true_id, &true_id, region) {
                    eprintln!("IO_ERROR: cannot save nested buffer content: {}", e);
                    return 1;
                }
            }
            
            // Save buffer metadata for v1.2.0 compatibility
            let buffer_meta = storage::BufferMeta {
                true_id: true_id.clone(),
                parent_true_id: parent_true_id.clone(),
                region_hash: h.clone(),
            };
            if let Err(e) = storage::save_buffer_metadata(&file_hash, &true_id, &buffer_meta) {
                eprintln!("IO_ERROR: cannot save buffer metadata: {}", e);
                return 1;
            }
            
            // For v1.2.0: output both label (v1.1.0 compat) and true_id
            // label is the region hash for v1.1.0 compatibility
            println!("label={}", h);
            println!("true_id={}", true_id);
            0
        }
    }
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
