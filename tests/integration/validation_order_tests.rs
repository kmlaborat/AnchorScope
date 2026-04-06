use crate::test_helpers::*;
use tempfile::TempDir;

#[test]
fn test_validation_precedes_normalization_in_read() {
    // Create file with invalid UTF-8 sequence
    let invalid_utf8 = vec![0x48, 0x69, 0x80, 0x21]; // "Hi" + invalid byte + "!"
    let (temp_dir, file_path) = {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("bad_utf8.txt");
        std::fs::write(&path, &invalid_utf8).unwrap();
        (dir, path)
    };

    // Try to read with any anchor - should fail with IO_ERROR: invalid UTF-8
    let output = run_anchorscope(&[
        "read",
        "--file", file_path.to_str().unwrap(),
        "--anchor", "Hi"
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IO_ERROR: invalid UTF-8"));
    // Should NOT report any other error
}

#[test]
fn test_anchor_file_validation_precedes_normalization() {
    // Create anchor file with invalid UTF-8
    let invalid_anchor = vec![0x80, 0x81, 0x82];
    let (_anchor_dir, anchor_path) = {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("bad_anchor.txt");
        std::fs::write(&path, &invalid_anchor).unwrap();
        (dir, path)
    };

    // Valid file content
    let (_file_dir, file_path) = create_temp_file("Hello\nWorld");

    // Try to read using anchor_file
    let output = run_anchorscope(&[
        "read",
        "--file", file_path.to_str().unwrap(),
        "--anchor-file", anchor_path.to_str().unwrap(),
    ]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("stderr: |||{}|||", stderr);
        assert!(stderr.contains("IO_ERROR: invalid UTF-8"));
    } else {
        panic!("expected failure but command succeeded");
    }
}
