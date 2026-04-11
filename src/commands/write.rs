use std::fs;
use crate::storage;
use crate::buffer_path;

/// Write: locate anchor, verify hash, replace, write back. Exit 0 or 1.
pub fn execute(
    file_path: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    expected_hash: Option<&str>,
    label: Option<&str>,
    replacement: &str,
    from_replacement: bool,
) -> i32 {
    // Validate replacement source - cannot use both options
    if from_replacement && !replacement.is_empty() {
        eprintln!("AMBIGUOUS_REPLACEMENT");
        return 1;
    }

    // Resolve file, anchor_bytes, expected_hash, and track label for cleanup
    let (target_file, anchor_bytes, expected_hash, used_label, replacement_bytes): (String, Vec<u8>, String, Option<String>, Vec<u8>) = if let Some(label_name) = label {
        // Label mode: resolve label -> true_id -> anchor metadata
        let true_id = match crate::storage::load_label_target(label_name) {
            Ok(tid) => tid,
            Err(e) => {
                eprintln!("{}", e);
                return 1;
            }
        };
        
        // Check for DUPLICATE_TRUE_ID per SPEC: same true_id in multiple locations within the same file_hash directory
        // Find all file_hash directories where this true_id exists, then check each for duplicates
        let temp_dir = std::env::temp_dir();
        let anchorscope_dir = temp_dir.join("anchorscope");
        let mut duplicate_check_error: Option<String> = None;
        
        if let Ok(entries) = std::fs::read_dir(&anchorscope_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let file_hash = entry.file_name();
                    let file_hash_str = file_hash.to_string_lossy();
                    
                    // Skip special subdirectories like "anchors" and "labels"
                    if file_hash_str == "anchors" || file_hash_str == "labels" {
                        continue;
                    }
                    
                    // Check if true_id exists in this file_hash directory (including nested)
                    // We need to use a simpler approach since find_true_id_in_dir_recursive is private
                    // Use file_hash_exists_in_dir_with_count which returns (found, count)
                    let (found, count) = crate::storage::file_hash_exists_in_dir_with_count(
                        &buffer_path::file_dir(&file_hash_str),
                        &true_id
                    );
                    if found && count > 1 {
                        // true_id found in this file_hash, now check for duplicates within this file_hash
                        match storage::check_duplicate_true_id_in_file_hash(&file_hash_str, &true_id) {
                            Ok(_) => {
                                // Single location - OK
                            }
                            Err(_) => {
                                // Multiple locations within same file_hash - DUPLICATE_TRUE_ID
                                duplicate_check_error = Some("DUPLICATE_TRUE_ID".to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        if let Some(err) = duplicate_check_error {
            eprintln!("{}", err);
            return 1;
        }
        
        let meta = match crate::storage::load_anchor_metadata_by_true_id(&true_id) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{}", e);
                return 1;
            }
        };
        
        let meta = match crate::storage::load_anchor_metadata_by_true_id(&true_id) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{}", e);
                return 1;
            }
        };
        // Determine replacement content
        let rep_bytes = if from_replacement {
            match crate::storage::load_replacement_content(&meta.file, &true_id) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
            }
        } else {
            // Use inline replacement
            crate::matcher::normalize_line_endings(replacement.as_bytes())
        };
        (meta.file, meta.anchor.into_bytes(), meta.hash, Some(label_name.to_string()), rep_bytes)
    } else {
        // Direct mode: use provided args (must have anchor and expected_hash)
        let anchor_bytes = match crate::load_anchor(anchor, anchor_file) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{}", e);
                return 1;
            }
        };
        let expected_hash = match expected_hash {
            Some(h) => h.to_string(),
            None => {
                eprintln!("ERROR: expected-hash required when not using label");
                return 1;
            }
        };
        // Determine replacement content
        let rep_bytes = if from_replacement {
            eprintln!("IO_ERROR: cannot use --from-replacement without --label");
            return 1;
        } else {
            crate::matcher::normalize_line_endings(replacement.as_bytes())
        };
        (file_path.to_string(), anchor_bytes, expected_hash, None, rep_bytes)
    };

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

    let normalized = crate::matcher::normalize_line_endings(&raw);

    // replacement_bytes is already computed in the branch above

    let m = match crate::matcher::resolve(&normalized, &anchor_bytes) {
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
        Ok(m) => m,
    };

    let region = &normalized[m.byte_start..m.byte_end];
    let actual_hash = crate::hash::compute(region);

    if actual_hash != expected_hash {
        eprintln!("HASH_MISMATCH");
        return 1;
    }

    // Splice: prefix + replacement + suffix (all in normalized space).
    let mut result: Vec<u8> = Vec::with_capacity(normalized.len());
    result.extend_from_slice(&normalized[..m.byte_start]);
    result.extend_from_slice(&replacement_bytes);
    result.extend_from_slice(&normalized[m.byte_end..]);

    match fs::write(&target_file, &result) {
        Ok(_) => {
            // Clean up buffer artifacts BEFORE invalidating the label
            if let Some(ref label_name) = used_label {
                match crate::storage::load_label_target(label_name) {
                    Ok(true_id) => {
                        match crate::storage::file_hash_for_true_id(&true_id) {
                            Ok(file_hash) => {
                                let _ = crate::storage::invalidate_true_id_hierarchy(&file_hash, &true_id);
                            }
                            Err(_) => {}
                        }
                    }
                    Err(_) => {}
                }
            }
            
            // Clean up ephemeral files after successful write (SPEC §3.3)
            if let Some(ref lname) = used_label {
                crate::storage::invalidate_label(lname);
            }
            crate::storage::invalidate_anchor(&expected_hash);
            
            println!("OK: written {} bytes", result.len());
            0
        }
        Err(e) => {
            eprintln!("{}", crate::map_io_error_write(e));
            1
        }
    }
}
