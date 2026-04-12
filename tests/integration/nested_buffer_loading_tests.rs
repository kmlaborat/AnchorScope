use std::env;
use std::fs;
use tempfile::tempdir;

use anchor_scope::{buffer_path, storage};
use serde_json;

mod integration_test_helpers;
use integration_test_helpers::{create_temp_file, run_anchorscope};

fn parse_output(output: &str) -> std::collections::HashMap<String, String> {
    let mut result = std::collections::HashMap::new();
    for line in output.lines() {
        if let Some((key, value)) = line.split_once('=') {
            result.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    result
}

#[test]
fn test_nested_buffer_loading_flat() {
    let tmp_dir = tempdir().unwrap();
    env::set_var("TMPDIR", tmp_dir.path());

    let file_hash = "abc123def4567890";
    let true_id = "level1abc1234567890";

    storage::save_file_content(&file_hash, b"test content").unwrap();
    storage::save_buffer_content(&file_hash, true_id, b"flat content").unwrap();
    storage::save_buffer_metadata(
        &file_hash,
        true_id,
        &storage::BufferMeta {
            true_id: true_id.to_string(),
            parent_true_id: None,
            scope_hash: "scope1234567890123456".to_string(),
            anchor: "level1".to_string(),
        },
    ).unwrap();

    let result = storage::load_buffer_content(&file_hash, true_id);
    assert!(result.is_ok(), "Should load flat buffer content");
    assert_eq!(result.unwrap(), b"flat content");

    storage::invalidate_true_id_hierarchy(&file_hash, true_id).unwrap();
}

#[test]
fn test_nested_buffer_loading_deep_nesting() {
    let tmp_dir = tempdir().unwrap();
    env::set_var("TMPDIR", tmp_dir.path());

    let file_hash = "abc123def4567890";
    let level1_id = "level1abc1234567890";
    let level2_id = "level2abc1234567890";
    let level3_id = "level3abc1234567890";

    storage::save_file_content(&file_hash, b"test content").unwrap();
    storage::save_buffer_content(&file_hash, level1_id, b"level1").unwrap();
    storage::save_buffer_metadata(
        &file_hash, level1_id,
        &storage::BufferMeta {
            true_id: level1_id.to_string(),
            parent_true_id: None,
            scope_hash: "scope1234567890123456".to_string(),
            anchor: "level1".to_string(),
        },
    ).unwrap();

    let level1_dir = buffer_path::true_id_dir(&file_hash, level1_id);
    std::fs::create_dir_all(&level1_dir).unwrap();

    let level2_path = level1_dir.join(level2_id).join("content");
    std::fs::create_dir_all(level2_path.parent().unwrap()).unwrap();
    std::fs::write(&level2_path, b"level2").unwrap();

    let level2_dir = level1_dir.join(level2_id);
    let level3_path = level2_dir.join(level3_id).join("content");
    std::fs::create_dir_all(level3_path.parent().unwrap()).unwrap();
    std::fs::write(&level3_path, b"level3").unwrap();

    let result = storage::load_buffer_content(&file_hash, level3_id);
    assert!(result.is_ok(), "Should load deeply nested buffer content");
    assert_eq!(result.unwrap(), b"level3");

    let result = storage::load_buffer_content(&file_hash, level2_id);
    assert!(result.is_ok(), "Should load level2 buffer content");
    assert_eq!(result.unwrap(), b"level2");

    let result = storage::load_buffer_content(&file_hash, level1_id);
    assert!(result.is_ok(), "Should load level1 buffer content");
    assert_eq!(result.unwrap(), b"level1");

    storage::invalidate_true_id_hierarchy(&file_hash, level1_id).unwrap();
    storage::invalidate_true_id_hierarchy(&file_hash, level2_id).unwrap();
    storage::invalidate_true_id_hierarchy(&file_hash, level3_id).unwrap();
}

#[test]
fn test_nested_buffer_loading_round_trip() {
    let tmp_dir = tempdir().unwrap();
    env::set_var("TMPDIR", tmp_dir.path());

    let file_hash = "def456abc7890123";
    let parent_id = "parent1234567890123456";
    let child_id = "child12345678901234567890";

    storage::save_file_content(&file_hash, b"original").unwrap();
    storage::save_buffer_content(&file_hash, parent_id, b"parent content").unwrap();

    let parent_dir = buffer_path::true_id_dir(&file_hash, parent_id);
    std::fs::create_dir_all(&parent_dir).unwrap();

    let child_content = b"nested child content";
    let child_path = parent_dir.join(child_id).join("content");
    std::fs::create_dir_all(child_path.parent().unwrap()).unwrap();
    std::fs::write(&child_path, child_content).unwrap();

    let child_meta = storage::BufferMeta {
        true_id: child_id.to_string(),
        parent_true_id: Some(parent_id.to_string()),
        scope_hash: "child_scope_hash_here".to_string(),
        anchor: "child".to_string(),
    };
    let meta_json = serde_json::to_string_pretty(&child_meta).unwrap();
    std::fs::write(parent_dir.join(child_id).join("metadata.json"), meta_json).unwrap();

    let loaded = storage::load_buffer_content(&file_hash, child_id).unwrap();
    assert_eq!(loaded, child_content);

    storage::invalidate_true_id_hierarchy(&file_hash, parent_id).unwrap();
    storage::invalidate_true_id_hierarchy(&file_hash, child_id).unwrap();
}

#[test]
fn test_nested_buffer_loading_multiple_children() {
    let tmp_dir = tempdir().unwrap();
    env::set_var("TMPDIR", tmp_dir.path());

    let file_hash = "multi12345678901234";
    let parent_id = "parent1234567890123456";
    let child1_id = "child1abc12345678901234";
    let child2_id = "child2def4567890123456";
    let child3_id = "child3ghi7890123456789";

    storage::save_file_content(&file_hash, b"original").unwrap();
    storage::save_buffer_content(&file_hash, parent_id, b"parent").unwrap();

    let parent_dir = buffer_path::true_id_dir(&file_hash, parent_id);
    std::fs::create_dir_all(&parent_dir).unwrap();

    let child1_path = parent_dir.join(child1_id).join("content");
    std::fs::create_dir_all(child1_path.parent().unwrap()).unwrap();
    std::fs::write(&child1_path, b"child1 content").unwrap();

    let child2_path = parent_dir.join(child2_id).join("content");
    std::fs::create_dir_all(child2_path.parent().unwrap()).unwrap();
    std::fs::write(&child2_path, b"child2 content").unwrap();

    let child3_path = parent_dir.join(child3_id).join("content");
    std::fs::create_dir_all(child3_path.parent().unwrap()).unwrap();
    std::fs::write(&child3_path, b"child3 content").unwrap();

    assert_eq!(
        storage::load_buffer_content(&file_hash, child1_id).unwrap(),
        b"child1 content"
    );
    assert_eq!(
        storage::load_buffer_content(&file_hash, child2_id).unwrap(),
        b"child2 content"
    );
    assert_eq!(
        storage::load_buffer_content(&file_hash, child3_id).unwrap(),
        b"child3 content"
    );

    assert_eq!(
        storage::load_buffer_content(&file_hash, parent_id).unwrap(),
        b"parent"
    );

    storage::invalidate_true_id_hierarchy(&file_hash, parent_id).unwrap();
    storage::invalidate_true_id_hierarchy(&file_hash, child1_id).unwrap();
    storage::invalidate_true_id_hierarchy(&file_hash, child2_id).unwrap();
    storage::invalidate_true_id_hierarchy(&file_hash, child3_id).unwrap();
}

#[test]
fn test_full_read_write_workflow_with_nested_buffers() {
    let content = "def foo():\n    pass\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    let anchor = "def foo()";

    // 1. read to create buffer and get true_id
    let read_out = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor,
    ]);
    assert!(
        read_out.status.success(),
        "read command failed: {}",
        String::from_utf8_lossy(&read_out.stderr)
    );

    let stdout = String::from_utf8(read_out.stdout).expect("output is not valid UTF-8");
    let result: std::collections::HashMap<String, String> = parse_output(&stdout);
    let true_id = result
        .get("true_id")
        .expect("true_id should be present")
        .clone();

    // 2. create human-readable label
    let label_out = run_anchorscope(&["label", "--name", "my_function", "--true-id", &true_id]);
    assert!(
        label_out.status.success(),
        "label command failed: {}",
        String::from_utf8_lossy(&label_out.stderr)
    );

    // 3. read again using label to create nested buffer
    let read2_out = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "pass",
        "--label",
        "my_function",
    ]);

    assert!(
        read2_out.status.success(),
        "read with label should succeed: {}",
        String::from_utf8_lossy(&read2_out.stderr)
    );

    // Should produce output with a new true_id
    let stdout2 = String::from_utf8(read2_out.stdout).expect("output is not valid UTF-8");
    let result2: std::collections::HashMap<String, String> = parse_output(&stdout2);
    let nested_true_id = result2
        .get("true_id")
        .expect("true_id should be present in nested read")
        .clone();

    // Verify the nested True ID is different from parent
    assert_ne!(true_id, nested_true_id, "Nested True ID should be different");

    // 4. verify nested buffer was created
    let parent_dir = buffer_path::true_id_dir(
        result.get("label").expect("label should be present").clone(),
        &true_id
    );
    let nested_content_path = parent_dir.join(&nested_true_id).join("content");
    assert!(
        nested_content_path.exists(),
        "Nested buffer content should exist at {}",
        nested_content_path.display()
    );

    // 5. write using nested True ID
    let write_out = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--true-id",
        &nested_true_id,
        "--expected-hash",
        result2.get("hash").expect("hash should be present").clone(),
        "--replacement",
        "    return 42\n",
    ]);

    assert!(
        write_out.status.success(),
        "write should succeed: {}",
        String::from_utf8_lossy(&write_out.stderr)
    );

    // 6. verify file was modified correctly
    let modified_content = fs::read_to_string(&file_path).unwrap();
    assert!(
        modified_content.contains("return 42"),
        "File should contain 'return 42' after write"
    );
}
