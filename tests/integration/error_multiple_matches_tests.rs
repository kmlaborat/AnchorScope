use crate::test_helpers::{create_temp_file, run_anchorscope};

#[test]
fn read_multiple_matches_two_occurrences() {
    // Create a temp file where the anchor appears exactly twice (non-overlapping)
    let content = "prefix ANCHOR middle ANCHOR suffix\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    let anchor = "ANCHOR";

    // Run the read command
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
        "read command should have failed for multiple matches"
    );

    // Assert stderr is exactly "MULTIPLE_MATCHES"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.trim() == "MULTIPLE_MATCHES",
        "stderr should be exactly MULTIPLE_MATCHES, got: {}",
        stderr
    );
}

#[test]
fn read_multiple_matches_overlapping() {
    // Create a temp file where the anchor appears with overlapping positions
    // "aaa" in "aaaa" gives matches at positions 0 and 1 (overlapping)
    // All exact matches must be counted, so we expect MULTIPLE_MATCHES (2).
    let content = "aaaa\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    let anchor = "aaa";

    // Run the read command
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
        "read command should have failed for overlapping matches"
    );

    // Expect MULTIPLE_MATCHES
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.trim() == "MULTIPLE_MATCHES",
        "stderr should be exactly MULTIPLE_MATCHES, got: {}",
        stderr
    );
}

#[test]
fn write_multiple_matches_rejected() {
    // Create a temp file where the anchor appears multiple times
    let content = "start ANCHOR middle ANCHOR end\n";
    let (_temp_dir, file_path) = create_temp_file(content);

    let anchor = "ANCHOR";
    let replacement = "REPLACED";
    // Use a dummy hash (won't be checked because multiple matches error occurs first)
    let expected_hash = "0123456789abcdef0123456789abcdef";

    // Run the write command
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
        "write command should have failed for multiple matches"
    );

    // Assert stderr is exactly "MULTIPLE_MATCHES"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.trim() == "MULTIPLE_MATCHES",
        "stderr should be exactly MULTIPLE_MATCHES, got: {}",
        stderr
    );
}
