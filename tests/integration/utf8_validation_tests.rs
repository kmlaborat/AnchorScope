use crate::test_helpers::run_anchorscope;
use std::fs;
use tempfile::TempDir;

#[test]
fn read_invalid_utf8_should_fail() {
    // Create a file with invalid UTF-8 bytes (e.g., 0xFF)
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let file_path = temp_dir.path().join("invalid_utf8.txt");
    // Write a single invalid byte
    fs::write(&file_path, b"\xFF").expect("failed to write file with invalid UTF-8");

    // Attempt to read with any anchor
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "any",
    ]);

    // Expect failure (exit code 1)
    assert!(
        !output.status.success(),
        "read should have failed for invalid UTF-8 file"
    );

    // stderr should contain IO_ERROR and mention invalid UTF-8
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("IO_ERROR"),
        "stderr should contain IO_ERROR, got: {}",
        stderr
    );
    assert!(
        stderr.contains("invalid UTF-8"),
        "stderr should mention invalid UTF-8, got: {}",
        stderr
    );
}

#[test]
fn write_invalid_utf8_should_fail() {
    // Create a file with invalid UTF-8 bytes
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let file_path = temp_dir.path().join("invalid_utf8.txt");
    fs::write(&file_path, b"\xFF\xFE").expect("failed to write file with invalid UTF-8");

    // Attempt to write (anchor and hash don't matter, will fail first on UTF-8 check)
    let output = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "any",
        "--expected-hash",
        "0123456789abcdef0123456789abcdef",
        "--replacement",
        "new",
    ]);

    // Expect failure
    assert!(
        !output.status.success(),
        "write should have failed for invalid UTF-8 file"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IO_ERROR"));
    assert!(stderr.contains("invalid UTF-8"));
}


