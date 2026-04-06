use crate::test_helpers::{create_temp_file, parse_output, read_file, run_anchorscope};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

#[test]
fn write_simple_replacement() {
    // Create a temp file with a simple single-line anchor
    let content = "Line 1\nANCHOR\nLine 2\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    let anchor = "ANCHOR";

    // First, use read command to obtain the hash of the anchor region
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
    let replacement = "NEW_CONTENT";
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

    // Read the file content and verify the changes
    let final_content = read_file(&file_path);
    let expected_content = "Line 1\nNEW_CONTENT\nLine 2\n";
    assert_eq!(final_content, expected_content);
}

#[test]
fn write_multiline_replacement() {
    // Create a temp file with a multi-line anchor spanning lines 2-4
    let content = "Before\nLINE1\nLINE2\nLINE3\nAfter\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    let anchor = "LINE1\nLINE2\nLINE3";

    // First, use read command to obtain the hash of the anchor region
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

    // Now call write command with a multi-line replacement
    let replacement = "NEW1\nNEW2\nNEW3";
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
    // - The anchor region is replaced
    // - Other parts unchanged
    let final_content = read_file(&file_path);
    let expected_content = "Before\nNEW1\nNEW2\nNEW3\nAfter\n";
    assert_eq!(final_content, expected_content);
}

#[test]
fn write_normalizes_replacement_to_lf() {
    // Create a temp file with CRLF line endings (raw bytes)
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    let content_bytes = b"Line 1\r\nANCHOR\r\nLine 2\r\n";
    fs::write(&file_path, content_bytes).expect("failed to write temp file with CRLF");

    let anchor = "ANCHOR";

    // First, use read command to obtain the hash of the anchor region
    // The read command will normalize the file content internally
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

    // Now call write command with a replacement string that contains newlines
    // We pass a string with explicit \n (or could be \r\n, but it will be normalized)
    let replacement = "NEW\r\nCONTENT"; // contains CRLF which should be normalized to LF
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
    // - The anchor region is replaced
    // - File content is normalized to LF line endings (no CR bytes)
    // - Other parts unchanged
    let final_bytes = fs::read(&file_path).expect("failed to read final file");
    assert!(
        !final_bytes.contains(&b'\r'),
        "File should not contain CR bytes; line endings should be normalized to LF"
    );
    let final_content = String::from_utf8(final_bytes).expect("final file is not valid UTF-8");
    let expected_content = "Line 1\nNEW\nCONTENT\nLine 2\n";
    assert_eq!(final_content, expected_content);
}

#[test]
fn write_using_label() {
    // Full workflow: read -> label -> write
    let content = "fn main() { println!(\"old\"); }";
    let (_temp_dir, file_path) = create_temp_file(content);
    let anchor = "old";

    // 1. read to get the internal label (auto-generated)
    let out_read = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(), "--anchor", anchor
    ]);
    assert!(out_read.status.success(), "read failed: {}", String::from_utf8_lossy(&out_read.stderr));
    let res = parse_output(&String::from_utf8_lossy(&out_read.stdout));
    let internal_label = res.get("label").unwrap().clone();

    // 2. create human-readable label
    let out_label = run_anchorscope(&[
        "label", "--name", "my_anchor", "--internal-label", &internal_label
    ]);
    assert!(out_label.status.success(), "label failed: {}", String::from_utf8_lossy(&out_label.stderr));

    // 3. write using label
    let out_write = run_anchorscope(&[
        "write", "--label", "my_anchor", "--replacement", "new", "--file", file_path.to_str().unwrap()
    ]);
    assert!(out_write.status.success(), "write via label failed: {}", String::from_utf8_lossy(&out_write.stderr));

    // 4. verify file changed
    let final_content = read_file(&file_path);
    assert!(final_content.contains("println!(\"new\");"));
}
