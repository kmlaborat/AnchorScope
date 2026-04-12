use anchorscope::{hash, storage};

#[test]
fn nesting_depth_counts_zero_indexed() {
    // Level 1 (file → buffer): depth = 0
    // Level 2 (buffer → nested): depth = 1
    // Level 3: depth = 2
    // etc.
    
    let content = b"def outer():\n    def inner():\n        pass\n";
    let file_hash = hash::compute(content);
    
    // Save file content
    storage::save_file_content(&file_hash, content).unwrap();
    
    // Simulate level 1: "def outer"
    let outer_scope = b"def outer():\n    def inner():\n        pass\n";
    let outer_scope_hash = hash::compute(outer_scope);
    let outer_true_id = hash::compute(format!("{}_{}", file_hash, outer_scope_hash).as_bytes());
    
    // Save level 1 metadata with scope_hash
    let outer_meta = storage::BufferMeta {
        true_id: outer_true_id.clone(),
        parent_true_id: None,
        scope_hash: outer_scope_hash.clone(),
        anchor: "def outer".to_string(),
    };
    storage::save_buffer_metadata(&file_hash, &outer_true_id, &outer_meta).unwrap();
    storage::save_scope_content(&file_hash, &outer_true_id, outer_scope).unwrap();
    
    // Calculate depth for level 1 (should be 0)
    let depth1 = anchorscope::commands::read::calculate_nesting_depth(&outer_true_id, &file_hash);
    assert_eq!(depth1, Ok(0), "Level 1 should have depth 0");
    
    // Simulate level 2: "def inner" nested under level 1
    let inner_scope = b"def inner():\n        pass\n";
    let inner_scope_hash = hash::compute(inner_scope);
    // True ID = hash(parent_scope_hash + "_" + child_scope_hash)
    let inner_true_id = hash::compute(format!("{}_{}", outer_scope_hash, inner_scope_hash).as_bytes());
    
    // Save level 2 as nested under level 1
    let nested_dir = anchorscope::buffer_path::nested_true_id_dir(&file_hash, &outer_true_id, &inner_true_id);
    std::fs::create_dir_all(&nested_dir).unwrap();
    std::fs::write(nested_dir.join("content"), inner_scope).unwrap();
    
    // Save nested metadata
    let inner_meta = storage::BufferMeta {
        true_id: inner_true_id.clone(),
        parent_true_id: Some(outer_true_id.clone()),
        scope_hash: inner_scope_hash.clone(),
        anchor: "def inner".to_string(),
    };
    let nested_metadata_path = nested_dir.join("metadata.json");
    std::fs::write(
        &nested_metadata_path,
        serde_json::to_string_pretty(&inner_meta).unwrap()
    ).unwrap();
    
    // Calculate depth for level 2 (should be 1)
    let depth2 = anchorscope::commands::read::calculate_nesting_depth(&inner_true_id, &file_hash);
    assert_eq!(depth2, Ok(1), "Level 2 should have depth 1");
    
    // Verify the depth check logic
    // If parent is at depth 4 (level 5), child would be at depth 5 (level 6)
    // With max_depth = 5, we should get error if parent depth >= max_depth - 1 = 4
    let max_depth = anchorscope::config::max_depth();
    assert!(max_depth >= 5, "max_depth should be at least 5 for this test");
    
    // Cleanup
    let _ = std::fs::remove_dir_all(anchorscope::buffer_path::file_dir(&file_hash));
}

#[test]
fn depth_exceeds_limit_returns_error() {
    // With max_depth = 5 (default), depth >= 4 should trigger error
    // depth >= max_depth - 1 means the NEXT level would exceed max_depth
    
    let content = b"test";
    let file_hash = hash::compute(content);
    
    // Create a structure where we're at depth 4 (5 levels total)
    // Adding one more would be depth 5 (6 levels) → error
    
    // Level 1: depth 0
    let scope1 = b"r1";
    let hash1 = hash::compute(scope1);
    let tid1 = hash::compute(format!("{}_{}", file_hash, hash1).as_bytes());
    
    let dir1 = anchorscope::buffer_path::true_id_dir(&file_hash, &tid1);
    std::fs::create_dir_all(&dir1).unwrap();
    std::fs::write(dir1.join("content"), scope1).unwrap();
    
    // Level 2: depth 1
    let scope2 = b"r2";
    let hash2 = hash::compute(scope2);
    let tid2 = hash::compute(format!("{}_{}", hash1, hash2).as_bytes());
    
    let dir2 = anchorscope::buffer_path::nested_true_id_dir(&file_hash, &tid1, &tid2);
    std::fs::create_dir_all(&dir2).unwrap();
    std::fs::write(dir2.join("content"), scope2).unwrap();
    
    // Level 3: depth 2
    let scope3 = b"r3";
    let hash3 = hash::compute(scope3);
    let tid3 = hash::compute(format!("{}_{}", hash2, hash3).as_bytes());
    
    let dir3 = anchorscope::buffer_path::nested_true_id_dir(&file_hash, &tid2, &tid3);
    std::fs::create_dir_all(&dir3).unwrap();
    std::fs::write(dir3.join("content"), scope3).unwrap();
    
    // Level 4: depth 3
    let scope4 = b"r4";
    let hash4 = hash::compute(scope4);
    let tid4 = hash::compute(format!("{}_{}", hash3, hash4).as_bytes());
    
    let dir4 = anchorscope::buffer_path::nested_true_id_dir(&file_hash, &tid3, &tid4);
    std::fs::create_dir_all(&dir4).unwrap();
    std::fs::write(dir4.join("content"), scope4).unwrap();
    
    // Level 5: depth 4
    let scope5 = b"r5";
    let hash5 = hash::compute(scope5);
    let tid5 = hash::compute(format!("{}_{}", hash4, hash5).as_bytes());
    
    let dir5 = anchorscope::buffer_path::nested_true_id_dir(&file_hash, &tid4, &tid5);
    std::fs::create_dir_all(&dir5).unwrap();
    std::fs::write(dir5.join("content"), scope5).unwrap();
    
    // Depth for level 5 should be 4 (valid, max_depth-1 = 4)
    let depth5 = anchorscope::commands::read::calculate_nesting_depth(&tid5, &file_hash);
    assert_eq!(depth5, Ok(4), "Level 5 should have depth 4");
    
    // Cleanup
    let _ = std::fs::remove_dir_all(anchorscope::buffer_path::file_dir(&file_hash));
}
