use crate::test_helpers::{create_temp_file, parse_output, run_anchorscope};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

#[test]
fn normalization_crlf_file_lf_anchor() {
    // Create a temp file with CRLF line endings (raw bytes)
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    // File contains: prefix\r\nTARGET\r\nsuffix
    let content_bytes = b"prefix\r\nTARGET\r\nsuffix";
    fs::write(&file_path, content_bytes).expect("failed to write temp file with CRLF");

    let anchor = "TARGET"; // LF normalized internally

    // First, use read command to obtain the hash of the anchor scope
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
    let hash = result.get("hash").expect("hash should be present").clone();

    // Now call write command with the hash and replacement
    let replacement = "NEW";
    let output = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor,
        "--expected-hash",
        &hash,
        "--replacement",
        replacement,
    ]);

    // Assert exit code is 0 (success)
    assert!(
        output.status.success(),
        "write command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Read the file content and verify:
    // - The anchor scope is replaced with "NEW"
    // - File content is normalized to LF line endings (no CR bytes)
    let final_bytes = fs::read(&file_path).expect("failed to read final file");
    assert!(
        !final_bytes.contains(&b'\r'),
        "File should not contain CR bytes; line endings should be normalized to LF"
    );
    let final_content = String::from_utf8(final_bytes).expect("final file is not valid UTF-8");
    let expected_content = "prefix\nNEW\nsuffix";
    assert_eq!(final_content, expected_content);
}

#[test]
fn normalization_anchor_with_crlf() {
    // Create a temp file with LF line endings
    let content = "line1\nline2\nline3";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Anchor string contains CRLF, e.g., "\r\nline2\r\n"
    // After normalization, this should match "line2"
    let anchor = "\r\nline2\r\n";

    // Run read command with this anchor
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor,
    ]);

    // Should succeed because after normalization, both file and anchor become "line2"
    assert!(
        output.status.success(),
        "read command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("output is not valid UTF-8");
    let result: HashMap<String, String> = parse_output(&stdout);

    // Verify content returned is the normalized anchor which includes surrounding newlines
    // "\r\nline2\r\n" normalizes to "\nline2\n"
    let content_value = result.get("content").expect("content should be present");
    assert_eq!(content_value, "\nline2\n");
}

#[test]
fn normalization_replacement_with_crlf() {
    // Create a temp file with LF line endings
    let content = "before\nTARGET\nafter";
    let (_temp_dir, file_path) = create_temp_file(content);

    let anchor = "TARGET";

    // First, use read command to obtain the hash of the anchor scope
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
    let hash = result.get("hash").expect("hash should be present").clone();

    // Write with replacement that contains CRLF: "A\r\nB"
    let replacement = "A\r\nB";
    let output = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor,
        "--expected-hash",
        &hash,
        "--replacement",
        replacement,
    ]);

    // Assert exit code is 0 (success)
    assert!(
        output.status.success(),
        "write command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Read the file content and verify:
    // - The anchor scope is replaced
    // - File content is normalized to LF line endings (no CR bytes)
    let final_bytes = fs::read(&file_path).expect("failed to read final file");
    assert!(
        !final_bytes.contains(&b'\r'),
        "File should not contain CR bytes; line endings should be normalized to LF"
    );
    let final_content = String::from_utf8(final_bytes).expect("final file is not valid UTF-8");
    let expected_content = "before\nA\nB\nafter";
    assert_eq!(final_content, expected_content);
}
