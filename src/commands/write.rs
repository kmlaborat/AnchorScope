use std::fs;

/// Write: locate anchor, verify hash, replace, write back. Exit 0 or 1.
pub fn execute(
    file_path: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    expected_hash: &str,
    replacement: &str,
) -> i32 {
    let raw = match fs::read(file_path) {
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
    let anchor_bytes = match crate::load_anchor(anchor, anchor_file) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

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

    match fs::write(file_path, &result) {
        Ok(_) => {
            println!("OK: written {} bytes", result.len());
            0
        }
        Err(e) => {
            eprintln!("{}", crate::map_io_error_write(e));
            1
        }
    }
}
