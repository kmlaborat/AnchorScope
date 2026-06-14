use crate::error::{self, IoErrorKind};
use crate::hash;
use crate::matcher;
use std::fs;

/// Execute the write command.
/// Returns 0 on success, 1 on error.
pub fn execute(
    file: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    expected_hash: &str,
    replacement: Option<&str>,
    replacement_file: Option<&str>,
) -> i32 {
    // Load anchor bytes (raw, not yet normalized)
    let anchor_raw = match load_anchor(anchor, anchor_file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    // Load replacement bytes
    let replacement_bytes = match load_replacement(replacement, replacement_file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    // Read file
    let raw = match fs::read(file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", error::io_error(error::map_io_error_read(&e)));
            return 1;
        }
    };

    // Validate UTF-8
    if std::str::from_utf8(&raw).is_err() {
        eprintln!("{}", error::io_error(IoErrorKind::InvalidUtf8));
        return 1;
    }

    // Normalize file content (CRLF → LF) — in-memory only, with offset map
    let (file_normalized, offset_map) = matcher::normalize_line_endings(&raw);

    // Normalize anchor (CRLF → LF)
    let anchor_normalized = matcher::normalize_line_endings(&anchor_raw).0;

    // Match against normalized content
    let m = match matcher::resolve(&file_normalized, &anchor_normalized) {
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
        Ok(m) => m,
    };

    // Hash verification (on normalized matched scope)
    let scope_normalized = &file_normalized[m.byte_start..m.byte_end];
    let actual_hash = hash::compute(scope_normalized);
    if actual_hash != expected_hash {
        eprintln!("{}", error::hash_mismatch());
        return 1;
    }

    // Map normalized match range back to original file byte range
    let (orig_start, orig_end) = matcher::map_to_original(
        &raw,
        &file_normalized,
        &offset_map,
        m.byte_start,
        m.byte_end,
        raw.len(),
    );

    // Verify offsets using the formula:
    // original_offset == normalized_offset + number_of_CR_before_original_offset
    if !matcher::verify_offset(&raw, m.byte_start, orig_start) {
        eprintln!("{}", error::io_error(IoErrorKind::WriteFailure));
        return 1;
    }
    if !matcher::verify_offset(&raw, m.byte_end, orig_end) {
        eprintln!("{}", error::io_error(IoErrorKind::WriteFailure));
        return 1;
    }

    // Build result: prefix (original) + replacement + suffix (original)
    let mut result: Vec<u8> =
        Vec::with_capacity(raw.len() - (orig_end - orig_start) + replacement_bytes.len());
    result.extend_from_slice(&raw[..orig_start]);
    result.extend_from_slice(&replacement_bytes);
    result.extend_from_slice(&raw[orig_end..]);

    // Write file
    if let Err(e) = fs::write(file, &result) {
        eprintln!("{}", error::io_error(error::map_io_error_write(&e)));
        return 1;
    }

    println!("OK: written {} bytes", result.len());
    0
}

/// Load anchor from either inline or file source.
/// Returns raw anchor bytes (not normalized).
fn load_anchor(anchor: Option<&str>, anchor_file: Option<&str>) -> Result<Vec<u8>, String> {
    match (anchor, anchor_file) {
        (None, None) => Err("ERROR: either --anchor or --anchor-file must be provided".to_string()),
        (Some(a), None) => {
            if a.is_empty() {
                Err("NO_MATCH".to_string())
            } else {
                Ok(a.as_bytes().to_vec())
            }
        }
        (None, Some(path)) => {
            let content = fs::read(path).map_err(|e| error::io_error(error::map_io_error_read(&e)))?;
            if std::str::from_utf8(&content).is_err() {
                return Err(error::io_error(IoErrorKind::InvalidUtf8));
            }
            let s = String::from_utf8(content).unwrap();
            if s.is_empty() {
                return Err("NO_MATCH".to_string());
            }
            Ok(s.into_bytes())
        }
        (Some(_), Some(_)) => {
            Err("ERROR: --anchor and --anchor-file are mutually exclusive".to_string())
        }
    }
}

/// Load replacement from either inline string or file.
/// Returns raw replacement bytes (not normalized — written as-is per SPEC §2.3).
fn load_replacement(replacement: Option<&str>, replacement_file: Option<&str>) -> Result<Vec<u8>, String> {
    match (replacement, replacement_file) {
        (Some(r), None) => Ok(r.as_bytes().to_vec()),
        (None, Some(path)) => {
            let content = fs::read(path).map_err(|e| error::io_error(error::map_io_error_read(&e)))?;
            if std::str::from_utf8(&content).is_err() {
                return Err(error::io_error(IoErrorKind::InvalidUtf8));
            }
            Ok(content)
        }
        (None, None) => Err("ERROR: either --replacement or --replacement-file must be provided".to_string()),
        (Some(_), Some(_)) => {
            Err("ERROR: --replacement and --replacement-file are mutually exclusive".to_string())
        }
    }
}
