use crate::test_helpers::{create_temp_file, parse_output, run_anchorscope};

#[test]
fn paths_with_true_id_returns_file_paths() {
    // Setup: Create buffer
    let content = "fn main() { x }\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    
    let read_output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "fn main() {",
    ]);
    assert!(read_output.status.success());
    let read_stdout = String::from_utf8(read_output.stdout).unwrap();
    let result = parse_output(&read_stdout);
    let true_id = result.get("true_id").unwrap().clone();
    
    // Get paths
    let output = run_anchorscope(&[
        "paths",
        "--true-id",
        &true_id,
    ]);
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    
    // Should have content: and replacement: lines
    let content_line = lines.iter().find(|l| l.starts_with("content:")).expect("content line missing");
    let replacement_line = lines.iter().find(|l| l.starts_with("replacement:")).expect("replacement line missing");
    
    let content_path = content_line.strip_prefix("content:").unwrap().trim();
    let replacement_path = replacement_line.strip_prefix("replacement:").unwrap().trim();
    
    // Verify paths exist (content path should exist)
    assert!(std::path::Path::new(content_path).exists());
    // Replacement may not exist yet, that's OK
}

#[test]
fn paths_with_label_returns_file_paths() {
    // Setup: Create buffer and label
    let content = "fn main() { x }\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    
    let read_output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "fn main() {",
    ]);
    assert!(read_output.status.success());
    let read_stdout = String::from_utf8(read_output.stdout).unwrap();
    let result = parse_output(&read_stdout);
    let true_id = result.get("true_id").unwrap().clone();
    
    let label_output = run_anchorscope(&[
        "label",
        "--name",
        "main_func",
        "--true-id",
        &true_id,
    ]);
    assert!(label_output.status.success());
    
    // Get paths with label
    let output = run_anchorscope(&[
        "paths",
        "--label",
        "main_func",
    ]);
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("content:"));
}

#[test]
fn paths_with_nonexistent_true_id_returns_error() {
    let output = run_anchorscope(&[
        "paths",
        "--true-id",
        "nonexistent12345678",
    ]);
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IO_ERROR"));
}

#[test]
fn paths_with_nonexistent_label_returns_error() {
    let output = run_anchorscope(&[
        "paths",
        "--label",
        "nonexistent_label",
    ]);
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IO_ERROR"));
}
