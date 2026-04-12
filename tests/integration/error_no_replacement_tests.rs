use crate::test_helpers::{create_temp_file, run_anchorscope};

#[test]
fn write_no_replacement_returns_error() {
    // Setup: Create a test file
    let content = "test content\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    let anchor = "test";

    // Try to write without --replacement and without --from-replacement
    // This should fail with NO_REPLACEMENT error
    let output = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor,
        "--expected-hash",
        "0000000000000000", // dummy hash
        "--replacement",
        "",
    ]);

    // Should fail with NO_REPLACEMENT error
    assert!(
        !output.status.success(),
        "write should fail without replacement"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("IO_ERROR: no replacement provided"),
        "error should be IO_ERROR: no replacement provided: {}",
        stderr
    );
}

#[test]
fn write_no_replacement_without_anchor_returns_error() {
    // Setup: Create a test file
    let content = "test content\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Try to write without anchor (which implies no --replacement)
    // This should fail with NO_REPLACEMENT error
    let output = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--replacement",
        "",
    ]);

    // Should fail with NO_REPLACEMENT error
    assert!(
        !output.status.success(),
        "write should fail without anchor and replacement"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("IO_ERROR: no replacement provided"),
        "error should be IO_ERROR: no replacement provided: {}",
        stderr
    );
}

#[test]
fn write_with_from_replacement_without_label_returns_error() {
    // Setup: Create a test file
    let content = "test content\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Try to use --from-replacement without --label (should fail)
    // Because no label means no buffer exists
    let output = run_anchorscope(&[
        "write",
        "--anchor",
        "test",
        "--from-replacement",
        "--file",
        file_path.to_str().unwrap(),
        "--replacement",
        "ignored",
    ]);

    // Should fail because --from-replacement requires a label
    assert!(
        !output.status.success(),
        "write should fail with from_replacement but no label"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Clap produces an error message about conflicting arguments
    assert!(
        stderr.contains("cannot be used with"),
        "error should mention conflicting arguments: {}",
        stderr
    );
}
