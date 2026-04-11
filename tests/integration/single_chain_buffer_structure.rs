use crate::test_helpers::*;

#[test]
fn nested_read_creates_single_hierarchical_chain() {
    // Create a file with nested anchors where inner anchor is in the matched region of outer
    // Using Python-like syntax where the inner function is inside the outer function body
    let content = "def outer():\n    x = 1\n    def inner():\n        pass\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    
    // Level 1: read outer anchor "def outer"
    let out1 = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(),
        "--anchor", "def outer"
    ]);
    assert!(out1.status.success(), "level 1 read failed: {}", String::from_utf8_lossy(&out1.stderr));
    let res1 = parse_output(&String::from_utf8_lossy(&out1.stdout));
    let level1_true_id = res1.get("true_id").unwrap().clone();
    
    // Level 2: read inner anchor using label pointing to level 1
    // The buffer content for level 1 is "def outer:\n    x = 1\n    def inner:\n        pass"
    // which contains "def inner"
    let label_name = format!("outer_anchor_{}", level1_true_id); // Unique label name
    let label_out = run_anchorscope(&[
        "label", "--name", &label_name, "--true-id", &level1_true_id
    ]);
    assert!(label_out.status.success(), "label failed: {}", String::from_utf8_lossy(&label_out.stderr));
    
    let out2 = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(),
        "--anchor", "def inner",
        "--label", &label_name
    ]);
    assert!(out2.status.success(), "level 2 read failed: {}", String::from_utf8_lossy(&out2.stderr));
    let res2 = parse_output(&String::from_utf8_lossy(&out2.stdout));
    let level2_true_id = res2.get("true_id").unwrap().clone();
    
    // Verify directory structure:
    // {file_hash}/{level1_true_id}/content (level 1 buffer)
    // {file_hash}/{level1_true_id}/{level2_true_id}/content (level 2 buffer)
    // There should NOT be a flat {file_hash}/{level2_true_id}/ directory
    
    let anchorscope_dir = std::env::temp_dir().join("anchorscope");
    let file_hash = {
        let mut found = None;
        if let Ok(entries) = std::fs::read_dir(&anchorscope_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let file_hash = entry.file_name();
                    let file_hash_str = file_hash.to_string_lossy();
                    let file_hash_str = file_hash_str.as_ref();
                    
                    // Check if {file_hash}/{true_id}/content exists
                    let content_path = anchorscope_dir.join(file_hash_str)
                        .join(&level1_true_id)
                        .join("content");
                    if content_path.exists() {
                        found = Some(file_hash_str.to_string());
                        break;
                    }
                    
                    // Check nested: {file_hash}/{parent_true_id}/{true_id}/content
                    let file_dir = anchorscope_dir.join(file_hash_str);
                    if let Ok(file_dir_entries) = std::fs::read_dir(&file_dir) {
                        for parent_entry in file_dir_entries.flatten() {
                            if parent_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                                let parent_true_id = parent_entry.file_name();
                                let parent_true_id_str = parent_true_id.to_string_lossy();
                                let nested_content_path = anchorscope_dir.join(file_hash_str)
                                    .join(parent_true_id_str.as_ref())
                                    .join(&level1_true_id)
                                    .join("content");
                                
                                if nested_content_path.exists() {
                                    found = Some(file_hash_str.to_string());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        found.unwrap_or_else(|| panic!("file_hash not found for true_id {}", level1_true_id))
    };
    
    // Level 2 should exist ONLY as nested under level 1 (the fix)
    let flat_level2_dir = anchorscope_dir.join(&file_hash).join(&level2_true_id);
    let nested_level2_dir = anchorscope_dir.join(&file_hash).join(&level1_true_id).join(&level2_true_id);
    
    // Level 1 should exist
    assert!(nested_level2_dir.join("..").join("content").exists(), 
            "level 1 buffer should exist");
    
    // Level 2 should exist ONLY as nested under level 1
    assert!(nested_level2_dir.join("content").exists(), "level 2 nested buffer should exist");
    assert!(nested_level2_dir.join("metadata.json").exists(), "level 2 nested metadata should exist");
    
    // Level 2 should NOT exist as flat directory (this is what we're testing for)
    assert!(!flat_level2_dir.exists(), "level 2 should NOT exist as flat directory (orphan prevention)");
    
    // Cleanup
    let level1_full = anchorscope_dir.join(&file_hash).join(&level1_true_id);
    let _ = std::fs::remove_dir_all(&level1_full);
}

#[test]
fn three_level_nesting_write_cleans_up_correctly() {
    let content = "def a():\n    def b():\n        def c():\n            pass\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    
    // Level 1: read "def a"
    let out1 = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(),
        "--anchor", "def a"
    ]);
    assert!(out1.status.success());
    let res1 = parse_output(&String::from_utf8_lossy(&out1.stdout));
    let level1_true_id = res1.get("true_id").unwrap().clone();
    
    // Label level 1
    let label_name1 = format!("level1_{}", level1_true_id);
    let label_out1 = run_anchorscope(&[
        "label", "--name", &label_name1, "--true-id", &level1_true_id
    ]);
    assert!(label_out1.status.success(), "label1 failed: {}", String::from_utf8_lossy(&label_out1.stderr));
    
    // Level 2: read "def b" using level1 label
    let out2 = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(),
        "--anchor", "def b",
        "--label", &label_name1
    ]);
    assert!(out2.status.success());
    let res2 = parse_output(&String::from_utf8_lossy(&out2.stdout));
    let level2_true_id = res2.get("true_id").unwrap().clone();
    
    // Label level 2
    let label_name2 = format!("level2_{}", level2_true_id);
    let label_out2 = run_anchorscope(&[
        "label", "--name", &label_name2, "--true-id", &level2_true_id
    ]);
    assert!(label_out2.status.success(), "label2 failed: {}", String::from_utf8_lossy(&label_out2.stderr));
    
    // Level 3: read "def c" using level2 label
    let out3 = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(),
        "--anchor", "def c",
        "--label", &label_name2
    ]);
    assert!(out3.status.success());
    let res3 = parse_output(&String::from_utf8_lossy(&out3.stdout));
    let level3_true_id = res3.get("true_id").unwrap().clone();
    
    // Label level 3
    let label_name3 = format!("level3_{}", level3_true_id);
    let label_out3 = run_anchorscope(&[
        "label", "--name", &label_name3, "--true-id", &level3_true_id
    ]);
    assert!(label_out3.status.success(), "label3 failed: {}", String::from_utf8_lossy(&label_out3.stderr));
    
    // Get file_hash
    let anchorscope_dir = std::env::temp_dir().join("anchorscope");
    let file_hash = {
        let mut found = None;
        if let Ok(entries) = std::fs::read_dir(&anchorscope_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let file_hash = entry.file_name();
                    let file_hash_str = file_hash.to_string_lossy();
                    let file_hash_str = file_hash_str.as_ref();
                    let level1_path = anchorscope_dir.join(file_hash_str)
                        .join(&level1_true_id)
                        .join("content");
                    if level1_path.exists() {
                        found = Some(file_hash_str.to_string());
                        break;
                    }
                }
            }
        }
        found.unwrap_or_else(|| panic!("file_hash not found for true_id {}", level1_true_id))
    };
    
    // Verify 3-level structure exists:
    let level1_path = anchorscope_dir.join(&file_hash).join(&level1_true_id);
    let level2_path = anchorscope_dir.join(&file_hash).join(&level1_true_id).join(&level2_true_id);
    let level3_path = anchorscope_dir.join(&file_hash).join(&level1_true_id).join(&level2_true_id).join(&level3_true_id);
    
    assert!(level1_path.join("content").exists(), "level 1 should exist");
    assert!(level2_path.join("content").exists(), "level 2 should exist as nested");
    assert!(level3_path.join("content").exists(), "level 3 should exist as nested");
    
    // Write at level 3 - should clean up level 3 and any children
    let write_out = run_anchorscope(&[
        "write", "--file", file_path.to_str().unwrap(),
        "--label", &label_name3,
        "--replacement", "def c():\n        print('c modified')\n"
    ]);
    assert!(write_out.status.success(), "write failed: {}", String::from_utf8_lossy(&write_out.stderr));
    
    // Level 3 should be deleted
    assert!(!level3_path.exists(), "level 3 should be deleted after write");
    
    // Level 1 and 2 should still exist (they weren't the target of the write)
    assert!(level1_path.join("content").exists(), "level 1 should still exist");
    assert!(level2_path.join("content").exists(), "level 2 should still exist");
    
    // Cleanup
    let level1_full = anchorscope_dir.join(&file_hash).join(&level1_true_id);
    let _ = std::fs::remove_dir_all(&level1_full);
}
