use crate::test_helpers::*;

#[test]
fn test_anchor_command_success() {
    // Setup: create file with known content
    let (temp_dir, file_path) = create_temp_file(
        "fn main() {\n    println!(\"Hello\");\n}\n\
         fn foo() {\n    println!(\"World\");\n}\n"
    );

    // Step 1: Use read to get hash of anchor region
    let output = run_anchorscope(&[
        "read",
        "--file", file_path.to_str().unwrap(),
        "--anchor", "fn main() {\n    println!(\"Hello\");\n}"
    ]);
    assert!(output.status.success());
    let result = parse_output(&String::from_utf8_lossy(&output.stdout));
    let hash = result.get("hash").unwrap().clone();

    // Step 2: Call anchor command with that hash
    let output = run_anchorscope(&[
        "anchor",
        "--file", file_path.to_str().unwrap(),
        "--label", "main_function",
        "--anchor", "fn main() {\n    println!(\"Hello\");\n}",
        "--expected-hash", &hash
    ]);
    assert!(output.status.success(), "anchor should succeed");
}
