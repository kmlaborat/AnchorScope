//! AnchorScope — Deterministic scoped editing protocol.
//!
//! # Library API
//!
//! ```no_run
//! use anchorscope::{read, write, ReadResult, WriteResult};
//!
//! // Read a scoped region
//! let result = read("Cargo.toml", b"package")?;
//! println!("scope_hash={}", result.scope_hash);
//!
//! // Write a replacement
//! let result = write("Cargo.toml", b"package", &result.scope_hash, b"new content")?;
//! println!("wrote {} bytes", result.bytes_written);
//! # Ok::<(), anchorscope::AnchorScopeError>(())
//! ```

mod error;
pub(crate) mod hash;
pub(crate) mod matcher;

use std::fs;
use std::fmt;

// ── Public types ──────────────────────────────────────────────

/// Result of a successful read operation.
#[derive(Debug)]
pub struct ReadResult {
    /// xxh3_64 hash of the matched scope (lowercase 16-char hex).
    pub scope_hash: String,
    /// Raw bytes of the matched scope (normalized: CRLF → LF).
    pub content: Vec<u8>,
}

/// Result of a successful write operation.
#[derive(Debug)]
pub struct WriteResult {
    /// Total number of bytes written to the file.
    pub bytes_written: usize,
}

/// Errors that can occur during read/write operations.
#[derive(Debug)]
pub enum AnchorScopeError {
    /// The anchor was not found in the file.
    NoMatch,
    /// The anchor matched multiple locations in the file.
    MultipleMatches,
    /// The computed hash does not match the expected hash.
    HashMismatch,
    /// I/O error with a description.
    IoError(String),
}

impl fmt::Display for AnchorScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnchorScopeError::NoMatch => write!(f, "NO_MATCH"),
            AnchorScopeError::MultipleMatches => write!(f, "MULTIPLE_MATCHES"),
            AnchorScopeError::HashMismatch => write!(f, "HASH_MISMATCH"),
            AnchorScopeError::IoError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AnchorScopeError {}

// ── Public API ────────────────────────────────────────────────

/// Read a scoped region from a file.
///
/// Matches `anchor` against the file content (with CRLF → LF normalization),
/// computes the xxh3_64 hash of the matched scope, and returns the hash
/// and the matched content bytes.
pub fn read(file_path: &str, anchor: &[u8]) -> Result<ReadResult, AnchorScopeError> {
    // Read file
    let raw = fs::read(file_path).map_err(|e| {
        AnchorScopeError::IoError(error::io_error(error::map_io_error_read(&e)))
    })?;

    // Validate UTF-8
    if std::str::from_utf8(&raw).is_err() {
        return Err(AnchorScopeError::IoError(
            error::io_error(error::IoErrorKind::InvalidUtf8),
        ));
    }

    // Normalize file content (CRLF → LF)
    let (file_normalized, _offset_map) = matcher::normalize_line_endings(&raw);

    // Normalize anchor (CRLF → LF)
    let anchor_normalized = matcher::normalize_line_endings(anchor).0;

    // Match against normalized content
    let m = matcher::resolve(&file_normalized, &anchor_normalized).map_err(|e| match e {
        matcher::MatchError::NoMatch => AnchorScopeError::NoMatch,
        matcher::MatchError::MultipleMatches => AnchorScopeError::MultipleMatches,
    })?;

    // Extract the matched scope (already normalized)
    let scope_normalized = &file_normalized[m.byte_start..m.byte_end];
    let scope_hash = hash::compute(scope_normalized);
    let content = scope_normalized.to_vec();

    Ok(ReadResult {
        scope_hash,
        content,
    })
}

/// Write a replacement into a scoped region of a file.
///
/// Matches `anchor`, verifies the scope hash against `expected_hash`,
/// then replaces the matched region with `replacement` in the original file.
/// CRLF line endings outside the matched scope are preserved.
pub fn write(
    file_path: &str,
    anchor: &[u8],
    expected_hash: &str,
    replacement: &[u8],
) -> Result<WriteResult, AnchorScopeError> {
    // Read file
    let raw = fs::read(file_path).map_err(|e| {
        AnchorScopeError::IoError(error::io_error(error::map_io_error_read(&e)))
    })?;

    // Validate UTF-8
    if std::str::from_utf8(&raw).is_err() {
        return Err(AnchorScopeError::IoError(
            error::io_error(error::IoErrorKind::InvalidUtf8),
        ));
    }

    // Normalize file content (CRLF → LF), with offset map
    let (file_normalized, offset_map) = matcher::normalize_line_endings(&raw);

    // Normalize anchor (CRLF → LF)
    let anchor_normalized = matcher::normalize_line_endings(anchor).0;

    // Match against normalized content
    let m = matcher::resolve(&file_normalized, &anchor_normalized).map_err(|e| match e {
        matcher::MatchError::NoMatch => AnchorScopeError::NoMatch,
        matcher::MatchError::MultipleMatches => AnchorScopeError::MultipleMatches,
    })?;

    // Hash verification (on normalized matched scope)
    let scope_normalized = &file_normalized[m.byte_start..m.byte_end];
    let actual_hash = hash::compute(scope_normalized);
    if actual_hash != expected_hash {
        return Err(AnchorScopeError::HashMismatch);
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

    // Verify offsets
    if !matcher::verify_offset(&raw, m.byte_start, orig_start) {
        return Err(AnchorScopeError::IoError(
            error::io_error(error::IoErrorKind::WriteFailure),
        ));
    }
    if !matcher::verify_offset(&raw, m.byte_end, orig_end) {
        return Err(AnchorScopeError::IoError(
            error::io_error(error::IoErrorKind::WriteFailure),
        ));
    }

    // Build result: prefix + replacement + suffix
    let mut result: Vec<u8> =
        Vec::with_capacity(raw.len() - (orig_end - orig_start) + replacement.len());
    result.extend_from_slice(&raw[..orig_start]);
    result.extend_from_slice(replacement);
    result.extend_from_slice(&raw[orig_end..]);

    // Write file
    fs::write(file_path, &result).map_err(|e| {
        AnchorScopeError::IoError(error::io_error(error::map_io_error_write(&e)))
    })?;

    Ok(WriteResult {
        bytes_written: result.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_basic() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, b"AAA\nBBB\nCCC\n").unwrap();

        let result = read(path.to_str().unwrap(), b"BBB").unwrap();
        assert_eq!(result.content, b"BBB");
        assert_eq!(result.scope_hash.len(), 16);
    }

    #[test]
    fn read_no_match() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, b"AAA\nBBB\nCCC\n").unwrap();

        let err = read(path.to_str().unwrap(), b"XXX").unwrap_err();
        assert!(matches!(err, AnchorScopeError::NoMatch));
    }

    #[test]
    fn read_multiple_matches() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, b"AAA\nBBB\nAAA\n").unwrap();

        let err = read(path.to_str().unwrap(), b"AAA").unwrap_err();
        assert!(matches!(err, AnchorScopeError::MultipleMatches));
    }

    #[test]
    fn write_basic() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, b"AAA\nBBB\nCCC\n").unwrap();

        let read_result = read(path.to_str().unwrap(), b"BBB").unwrap();
        let write_result =
            write(path.to_str().unwrap(), b"BBB", &read_result.scope_hash, b"REPLACED").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "AAA\nREPLACED\nCCC\n");
        assert!(write_result.bytes_written > 0);
    }

    #[test]
    fn write_hash_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, b"AAA\nBBB\nCCC\n").unwrap();

        let err = write(path.to_str().unwrap(), b"BBB", "0000000000000000", b"X").unwrap_err();
        assert!(matches!(err, AnchorScopeError::HashMismatch));
    }

    #[test]
    fn error_display() {
        assert_eq!(format!("{}", AnchorScopeError::NoMatch), "NO_MATCH");
        assert_eq!(
            format!("{}", AnchorScopeError::MultipleMatches),
            "MULTIPLE_MATCHES"
        );
        assert_eq!(
            format!("{}", AnchorScopeError::HashMismatch),
            "HASH_MISMATCH"
        );
        assert_eq!(
            format!(
                "{}",
                AnchorScopeError::IoError("IO_ERROR: file not found".to_string())
            ),
            "IO_ERROR: file not found"
        );
    }
}
