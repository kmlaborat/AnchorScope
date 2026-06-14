use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Path to the anchorscope binary (built in debug mode).
fn binary_path() -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("anchorscope")
}

/// Creates a temporary file with the given content.
fn create_temp_file(content: &[u8]) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    std::fs::write(&file_path, content).expect("failed to write temp file");
    (temp_dir, file_path)
}

/// Runs the anchorscope binary with the given arguments.
fn run_anchorscope(args: &[&str]) -> std::process::Output {
    Command::new(&binary_path())
        .args(args)
        .output()
        .expect("failed to execute anchorscope")
}

/// Parses anchorscope output (key=value per line) into a HashMap.
/// Supports multi-line values by treating lines without '=' as continuations.
fn parse_output(output: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut current_key: Option<String> = None;
    for line in output.lines() {
        if let Some(equal_index) = line.find('=') {
            let key = line[..equal_index].to_string();
            let value = line[equal_index + 1..].to_string();
            map.insert(key.clone(), value);
            current_key = Some(key);
        } else if let Some(key) = &current_key {
            let existing = map.get_mut(key).unwrap();
            existing.push('\n');
            existing.push_str(line);
        }
    }
    map
}

// ─── Read tests ───────────────────────────────────────────────

#[test]
fn read_single_match_returns_scope_hash_and_content() {
    let content = "Line 1\nLine 2: ANCHOR\nLine 3\n";
    let (_temp_dir, file_path) = create_temp_file(content.as_bytes());

    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "Line 2: ANCHOR",
    ]);

    assert!(
        output.status.success(),
        "read should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let result = parse_output(&stdout);

    // scope_hash is a 16-char lowercase hex string
    let hash = result.get("scope_hash").expect("scope_hash should be present");
    assert_eq!(hash.len(), 16);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

    // content matches the anchor
    assert_eq!(result.get("content"), Some(&"Line 2: ANCHOR".to_string()));

    // No start_line/end_line in v2 output
    assert!(
        !result.contains_key("start_line"),
        "v2 should not output start_line"
    );
    assert!(
        !result.contains_key("end_line"),
        "v2 should not output end_line"
    );
}

#[test]
fn read_no_match() {
    let content = "Line 1\nLine 2\nLine 3\n";
    let (_temp_dir, file_path) = create_temp_file(content.as_bytes());

    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "NOT_FOUND",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.trim(), "NO_MATCH");
}

#[test]
fn read_multiple_matches() {
    let content = "AAA\nBBB\nAAA\n";
    let (_temp_dir, file_path) = create_temp_file(content.as_bytes());

    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "AAA",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.trim(), "MULTIPLE_MATCHES");
}

// ─── Write tests ──────────────────────────────────────────────

#[test]
fn write_success() {
    let content = "AAA\nBBB\nCCC\n";
    let (_temp_dir, file_path) = create_temp_file(content.as_bytes());

    // Step 1: read to get scope_hash
    let read_output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "BBB",
    ]);
    assert!(read_output.status.success());
    let read_result = parse_output(&String::from_utf8(read_output.stdout).unwrap());
    let scope_hash = read_result.get("scope_hash").unwrap();

    // Step 2: write with replacement
    let write_output = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "BBB",
        "--expected-hash",
        scope_hash,
        "--replacement",
        "REPLACED",
    ]);

    assert!(
        write_output.status.success(),
        "write should succeed, stderr: {}",
        String::from_utf8_lossy(&write_output.stderr)
    );

    // Step 3: verify file content
    let new_content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(new_content, "AAA\nREPLACED\nCCC\n");
}

#[test]
fn write_hash_mismatch() {
    let content = "AAA\nBBB\nCCC\n";
    let (_temp_dir, file_path) = create_temp_file(content.as_bytes());

    let output = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "BBB",
        "--expected-hash",
        "0000000000000000",
        "--replacement",
        "REPLACED",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.trim(), "HASH_MISMATCH");
}

// ─── CRLF offset mapping test ─────────────────────────────────

#[test]
fn write_crlf_file_preserves_line_endings_outside_scope() {
    // File with CRLF line endings
    let content = b"AAA\r\nBBB\r\nCCC\r\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Step 1: read to get scope_hash
    // The anchor is specified with LF (normalized)
    let read_output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "BBB",
    ]);
    assert!(read_output.status.success());
    let read_result = parse_output(&String::from_utf8(read_output.stdout).unwrap());
    let scope_hash = read_result.get("scope_hash").unwrap();

    // Step 2: write with replacement
    let write_output = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "BBB",
        "--expected-hash",
        scope_hash,
        "--replacement",
        "REPLACED",
    ]);

    assert!(
        write_output.status.success(),
        "write should succeed, stderr: {}",
        String::from_utf8_lossy(&write_output.stderr)
    );

    // Step 3: verify that CRLF outside the matched scope is preserved
    let new_bytes = std::fs::read(&file_path).unwrap();
    // Expected: "AAA\r\nREPLACED\r\nCCC\r\n"
    // The CRLF before and after "BBB" should be preserved because we
    // only replace the exact byte range of the matched scope in the original file.
    assert_eq!(
        new_bytes,
        b"AAA\r\nREPLACED\r\nCCC\r\n",
        "CRLF line endings outside the matched scope should be preserved"
    );
}

#[test]
fn write_crlf_multiline_anchor() {
    // File with CRLF: match a multi-line scope
    let content = b"HEADER\r\nfn foo() {\r\n    return 1;\r\n}\r\nFOOTER\r\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Step 1: read with a multi-line anchor (LF in anchor, CRLF in file)
    let read_output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "fn foo() {\n    return 1;\n}",
    ]);
    assert!(
        read_output.status.success(),
        "read should succeed, stderr: {}",
        String::from_utf8_lossy(&read_output.stderr)
    );
    let read_result = parse_output(&String::from_utf8(read_output.stdout).unwrap());
    let scope_hash = read_result.get("scope_hash").unwrap();

    // Step 2: write replacement
    let write_output = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "fn foo() {\n    return 1;\n}",
        "--expected-hash",
        scope_hash,
        "--replacement",
        "fn foo() {\n    return 42;\n}",
    ]);

    assert!(
        write_output.status.success(),
        "write should succeed, stderr: {}",
        String::from_utf8_lossy(&write_output.stderr)
    );

    // Step 3: verify CRLF is preserved in non-matched regions
    let new_bytes = std::fs::read(&file_path).unwrap();
    // HEADER\r\n and \r\nFOOTER\r\n should be preserved
    assert!(
        new_bytes.starts_with(b"HEADER\r\n"),
        "CRLF after HEADER should be preserved"
    );
    assert!(
        new_bytes.ends_with(b"\r\nFOOTER\r\n"),
        "CRLF around FOOTER should be preserved"
    );
    // The replacement itself is written as-is (with LF)
    assert!(
        new_bytes.windows(27).any(|w| w == b"fn foo() {\n    return 42;\n}"),
        "Replacement should be present in the file"
    );
}

// ─── Anchor file tests ────────────────────────────────────────

#[test]
fn read_with_anchor_file() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");

    let file_path = temp_dir.path().join("target.txt");
    std::fs::write(&file_path, "Hello World\n").unwrap();

    let anchor_path = temp_dir.path().join("anchor.txt");
    std::fs::write(&anchor_path, "Hello World").unwrap();

    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor-file",
        anchor_path.to_str().unwrap(),
    ]);

    assert!(
        output.status.success(),
        "read should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let result = parse_output(&String::from_utf8(output.stdout).unwrap());
    assert_eq!(result.get("content"), Some(&"Hello World".to_string()));
}

#[test]
fn write_with_replacement_file() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");

    let file_path = temp_dir.path().join("target.txt");
    std::fs::write(&file_path, "AAA\nBBB\nCCC\n").unwrap();

    let anchor_path = temp_dir.path().join("anchor.txt");
    std::fs::write(&anchor_path, "BBB").unwrap();

    let replacement_path = temp_dir.path().join("replacement.txt");
    std::fs::write(&replacement_path, "REPLACED").unwrap();

    // Step 1: read
    let read_output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor-file",
        anchor_path.to_str().unwrap(),
    ]);
    assert!(read_output.status.success());
    let read_result = parse_output(&String::from_utf8(read_output.stdout).unwrap());
    let scope_hash = read_result.get("scope_hash").unwrap();

    // Step 2: write
    let write_output = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor-file",
        anchor_path.to_str().unwrap(),
        "--expected-hash",
        scope_hash,
        "--replacement-file",
        replacement_path.to_str().unwrap(),
    ]);

    assert!(
        write_output.status.success(),
        "write should succeed, stderr: {}",
        String::from_utf8_lossy(&write_output.stderr)
    );

    let new_content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(new_content, "AAA\nREPLACED\nCCC\n");
}

// ─── Error tests ──────────────────────────────────────────────

#[test]
fn read_file_not_found() {
    let output = run_anchorscope(&[
        "read",
        "--file",
        "/nonexistent/path/file.txt",
        "--anchor",
        "test",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.trim(), "IO_ERROR: file not found");
}

#[test]
fn write_file_not_found() {
    let output = run_anchorscope(&[
        "write",
        "--file",
        "/nonexistent/path/file.txt",
        "--anchor",
        "test",
        "--expected-hash",
        "0000000000000000",
        "--replacement",
        "replaced",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.trim(), "IO_ERROR: file not found");
}

// ─── Mixed line ending tests ──────────────────────────────────

#[test]
fn read_mixed_line_endings() {
    // File with mixed CRLF and LF: "AAA\r\nBBB\nCCC\r\n"
    let content = b"AAA\r\nBBB\nCCC\r\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Read with anchor "BBB" (LF anchor against mixed file)
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "BBB",
    ]);

    assert!(
        output.status.success(),
        "read should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let result = parse_output(&stdout);

    let hash = result.get("scope_hash").expect("scope_hash should be present");
    assert_eq!(hash.len(), 16);
    assert_eq!(result.get("content"), Some(&"BBB".to_string()));
}

#[test]
fn read_mixed_line_endings_multiline_anchor() {
    // File: "AAA\r\nBBB\nCCC\r\n"
    // Match across mixed line endings: "BBB\nCCC" (spanning LF region)
    let content = b"AAA\r\nBBB\nCCC\r\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "BBB\nCCC",
    ]);

    assert!(
        output.status.success(),
        "read should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let result = parse_output(&stdout);

    assert_eq!(
        result.get("content"),
        Some(&"BBB\nCCC".to_string()),
        "content should be the normalized matched scope"
    );
}

#[test]
fn write_mixed_line_endings_preserves_outside_scope() {
    // File: "AAA\r\nBBB\nCCC\r\n"
    // Replace "BBB" with "REPLACED" — CRLF before and after should be preserved
    let content = b"AAA\r\nBBB\nCCC\r\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Step 1: read
    let read_output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "BBB",
    ]);
    assert!(read_output.status.success());
    let read_result = parse_output(&String::from_utf8(read_output.stdout).unwrap());
    let scope_hash = read_result.get("scope_hash").unwrap();

    // Step 2: write
    let write_output = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "BBB",
        "--expected-hash",
        scope_hash,
        "--replacement",
        "REPLACED",
    ]);
    assert!(
        write_output.status.success(),
        "write should succeed, stderr: {}",
        String::from_utf8_lossy(&write_output.stderr)
    );

    // Step 3: verify — CRLF before BBB and LF+CCC+CRLF after should be preserved
    let new_bytes = std::fs::read(&file_path).unwrap();
    assert_eq!(
        new_bytes,
        b"AAA\r\nREPLACED\nCCC\r\n",
        "mixed line endings outside the matched scope should be preserved"
    );
}

#[test]
fn write_mixed_line_endings_multiline_scope() {
    // File: "HDR\r\nAAA\r\nBBB\nCCC\nFOOTER\r\n"
    // Replace "AAA\r\nBBB\nCCC" (spanning CRLF and LF) with "REPLACED"
    let content = b"HDR\r\nAAA\r\nBBB\nCCC\nFOOTER\r\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Step 1: read with multiline anchor (LF in anchor)
    let read_output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "AAA\nBBB\nCCC",
    ]);
    assert!(
        read_output.status.success(),
        "read should succeed, stderr: {}",
        String::from_utf8_lossy(&read_output.stderr)
    );
    let read_result = parse_output(&String::from_utf8(read_output.stdout).unwrap());
    let scope_hash = read_result.get("scope_hash").unwrap();

    // Step 2: write
    let write_output = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "AAA\nBBB\nCCC",
        "--expected-hash",
        scope_hash,
        "--replacement",
        "REPLACED",
    ]);
    assert!(
        write_output.status.success(),
        "write should succeed, stderr: {}",
        String::from_utf8_lossy(&write_output.stderr)
    );

    // Step 3: verify
    let new_bytes = std::fs::read(&file_path).unwrap();
    // HDR\r\n should be preserved, REPLACED written as-is, \nFOOTER\r\n preserved
    assert!(
        new_bytes.starts_with(b"HDR\r\n"),
        "CRLF after HDR should be preserved"
    );
    assert!(
        new_bytes.ends_with(b"\nFOOTER\r\n"),
        "LF before FOOTER and CRLF after should be preserved"
    );
    assert!(
        new_bytes.windows(8).any(|w| w == b"REPLACED"),
        "Replacement should be present"
    );
}
