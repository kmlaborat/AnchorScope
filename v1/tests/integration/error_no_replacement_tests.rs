use crate::test_helpers::{create_temp_file, run_anchorscope};

#[test]
fn write_ambiguous_replacement_returns_error() {
    // Setup: Create a test file
    let content = "test content\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    let anchor = "test";

    // First, use read command to create buffer
    let read_out = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        anchor,
    ]);
    assert!(
        read_out.status.success(),
        "read command failed: {}",
        String::from_utf8_lossy(&read_out.stderr)
    );

    let stdout = String::from_utf8(read_out.stdout).expect("output is not valid UTF-8");
    let result: std::collections::HashMap<String, String> = parse_output(&stdout);
    let true_id = result
        .get("true_id")
        .expect("true_id should be present")
        .clone();

    // Create a label
    let label_out = run_anchorscope(&["label", "--name", "test_label", "--true-id", &true_id]);
    assert!(
        label_out.status.success(),
        "label command failed: {}",
        String::from_utf8_lossy(&label_out.stderr)
    );

    // Try to use both --replacement and --from-replacement (should fail)
    let write_out = run_anchorscope(&[
        "write",
        "--label",
        "test_label",
        "--replacement",
        "CONFLICT",
        "--from-replacement",
        "--file",
        file_path.to_str().unwrap(),
    ]);

    assert!(
        !write_out.status.success(),
        "write with both replacement options should fail"
    );

    let stderr = String::from_utf8_lossy(&write_out.stderr);
    // Clap produces an error message about conflicting arguments
    assert!(
        stderr.contains("cannot be used with"),
        "error should mention conflicting arguments: {}",
        stderr
    );
}

fn parse_output(output: &str) -> std::collections::HashMap<String, String> {
    let mut result = std::collections::HashMap::new();
    for line in output.lines() {
        if let Some((key, value)) = line.split_once('=') {
            result.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    result
}

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
        stderr.contains("NO_REPLACEMENT"),
        "error should be NO_REPLACEMENT: {}",
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
        stderr.contains("NO_REPLACEMENT"),
        "error should be NO_REPLACEMENT: {}",
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
