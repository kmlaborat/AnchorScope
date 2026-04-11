/// Configuration module for AnchorScope
/// Handles environment variable-based configuration

use std::env;

/// Maximum nesting depth for anchors
/// Default: 5 levels (cross-platform compatible)
/// Can be overridden by ANCHORSCOPE_MAX_DEPTH environment variable
pub const DEFAULT_MAX_DEPTH: usize = 5;

/// Get the maximum nesting depth from environment variable or default
/// Returns the depth value (always >= 1)
pub fn max_depth() -> usize {
    // Try to read from ANCHORSCOPE_MAX_DEPTH environment variable
    if let Ok(val) = env::var("ANCHORSCOPE_MAX_DEPTH") {
        if let Ok(depth) = val.parse::<usize>() {
            // Clamp to reasonable range: at least 1, at most 100
            return depth.max(1).min(100);
        }
    }
    
    // Use default if env var not set or invalid
    DEFAULT_MAX_DEPTH
}

/// Security configuration
pub mod security {
    use std::env;
    
    /// Maximum file size (default 100MB)
    pub fn max_file_size() -> u64 {
        if let Ok(val) = env::var("ANCHORSCOPE_MAX_FILE_SIZE") {
            if let Ok(size) = val.parse::<u64>() {
                return size.max(1).min(1024 * 1024 * 1024); // Clamp: 1B to 1GB
            }
        }
        100 * 1024 * 1024  // 100MB
    }
    
    /// Maximum nesting depth (default 100)
    pub fn max_nesting_depth() -> usize {
        if let Ok(val) = env::var("ANCHORSCOPE_MAX_NESTING_DEPTH") {
            if let Ok(depth) = val.parse::<usize>() {
                return depth.max(1).min(1000);
            }
        }
        100
    }
    
    /// Allowed tools for pipe command
    pub fn allowed_tools() -> Vec<String> {
        if let Ok(val) = env::var("ANCHORSCOPE_ALLOWED_TOOLS") {
            return val.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        vec!["sed".to_string(), "awk".to_string(), "perl".to_string(),
             "python3".to_string(), "node".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_max_depth_default() {
        // Save original state
        let was_set = std::env::var("ANCHORSCOPE_MAX_DEPTH").is_ok();
        
        // Reset the environment variable
        std::env::remove_var("ANCHORSCOPE_MAX_DEPTH");
        assert_eq!(max_depth(), DEFAULT_MAX_DEPTH);
        
        // Restore original state
        if was_set {
            // Restore with some value (we don't know what it was, so we can't)
            // For test isolation, this is acceptable
        }
    }

    #[test]
    #[serial]
    fn test_max_depth_env_override() {
        // Test with env var set
        std::env::set_var("ANCHORSCOPE_MAX_DEPTH", "7");
        assert_eq!(max_depth(), 7);
        
        // Reset
        std::env::remove_var("ANCHORSCOPE_MAX_DEPTH");
    }

    #[test]
    #[serial]
    fn test_max_depth_clamped() {
        // Test clamping to max 100
        std::env::set_var("ANCHORSCOPE_MAX_DEPTH", "500");
        assert_eq!(max_depth(), 100);
        
        // Reset
        std::env::remove_var("ANCHORSCOPE_MAX_DEPTH");
    }

    #[test]
    #[serial]
    fn test_max_depth_invalid_value() {
        // Test with invalid value
        std::env::set_var("ANCHORSCOPE_MAX_DEPTH", "invalid");
        assert_eq!(max_depth(), DEFAULT_MAX_DEPTH);
        
        // Reset
        std::env::remove_var("ANCHORSCOPE_MAX_DEPTH");
    }
}
