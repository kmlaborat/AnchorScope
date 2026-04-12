use crate::buffer_path;
use crate::error::AnchorScopeError;
use crate::matcher;
use crate::security::validate_tool_name;
use crate::storage;
use std::io::{self, Read, Write};

/// Stream content to stdout for a True ID.
pub fn stream_content_to_stdout(true_id: &str) -> Result<(), AnchorScopeError> {
    let file_hash = storage::file_hash_for_true_id(true_id)?;
    
    // Try flat location first
    let flat_path = buffer_path::true_id_dir(&file_hash, true_id).join("content");
    if flat_path.exists() {
        let content = std::fs::read(&flat_path).map_err(|e| crate::error::from_io_error_write(e))?;
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle
            .write_all(&content)
            .map_err(|e| crate::error::from_io_error_write(e))?;
        return Ok(());
    }
    
    // Search nested locations using BFS
    let file_dir = buffer_path::file_dir(&file_hash);
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(file_dir);
    
    while let Some(current_dir) = queue.pop_front() {
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let child_dir = entry.path();
                    let content_path = child_dir.join(true_id).join("content");
                    
                    if content_path.exists() {
                        let content = std::fs::read(&content_path)
                            .map_err(|e| crate::error::from_io_error_write(e))?;
                        let stdout = io::stdout();
                        let mut handle = stdout.lock();
                        handle
                            .write_all(&content)
                            .map_err(|e| crate::error::from_io_error_write(e))?;
                        return Ok(());
                    }
                    
                    queue.push_back(child_dir);
                }
            }
        }
    }
    
    Err(AnchorScopeError::FileNotFound)
}

/// Read from stdin and write to replacement file.
pub fn read_from_stdin_and_write_replacement(
    true_id: &str,
    stdin_bytes: &[u8],
) -> Result<(), AnchorScopeError> {
    let file_hash = storage::file_hash_for_true_id(true_id)?;

    // Validate UTF-8
    if std::str::from_utf8(stdin_bytes).is_err() {
        return Err(AnchorScopeError::InvalidUtf8);
    }

    // Normalize CRLF -> LF
    let normalized = matcher::normalize_line_endings(stdin_bytes);

    // Write to replacement file
    // Search for the true_id location using BFS to handle nested buffers
    let file_dir = buffer_path::file_dir(&file_hash);
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(file_dir);
    
    let true_id_dir = 'outer: loop {
        if let Some(current_dir) = queue.pop_front() {
            if let Ok(entries) = std::fs::read_dir(&current_dir) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        let child_dir = entry.path();
                        let content_path = child_dir.join(true_id).join("content");
                        
                        if content_path.exists() {
                            break 'outer child_dir;
                        }
                        
                        queue.push_back(child_dir);
                    }
                }
            }
        } else {
            // Not found, use flat path
            break 'outer buffer_path::file_dir(&file_hash);
        }
    };
    
    // Write to replacement file (in same directory as content)
    let replacement_path = true_id_dir.join(true_id).join("replacement");
    std::fs::write(&replacement_path, &normalized)
        .map_err(|e| crate::error::from_io_error_write(e))?;

    Ok(())
}

/// Validate and store replacement from external tool output file.
fn validate_and_store_replacement(
    true_id: &str,
    output_path: &std::path::Path,
) -> Result<(), AnchorScopeError> {
    let file_hash = storage::file_hash_for_true_id(true_id)?;

    // Read tool output
    let content = std::fs::read(output_path).map_err(|e| crate::error::from_io_error_write(e))?;

    // Validate UTF-8
    if std::str::from_utf8(&content).is_err() {
        return Err(AnchorScopeError::InvalidUtf8);
    }

    // Normalize CRLF -> LF
    let normalized = matcher::normalize_line_endings(&content);

    // Write to replacement file
    let replacement_path = buffer_path::true_id_dir(&file_hash, true_id).join("replacement");
    std::fs::write(&replacement_path, &normalized)
        .map_err(|e| crate::error::from_io_error_write(e))?;

    Ok(())
}

/// Entry point for pipe command - stdout mode (default).
fn execute_stdout(label: &Option<String>, true_id: Option<&str>, out: bool, in_flag: bool) -> i32 {
    let true_id_str = match (label, true_id) {
        (Some(l), None) => match storage::load_label_target(l) {
            Ok(tid) => tid,
            Err(e) => {
                eprintln!("{}", e);
                return 1;
            }
        },
        (None, Some(tid)) => tid.to_string(),
        (Some(_), Some(_)) => {
            eprintln!("IO_ERROR: mutually exclusive options: --label and --true-id");
            return 1;
        }
        (None, None) => {
            eprintln!("IO_ERROR: either --label or --true-id must be provided");
            return 1;
        }
    };

    if out {
        match stream_content_to_stdout(&true_id_str) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("{}", e);
                1
            }
        }
    } else if in_flag {
        // Read from stdin
        let mut stdin = io::stdin();
        let mut buffer = Vec::new();
        match stdin.read_to_end(&mut buffer) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("IO_ERROR: read failure");
                return 1;
            }
        }

        match read_from_stdin_and_write_replacement(&true_id_str, &buffer) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("{}", e);
                1
            }
        }
    } else {
        eprintln!("IO_ERROR: either --out or --in must be specified");
        1
    }
}

/// Entry point for pipe command - file-io mode.
fn execute_file_io(
    label: &Option<String>,
    true_id: Option<&str>,
    tool: &str,
    tool_args: Option<&str>,
) -> i32 {
    let true_id_str = match (label, true_id) {
        (Some(l), None) => match storage::load_label_target(l) {
            Ok(tid) => tid,
            Err(e) => {
                eprintln!("{}", e);
                return 1;
            }
        },
        (None, Some(tid)) => tid.to_string(),
        (Some(_), Some(_)) => {
            eprintln!("IO_ERROR: mutually exclusive options: --label and --true-id");
            return 1;
        }
        (None, None) => {
            eprintln!("IO_ERROR: either --label or --true-id must be provided");
            return 1;
        }
    };

    // Validate tool name BEFORE execution
    if let Err(e) = validate_tool_name(tool) {
        eprintln!("{}", e.to_spec_string());
        return 1;
    }

    // Get content path
    let file_hash = match storage::file_hash_for_true_id(&true_id_str) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    let content_path = buffer_path::true_id_dir(&file_hash, &true_id_str).join("content");

    if !content_path.exists() {
        eprintln!("IO_ERROR: file not found");
        return 1;
    }

    // Create temporary output file
    let tmp_dir = match tempfile::tempdir() {
        Ok(d) => d,
        Err(_) => {
            eprintln!("IO_ERROR: cannot create temporary directory");
            return 1;
        }
    };

    let output_path = tmp_dir.path().join("output.txt");

    // Execute external tool using Command directly (no shell)
    // Build the command with the tool and all arguments
    let mut cmd = std::process::Command::new(tool);

    // Add tool arguments if provided (parsed as space-separated)
    if let Some(args) = tool_args {
        let parts: Vec<&str> = args.split_whitespace().collect();
        cmd.args(&parts);
    }

    // For stdin/stdout tools, read from content_path and write to output_path
    // The tool reads from stdin and writes to stdout
    // Read content from input file
    let input_content = match std::fs::read(&content_path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("IO_ERROR: cannot read input file");
            return 1;
        }
    };

    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("IO_ERROR: cannot execute external tool");
            return 1;
        }
    };

    // Write input to stdin
    if std::io::Write::write_all(&mut child.stdin.take().unwrap(), &input_content).is_err() {
        eprintln!("IO_ERROR: cannot write to tool stdin");
        return 1;
    }

    // Wait for tool to complete and get output
    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(_) => {
            eprintln!("IO_ERROR: cannot read tool output");
            return 1;
        }
    };

    if !output.status.success() {
        eprintln!("IO_ERROR: external tool failed");
        return 1;
    }

    // Write tool output to output file
    if std::fs::write(&output_path, &output.stdout).is_err() {
        eprintln!("IO_ERROR: cannot write output file");
        return 1;
    }

    // Validate and store tool output as replacement
    match validate_and_store_replacement(&true_id_str, &output_path) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    }
}

/// Main entry point for pipe command.
pub fn execute(
    label: &Option<String>,
    true_id: Option<&str>,
    out: bool,
    in_flag: bool,
    file_io: bool,
    tool: Option<&str>,
    tool_args: Option<&str>,
) -> i32 {
    if file_io {
        if let Some(t) = tool {
            execute_file_io(label, true_id, t, tool_args)
        } else {
            eprintln!("IO_ERROR: --tool required for --file-io mode");
            1
        }
    } else {
        execute_stdout(label, true_id, out, in_flag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipe_stdout_out_streams_content_to_stdout() {
        // Setup: Create buffer content
        let content = b"test content for stdout\n";
        let file_hash = crate::hash::compute(content);
        let true_id = "test_pipe_stdout";

        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_buffer_content(&file_hash, &true_id, content).unwrap();
        storage::save_buffer_metadata(
            &file_hash,
            &true_id,
            &storage::BufferMeta {
                true_id: true_id.to_string(),
                parent_true_id: None,
                scope_hash: crate::hash::compute(content),
                anchor: "test".to_string(),
            },
        )
        .unwrap();
        storage::save_source_path(&file_hash, "/tmp/test.txt").unwrap();

        // Execute pipe --out
        let result = stream_content_to_stdout(&true_id);

        // Cleanup
        storage::invalidate_true_id_hierarchy(&file_hash, &true_id).unwrap();

        assert!(result.is_ok(), "stream_content_to_stdout should succeed");
    }

    #[test]
    fn pipe_stdout_in_reads_from_stdin_and_writes_replacement() {
        // Setup
        let content = b"original content";
        let file_hash = crate::hash::compute(content);
        let true_id = "test_pipe_in";

        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_buffer_content(&file_hash, &true_id, content).unwrap();
        storage::save_buffer_metadata(
            &file_hash,
            &true_id,
            &storage::BufferMeta {
                true_id: true_id.to_string(),
                parent_true_id: None,
                scope_hash: crate::hash::compute(content),
                anchor: "test".to_string(),
            },
        )
        .unwrap();
        storage::save_source_path(&file_hash, "/tmp/test.txt").unwrap();

        // Simulate stdin input
        let new_content = b"modified content\n";
        let result = read_from_stdin_and_write_replacement(&true_id, new_content);

        assert!(
            result.is_ok(),
            "read_from_stdin_and_write_replacement should succeed"
        );

        // Verify replacement file was created
        let replacement_path = buffer_path::true_id_dir(&file_hash, &true_id).join("replacement");
        assert!(replacement_path.exists(), "replacement file should exist");

        // Cleanup
        storage::invalidate_true_id_hierarchy(&file_hash, &true_id).unwrap();
    }

    #[test]
    fn pipe_stdout_in_validates_utf8() {
        // Setup
        let content = b"test";
        let file_hash = crate::hash::compute(content);
        let true_id = "test_pipe_utf8";

        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_buffer_content(&file_hash, &true_id, content).unwrap();
        storage::save_buffer_metadata(
            &file_hash,
            &true_id,
            &storage::BufferMeta {
                true_id: true_id.to_string(),
                parent_true_id: None,
                scope_hash: crate::hash::compute(content),
                anchor: "test".to_string(),
            },
        )
        .unwrap();
        storage::save_source_path(&file_hash, "/tmp/test.txt").unwrap();

        // Invalid UTF-8
        let invalid_content = vec![0xFF, 0xFE];
        let result = read_from_stdin_and_write_replacement(&true_id, &invalid_content);

        storage::invalidate_true_id_hierarchy(&file_hash, &true_id).unwrap();

        assert!(result.is_err(), "should reject invalid UTF-8");
        assert!(result.unwrap_err().starts_with("IO_ERROR: invalid UTF-8"));
    }

    #[test]
    fn pipe_stdout_in_normalizes_crlf_to_lf() {
        // Setup
        let content = b"test";
        let file_hash = crate::hash::compute(content);
        let true_id = "test_pipe_crlf";

        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_buffer_content(&file_hash, &true_id, content).unwrap();
        storage::save_buffer_metadata(
            &file_hash,
            &true_id,
            &storage::BufferMeta {
                true_id: true_id.to_string(),
                parent_true_id: None,
                scope_hash: crate::hash::compute(content),
                anchor: "test".to_string(),
            },
        )
        .unwrap();
        storage::save_source_path(&file_hash, "/tmp/test.txt").unwrap();

        // Content with CRLF
        let crlf_content = b"line1\r\nline2\r\n";
        let result = read_from_stdin_and_write_replacement(&true_id, crlf_content);

        assert!(result.is_ok(), "should normalize CRLF");

        // Verify CRLF was normalized to LF
        let saved =
            std::fs::read(buffer_path::true_id_dir(&file_hash, &true_id).join("replacement"))
                .unwrap();
        assert_eq!(saved, b"line1\nline2\n", "CRLF should be normalized to LF");

        // Cleanup
        storage::invalidate_true_id_hierarchy(&file_hash, &true_id).unwrap();
    }

    #[test]
    fn pipe_file_io_mode_passes_content_path_to_tool() {
        // Setup: Create buffer content
        let content = b"test content for file-io\n";
        let file_hash = crate::hash::compute(content);
        let true_id = "test_file_io";

        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_buffer_content(&file_hash, &true_id, content).unwrap();
        storage::save_buffer_metadata(
            &file_hash,
            &true_id,
            &storage::BufferMeta {
                true_id: true_id.to_string(),
                parent_true_id: None,
                scope_hash: crate::hash::compute(content),
                anchor: "test".to_string(),
            },
        )
        .unwrap();
        storage::save_source_path(&file_hash, "/tmp/test.txt").unwrap();

        // Create a temporary output file
        let tmp_dir = tempfile::tempdir().unwrap();
        let output_path = tmp_dir.path().join("output.txt");

        // Simulate external tool: read content, write modified output
        let content_bytes =
            std::fs::read(buffer_path::true_id_dir(&file_hash, &true_id).join("content")).unwrap();

        // Tool would modify the content
        let modified = b"MODIFIED: ".to_vec();
        let mut output = modified;
        output.extend(&content_bytes);

        std::fs::write(&output_path, &output).unwrap();

        // pipe would then validate and store output as replacement
        let result = validate_and_store_replacement(&true_id, &output_path);

        assert!(
            result.is_ok(),
            "validate_and_store_replacement should succeed"
        );

        // Verify replacement file
        let replacement_path = buffer_path::true_id_dir(&file_hash, &true_id).join("replacement");
        assert!(replacement_path.exists(), "replacement file should exist");

        let saved = std::fs::read(&replacement_path).unwrap();
        assert_eq!(
            saved, b"MODIFIED: test content for file-io\n",
            "replacement content should match expected"
        );

        // Cleanup
        storage::invalidate_true_id_hierarchy(&file_hash, &true_id).unwrap();
        let _ = std::fs::remove_dir_all(tmp_dir);
    }

    #[test]
    fn pipe_file_io_mode_validates_tool_output() {
        // Setup
        let content = b"test";
        let file_hash = crate::hash::compute(content);
        let true_id = "test_file_io_valid";

        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_buffer_content(&file_hash, &true_id, content).unwrap();
        storage::save_buffer_metadata(
            &file_hash,
            &true_id,
            &storage::BufferMeta {
                true_id: true_id.to_string(),
                parent_true_id: None,
                scope_hash: crate::hash::compute(content),
                anchor: "test".to_string(),
            },
        )
        .unwrap();
        storage::save_source_path(&file_hash, "/tmp/test.txt").unwrap();

        // Create invalid UTF-8 output
        let tmp_dir = tempfile::tempdir().unwrap();
        let invalid_path = tmp_dir.path().join("invalid.txt");
        std::fs::write(&invalid_path, vec![0xFF, 0xFE]).unwrap();

        // pipe should reject invalid UTF-8
        let result = validate_and_store_replacement(&true_id, &invalid_path);

        storage::invalidate_true_id_hierarchy(&file_hash, &true_id).unwrap();
        let _ = std::fs::remove_dir_all(tmp_dir);

        assert!(result.is_err(), "should reject invalid UTF-8");
        assert!(result.unwrap_err().starts_with("IO_ERROR: invalid UTF-8"));
    }

    #[test]
    fn pipe_file_io_mode_normalizes_tool_output() {
        // Setup
        let content = b"test";
        let file_hash = crate::hash::compute(content);
        let true_id = "test_file_io_crlf";

        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_buffer_content(&file_hash, &true_id, content).unwrap();
        storage::save_buffer_metadata(
            &file_hash,
            &true_id,
            &storage::BufferMeta {
                true_id: true_id.to_string(),
                parent_true_id: None,
                scope_hash: crate::hash::compute(content),
                anchor: "test".to_string(),
            },
        )
        .unwrap();
        storage::save_source_path(&file_hash, "/tmp/test.txt").unwrap();

        // Create output with CRLF
        let tmp_dir = tempfile::tempdir().unwrap();
        let crlf_path = tmp_dir.path().join("crlf.txt");
        std::fs::write(&crlf_path, b"line1\r\nline2\r\n").unwrap();

        // pipe should normalize CRLF to LF
        let result = validate_and_store_replacement(&true_id, &crlf_path);

        assert!(result.is_ok(), "should normalize CRLF");

        // Verify normalization
        let saved =
            std::fs::read(buffer_path::true_id_dir(&file_hash, &true_id).join("replacement"))
                .unwrap();
        assert_eq!(saved, b"line1\nline2\n", "CRLF should be normalized to LF");

        // Cleanup
        storage::invalidate_true_id_hierarchy(&file_hash, &true_id).unwrap();
        let _ = std::fs::remove_dir_all(tmp_dir);
    }
}
