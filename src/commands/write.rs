use std::fs;

/// Write: locate anchor, verify hash, replace, write back. Exit 0 or 1.
pub fn execute(
    file_path: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    expected_hash: Option<&str>,
    label: Option<&str>,
    replacement: &str,
) -> i32 {
    // Resolve file, anchor_bytes, expected_hash, and track label for cleanup
    let (target_file, anchor_bytes, expected_hash, used_label): (String, Vec<u8>, String, Option<String>) = if let Some(label_name) = label {
        // Label mode: resolve label -> true_id -> anchor metadata
        let true_id = match crate::storage::load_label_target(label_name) {
            Ok(tid) => tid,
            Err(e) => {
                eprintln!("{}", e);
                return 1;
            }
        };
        let meta = match crate::storage::load_anchor_metadata_by_true_id(&true_id) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{}", e);
                eprintln!("DEBUG: true_id = {}", true_id);
                eprintln!("DEBUG: Searching for buffers...");
                crate::storage::print_all_buffers();
                eprintln!("DEBUG: done searching");
                return 1;
            }
        };
        (meta.file, meta.anchor.into_bytes(), meta.hash, Some(label_name.to_string()))
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
        (file_path.to_string(), anchor_bytes, expected_hash, None)
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

    // Validate replacement is valid UTF-8 (defensive check; Rust &str should always satisfy this)
    if let Err(e) = crate::validate_utf8(replacement.as_bytes()) {
        eprintln!("{}", e);
        return 1;
    }

    let replacement_bytes = crate::matcher::normalize_line_endings(replacement.as_bytes());

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
        eprintln!("HASH_MISMATCH: expected={} actual={}", expected_hash, actual_hash);
        return 1;
    }

    // Splice: prefix + replacement + suffix (all in normalized space).
    let mut result: Vec<u8> = Vec::with_capacity(normalized.len());
    result.extend_from_slice(&normalized[..m.byte_start]);
    result.extend_from_slice(&replacement_bytes);
    result.extend_from_slice(&normalized[m.byte_end..]);

    match fs::write(&target_file, &result) {
        Ok(_) => {
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
