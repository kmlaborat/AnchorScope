use crate::test_helpers::*;

#[test]
fn test_anchor_command_success() {
    // Setup: create file with known content
    let (_temp_dir, file_path) = create_temp_file(
        "fn main() {\n    println!(\"Hello\");\n}\n\
         fn foo() {\n    println!(\"World\");\n}\n"
    );

    // Step 1: Use read to get hash of anchor region
    let output = run_anchorscope(&[
        "read",
        "--file", file_path.to_str().unwrap(),
        "--anchor", "fn main() {\n    println!(\"Hello\");\n}"
    ]);
    assert!(output.status.success());
    let result = parse_output(&String::from_utf8_lossy(&output.stdout));
    let hash = result.get("hash").unwrap().clone();

    // Step 2: Call anchor command with that hash
    let output = run_anchorscope(&[
        "anchor",
        "--file", file_path.to_str().unwrap(),
        "--label", "main_function",
        "--anchor", "fn main() {\n    println!(\"Hello\");\n}",
        "--expected-hash", &hash
    ]);
    assert!(output.status.success(), "anchor should succeed");
}

#[test]
fn test_anchor_nomatch() {
    let (_temp_dir, file_path) = create_temp_file("Hello\nWorld");

    // Use any hash; anchor not present will cause NO_MATCH before hash check
    let output = run_anchorscope(&[
        "anchor",
        "--file", file_path.to_str().unwrap(),
        "--label", "nonexistent",
        "--anchor", "not present in file",
        "--expected-hash", "0000000000000000"
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("NO_MATCH"));
}

#[test]
fn test_anchor_multiple_matches() {
    let (_temp_dir, file_path) = create_temp_file("x\nx\n"); // two identical lines

    // Anchor "x" appears twice -> MULTIPLE_MATCHES, before hash check
    let output = run_anchorscope(&[
        "anchor",
        "--file", file_path.to_str().unwrap(),
        "--label", "duplicate",
        "--anchor", "x",
        "--expected-hash", "0000000000000000" // hash irrelevant
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("MULTIPLE_MATCHES"));
}

#[test]
fn test_anchor_hash_mismatch() {
    let (_temp_dir, file_path) = create_temp_file("Hello\nWorld");

    // Compute actual hash of anchor
    let output = run_anchorscope(&[
        "read",
        "--file", file_path.to_str().unwrap(),
        "--anchor", "Hello"
    ]);
    assert!(output.status.success());
    let result = parse_output(&String::from_utf8_lossy(&output.stdout));
    let _real_hash = result.get("hash").unwrap().clone();

    // Use a different (wrong) hash
    let wrong_hash = "ffffffffffffffff";
    let output = run_anchorscope(&[
        "anchor",
        "--file", file_path.to_str().unwrap(),
        "--label", "mismatch",
        "--anchor", "Hello",
        "--expected-hash", wrong_hash
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("HASH_MISMATCH") && stderr.contains(wrong_hash));
}

#[test]
fn test_anchor_invalid_utf8_anchor_via_file() {
    use tempfile::TempDir;
    let (_temp_dir, file_path) = create_temp_file("Hello\nWorld");

    // Create anchor file with invalid UTF-8
    let invalid_anchor = vec![0x80, 0x81, 0x82];
    let anchor_dir = TempDir::new().unwrap();
    let anchor_path = anchor_dir.path().join("bad_anchor.txt");
    std::fs::write(&anchor_path, &invalid_anchor).unwrap();

    let output = run_anchorscope(&[
        "anchor",
        "--file", file_path.to_str().unwrap(),
        "--label", "invalid_anchor",
        "--anchor-file", anchor_path.to_str().unwrap(),
        "--expected-hash", "0000000000000000"
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IO_ERROR: invalid UTF-8"));
}

#[test]
fn test_anchor_io_error_file_not_found() {
    // Use non-existent file
    let output = run_anchorscope(&[
        "anchor",
        "--file", "/nonexistent/path.txt",
        "--label", "test",
        "--anchor", "Hello",
        "--expected-hash", "0000000000000000"
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IO_ERROR: file not found"));
}

#[test]
fn test_anchor_label_collision() {
    let (_temp_dir, file_path) = create_temp_file(
        "fn main() {\n    println!(\"Hello\");\n}\n\
         fn foo() {\n    println!(\"World\");\n}\n"
    );

    // First define the label for main_function
    let output1 = run_anchorscope(&[
        "read",
        "--file", file_path.to_str().unwrap(),
        "--anchor", "fn main() {\n    println!(\"Hello\");\n}"
    ]);
    assert!(output1.status.success());
    let result = parse_output(&String::from_utf8_lossy(&output1.stdout));
    let hash_main = result.get("hash").unwrap().clone();

    let output_anchor1 = run_anchorscope(&[
        "anchor",
        "--file", file_path.to_str().unwrap(),
        "--label", "main_function",
        "--anchor", "fn main() {\n    println!(\"Hello\");\n}",
        "--expected-hash", &hash_main
    ]);
    assert!(output_anchor1.status.success(), "first anchor should succeed");

    // Attempt to define the same label with a different anchor (different hash)
    // Need correct hash for that different anchor to pass hash check, to reach label collision check
    let output_read_foo = run_anchorscope(&[
        "read",
        "--file", file_path.to_str().unwrap(),
        "--anchor", "fn foo() {\n    println!(\"World\");\n}"
    ]);
    assert!(output_read_foo.status.success());
    let result_foo = parse_output(&String::from_utf8_lossy(&output_read_foo.stdout));
    let hash_foo = result_foo.get("hash").unwrap().clone();

    let output2 = run_anchorscope(&[
        "anchor",
        "--file", file_path.to_str().unwrap(),
        "--label", "main_function",
        "--anchor", "fn foo() {\n    println!(\"World\");\n}", // different anchor
        "--expected-hash", &hash_foo // correct hash for this anchor
    ]);
    assert!(!output2.status.success());
    let stderr = String::from_utf8_lossy(&output2.stderr);
    assert!(stderr.contains("LABEL_EXISTS"));

    // Attempt to define the same label with the same anchor+hash (idempotent should succeed)
    let output3 = run_anchorscope(&[
        "anchor",
        "--file", file_path.to_str().unwrap(),
        "--label", "main_function",
        "--anchor", "fn main() {\n    println!(\"Hello\");\n}",
        "--expected-hash", &hash_main
    ]);
    assert!(output3.status.success(), "re-define identical anchor should be idempotent");
}
