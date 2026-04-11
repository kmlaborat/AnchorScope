use std::path::PathBuf;
use crate::storage;
use crate::buffer_path;

/// Result of paths command.
#[derive(Debug, Clone)]
pub struct PathsResult {
    pub content_path: PathBuf,
    pub replacement_path: PathBuf,
}

/// Resolve label to true_id and call execute_for_true_id.
pub fn execute_for_label(label: &str) -> Result<PathsResult, String> {
    let true_id = storage::load_label_target(label)?;
    execute_for_true_id(&true_id)
}

/// Return content and replacement paths for a True ID.
pub fn execute_for_true_id(true_id: &str) -> Result<PathsResult, String> {
    // Find the file_hash containing this true_id
    let file_hash = match storage::file_hash_for_true_id(true_id) {
        Ok(h) => h,
        Err(ref msg) if msg.starts_with("DUPLICATE_TRUE_ID") => {
            return Err("DUPLICATE_TRUE_ID".to_string());
        }
        Err(e) => return Err(e),
    };
    
    // Build paths
    let content_path = buffer_path::true_id_dir(&file_hash, true_id).join("content");
    let replacement_path = buffer_path::true_id_dir(&file_hash, true_id).join("replacement");
    
    // Verify content file exists
    if !content_path.exists() {
        return Err("IO_ERROR: file not found".to_string());
    }
    
    Ok(PathsResult {
        content_path,
        replacement_path,
    })
}

/// Entry point for paths command.
pub fn execute(label: &Option<String>, true_id: Option<&str>) -> i32 {
    let result = match (label, true_id) {
        (Some(l), None) => execute_for_label(l),
        (None, Some(tid)) => execute_for_true_id(tid),
        (Some(_), Some(_)) => {
            eprintln!("AMBIGUOUS_ANCHOR");
            return 1;
        }
        (None, None) => {
            eprintln!("IO_ERROR: either --label or --true-id must be provided");
            return 1;
        }
    };
    
    match result {
        Ok(paths) => {
            println!("content:     {}", paths.content_path.display());
            println!("replacement: {}", paths.replacement_path.display());
            0
        }
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hash, storage};

    #[test]
    fn paths_returns_content_and_replacement_paths() {
        // Setup: Create a buffer structure
        let content = b"test content";
        let file_hash = hash::compute(content);
        let true_id = "test_true_id_123";
        
        // Save buffer content
        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_buffer_content(&file_hash, &true_id, content).unwrap();
        storage::save_buffer_metadata(&file_hash, &true_id, &storage::BufferMeta {
            true_id: true_id.to_string(),
            parent_true_id: None,
            region_hash: hash::compute(content),
            anchor: "test".to_string(),
        }).unwrap();
        storage::save_source_path(&file_hash, "/tmp/test.txt").unwrap();
        
        // Execute paths command
        let result = execute_for_true_id(&true_id).unwrap();
        
        // Verify content path
        let expected_content_path = buffer_path::true_id_dir(&file_hash, &true_id).join("content");
        assert_eq!(result.content_path, expected_content_path);
        
        // Verify replacement path (may not exist)
        let expected_replacement_path = buffer_path::true_id_dir(&file_hash, &true_id).join("replacement");
        assert_eq!(result.replacement_path, expected_replacement_path);
        
        // Cleanup
        storage::invalidate_true_id_hierarchy(&file_hash, &true_id).unwrap();
    }

    #[test]
    fn paths_resolves_label_to_true_id() {
        // Setup: Create label mapping
        let content = b"test content";
        let file_hash = hash::compute(content);
        let true_id = "test_true_id_456";
        
        storage::save_label_mapping("my_function", &true_id).unwrap();
        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_buffer_content(&file_hash, &true_id, content).unwrap();
        storage::save_source_path(&file_hash, "/tmp/test.txt").unwrap();
        
        // Execute paths command with label
        let result = execute_for_label("my_function");
        
        // Should resolve to same true_id
        let expected_content_path = buffer_path::true_id_dir(&file_hash, &true_id).join("content");
        assert_eq!(result.unwrap().content_path, expected_content_path);
        
        // Cleanup
        storage::invalidate_label("my_function");
        storage::invalidate_true_id_hierarchy(&file_hash, &true_id).unwrap();
    }

    #[test]
    fn paths_error_when_true_id_not_found() {
        let result = execute_for_true_id("nonexistent_true_id");
        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("IO_ERROR:"));
    }

    #[test]
    fn paths_error_when_label_not_found() {
        let result = execute_for_label("nonexistent_label");
        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("IO_ERROR:"));
    }
}
