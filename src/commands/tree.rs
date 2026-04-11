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

    // Display buffer structure recursively
    show_buffer_hierarchy(&buffer_path::file_dir(&file_hash), "", &file_hash);

    0
}

/// Recursively display buffer hierarchy with proper indentation
fn show_buffer_hierarchy(dir: &std::path::Path, prefix: &str, file_hash: &str) {
    // Show True IDs in this directory
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut true_ids: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();

        // Sort for consistent output
        true_ids.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        for (i, entry) in true_ids.iter().enumerate() {
            let is_last = i == true_ids.len() - 1;
            let current_prefix = if is_last { "└── " } else { "├── " };
            let next_prefix = if is_last { "    " } else { "│   " };

            let true_id_name = entry.file_name().to_string_lossy().to_string();
            let true_id_str = &true_id_name;

            // Check if there's an alias for this True ID
            let alias = load_alias_for_true_id(file_hash, true_id_str);

            println!("{}{}{}  [{}]", prefix, current_prefix, true_id_str, alias);

            // Recursively show nested True IDs
            let nested_dir = entry.path();
            show_buffer_hierarchy(
                &nested_dir,
                &format!("{}{}", prefix, next_prefix),
                file_hash,
            );
        }
    }
}

/// Load alias for a True ID from labels directory
fn load_alias_for_true_id(_file_hash: &str, true_id: &str) -> String {
    // Search all label files for matching true_id
    if let Ok(entries) = std::fs::read_dir(buffer_path::labels_dir()) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(label_meta) =
                        serde_json::from_str::<crate::storage::LabelMeta>(&content)
                    {
                        if label_meta.true_id == true_id {
                            let name = entry.file_name();
                            return name.to_string_lossy().trim_end_matches(".json").to_string();
                        }
                    }
                }
            }
        }
    }
    true_id.to_string()
}
