use crate::test_helpers::{create_temp_file, run_anchorscope};

#[test]
fn read_nomatch_anchor_missing() {
    // Create a temp file with content that does NOT contain the anchor
    let content = "Line 1\nLine 2\nLine 3\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    let anchor = "NONEXISTENT_ANCHOR";

    // Run the read command with an anchor that is not in the file
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor,
    ]);

    // Assert exit code is 1 (failure)
    assert!(
        !output.status.success(),
        "read command should have failed for missing anchor"
    );

    // Assert stderr contains exactly "NO_MATCH"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.trim(), "NO_MATCH");
}

#[test]
fn read_nomatch_empty_anchor() {
    // Create a temp file with any content
    let content = "Line 1\nLine 2\nLine 3\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    // Run the read command with an empty anchor string
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "",
    ]);

    // Assert exit code is 1 (failure)
    assert!(
        !output.status.success(),
        "read command should have failed for empty anchor"
    );

    // Assert stderr contains exactly "NO_MATCH"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.trim(), "NO_MATCH");
}

#[test]
fn write_nomatch_anchor_missing() {
    // Create a temp file with content that does NOT contain the anchor
    let content = "Line 1\nLine 2\nLine 3\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    let anchor = "NONEXISTENT_ANCHOR";
    let replacement = "NEW_CONTENT";
    // Use a dummy hash (won't be checked because anchor is missing)
    let expected_hash = "0000000000000000";

    // Run the write command with an anchor that is not in the file
    let output = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor,
        "--expected-hash",
        expected_hash,
        "--replacement",
        replacement,
    ]);

    // Assert exit code is 1 (failure)
    assert!(
        !output.status.success(),
        "write command should have failed for missing anchor"
    );

    // Assert stderr contains exactly "NO_MATCH"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.trim(), "NO_MATCH");
}
