use std::path::{Path, PathBuf};

/// Maximum path length to prevent buffer overflow and resource exhaustion
pub const MAX_PATH_LENGTH: usize = 2048;

/// Maximum file size (100MB default)
pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// List of allowed tools for pipe command
pub const ALLOWED_TOOLS: &[&str] = &["sed", "awk", "perl", "python3", "node"];

/// Validate that a path is within allowed boundaries
pub fn validate_path_safety(path: &Path, allowed_base: &Path) -> Result<(), crate::error::AnchorScopeError> {
    // Check path length
    if path.as_os_str().len() > MAX_PATH_LENGTH {
        return Err(crate::error::AnchorScopeError::PermissionDenied);
    }
    
    // Get canonicalized allowed base
    let canonicalized_allowed = std::fs::canonicalize(allowed_base)
        .map_err(|_| crate::error::AnchorScopeError::FileNotFound)?;
    
    // Get canonicalized resolved path
    let resolved = std::fs::canonicalize(path)
        .map_err(|_| crate::error::AnchorScopeError::FileNotFound)?;
    
    // Verify resolved path is within allowed base
    if !resolved.starts_with(&canonicalized_allowed) {
        return Err(crate::error::AnchorScopeError::PermissionDenied);
    }
    
    Ok(())
}

/// Check if a path contains symbolic links
pub fn ensure_no_symlinks(path: &Path) -> Result<(), crate::error::AnchorScopeError> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| crate::error::AnchorScopeError::FileNotFound)?;
    
    if metadata.file_type().is_symlink() {
        return Err(crate::error::AnchorScopeError::PermissionDenied);
    }
    
    Ok(())
}

/// Validate file size
pub fn validate_file_size(path: &Path) -> Result<(), crate::error::AnchorScopeError> {
    let metadata = std::fs::metadata(path)
        .map_err(|_| crate::error::AnchorScopeError::FileNotFound)?;
    
    if metadata.len() > MAX_FILE_SIZE {
        return Err(crate::error::AnchorScopeError::PermissionDenied);
    }
    
    Ok(())
}

/// Validate tool name to prevent command injection
pub fn validate_tool_name(tool: &str) -> Result<(), crate::error::AnchorScopeError> {
    // Check for path separators (prevent absolute/relative paths)
    if tool.contains('/') || tool.contains('\\') {
        return Err(crate::error::AnchorScopeError::PermissionDenied);
    }
    
    // Check for shell metacharacters
    let dangerous_chars = [';', '|', '&', '$', '`', '\n', '\r', '\t'];
    for c in dangerous_chars {
        if tool.contains(c) {
            return Err(crate::error::AnchorScopeError::PermissionDenied);
        }
    }
    
    // Check against whitelist
    if !ALLOWED_TOOLS.contains(&tool) {
        return Err(crate::error::AnchorScopeError::PermissionDenied);
    }
    
    Ok(())
}

/// Validate file path for security
/// Returns the resolved path if valid
/// Prevents path traversal attacks using .. components
pub fn validate_file_path(
    path: &str,
    working_dir: &Path,
) -> Result<PathBuf, crate::error::AnchorScopeError> {
    let path = Path::new(path);
    
    // Validate path length
    if path.as_os_str().len() > MAX_PATH_LENGTH {
        return Err(crate::error::AnchorScopeError::PermissionDenied);
    }
    
    // Check for path traversal attempts in the path string itself
    // This is the primary security check
    let path_str = path.to_string_lossy();
    if path_str.contains("..") {
        return Err(crate::error::AnchorScopeError::PermissionDenied);
    }
    
    // Resolve relative to working directory
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        working_dir.join(path)
    };
    
    // Check for symlinks - only if file exists
    if resolved.exists() {
        ensure_no_symlinks(&resolved)?;
    }
    
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_safety_allows_valid_path() {
        let temp_dir = std::env::temp_dir();
        let valid_path = temp_dir.join("test_file.txt");
        std::fs::write(&valid_path, "test").unwrap();
        
        assert!(validate_path_safety(&valid_path, &temp_dir).is_ok());
    }

    #[test]
    fn test_validate_path_safety_blocks_path_traversal() {
        let temp_dir = std::env::temp_dir();
        let malicious_path = temp_dir.join("allowed/../etc/passwd");
        assert!(validate_path_safety(&malicious_path, &temp_dir).is_err());
    }

    #[test]
    fn test_ensure_no_symlinks_allows_regular_file() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_regular.txt");
        std::fs::write(&test_file, "test").unwrap();
        assert!(ensure_no_symlinks(&test_file).is_ok());
    }

    #[test]
    fn test_ensure_no_symlinks_blocks_symlink() {
        let temp_dir = std::env::temp_dir();
        let target = temp_dir.join("target.txt");
        let link = temp_dir.join("symlink.txt");
        
        std::fs::write(&target, "test").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &link).unwrap();
        
        if cfg!(unix) {
            assert!(ensure_no_symlinks(&link).is_err());
        }
    }

    #[test]
    fn test_validate_file_size_allows_small_file() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("small_file.txt");
        std::fs::write(&test_file, "small content").unwrap();
        assert!(validate_file_size(&test_file).is_ok());
    }

    #[test]
    fn test_validate_tool_name_allows_whitelisted_tool() {
        assert!(validate_tool_name("sed").is_ok());
        assert!(validate_tool_name("awk").is_ok());
        assert!(validate_tool_name("perl").is_ok());
    }

    #[test]
    fn test_validate_tool_name_blocks_path() {
        assert!(validate_tool_name("/bin/sh").is_err());
        assert!(validate_tool_name("../tmp/malicious").is_err());
    }

    #[test]
    fn test_validate_tool_name_blocks_injection() {
        assert!(validate_tool_name("sed;rm -rf /").is_err());
        assert!(validate_tool_name("tool|cat /etc/passwd").is_err());
        assert!(validate_tool_name("tool$()").is_err());
    }
}
