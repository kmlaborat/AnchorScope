use crate::test_helpers::*;

#[test]
fn tree_shows_buffer_structure_for_file() {
    // Setup: Create a file and read an anchor to create a buffer
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
    
    // Run tree with --file
    let output = run_anchorscope(&[
        "tree",
        "--file",
        file_path.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    // Should contain a True ID (16-char hex) in the output
    // The output format is: <file_hash>  (<file_path>)\n├── <true_id>  [<alias>]
    let true_id_pattern: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains("──"))
        .map(|line| {
            line.split("──")
                .nth(1)
                .unwrap_or("")
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("")
        })
        .collect();
    
    assert!(
        !true_id_pattern.is_empty(),
        "Should have at least one buffer entry"
    );
    
    // Verify the True ID is 16 hex characters
    let true_id = true_id_pattern[0];
    assert_eq!(true_id.len(), 16, "True ID should be 16 characters");
    assert!(
        true_id.chars().all(|c| c.is_ascii_hexdigit()),
        "True ID should be all hex digits"
    );
}

#[test]
fn tree_with_nonexistent_file_shows_error() {
    let output = run_anchorscope(&[
        "tree",
        "--file",
        "/nonexistent/path/file.txt",
    ]);
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IO_ERROR") || stderr.contains("file not found"));
}

#[test]
fn tree_shows_nested_buffer_structure_with_label() {
    // Setup: Create nested buffers using label pattern
    // Use Python-style content to trigger full function extraction
    let content = "def foo():\n    x\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    
    // Read outer function (starts with "def " so full body is extracted)
    let read1 = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "def foo()",
    ]);
    assert!(read1.status.success());
    let read_stdout1 = String::from_utf8(read1.stdout).unwrap();
    let result1 = parse_output(&read_stdout1);
    let true_id1 = result1.get("true_id").unwrap().clone();
    
    // Create label for outer function
    let label_output = run_anchorscope(&[
        "label",
        "--name",
        "foo_func",
        "--true-id",
        &true_id1,
    ]);
    assert!(label_output.status.success());
    
    // Read nested anchor using label (full function body is available)
    let read2 = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--label",
        "foo_func",
        "--anchor",
        "x",
    ]);
    assert!(read2.status.success());
    
    // Run tree with --file
    let output = run_anchorscope(&[
        "tree",
        "--file",
        file_path.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    // Should show the nested structure with True IDs
    // Extract all True IDs from the output
    let tree_ids: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains("──"))
        .map(|line| {
            line.split("──")
                .nth(1)
                .unwrap_or("")
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("")
        })
        .filter(|s| !s.is_empty())
        .collect();
    
    // Should have at least 2 entries (outer + nested)
    assert!(
        tree_ids.len() >= 2,
        "Should have at least 2 buffer entries, got: {:?}",
        tree_ids
    );
    
    // The first True ID should be in the output
    assert!(
        tree_ids.contains(&true_id1.as_str()),
        "First True ID {} should be in tree output: {}",
        true_id1,
        stdout
    );
}
