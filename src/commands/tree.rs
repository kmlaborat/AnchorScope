use crate::buffer_path;
use std::path::PathBuf;

/// Tree: Display current buffer structure.
/// Shows the hierarchical structure of anchors in the Anchor Buffer.
pub fn execute(file_path: &str) -> i32 {
    // Compute file hash from the file
    let raw = match std::fs::read(file_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", crate::map_io_error_read(e));
            return 1;
        }
    };

    // Validate UTF-8
    if std::str::from_utf8(&raw).is_err() {
        eprintln!("IO_ERROR: invalid UTF-8");
        return 1;
    }

    let normalized = crate::matcher::normalize_line_endings(&raw);

    // Compute file hash
    let file_hash = crate::hash::compute(&normalized);
    
    // Display root
    let source_path = PathBuf::from(file_path)
        .canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| file_path.to_string());
    
    println!("{}  ({})", file_hash, source_path);
    
    // Show aliases for this file hash
    show_aliases(&file_hash, buffer_path::labels_dir());
    
    0
}

/// Display all aliases and their True IDs
fn show_aliases(_file_hash: &str, labels_dir: std::path::PathBuf) {
    if let Ok(entries) = std::fs::read_dir(&labels_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                let alias = name_str.trim_end_matches(".json");
                
                // Load the label mapping
                if let Ok(label_content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(label_meta) = serde_json::from_str::<crate::storage::LabelMeta>(&label_content) {
                        let true_id = &label_meta.true_id;
                        println!("└── {}  [{}]", true_id, alias);
                    }
                }
            }
        }
    }
}
