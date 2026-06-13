use crate::test_helpers::*;
use std::path::PathBuf;

fn anchorscope_temp_dir() -> PathBuf {
    std::env::temp_dir().join("anchorscope")
}

#[test]
fn test_anchor_and_label_files_created() {
    let (_temp_dir, file_path) = create_temp_file("Hello\nWorld\n");

    let output = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "Hello",
    ]);
    assert!(
        output.status.success(),
        "read failed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result = parse_output(&String::from_utf8_lossy(&output.stdout));
    let label_hash = result.get("label").unwrap().clone();

    let anchor_file = anchorscope_temp_dir()
        .join("anchors")
        .join(format!("{}.json", label_hash));
    assert!(
        anchor_file.exists(),
        "anchor metadata should exist after read"
    );

    run_anchorscope(&["label", "--name", "greeting", "--true-id", &label_hash]);

    let label_file = anchorscope_temp_dir().join("labels").join("greeting.json");
    assert!(
        label_file.exists(),
        "label mapping should exist after label command"
    );
}

#[test]
fn test_write_using_label_invalidates_files() {
    let (_temp_dir, file_path) = create_temp_file("Hello\nWorld\n");

    let out = run_anchorscope(&[
        "read",
        "--file",
        file_path.to_str().unwrap(),
        "--anchor",
        "Hello",
    ]);
    assert!(out.status.success());
    let result = parse_output(&String::from_utf8_lossy(&out.stdout));
    let label_hash = result.get("label").unwrap().clone(); // The label is the scope hash for v1.1.0 compat

    let label_out = run_anchorscope(&["label", "--name", "greet", "--true-id", &label_hash]);
    assert!(label_out.status.success());

    let anchor_file = anchorscope_temp_dir()
        .join("anchors")
        .join(format!("{}.json", label_hash));
    let label_file = anchorscope_temp_dir().join("labels").join("greet.json");

    assert!(anchor_file.exists());
    assert!(label_file.exists());

    let write_out = run_anchorscope(&[
        "write",
        "--label",
        "greet",
        "--replacement",
        "Hi",
        "--file",
        file_path.to_str().unwrap(),
    ]);
    assert!(
        write_out.status.success(),
        "write failed: {}",
        String::from_utf8_lossy(&write_out.stderr)
    );

    let anchor_file = anchorscope_temp_dir()
        .join("anchors")
        .join(format!("{}.json", label_hash));
    let label_file = anchorscope_temp_dir().join("labels").join("greet.json");

    assert!(
        !anchor_file.exists(),
        "anchor file should be invalidated after write"
    );
    assert!(
        !label_file.exists(),
        "label file should be invalidated after write"
    );
}
