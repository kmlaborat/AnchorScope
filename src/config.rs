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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
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
    fn test_max_depth_env_override() {
        // Test with env var set
        std::env::set_var("ANCHORSCOPE_MAX_DEPTH", "7");
        assert_eq!(max_depth(), 7);
        
        // Reset
        std::env::remove_var("ANCHORSCOPE_MAX_DEPTH");
    }

    #[test]
    fn test_max_depth_clamped() {
        // Test clamping to max 100
        std::env::set_var("ANCHORSCOPE_MAX_DEPTH", "500");
        assert_eq!(max_depth(), 100);
        
        // Reset
        std::env::remove_var("ANCHORSCOPE_MAX_DEPTH");
    }

    #[test]
    fn test_max_depth_invalid_value() {
        // Test with invalid value
        std::env::set_var("ANCHORSCOPE_MAX_DEPTH", "invalid");
        assert_eq!(max_depth(), DEFAULT_MAX_DEPTH);
        
        // Reset
        std::env::remove_var("ANCHORSCOPE_MAX_DEPTH");
    }
}
