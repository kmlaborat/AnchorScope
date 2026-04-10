use anchorscope::{hash, storage};

#[test]
fn duplicate_true_id_triggers_ambiguous_anchor() {
    // Create a scenario where the same true_id exists in multiple locations
    // This should trigger AMBIGUOUS_ANCHOR error
    
    let content = b"test content";
    let file_hash = hash::compute(content);
    
    // Save file content
    storage::save_file_content(&file_hash, content).unwrap();
    
    // Create a true_id manually
    let true_id = "duplicate_true_id_test_123";
    
    // Save to two different locations
    let dir1 = anchorscope::buffer_path::true_id_dir(&file_hash, true_id);
    std::fs::create_dir_all(&dir1).unwrap();
    std::fs::write(dir1.join("content"), b"content1").unwrap();
    
    let dir2 = anchorscope::buffer_path::file_dir(&file_hash).join("another_parent").join(true_id);
    std::fs::create_dir_all(&dir2).unwrap();
    std::fs::write(dir2.join("content"), b"content2").unwrap();
    
    // Try to load buffer metadata - should detect duplicate
    let result = storage::load_buffer_metadata(&file_hash, true_id);
    
    // Should fail with AMBIGUOUS_ANCHOR error
    assert!(result.is_err(), "load_buffer_metadata should fail with duplicate true_id");
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("ambiguous") || err_msg.contains("Ambiguous") || err_msg.contains("multiple"), 
            "Error should mention ambiguity: {}", err_msg);
    
    // Cleanup
    let _ = std::fs::remove_dir_all(anchorscope::buffer_path::file_dir(&file_hash));
}

#[test]
fn ambiguous_in_multiple_file_hashes_detected() {
    // Create a scenario where the same true_id exists in multiple file_hash directories
    // This should trigger AMBIGUOUS_ANCHOR error
    
    let content1 = b"content1";
    let file_hash1 = hash::compute(content1);
    
    let content2 = b"content2";
    let file_hash2 = hash::compute(content2);
    
    // Save file contents
    storage::save_file_content(&file_hash1, content1).unwrap();
    storage::save_file_content(&file_hash2, content2).unwrap();
    
    // Use the same true_id in both file_hash directories
    let true_id = "ambiguous_test_456";
    
    let dir1 = anchorscope::buffer_path::true_id_dir(&file_hash1, true_id);
    std::fs::create_dir_all(&dir1).unwrap();
    std::fs::write(dir1.join("content"), b"content1").unwrap();
    
    let dir2 = anchorscope::buffer_path::true_id_dir(&file_hash2, true_id);
    std::fs::create_dir_all(&dir2).unwrap();
    std::fs::write(dir2.join("content"), b"content2").unwrap();
    
    // Try to load buffer metadata - should detect duplicate
    let result = storage::load_buffer_metadata(&file_hash1, true_id);
    
    // Should fail with AMBIGUOUS_ANCHOR error
    assert!(result.is_err(), "load_buffer_metadata should fail with duplicate true_id");
    
    // Cleanup
    let _ = std::fs::remove_dir_all(anchorscope::buffer_path::file_dir(&file_hash1));
    let _ = std::fs::remove_dir_all(anchorscope::buffer_path::file_dir(&file_hash2));
}
