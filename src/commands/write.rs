use crate::error::AnchorScopeError;
use crate::security::{ensure_no_symlinks, validate_file_path, validate_file_size};
use crate::storage;
use std::fs;
use tempfile::NamedTempFile;

/// Write file atomically using temp file and rename
fn atomic_write_file(path: &std::path::Path, content: &[u8]) -> Result<(), AnchorScopeError> {
    use std::io::Write;

    let parent = match path.parent() {
        Some(p) => p,
        None => {
            return Err(AnchorScopeError::WriteFailure(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "path has no parent",
            )));
        }
    };

    let mut temp_file = match NamedTempFile::new_in(parent) {
        Ok(f) => f,
        Err(e) => {
            return Err(AnchorScopeError::WriteFailure(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("tempfile creation error: {}", e),
            )));
        }
    };

    if let Err(e) = temp_file.write_all(content) {
        return Err(AnchorScopeError::WriteFailure(e));
    }

    // Atomic rename
    match temp_file.persist(path) {
        Ok(_) => Ok(()),
        Err(e) => Err(AnchorScopeError::WriteFailure(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("tempfile persist error: {}", e),
        ))),
    }
}

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
    // If neither source is provided, report missing replacement
    if !from_replacement && replacement.is_empty() {
        eprintln!("NO_REPLACEMENT");
        return 1;
    }

    // Get current working directory for path validation
    let working_dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(_) => {
            eprintln!("IO_ERROR: cannot get current directory");
            return 1;
        }
    };

    // Resolve file, anchor_bytes, expected_hash, and track label for cleanup
    let (target_file, anchor_bytes, expected_hash, used_label, replacement_bytes): (
        String,
        Vec<u8>,
        String,
        Option<String>,
        Vec<u8>,
    ) = if let Some(label_name) = label {
        // Label mode: resolve label -> true_id -> anchor metadata
        let true_id = match crate::storage::load_label_target(label_name) {
            Ok(tid) => tid,
            Err(e) => {
                eprintln!("{}", e);
                return 1;
            }
        };

        // Check for DUPLICATE_TRUE_ID per SPEC: same true_id in multiple locations within the same file_hash directory
        // Only check if the true_id exists in the buffer (not for old-format v1.1.0 anchors)
        if let Ok(file_hash) = storage::file_hash_for_true_id(&true_id) {
            // Only check for duplicates within this file_hash
            match storage::check_duplicate_true_id_in_file_hash(&file_hash, &true_id) {
                Ok(_) => {
                    // Single location - OK
                }
                Err(_) => {
                    eprintln!("DUPLICATE_TRUE_ID");
                    return 1;
                }
            }
        }
        // If file_hash_for_true_id fails, the true_id doesn't exist in the buffer
        // (e.g., old v1.1.0 format), so skip the duplicate check

        let meta = match crate::storage::load_anchor_metadata_by_true_id(&true_id) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{}", e);
                return 1;
            }
        };

        // Get file_hash for this true_id (required for loading replacement content)
        // For v1.1.0 format anchors (stored in anchors/ directory), file_hash_for_true_id will fail
        // In that case, --from-replacement is not supported
        let file_hash_or_error = storage::file_hash_for_true_id(&true_id);

        // Determine replacement content
        let rep_bytes = if from_replacement {
            match file_hash_or_error {
                Ok(file_hash) => {
                    match crate::storage::load_replacement_content(&file_hash, &true_id) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("{}", e);
                            return 1;
                        }
                    }
                }
                Err(_) => {
                    // v1.1.0 format anchor - no replacement file exists
                    eprintln!(
                        "IO_ERROR: --from-replacement not supported for v1.1.0 format anchors"
                    );
                    return 1;
                }
            }
        } else {
            // Use inline replacement
            crate::matcher::normalize_line_endings(replacement.as_bytes())
        };
        (
            meta.file,
            meta.anchor.into_bytes(),
            meta.hash,
            Some(label_name.to_string()),
            rep_bytes,
        )
    } else {
        // Direct mode: use provided args (must have anchor and expected_hash)

        // Validate target file path
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

        // Validate anchor file if provided
        if let Some(ref anchor_file_path) = anchor_file {
            let anchor_path = match validate_file_path(anchor_file_path, &working_dir) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{}", e.to_spec_string());
                    return 1;
                }
            };

            if let Err(e) = ensure_no_symlinks(&anchor_path) {
                eprintln!("{}", e.to_spec_string());
                return 1;
            }
        }

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
                eprintln!("NO_REPLACEMENT");
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
        (
            target_path.to_string_lossy().to_string(),
            anchor_bytes,
            expected_hash,
            None,
            rep_bytes,
        )
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

    let scope = &normalized[m.byte_start..m.byte_end];
    let actual_hash = crate::hash::compute(scope);

    if actual_hash != expected_hash {
        eprintln!("HASH_MISMATCH");
        return 1;
    }

    // Splice: prefix + replacement + suffix (all in normalized space).
    let mut result: Vec<u8> = Vec::with_capacity(normalized.len());
    result.extend_from_slice(&normalized[..m.byte_start]);
    result.extend_from_slice(&replacement_bytes);
    result.extend_from_slice(&normalized[m.byte_end..]);

    // Write file atomically using temp file and rename
    match atomic_write_file(std::path::Path::new(&target_file), &result) {
        Ok(_) => {
            // Clean up buffer artifacts BEFORE invalidating the label
            if let Some(ref label_name) = used_label {
                match crate::storage::load_label_target(label_name) {
                    Ok(true_id) => match crate::storage::file_hash_for_true_id(&true_id) {
                        Ok(file_hash) => {
                            let _ =
                                crate::storage::invalidate_true_id_hierarchy(&file_hash, &true_id);
                        }
                        Err(_) => {}
                    },
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
            eprintln!("{}", e.to_spec_string());
            1
        }
    }
}
