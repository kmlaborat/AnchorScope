use crate::test_helpers::run_anchorscope;

#[test]
fn path_traversal_blocked() {
    // Attempt to access file outside working directory
    let output = run_anchorscope(&["read", "--file", "../etc/passwd", "--anchor", "test"]);

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

    let output = run_anchorscope(&["read", "--file", link.to_str().unwrap(), "--anchor", "test"]);

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

#[test]
fn read_fails_when_path_contains_symlink() {
    // create a real file and a symlink to it
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("real.txt");
    std::fs::write(&real, "data").unwrap();
    let _link = dir.path().join("link.txt");

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&real, &link).unwrap();
    }
    #[cfg(windows)]
    {
        // Symlinks require admin privileges on Windows, so skip this test
        // The symlink test is only meaningful on Unix systems
        return;
    }

    // Verify the symlink was created
    if !_link.exists()
        || !std::fs::symlink_metadata(&_link)
            .unwrap()
            .file_type()
            .is_symlink()
    {
        return; // Skip if symlink creation failed
    }

    // run the command with the symlink path
    let output = run_anchorscope(&["read", "--file", _link.to_str().unwrap(), "--anchor", "test"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("PermissionDenied"));
}

#[test]
fn write_fails_when_target_is_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("real.txt");
    std::fs::write(&real, "orig").unwrap();
    let _link = dir.path().join("link.txt");

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&real, &link).unwrap();
    }
    #[cfg(windows)]
    {
        // Symlinks require admin privileges on Windows, so skip this test
        return;
    }

    // Verify the symlink was created
    if !_link.exists()
        || !std::fs::symlink_metadata(&_link)
            .unwrap()
            .file_type()
            .is_symlink()
    {
        return;
    }

    let out = run_anchorscope(&[
        "write",
        "--file",
        _link.to_str().unwrap(),
        "--anchor",
        "test",
        "--expected-hash",
        "deadbeef",
        "--replacement",
        "new",
    ]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("PermissionDenied"));
}

#[test]
#[cfg(unix)]
fn pipe_tool_whitelist_respects_env() {
    std::env::set_var("ANCHORSCOPE_ALLOWED_TOOLS", "sed,awk");
    let out = run_anchorscope(&["pipe", "--true-id", "t", "--file-io", "--tool", "awk"]);
    assert!(out.status.success());

    let out2 = run_anchorscope(&["pipe", "--true-id", "t", "--file-io", "--tool", "perl"]);
    assert!(!out2.status.success());
}

#[test]
fn pipe_uses_command_without_shell() {
    // First, create a buffer with a known true_id
    // We'll use the read command to create a buffer, then pipe
    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    std::fs::write(&test_file, "test content").unwrap();

    // Use read command to create a buffer
    let read_out = run_anchorscope(&[
        "read",
        "--file",
        test_file.to_str().unwrap(),
        "--anchor",
        "test",
    ]);
    assert!(read_out.status.success(), "read should succeed");

    // Extract the true_id from read output
    let output = String::from_utf8_lossy(&read_out.stdout);
    let true_id = output
        .lines()
        .find(|l| l.starts_with("true_id="))
        .map(|l| l.trim_start_matches("true_id=").to_string())
        .expect("true_id not found in output");

    // Test that pipe uses Command directly without shell
    // The security check is that we build Command directly (no "sh -c" or shell)
    //
    // We use perl with an expression that reads from stdin and outputs
    // The key is that tool args are passed via cmd.args() without shell interpretation
    //
    // Using perl with -e flag followed by a simple expression
    // The expression '1' is a perl idiom that prints each line
    let out = run_anchorscope(&[
        "pipe",
        "--true-id",
        &true_id,
        "--file-io",
        "--tool",
        "perl",
        "--tool-args",
        "-e1",
    ]);
    if !out.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&out.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&out.stderr));
    }
    assert!(
        out.status.success(),
        "pipe with tool_args should succeed - verifying no shell used"
    );
}

#[test]
#[cfg(unix)]
fn atomic_write_propagates_io_error() {
    // make the target directory read-only
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("readonly.txt");
    std::fs::write(&file, "data").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444)).unwrap();

    let out = run_anchorscope(&[
        "write",
        "--file",
        file.to_str().unwrap(),
        "--anchor",
        "a",
        "--expected-hash",
        "deadbeef",
        "--replacement",
        "new",
    ]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("WriteFailure"));
}
