use crate::test_helpers::{create_temp_file, parse_output, run_anchorscope};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

#[test]
fn read_single_line_anchor_from_file() {
    // Create a temp file with a simple single-line anchor
    let content = "Line 1\nLine 2: ANCHOR\nLine 3\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Create an anchor file
    let anchor_content = "Line 2: ANCHOR";
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let anchor_file_path = temp_dir.path().join("anchor.txt");
    fs::write(&anchor_file_path, anchor_content).expect("failed to write anchor file");

    // Run the read command with --anchor-file
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor-file",
        anchor_file_path.to_str().unwrap(),
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
    assert_eq!(content_value, anchor_content);
}

#[test]
fn read_multiline_anchor_from_file() {
    // Create a temp file with a multi-line anchor spanning lines 2-4
    let content = "Line 1\nLine 2: start\nLine 3: middle\nLine 4: end\nLine 5\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Create an anchor file with multi-line content
    let anchor_content = "Line 2: start\nLine 3: middle\nLine 4: end";
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let anchor_file_path = temp_dir.path().join("anchor.txt");
    fs::write(&anchor_file_path, anchor_content).expect("failed to write anchor file");

    // Run the read command with --anchor-file
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor-file",
        anchor_file_path.to_str().unwrap(),
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
    assert_eq!(content_value, anchor_content);
}

#[test]
fn write_with_anchor_file() {
    // Create a temp file with an anchor region
    let content = "before\nTARGET\nafter";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Create an anchor file
    let anchor_content = "TARGET";
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let anchor_file_path = temp_dir.path().join("anchor.txt");
    fs::write(&anchor_file_path, anchor_content).expect("failed to write anchor file");

    // First, use read command to obtain the hash of the anchor region
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor-file",
        anchor_file_path.to_str().unwrap(),
    ]);

    assert!(
        output.status.success(),
        "read command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("output is not valid UTF-8");
    let result: HashMap<String, String> = parse_output(&stdout);
    let hash = result.get("hash").expect("hash should be present").clone();

    // Now use write command with --anchor-file
    let replacement = "NEW_CONTENT";
    let output = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor-file",
        anchor_file_path.to_str().unwrap(),
        "--expected-hash",
        &hash,
        "--replacement",
        replacement,
    ]);

    // Assert exit code is 0 (success)
    assert!(
        output.status.success(),
        "write command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify the file was modified
    let final_content = std::fs::read_to_string(&file_path).expect("failed to read final file");
    let expected_content = "before\nNEW_CONTENT\nafter";
    assert_eq!(final_content, expected_content);
}

#[test]
fn empty_anchor_file_returns_error() {
    // Create a temp file with content
    let content = "Line 1\nLine 2: ANCHOR\nLine 3\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Create an empty anchor file
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let anchor_file_path = temp_dir.path().join("empty_anchor.txt");
    fs::write(&anchor_file_path, "").expect("failed to write empty anchor file");

    // Run the read command with empty anchor file
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor-file",
        anchor_file_path.to_str().unwrap(),
    ]);

    // Assert exit code is not 0 (failure)
    assert!(
        !output.status.success(),
        "read command should have failed for empty anchor file"
    );

    // Assert stderr contains "NO_MATCH"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("NO_MATCH"),
        "stderr should indicate NO_MATCH error for empty anchor, got: {}",
        stderr
    );
}

#[test]
fn nonexistent_anchor_file_returns_io_error() {
    // Create a temp file with content
    let content = "Line 1\nLine 2: ANCHOR\nLine 3\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Use a non-existent anchor file path
    let non_existent_anchor_file = "/nonexistent_path_12345/anchor.txt";

    // Run the read command with non-existent anchor file
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor-file",
        non_existent_anchor_file,
    ]);

    // Assert exit code is not 0 (failure)
    assert!(
        !output.status.success(),
        "read command should have failed for non-existent anchor file"
    );

    // Assert stderr starts with "IO_ERROR:"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.starts_with("IO_ERROR:"),
        "stderr should start with IO_ERROR:, got: {}",
        stderr
    );
}

#[test]
fn anchor_file_with_crlf_normalization() {
    // Create a temp file with LF line endings
    let content = "line1\nTARGET\nline3";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Create an anchor file with CRLF line endings
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let anchor_file_path = temp_dir.path().join("anchor_crlf.txt");
    let anchor_content_with_crlf = "TARGET\r\n";
    fs::write(&anchor_file_path, anchor_content_with_crlf).expect("failed to write anchor file with CRLF");

    // Run the read command with anchor file containing CRLF
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor-file",
        anchor_file_path.to_str().unwrap(),
    ]);

    // Should succeed because after normalization, both file and anchor match
    assert!(
        output.status.success(),
        "read command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("output is not valid UTF-8");
    let result: HashMap<String, String> = parse_output(&stdout);

    // Verify content returned is the normalized anchor (without CR)
    let content_value = result.get("content").expect("content should be present");
    assert_eq!(content_value, "TARGET\n");
}

#[test]
fn anchor_and_anchor_file_mutually_exclusive() {
    // Create a temp file with content
    let content = "Line 1\nLine 2: ANCHOR\nLine 3\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Create an anchor file
    let anchor_content = "Line 2: ANCHOR";
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let anchor_file_path = temp_dir.path().join("anchor.txt");
    fs::write(&anchor_file_path, anchor_content).expect("failed to write anchor file");

    // Run the read command with BOTH --anchor and --anchor-file
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "SOME_ANCHOR",
        "--anchor-file",
        anchor_file_path.to_str().unwrap(),
    ]);

    // Assert exit code is not 0 (failure)
    assert!(
        !output.status.success(),
        "read command should have failed when both anchor and anchor-file are provided"
    );

    // Assert stderr contains the mutual exclusivity error
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("mutually exclusive"),
        "stderr should indicate mutual exclusivity error, got: {}",
        stderr
    );
}
