use crate::test_helpers::*;

// Import helper functions directly since test crates can't access the main crate
fn normalize_line_endings(raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'\r' && i + 1 < raw.len() && raw[i + 1] == b'\n' {
            i += 1;
        } else {
            out.push(raw[i]);
            i += 1;
        }
    }
    out
}

fn compute_hash(bytes: &[u8]) -> String {
    use xxhash_rust::xxh3::xxh3_64;
    let h = xxh3_64(bytes);
    format!("{:016x}", h)
}

/// Tests for nested buffer support (multi-level anchoring)

#[test]
fn test_nested_buffer_structure_created_on_read() {
    // SPEC §4.3: read on original file creates {file_hash}/content, {file_hash}/{true_id}/content
    let (_temp_dir, file_path) = create_temp_file("Hello\nWorld\n");

    let output = run_anchorscope(&[
        "read",
        "--file", file_path.to_str().unwrap(),
        "--anchor", "Hello"
    ]);
    assert!(output.status.success());

    // Get file hash from output
    let result = parse_output(&String::from_utf8_lossy(&output.stdout));
    let true_id = result.get("true_id").unwrap().clone();
    
    // Compute expected file_hash
    let file_raw = std::fs::read(&file_path).unwrap();
    let normalized = normalize_line_endings(&file_raw);
    let file_hash = compute_hash(&normalized);

    let temp_dir = std::env::temp_dir().join("anchorscope");
    let file_dir = temp_dir.join(&file_hash);

    // Verify root content exists
    let root_content_path = file_dir.join("content");
    assert!(root_content_path.exists(), "Root buffer content should exist");

    // Verify True ID content exists
    let true_id_dir = file_dir.join(&true_id);
    let true_id_content_path = true_id_dir.join("content");
    assert!(true_id_content_path.exists(), "True ID buffer content should exist");
}

#[test]
fn test_label_mapping_stored() {
    // Verify label creates proper JSON mapping
    let (_temp_dir, file_path) = create_temp_file("Hello\nWorld\n");

    let output = run_anchorscope(&[
        "read",
        "--file", file_path.to_str().unwrap(),
        "--anchor", "Hello"
    ]);
    assert!(output.status.success());

    let result = parse_output(&String::from_utf8_lossy(&output.stdout));
    let true_id = result.get("true_id").unwrap().clone();

    let label_out = run_anchorscope(&[
        "label",
        "--name", "test_label",
        "--true-id", &true_id
    ]);
    assert!(label_out.status.success());

    // Verify label file exists
    let temp_dir = std::env::temp_dir().join("anchorscope");
    let label_file = temp_dir.join("labels").join("test_label.json");
    assert!(label_file.exists(), "Label file should exist");

    // Verify label content
    let label_content = std::fs::read_to_string(&label_file).unwrap();
    assert!(label_content.contains(&true_id), "Label should map to correct true_id");
}

#[test]
fn test_tree_shows_buffer_structure() {
    // SPEC §6.5: tree command displays buffer structure
    let (_temp_dir, file_path) = create_temp_file("Hello\nWorld\n");

    // Create buffer entries first
    let read_out = run_anchorscope(&[
        "read",
        "--file", file_path.to_str().unwrap(),
        "--anchor", "Hello"
    ]);
    assert!(read_out.status.success());

    let result = parse_output(&String::from_utf8_lossy(&read_out.stdout));
    let true_id = result.get("true_id").unwrap().clone();

    // Use a unique label name to avoid collision
    let label_out = run_anchorscope(&[
        "label",
        "--name", "tree_test",
        "--true-id", &true_id
    ]);
    assert!(label_out.status.success(), "label command failed: {}", String::from_utf8_lossy(&label_out.stderr));

    // Run tree command
    let tree_out = run_anchorscope(&[
        "tree",
        "--file", file_path.to_str().unwrap()
    ]);
    assert!(tree_out.status.success());

    let tree_output = String::from_utf8_lossy(&tree_out.stdout);
    
    // Tree should show alias (true_id is displayed, not matched content)
    assert!(tree_output.contains("tree_test"), "Tree should show alias");
}
