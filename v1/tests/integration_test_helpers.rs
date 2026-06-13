use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Creates a temporary file with the given content.
/// Returns the TempDir (to keep it alive) and the PathBuf to the file.
pub fn create_temp_file(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    std::fs::write(&file_path, content).expect("failed to write temp file");
    (temp_dir, file_path)
}

/// Reads the entire contents of a file into a String.
pub fn read_file(path: &PathBuf) -> String {
    std::fs::read_to_string(path).expect("failed to read file")
}

/// Runs the anchorscope binary with the given arguments.
/// Returns the process Output.
pub fn run_anchorscope(args: &[&str]) -> std::process::Output {
    // Build first to ensure binary is up to date
    let build_output = Command::new("cargo")
        .arg("build")
        .arg("--quiet")
        .arg("--bin")
        .arg("anchorscope")
        .output()
        .expect("failed to build anchorscope");

    if !build_output.status.success() {
        panic!(
            "Build failed: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Run the built binary directly
    let binary_path = std::path::Path::new("target")
        .join("debug")
        .join("anchorscope");
    let output = Command::new(&binary_path)
        .args(args)
        .output()
        .expect("failed to execute anchorscope");
    output
}

/// Parses anchorscope output (key=value per line) into a HashMap.
/// Supports multi-line values by treating lines without '=' as continuations
/// of the most recent key.
pub fn parse_output(output: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut current_key: Option<String> = None;
    for line in output.lines() {
        if let Some(equal_index) = line.find('=') {
            let key = line[..equal_index].to_string();
            let value = line[equal_index + 1..].to_string();
            map.insert(key.clone(), value);
            current_key = Some(key);
        } else if let Some(key) = &current_key {
            // Continuation line: append with newline
            let existing = map.get_mut(key).unwrap();
            existing.push('\n');
            existing.push_str(line);
        }
    }
    map
}
