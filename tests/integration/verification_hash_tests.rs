use crate::test_helpers::{create_temp_file, parse_output, run_anchorscope};
use xxhash_rust::xxh3::xxh3_64;

/// Test 1: Verify same anchor content produces identical hash across multiple runs/files
#[test]
fn hash_determinism() {
    // Create two separate temp files with identical content containing the same anchor
    let content = "Some content before\nANCHOR_CONTENT\nSome content after";
    let (_temp_dir1, file_path1) = create_temp_file(content);
    let (_temp_dir2, file_path2) = create_temp_file(content);

    let anchor = "ANCHOR_CONTENT";

    // Run read on first file
    let output1 = run_anchorscope(&[
        "read",
        "--file",
        file_path1.to_str().unwrap(),
        "--anchor",
        anchor,
    ]);

    // Run read on second file
    let output2 = run_anchorscope(&[
        "read",
        "--file",
        file_path2.to_str().unwrap(),
        "--anchor",
        anchor,
    ]);

    // Both should succeed
    assert!(
        output1.status.success(),
        "first read failed with stderr: {}",
        String::from_utf8_lossy(&output1.stderr)
    );
    assert!(
        output2.status.success(),
        "second read failed with stderr: {}",
        String::from_utf8_lossy(&output2.stderr)
    );

    // Parse outputs
    let stdout1 = String::from_utf8(output1.stdout).expect("output1 is not valid UTF-8");
    let stdout2 = String::from_utf8(output2.stdout).expect("output2 is not valid UTF-8");
    let result1: std::collections::HashMap<String, String> = parse_output(&stdout1);
    let result2: std::collections::HashMap<String, String> = parse_output(&stdout2);

    let hash1 = result1
        .get("hash")
        .expect("hash should be present in first output");
    let hash2 = result2
        .get("hash")
        .expect("hash should be present in second output");

    // Verify hashes are identical
    assert_eq!(
        hash1, hash2,
        "hashes should be identical for same anchor content"
    );
}

/// Test 2: Verify different anchor content produces different hashes
#[test]
fn hash_differentiates_similar_content() {
    // Create a file with two different anchors
    let content = "First anchor: CONTENT_A\nSome middle content\nSecond anchor: CONTENT_B";
    let (_temp_dir, file_path) = create_temp_file(content);

    let anchor_a = "First anchor: CONTENT_A";
    let anchor_b = "Second anchor: CONTENT_B";

    // Get hash for anchor A
    let output_a = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor_a,
    ]);

    // Get hash for anchor B
    let output_b = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor_b,
    ]);

    assert!(
        output_a.status.success(),
        "read for anchor A failed with stderr: {}",
        String::from_utf8_lossy(&output_a.stderr)
    );
    assert!(
        output_b.status.success(),
        "read for anchor B failed with stderr: {}",
        String::from_utf8_lossy(&output_b.stderr)
    );

    let stdout_a = String::from_utf8(output_a.stdout).expect("output_a is not valid UTF-8");
    let stdout_b = String::from_utf8(output_b.stdout).expect("output_b is not valid UTF-8");
    let result_a: std::collections::HashMap<String, String> = parse_output(&stdout_a);
    let result_b: std::collections::HashMap<String, String> = parse_output(&stdout_b);

    let hash_a = result_a
        .get("hash")
        .expect("hash should be present for anchor A");
    let hash_b = result_b
        .get("hash")
        .expect("hash should be present for anchor B");

    // Verify hashes are different
    assert_ne!(
        hash_a, hash_b,
        "hashes should be different for different anchor content"
    );
}

/// Test 3: Verify hash is computed on normalized (LF) bytes only
#[test]
fn hash_on_normalized_content() {
    // Create a file with CRLF line endings containing an anchor
    // The file content uses CRLF, but the anchor itself should be found and normalized
    let content_bytes = b"Some prefix\r\nTARGET\r\nSome suffix\r\n";
    let (_temp_dir, file_path) = create_temp_file(std::str::from_utf8(content_bytes).unwrap());

    let anchor = "TARGET";

    // Run read command
    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor,
    ]);

    assert!(
        output.status.success(),
        "read failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("output is not valid UTF-8");
    let result: std::collections::HashMap<String, String> = parse_output(&stdout);

    let hash = result.get("hash").expect("hash should be present");

    // Compute expected hash manually on the normalized matched scope
    // The anchor "TARGET" appears on a line by itself. After normalization, the matched
    // scope should be just "TARGET" without the CRLF.
    // Note: The anchor itself is "TARGET" which has no newlines. The CRLF is part of the
    // line ending in the file, which is normalized away during matching.
    let expected_hash = format!("{:016x}", xxh3_64(b"TARGET"));

    // Verify the returned hash matches the manual computation
    assert_eq!(
        hash, &expected_hash,
        "hash should be computed on normalized content (anchor without CRLF)"
    );
}
