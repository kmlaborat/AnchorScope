use crate::test_helpers::*;

#[test]
fn test_label_command_success() {
    // Setup: create file with known content
    let (_temp_dir, file_path) = create_temp_file(
        "fn main() {\n    println!(\"Hello\");\n}\n"
    );

    // Step 1: Use read to get internal label (hash)
    let output = run_anchorscope(&[
        "read",
        "--file", file_path.to_str().unwrap(),
        "--anchor", "fn main() {\n    println!(\"Hello\");\n}"
    ]);
    assert!(output.status.success());
    let result = parse_output(&String::from_utf8_lossy(&output.stdout));
    let internal_label = result.get("label").unwrap().clone();

    // Step 2: Call label command to assign human-readable name
    let output = run_anchorscope(&[
        "label",
        "--name", "main_function",
        "--internal-label", &internal_label
    ]);
    assert!(output.status.success(), "label should succeed, stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("OK: label 'main_function' defined"));
}

#[test]
fn test_label_unknown_internal() {
    // Try to label a non-existent internal label
    let output = run_anchorscope(&[
        "label",
        "--name", "test",
        "--internal-label", "0000000000000000"
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IO_ERROR: unknown internal label"));
}

#[test]
fn test_label_name_collision() {
    // Create a file and get two different anchors
    let (_temp_dir, file_path) = create_temp_file(
        "fn main() {\n    println!(\"Hello\");\n}\n\
         fn foo() {\n    println!(\"World\");\n}\n"
    );

    // First anchor: main
    let out1 = run_anchorscope(&[
        "read",
        "--file", file_path.to_str().unwrap(),
        "--anchor", "fn main() {\n    println!(\"Hello\");\n}"
    ]);
    assert!(out1.status.success());
    let res1 = parse_output(&String::from_utf8_lossy(&out1.stdout));
    let label1 = res1.get("label").unwrap().clone();

    // Assign name "func" to first anchor
    let out_label1 = run_anchorscope(&[
        "label",
        "--name", "func",
        "--internal-label", &label1
    ]);
    assert!(out_label1.status.success());

    // Second anchor: foo
    let out2 = run_anchorscope(&[
        "read",
        "--file", file_path.to_str().unwrap(),
        "--anchor", "fn foo() {\n    println!(\"World\");\n}"
    ]);
    assert!(out2.status.success());
    let res2 = parse_output(&String::from_utf8_lossy(&out2.stdout));
    let label2 = res2.get("label").unwrap().clone();

    // Try to assign same name "func" to different internal label -> should fail
    let out_label2 = run_anchorscope(&[
        "label",
        "--name", "func",
        "--internal-label", &label2
    ]);
    assert!(!out_label2.status.success());
    let stderr = String::from_utf8_lossy(&out_label2.stderr);
    assert!(stderr.contains("LABEL_EXISTS"));
}
