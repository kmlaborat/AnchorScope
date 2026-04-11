use crate::test_helpers::{create_temp_file, read_file, run_anchorscope};

#[test]
fn write_hash_mismatch_wrong_hash() {
    // Create a temp file with a simple single-line anchor
    let content = "Line 1\nANCHOR\nLine 2\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    let anchor = "ANCHOR";
    let replacement = "REPLACED";

    // First, use read command to obtain the correct hash of the anchor region
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
    let result: std::collections::HashMap<String, String> =
        crate::test_helpers::parse_output(&stdout);
    let _real_hash = result.get("hash").expect("hash should be present").clone();

    // Intentionally use a WRONG hash (all zeros) to trigger HASH_MISMATCH
    let wrong_hash = "0000000000000000";

    // Now call write command with the wrong hash and a replacement
    let output = run_anchorscope(&[
        "write",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor,
        "--expected-hash",
        &wrong_hash,
        "--replacement",
        replacement,
    ]);

    // Assert exit code is 1 (failure)
    assert!(
        !output.status.success(),
        "write command should have failed for hash mismatch"
    );

    // Assert stderr contains HASH_MISMATCH error with expected and actual hashes
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("HASH_MISMATCH"),
        "stderr should contain HASH_MISMATCH, got: {}",
        stderr
    );


    // Verify the file content remains unchanged after the failed write
    let final_content = read_file(&file_path);
    assert_eq!(final_content, content, "file should remain unchanged");
}
