use thiserror::Error;

/// AnchorScope-specific errors per SPEC §4.5
#[derive(Error, Debug)]
pub enum AnchorScopeError {
    #[error("NO_MATCH")]
    NoMatch,

    #[error("MULTIPLE_MATCHES")]
    MultipleMatches,

    #[error("HASH_MISMATCH")]
    HashMismatch,

    #[error("DUPLICATE_TRUE_ID")]
    DuplicateTrueId,

    #[error("LABEL_EXISTS")]
    LabelExists,

    #[error("AMBIGUOUS_REPLACEMENT")]
    AmbiguousReplacement,

    #[error("NO_REPLACEMENT")]
    NoReplacement,

    #[error("IO_ERROR: file not found")]
    FileNotFound,

    #[error("IO_ERROR: permission denied")]
    PermissionDenied,

    #[error("IO_ERROR: invalid UTF-8")]
    InvalidUtf8,

    #[error("IO_ERROR: read failure")]
    ReadFailure,

    #[error("IO_ERROR: write failure")]
    WriteFailure,

    #[error("IO_ERROR: buffer metadata for true_id '{0}' not found")]
    BufferMetadataNotFound(String),

    #[error("IO_ERROR: parent buffer metadata corrupted: {0}")]
    ParentBufferMetadataCorrupted(String),

    #[error("IO_ERROR: cannot load source path: {0}")]
    CannotLoadSourcePath(String),

    #[error("IO_ERROR: cannot save file content: {0}")]
    CannotSaveFileContent(String),

    #[error("IO_ERROR: cannot save source path: {0}")]
    CannotSaveSourcePath(String),

    #[error("IO_ERROR: cannot save scope content: {0}")]
    CannotSaveScopeContent(String),

    #[error("IO_ERROR: cannot save buffer metadata: {0}")]
    CannotSaveBufferMetadata(String),

    #[error("IO_ERROR: JSON serialization failed: {0}")]
    JsonSerializationFailed(String),

    #[error("IO_ERROR: label mapping corrupted: {0}")]
    LabelMappingCorrupted(String),

    #[error("IO_ERROR: cannot load buffer content")]
    CannotLoadBufferContent,

    #[error("IO_ERROR: parent directory for true_id '{0}' not found")]
    ParentDirectoryNotFound(String),

    #[error("IO_ERROR: maximum nesting depth ({0}) exceeded")]
    MaximumNestingDepthExceeded(usize),

    #[error("IO_ERROR: external tool failed")]
    ExternalToolFailed,

    #[error("IO_ERROR: cannot execute external tool")]
    CannotExecuteExternalTool,

    #[error("IO_ERROR: cannot create temporary directory")]
    CannotCreateTempDirectory,

    #[error("IO_ERROR: buffer not found")]
    BufferNotFound,

    #[error("IO_ERROR: replacement not found")]
    ReplacementNotFound,

    #[error("IO_ERROR: label mapping for '{0}' not found")]
    LabelMappingNotFound(String),
}

/// Convert AnchorScopeError to SPEC-compliant error string
impl AnchorScopeError {
    pub fn to_spec_string(&self) -> String {
        match self {
            AnchorScopeError::NoMatch => "NO_MATCH".to_string(),
            AnchorScopeError::MultipleMatches => "MULTIPLE_MATCHES".to_string(),
            AnchorScopeError::HashMismatch => "HASH_MISMATCH".to_string(),
            AnchorScopeError::DuplicateTrueId => "DUPLICATE_TRUE_ID".to_string(),
            AnchorScopeError::LabelExists => "LABEL_EXISTS".to_string(),
            AnchorScopeError::AmbiguousReplacement => "AMBIGUOUS_REPLACEMENT".to_string(),
            AnchorScopeError::NoReplacement => "NO_REPLACEMENT".to_string(),
            AnchorScopeError::FileNotFound => "IO_ERROR: file not found".to_string(),
            AnchorScopeError::PermissionDenied => "IO_ERROR: permission denied".to_string(),
            AnchorScopeError::InvalidUtf8 => "IO_ERROR: invalid UTF-8".to_string(),
            AnchorScopeError::ReadFailure => "IO_ERROR: read failure".to_string(),
            AnchorScopeError::WriteFailure => "IO_ERROR: write failure".to_string(),
            AnchorScopeError::BufferMetadataNotFound(_) => "IO_ERROR: buffer metadata for true_id not found".to_string(),
            AnchorScopeError::ParentBufferMetadataCorrupted(_) => "IO_ERROR: parent buffer metadata corrupted".to_string(),
            AnchorScopeError::CannotLoadSourcePath(_) => "IO_ERROR: cannot load source path".to_string(),
            AnchorScopeError::CannotSaveFileContent(_) => "IO_ERROR: cannot save file content".to_string(),
            AnchorScopeError::CannotSaveSourcePath(_) => "IO_ERROR: cannot save source path".to_string(),
            AnchorScopeError::CannotSaveScopeContent(_) => "IO_ERROR: cannot save scope content".to_string(),
            AnchorScopeError::CannotSaveBufferMetadata(_) => "IO_ERROR: cannot save buffer metadata".to_string(),
            AnchorScopeError::JsonSerializationFailed(_) => "IO_ERROR: JSON serialization failed".to_string(),
            AnchorScopeError::LabelMappingCorrupted(_) => "IO_ERROR: label mapping corrupted".to_string(),
            AnchorScopeError::CannotLoadBufferContent => "IO_ERROR: cannot load buffer content".to_string(),
            AnchorScopeError::ParentDirectoryNotFound(_) => "IO_ERROR: parent directory for true_id not found".to_string(),
            AnchorScopeError::MaximumNestingDepthExceeded(_) => "IO_ERROR: maximum nesting depth exceeded".to_string(),
            AnchorScopeError::ExternalToolFailed => "IO_ERROR: external tool failed".to_string(),
            AnchorScopeError::CannotExecuteExternalTool => "IO_ERROR: cannot execute external tool".to_string(),
            AnchorScopeError::CannotCreateTempDirectory => "IO_ERROR: cannot create temporary directory".to_string(),
            AnchorScopeError::BufferNotFound => "IO_ERROR: buffer not found".to_string(),
            AnchorScopeError::ReplacementNotFound => "IO_ERROR: replacement not found".to_string(),
            AnchorScopeError::LabelMappingNotFound(_) => "IO_ERROR: label mapping not found".to_string(),
        }
    }
}

/// Convert std::io::Error to AnchorScopeError (read version)
impl From<std::io::Error> for AnchorScopeError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => AnchorScopeError::FileNotFound,
            std::io::ErrorKind::PermissionDenied => AnchorScopeError::PermissionDenied,
            _ => AnchorScopeError::ReadFailure,
        }
    }
}

/// Convert std::io::Error to AnchorScopeError for write operations
/// NotFound is mapped to WriteFailure for backward compatibility with SPEC
pub fn from_io_error_write(err: std::io::Error) -> AnchorScopeError {
    match err.kind() {
        std::io::ErrorKind::NotFound => AnchorScopeError::WriteFailure,
        std::io::ErrorKind::PermissionDenied => AnchorScopeError::PermissionDenied,
        _ => AnchorScopeError::WriteFailure,
    }
}
