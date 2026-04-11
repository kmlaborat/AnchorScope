use crate::test_helpers::{create_temp_file, run_anchorscope};

#[test]
fn path_traversal_blocked() {
    // Attempt to access file outside working directory
    let output = run_anchorscope(&[
        "read",
        "--file",
        "../etc/passwd",
        "--anchor",
        "test",
    ]);
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IO_ERROR") || stderr.contains("permission"));
}

#[test]
fn symlink_blocked() {
    let tmp = std::env::temp_dir();
    let target = tmp.join("test_target.txt");
    let link = tmp.join("test_link.txt");
    
    std::fs::write(&target, "test content").unwrap();
    
    // Skip on Windows (symlinks require elevated privileges)
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &link).unwrap();
    
    // Skip test if symlink creation failed (Windows)
    if !link.exists() {
        return;
    }
    
    let output = run_anchorscope(&[
        "read",
        "--file",
        link.to_str().unwrap(),
        "--anchor",
        "test",
    ]);
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IO_ERROR") || stderr.contains("permission"));
}

#[test]
fn command_injection_blocked() {
    let output = run_anchorscope(&[
        "pipe",
        "--true-id",
        "test",
        "--file-io",
        "--tool",
        "sed;rm -rf /",
    ]);
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IO_ERROR") || stderr.contains("permission"));
}

#[test]
fn invalid_tool_blocked() {
    let output = run_anchorscope(&[
        "pipe",
        "--true-id",
        "test",
        "--file-io",
        "--tool",
        "malicious_tool",
    ]);
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IO_ERROR") || stderr.contains("permission"));
}

#[test]
fn absolute_path_blocked() {
    let output = run_anchorscope(&[
        "pipe",
        "--true-id",
        "test",
        "--file-io",
        "--tool",
        "/bin/sh",
    ]);
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IO_ERROR") || stderr.contains("permission"));
}
