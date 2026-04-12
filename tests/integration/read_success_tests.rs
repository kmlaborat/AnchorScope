use crate::test_helpers::{create_temp_file, parse_output, run_anchorscope};
use std::collections::HashMap;

#[test]
fn read_single_line_anchor_found() {
    // Create a temp file with a simple single-line anchor
    let content = "Line 1\nLine 2: ANCHOR\nLine 3\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    let anchor = "Line 2: ANCHOR";

    // Run the read command
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor,
    ]);

    // Assert exit code is 0 (success)
    assert!(
        output.status.success(),
        "read command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Parse stdout
    let stdout = String::from_utf8(output.stdout).expect("output is not valid UTF-8");
    let result: HashMap<String, String> = parse_output(&stdout);

    // Verify start_line and end_line
    assert_eq!(result.get("start_line"), Some(&"2".to_string()));
    assert_eq!(result.get("end_line"), Some(&"2".to_string()));

    // Verify hash is a 16-character lowercase hex string
    let hash = result.get("hash").expect("hash should be present");
    assert_eq!(hash.len(), 16, "hash should be 16 characters");
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "hash should be all hex digits"
    );

    // Verify content matches the anchor (normalized)
    let content_value = result.get("content").expect("content should be present");
    assert_eq!(content_value, anchor);

    // Verify label is present and equals hash
    let label = result.get("label").expect("label should be present");
    assert_eq!(label, hash);
}

#[test]
fn read_multiline_anchor_found() {
    // Create a temp file with a multi-line anchor spanning lines 2-4
    let content = "Line 1\nLine 2: start\nLine 3: middle\nLine 4: end\nLine 5\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    let anchor = "Line 2: start\nLine 3: middle\nLine 4: end";

    // Run the read command
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor,
    ]);

    // Assert exit code is 0 (success)
    assert!(
        output.status.success(),
        "read command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Parse stdout
    let stdout = String::from_utf8(output.stdout).expect("output is not valid UTF-8");
    let result: HashMap<String, String> = parse_output(&stdout);

    // Verify start_line and end_line (anchor spans lines 2, 3, and 4)
    assert_eq!(result.get("start_line"), Some(&"2".to_string()));
    assert_eq!(result.get("end_line"), Some(&"4".to_string()));

    // Verify hash is a 16-character lowercase hex string
    let hash = result.get("hash").expect("hash should be present");
    assert_eq!(hash.len(), 16, "hash should be 16 characters");
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "hash should be all hex digits"
    );

    // Verify content matches the anchor (normalized)
    let content_value = result.get("content").expect("content should be present");
    assert_eq!(content_value, anchor);

    // Verify label is present and equals hash
    let label = result.get("label").expect("label should be present");
    assert_eq!(label, hash);
}

#[test]
fn read_substring_match_without_boundaries() {
    // Per spec Section 5.3: "If the full anchor byte sequence appears within a larger sequence, it is considered a valid match."
    // This test verifies that an anchor embedded in a larger non-boundaried token matches.
    let content = "XYZABCXYZ";
    let (_temp_dir, file_path) = create_temp_file(content);

    let anchor = "ABC";

    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor,
    ]);

    assert!(
        output.status.success(),
        "read should succeed for substring match without boundaries: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("output is not valid UTF-8");
    let result: HashMap<String, String> = parse_output(&stdout);

    // Should match at line 1 (no newlines in file)
    assert_eq!(result.get("start_line"), Some(&"1".to_string()));
    assert_eq!(result.get("end_line"), Some(&"1".to_string()));
    assert_eq!(result.get("content"), Some(&"ABC".to_string()));

    // Verify hash and label
    let hash = result.get("hash").expect("hash should be present");
    let label = result.get("label").expect("label should be present");
    assert_eq!(label, hash);
}

#[test]
fn read_with_true_id() {
    // Create a temp file with a simple anchor
    let content = "Line 1: ANCHOR\nLine 2\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    let anchor = "Line 1: ANCHOR";

    // Step 1: Read to get the true_id
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor,
    ]);

    assert!(
        output.status.success(),
        "read command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("output is not valid UTF-8");
    let result: HashMap<String, String> = parse_output(&stdout);
    let true_id = result.get("true_id").expect("true_id should be present").clone();

    // Step 2: Read using --true-id to create nested anchor
    let output2 = run_anchorscope(&[
        "read",
        "--true-id",
        &true_id,
        "--anchor",
        "ANCHOR",
    ]);

    assert!(
        output2.status.success(),
        "read with --true-id should succeed: {}",
        String::from_utf8_lossy(&output2.stderr)
    );

    let stdout2 = String::from_utf8(output2.stdout).expect("output is not valid UTF-8");
    let result2: HashMap<String, String> = parse_output(&stdout2);

    // Verify the nested read
    let nested_true_id = result2.get("true_id").expect("true_id should be present").clone();
    let nested_hash = result2.get("hash").expect("hash should be present").clone();

    // The nested true_id should be different from parent
    assert_ne!(true_id, nested_true_id, "Nested true_id should be different");

    // Verify content
    let content_value = result2.get("content").expect("content should be present");
    assert_eq!(content_value, "ANCHOR");

    // Verify hash in output
    assert!(result2.contains_key("hash"), "hash should be present");
    let output_hash = result2.get("hash").expect("hash should be present").clone();
    assert_eq!(output_hash, nested_hash);
}

#[test]
fn write_with_true_id() {
    // Create a temp file with a simple anchor
    let content = "Line 1: ANCHOR\nLine 2\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    let anchor = "Line 1: ANCHOR";

    // Step 1: Read to get the true_id and scope_hash
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor,
    ]);

    assert!(
        output.status.success(),
        "read command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("output is not valid UTF-8");
    let result: HashMap<String, String> = parse_output(&stdout);
    let true_id = result.get("true_id").expect("true_id should be present").clone();
    let scope_hash = result.get("hash").expect("hash should be present").clone();

    // Step 2: Create a replacement file
    let replacement = "REPLACED";

    // Step 3: Write using --true-id
    let output2 = run_anchorscope(&[
        "write",
        "--true-id",
        &true_id,
        "--expected-hash",
        &scope_hash,
        "--replacement",
        replacement,
    ]);

    assert!(
        output2.status.success(),
        "write with --true-id should succeed: {}",
        String::from_utf8_lossy(&output2.stderr)
    );

    // Step 4: Verify the file was modified
    let new_content = std::fs::read_to_string(&file_path).unwrap();
    assert!(
        new_content.contains("REPLACED"),
        "File should contain 'REPLACED' after write"
    );
}
