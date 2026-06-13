use crate::test_helpers::{create_temp_file, run_anchorscope};

#[test]
fn forbid_partial_matching() {
    // File contains only a prefix ("AB") of the anchor ("ABC")
    // This should fail because the tool must not perform partial/prefix matching.
    // The full anchor "ABC" is not present; only its prefix "AB" exists.
    let content = "some AB content";
    let (_temp_dir, file_path) = create_temp_file(content);

    let anchor = "ABC";

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
        "read command should have failed for partial matching"
    );

    // Assert stderr contains exactly "NO_MATCH"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.trim(), "NO_MATCH");
}

#[test]
fn forbid_fuzzy_matching() {
    // File contains "TARGET" but we search for "TARGT" (one char different)
    // This should fail because fuzzy/approximate matching is forbidden
    let content = "some\nTARGET\ncontent";
    let (_temp_dir, file_path) = create_temp_file(content);

    let anchor = "TARGT";

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
        "read command should have failed for fuzzy matching"
    );

    // Assert stderr contains exactly "NO_MATCH"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.trim(), "NO_MATCH");
}

#[test]
fn forbid_auto_correction() {
    // File contains "TARGET" but we search for " TARGET" (with leading space)
    // This should fail because whitespace trimming/auto-correction is forbidden
    let content = "content\nTARGET\nmore";
    let (_temp_dir, file_path) = create_temp_file(content);

    let anchor = " TARGET";

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
        "read command should have failed for auto-correction"
    );

    // Assert stderr contains exactly "NO_MATCH"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.trim(), "NO_MATCH");
}
