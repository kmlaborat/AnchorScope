use std::fs;

/// Anchor: verify anchor matches expected_hash, then store label mapping.
/// Labels map to (file, anchor, hash) triples for future reference.
/// For v1.1.0, we store labels in ~/.anchorscope/labels/ as JSON.
pub fn execute(
    file_path: &str,
    label: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    expected_hash: &str,
) -> i32 {
    // 1. Read file
    let raw = match fs::read(file_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", crate::map_io_error_read(e));
            return 1;
        }
    };

    // 2. Validate UTF-8
    if std::str::from_utf8(&raw).is_err() {
        eprintln!("IO_ERROR: invalid UTF-8");
        return 1;
    }

    // 3. Normalize file content
    let normalized = crate::matcher::normalize_line_endings(&raw);

    // 4. Load and validate anchor
    let anchor_bytes = match crate::load_anchor(anchor, anchor_file) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    // 5. Resolve anchor
    let m = match crate::matcher::resolve(&normalized, &anchor_bytes) {
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
        Ok(m) => m,
    };

    // 6. Compute hash of matched region
    let region = &normalized[m.byte_start..m.byte_end];
    let actual_hash = crate::hash::compute(region);

    // 7. Verify hash
    if actual_hash != expected_hash {
        eprintln!("HASH_MISMATCH: expected={} actual={}", expected_hash, actual_hash);
        return 1;
    }

    // 8. Store label mapping
    let home = match dirs::home_dir() {
        Some(path) => path,
        None => {
            eprintln!("IO_ERROR: cannot determine home directory");
            return 1;
        }
    };
    let label_dir = home.join(".anchorscope").join("labels");
    if let Err(e) = std::fs::create_dir_all(&label_dir) {
        eprintln!("IO_ERROR: cannot create label directory: {}", e);
        return 1;
    }

    let label_file = label_dir.join(format!("{}.json", label));
    let anchor_str = String::from_utf8_lossy(&anchor_bytes).to_string();
    let record = serde_json::json!({
        "file": file_path,
        "anchor": anchor_str,
        "hash": actual_hash,
        "line_range": [m.start_line, m.end_line],
    });

    match serde_json::to_string_pretty(&record) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&label_file, json) {
                eprintln!("IO_ERROR: cannot write label file: {}", e);
                return 1;
            }
        }
        Err(e) => {
            eprintln!("IO_ERROR: JSON serialization failed: {}", e);
            return 1;
        }
    }

    println!("OK: anchor '{}' defined", label);
    0
}
