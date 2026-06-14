/// SPEC-compliant error output string for hash mismatch.
pub fn hash_mismatch() -> &'static str {
    "HASH_MISMATCH"
}

pub fn io_error(kind: IoErrorKind) -> String {
    match kind {
        IoErrorKind::FileNotFound => "IO_ERROR: file not found".to_string(),
        IoErrorKind::PermissionDenied => "IO_ERROR: permission denied".to_string(),
        IoErrorKind::InvalidUtf8 => "IO_ERROR: invalid UTF-8".to_string(),
        IoErrorKind::ReadFailure => "IO_ERROR: read failure".to_string(),
        IoErrorKind::WriteFailure => "IO_ERROR: write failure".to_string(),
    }
}

pub enum IoErrorKind {
    FileNotFound,
    PermissionDenied,
    InvalidUtf8,
    ReadFailure,
    WriteFailure,
}

/// Map a std::io::Error to IoErrorKind for read operations.
pub fn map_io_error_read(e: &std::io::Error) -> IoErrorKind {
    match e.kind() {
        std::io::ErrorKind::NotFound => IoErrorKind::FileNotFound,
        std::io::ErrorKind::PermissionDenied => IoErrorKind::PermissionDenied,
        _ => IoErrorKind::ReadFailure,
    }
}

/// Map a std::io::Error to IoErrorKind for write operations.
pub fn map_io_error_write(e: &std::io::Error) -> IoErrorKind {
    match e.kind() {
        std::io::ErrorKind::PermissionDenied => IoErrorKind::PermissionDenied,
        _ => IoErrorKind::WriteFailure,
    }
}
