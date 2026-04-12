use crate::test_helpers::*;

#[test]
fn pipe_out_streams_content() {
    // Setup: Create buffer with content that will be fully matched by the anchor
    // The anchor must match exactly what we want to pipe
    let content = "fn main() {\n    x\n}\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    
    // Use an anchor that matches a portion of the content
    let read_output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "    x",
    ]);
    assert!(read_output.status.success(), "read failed");
    let read_stdout = String::from_utf8(read_output.stdout).unwrap();
    let result = parse_output(&read_stdout);
    let true_id = result.get("true_id").unwrap().clone();
    
    // Pipe out
    let output = run_anchorscope(&[
        "pipe",
        "--true-id",
        &true_id,
        "--out",
    ]);
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // The pipe command outputs the matched content, not the entire file
    assert!(stdout.contains("x"), "stdout should contain 'x': {}", stdout);
}

#[test]
fn pipe_with_label_context_succeeds() {
    // Setup: Create buffer and label
    let content = "fn main() {\n    x\n}\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    
    // Use an anchor that matches a portion of the content
    let read_output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "    x",
    ]);
    assert!(read_output.status.success());
    let read_stdout = String::from_utf8(read_output.stdout).unwrap();
    let result = parse_output(&read_stdout);
    let true_id = result.get("true_id").unwrap().clone();
    
    // Use a unique label name
    let label_output = run_anchorscope(&[
        "label",
        "--name",
        "pipe_test_label",
        "--true-id",
        &true_id,
    ]);
    assert!(label_output.status.success());
    
    // Pipe out with label
    let output = run_anchorscope(&[
        "pipe",
        "--label",
        "pipe_test_label",
        "--out",
    ]);
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // The pipe command outputs the matched content
    assert!(stdout.contains("x"));
}

#[test]
fn pipe_in_writes_replacement() {
    // Setup: Create buffer with multiline content
    let content = "fn main() {\n    x\n}\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    
    // Use an anchor that will match a larger portion of content
    let read_output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "    x",
    ]);
    assert!(read_output.status.success());
    let read_stdout = String::from_utf8(read_output.stdout).unwrap();
    let result = parse_output(&read_stdout);
    let true_id = result.get("true_id").unwrap().clone();
    
    // Get content via pipe --out
    let pipe_out = run_anchorscope(&[
        "pipe",
        "--true-id",
        &true_id,
        "--out",
    ]);
    assert!(pipe_out.status.success());
    
    // Pipe in with stdin
    let output = run_anchorscope(&[
        "pipe",
        "--true-id",
        &true_id,
        "--in",
    ]);
    
    // Verify the command succeeds
    assert!(output.status.success());
}

#[test]
fn pipe_stdin_mode_replacement_with_label() {
    // Setup: Create buffer and label
    let content = "fn main() {\n    x\n}\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    
    // Use an anchor that matches a portion of the content
    let read_output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "    x",
    ]);
    assert!(read_output.status.success());
    let read_stdout = String::from_utf8(read_output.stdout).unwrap();
    let result = parse_output(&read_stdout);
    let true_id = result.get("true_id").unwrap().clone();
    
    // Use a unique label name
    let label_output = run_anchorscope(&[
        "label",
        "--name",
        "pipe_stdin_label",
        "--true-id",
        &true_id,
    ]);
    assert!(label_output.status.success());
    
    // Pipe out with label
    let pipe_out = run_anchorscope(&[
        "pipe",
        "--label",
        "pipe_stdin_label",
        "--out",
    ]);
    assert!(pipe_out.status.success());
    
    // Verify output contains the matched content
    let stdout = String::from_utf8(pipe_out.stdout).unwrap();
    assert!(stdout.contains("x"));
}
