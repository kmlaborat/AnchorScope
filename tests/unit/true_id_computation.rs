use anchorscope::{hash, storage};

#[test]
fn true_id_never_uses_parent_tid_as_parent_hash() {
    // Prepare a temporary file content with outer anchor
    let content = b"12345";
    let file_hash = hash::compute(content);
    
    // Save file content
    storage::save_file_content(&file_hash, content).unwrap();
    
    // Simulate outer anchor scope "234"
    let outer_scope = b"234";
    let outer_scope_hash = hash::compute(outer_scope);
    let outer_true_id = hash::compute(format!("{}_{}", file_hash, outer_scope_hash).as_bytes());
    
    // Save outer buffer metadata
    let outer_meta = storage::BufferMeta {
        true_id: outer_true_id.clone(),
        parent_true_id: None,
        scope_hash: outer_scope_hash.clone(),
        anchor: "234".to_string(),
    };
    storage::save_buffer_metadata(&file_hash, &outer_true_id, &outer_meta).unwrap();
    storage::save_scope_content(&file_hash, &outer_true_id, outer_scope).unwrap();
    
    // Save label mapping and source path
    storage::save_label_mapping("test_label", &outer_true_id).unwrap();
    
    // Create a temporary real file for source path
    let tmp_file_path = std::env::temp_dir().join("tmp_anchor_file.txt");
    std::fs::write(&tmp_file_path, content).expect("write tmp file");
    storage::save_source_path(&file_hash, tmp_file_path.to_str().unwrap()).unwrap();
    
    // Execute read in label mode with inner anchor
    // Inner anchor "3" is inside "234"
    let exit_code = anchorscope::commands::read::execute(
        "tmp_path",
        Some("3"),
        None,
        Some("test_label")
    );
    
    assert_eq!(exit_code, 0, "read should succeed with valid metadata");
    
    // Verify inner true_id was computed correctly
    // inner_scope_hash = hash("3")
    // expected_true_id = hash(outer_scope_hash + "_" + inner_scope_hash)
    let inner_scope_hash = hash::compute(b"3");
    let expected_true_id = hash::compute(format!("{}_{}", outer_scope_hash, inner_scope_hash).as_bytes());
    
    // Check that the inner true_id exists in the nested directory
    let file_dir = anchorscope::buffer_path::file_dir(&file_hash);
    let nested_dir = file_dir.join(&outer_true_id).join(&expected_true_id);
    
    assert!(nested_dir.join("content").exists(), "nested directory should exist");
    
    // Verify the metadata was stored correctly
    let nested_meta = storage::load_buffer_metadata(&file_hash, &expected_true_id).expect("nested metadata not found");
    assert_eq!(nested_meta.parent_true_id.as_deref(), Some(outer_true_id.as_str()));
    assert_eq!(nested_meta.scope_hash, inner_scope_hash);
    
    // Cleanup
    storage::invalidate_true_id_hierarchy(&file_hash, &outer_true_id).unwrap();
    storage::invalidate_label("test_label");
    let _ = std::fs::remove_file(tmp_file_path);
}

#[test]
fn true_id_fails_when_parent_metadata_missing() {
    // Prepare a temporary file content
    let content = b"12345";
    let file_hash = hash::compute(content);
    
    // Save file content
    storage::save_file_content(&file_hash, content).unwrap();
    
    // Simulate outer anchor scope "234" but DO NOT save metadata
    let outer_scope = b"234";
    let outer_scope_hash = hash::compute(outer_scope);
    let outer_true_id = hash::compute(format!("{}_{}", file_hash, outer_scope_hash).as_bytes());
    
    // Save scope content but NOT metadata (to simulate corruption)
    storage::save_scope_content(&file_hash, &outer_true_id, outer_scope).unwrap();
    
    // Save label mapping pointing to outer_true_id
    storage::save_label_mapping("test_label_missing_meta", &outer_true_id).unwrap();
    
    // Create a temporary real file for source path
    let tmp_file_path = std::env::temp_dir().join("tmp_anchor_file2.txt");
    std::fs::write(&tmp_file_path, content).expect("write tmp file");
    storage::save_source_path(&file_hash, tmp_file_path.to_str().unwrap()).unwrap();
    
    // Execute read in label mode - should fail because parent metadata is missing
    let exit_code = anchorscope::commands::read::execute(
        "tmp_path",
        Some("3"),
        None,
        Some("test_label_missing_meta")
    );
    
    // Should fail with IO_ERROR
    assert_ne!(exit_code, 0, "read should fail when parent metadata is missing");
    
    // Cleanup
    let _ = std::fs::remove_file(tmp_file_path);
}
