use crate::error::{self, IoErrorKind};
use crate::hash;
use crate::matcher;
use std::fs;

/// Execute the read command.
/// Returns 0 on success, 1 on error.
pub fn execute(
    file: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
) -> i32 {
    // Load anchor bytes (raw, not yet normalized)
    let anchor_raw = match load_anchor(anchor, anchor_file) {
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

    // Normalize file content (CRLF → LF) — in-memory only
    let (file_normalized, _offset_map) = matcher::normalize_line_endings(&raw);

    // Normalize anchor (CRLF → LF)
    let anchor_normalized = matcher::normalize_line_endings(&anchor_raw).0;

    // Match against normalized content
    match matcher::resolve(&file_normalized, &anchor_normalized) {
        Err(e) => {
            eprintln!("{}", e);
            1
        }
        Ok(m) => {
            // The matched scope is already normalized
            let scope_normalized = &file_normalized[m.byte_start..m.byte_end];
            let scope_hash = hash::compute(scope_normalized);
            let content = std::str::from_utf8(scope_normalized).unwrap();

            println!("scope_hash={}", scope_hash);
            println!("content={}", content);
            0
        }
    }
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
