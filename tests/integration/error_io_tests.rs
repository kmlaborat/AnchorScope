use crate::test_helpers::run_anchorscope;

#[test]
fn read_io_error_file_not_found() {
    // Use an absolute path that definitely does not exist
    let non_existent_file = "/nonexistent_path_12345/file.txt";

    // Run the read command on a file that doesn't exist
    let output = run_anchorscope(&["read", "--file", non_existent_file, "--anchor", "ANCHOR"]);

    // Assert exit code is 1 (failure)
    assert!(
        !output.status.success(),
        "read command should have failed for non-existent file"
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
fn write_io_error_invalid_path() {
    // Use an absolute path where the directory does not exist
    let invalid_path = "/nonexistent_path_12345/file.txt";

    // Run the write command targeting a file in a non-existent directory
    // The file read will fail before any write attempt
    let output = run_anchorscope(&[
        "write",
        "--file",
        invalid_path,
        "--anchor",
        "ANCHOR",
        "--expected-hash",
        "0000000000000000",
        "--replacement",
        "NEW_CONTENT",
    ]);

    // Assert exit code is 1 (failure)
    assert!(
        !output.status.success(),
        "write command should have failed for invalid path"
    );

    // Assert stderr starts with "IO_ERROR:"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.starts_with("IO_ERROR:"),
        "stderr should start with IO_ERROR:, got: {}",
        stderr
    );
}
