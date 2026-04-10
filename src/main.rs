mod buffer_path;
mod cli;
mod commands;
mod config;
mod hash;
mod matcher;
mod storage;

use clap::Parser;
use cli::{Cli, Command};
use matcher::normalize_line_endings;
use std::fs;
use std::process;

pub fn map_io_error_read(e: std::io::Error) -> String {
    match e.kind() {
        std::io::ErrorKind::NotFound => "IO_ERROR: file not found".to_string(),
        std::io::ErrorKind::PermissionDenied => "IO_ERROR: permission denied".to_string(),
        _ => "IO_ERROR: read failure".to_string(),
    }
}

pub fn map_io_error_write(e: std::io::Error) -> String {
    match e.kind() {
        std::io::ErrorKind::PermissionDenied => "IO_ERROR: permission denied".to_string(),
        _ => "IO_ERROR: write failure".to_string(),
    }
}

pub fn validate_utf8(bytes: &[u8]) -> Result<(), String> {
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
            label,
        } => commands::read::execute(&file, anchor.as_deref(), anchor_file.as_deref(), label.as_deref()),
        Command::Write {
            file,
            anchor,
            anchor_file,
            expected_hash,
            label,
            replacement,
        } => commands::write::execute(
            &file,
            anchor.as_deref(),
            anchor_file.as_deref(),
            expected_hash.as_deref(),
            label.as_deref(),
            &replacement,
        ),
        Command::Label {
            name,
            true_id,
        } => commands::label::execute(&name, &true_id),
        Command::Tree { file } => commands::tree::execute(&file),
    };

    process::exit(exit_code);
}

/// Load and validate anchor from either inline or file source.
/// Returns normalized anchor bytes (Vec<u8>) or error string.
pub fn load_anchor(anchor: Option<&str>, anchor_file: Option<&str>) -> Result<Vec<u8>, String> {
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
