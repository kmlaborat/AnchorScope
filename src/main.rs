mod cli;
mod hash;
mod matcher;

use clap::Parser;
use cli::{Cli, Command};
use dirs;
use matcher::normalize_line_endings;
use serde_json;
use std::fs;
use std::process;

fn map_io_error_read(e: std::io::Error) -> String {
    match e.kind() {
        std::io::ErrorKind::NotFound => "IO_ERROR: file not found".to_string(),
        std::io::ErrorKind::PermissionDenied => "IO_ERROR: permission denied".to_string(),
        _ => "IO_ERROR: read failure".to_string(),
    }
}

fn map_io_error_write(e: std::io::Error) -> String {
    match e.kind() {
        std::io::ErrorKind::PermissionDenied => "IO_ERROR: permission denied".to_string(),
        _ => "IO_ERROR: write failure".to_string(),
    }
}

fn validate_utf8(bytes: &[u8]) -> Result<(), String> {
    if std::str::from_utf8(bytes).is_err() {
        Err("IO_ERROR: invalid UTF-8".to_string())
    } else {
        Ok(())
    }
}

fn main() {
    let cli = Cli::parse();

    let exit_code = match cli.command {
        Command::Read {
            file,
            anchor,
            anchor_file,
        } => cmd_read(&file, anchor.as_deref(), anchor_file.as_deref()),
        Command::Write {
            file,
            anchor,
            anchor_file,
            expected_hash,
            replacement,
        } => cmd_write(
            &file,
            anchor.as_deref(),
            anchor_file.as_deref(),
            &expected_hash,
            &replacement,
        ),
        Command::Anchor {
            file,
            label,
            anchor,
            anchor_file,
            expected_hash,
        } => cmd_anchor(&file, &label, anchor.as_deref(), anchor_file.as_deref(), &expected_hash),
    };

    process::exit(exit_code);
}

/// Load and validate anchor from either inline or file source.
/// Returns normalized anchor bytes (Vec<u8>) or error string.
fn load_anchor(anchor: Option<&str>, anchor_file: Option<&str>) -> Result<Vec<u8>, String> {
    match (anchor, anchor_file) {
        (None, None) => return Err("ERROR: either --anchor or --anchor-file must be provided".to_string()),
        (Some(_), Some(_)) => return Err("IO_ERROR: mutually exclusive options".to_string()),
        _ => {}
    }

    let anchor_bytes = match anchor {
        Some(a) => {
            if a.is_empty() {
                return Err("NO_MATCH".to_string());
            }
            normalize_line_endings(a.as_bytes())
        }
        None => {
            let path = anchor_file.unwrap();
            let content = fs::read(path).map_err(|e| map_io_error_read(e))?;
            // Validate UTF-8
            if std::str::from_utf8(&content).is_err() {
                return Err("IO_ERROR: invalid UTF-8".to_string());
            }
            let s = String::from_utf8(content).unwrap(); // safe after check
            if s.is_empty() {
                return Err("NO_MATCH".to_string());
            }
            normalize_line_endings(s.as_bytes())
        }
    };

    Ok(anchor_bytes)
}

/// Read: locate anchor, print location + hash. Exit 0 on success, 1 on error.
fn cmd_read(file_path: &str, anchor: Option<&str>, anchor_file: Option<&str>) -> i32 {
    let raw = match fs::read(file_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", map_io_error_read(e));
            return 1;
        }
    };

    // Enforce UTF-8 validity per SPEC
    if std::str::from_utf8(&raw).is_err() {
        eprintln!("IO_ERROR: invalid UTF-8");
        return 1;
    }

    let normalized = normalize_line_endings(&raw);
    let anchor_bytes = match load_anchor(anchor, anchor_file) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    match matcher::resolve(&normalized, &anchor_bytes) {
        Err(e) => {
            eprintln!("{}", e);
            1
        }
        Ok(m) => {
            let region = &normalized[m.byte_start..m.byte_end];
            let h = hash::compute(region);
            // Output is machine-readable: one key=value per line.
            println!("start_line={}", m.start_line);
            println!("end_line={}", m.end_line);
            println!("hash={}", h);
            println!("content={}", String::from_utf8_lossy(region));
            0
        }
    }
}

/// Write: locate anchor, verify hash, replace, write back. Exit 0 or 1.
fn cmd_write(
    file_path: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    expected_hash: &str,
    replacement: &str,
) -> i32 {
    let raw = match fs::read(file_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", map_io_error_read(e));
            return 1;
        }
    };

    // Enforce UTF-8 validity per SPEC
    if std::str::from_utf8(&raw).is_err() {
        eprintln!("IO_ERROR: invalid UTF-8");
        return 1;
    }

    let normalized = normalize_line_endings(&raw);
    let anchor_bytes = match load_anchor(anchor, anchor_file) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    // Validate replacement is valid UTF-8 (defensive check; Rust &str should always satisfy this)
    if let Err(e) = validate_utf8(replacement.as_bytes()) {
        eprintln!("{}", e);
        return 1;
    }

    let replacement_bytes = normalize_line_endings(replacement.as_bytes());

    let m = match matcher::resolve(&normalized, &anchor_bytes) {
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
        Ok(m) => m,
    };

    let region = &normalized[m.byte_start..m.byte_end];
    let actual_hash = hash::compute(region);

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
            eprintln!("{}", map_io_error_write(e));
            1
        }
    }
}

/// Anchor: verify anchor matches expected_hash, then store label mapping.
/// Labels map to (file, anchor, hash) triples for future reference.
/// For v1.1.0, we store labels in ~/.anchorscope/labels/ as JSON.
fn cmd_anchor(
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
            eprintln!("{}", map_io_error_read(e));
            return 1;
        }
    };

    // 2. Validate UTF-8
    if std::str::from_utf8(&raw).is_err() {
        eprintln!("IO_ERROR: invalid UTF-8");
        return 1;
    }

    // 3. Normalize file content
    let normalized = normalize_line_endings(&raw);

    // 4. Load and validate anchor
    let anchor_bytes = match load_anchor(anchor, anchor_file) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    // 5. Resolve anchor
    let m = match matcher::resolve(&normalized, &anchor_bytes) {
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
        Ok(m) => m,
    };

    // 6. Compute hash of matched region
    let region = &normalized[m.byte_start..m.byte_end];
    let actual_hash = hash::compute(region);

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

    let json = match serde_json::to_string_pretty(&record) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("IO_ERROR: JSON serialization failed: {}", e);
            return 1;
        }
    };

    if let Err(e) = std::fs::write(&label_file, json) {
        eprintln!("IO_ERROR: cannot write label file: {}", e);
        return 1;
    }

    println!("OK: anchor '{}' defined", label);
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_io_error_read() {
        // Test NotFound
        let e = std::io::Error::from(std::io::ErrorKind::NotFound);
        assert_eq!(map_io_error_read(e), "IO_ERROR: file not found");

        // Test PermissionDenied
        let e = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        assert_eq!(map_io_error_read(e), "IO_ERROR: permission denied");

        // Test other errors (Interrupted, Unexpected, etc.)
        let e = std::io::Error::from(std::io::ErrorKind::Interrupted);
        assert_eq!(map_io_error_read(e), "IO_ERROR: read failure");

        let e = std::io::Error::from(std::io::ErrorKind::Other);
        assert_eq!(map_io_error_read(e), "IO_ERROR: read failure");
    }

    #[test]
    fn test_map_io_error_write() {
        // Test PermissionDenied
        let e = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        assert_eq!(map_io_error_write(e), "IO_ERROR: permission denied");

        // Test other errors (NotFound, Interrupted, etc.)
        let e = std::io::Error::from(std::io::ErrorKind::NotFound);
        assert_eq!(map_io_error_write(e), "IO_ERROR: write failure");

        let e = std::io::Error::from(std::io::ErrorKind::Interrupted);
        assert_eq!(map_io_error_write(e), "IO_ERROR: write failure");
    }

    #[test]
    fn test_validate_utf8_valid() {
        assert!(validate_utf8(b"hello").is_ok());
        assert!(validate_utf8(b"\xC3\xA9").is_ok()); // valid UTF-8: é
    }

    #[test]
    fn test_validate_utf8_invalid() {
        assert_eq!(
            validate_utf8(&[0xFF, 0xFE]),
            Err("IO_ERROR: invalid UTF-8".to_string())
        );
        assert_eq!(
            validate_utf8(b"\x80\x81\x82"),
            Err("IO_ERROR: invalid UTF-8".to_string())
        );
    }
}
