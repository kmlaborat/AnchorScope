use crate::storage;

/// Label: assign a human-readable name (alias) to a True ID.
///
/// Creates {TMPDIR}/anchorscope/labels/<name>.json containing the true_id.
/// This file points to the anchor metadata stored at
/// {TMPDIR}/anchorscope/{file_hash}/{true_id}/
///
/// Uses strict match-based error handling (no ? operator).
/// Error messages conform to SPEC §4.5.
pub fn execute(name: &str, true_id: &str) -> i32 {
    // For v1.2.0: Check if the true_id exists in either:
    // 1. Old location: {TMPDIR}/anchorscope/anchors/{true_id}.json (v1.1.0 format)
    // 2. New location: {TMPDIR}/anchorscope/{file_hash}/{true_id}/content
    
    let temp_dir = std::env::temp_dir().join("anchorscope");
    let anchors_dir = temp_dir.join("anchors");
    let true_id_json = anchors_dir.join(format!("{}.json", true_id));
    
    // Check old location first (v1.1.0 compatibility)
    let exists_in_old_location = true_id_json.exists();
    
    // If not found, check new buffer locations
    let exists_in_new_location = if exists_in_old_location {
        true
    } else if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .any(|e| {
                let file_hash_dir = e.path();
                let true_id_dir = file_hash_dir.join(true_id);
                let content_path = true_id_dir.join("content");
                content_path.exists()
            })
    } else {
        false
    };
    
    if !exists_in_old_location && !exists_in_new_location {
        eprintln!("IO_ERROR: buffer metadata for true_id '{}' not found", true_id);
        return 1;
    }
    
    // Validate arguments
    if name.is_empty() {
        eprintln!("IO_ERROR: label name must not be empty");
        return 1;
    }

    if true_id.is_empty() {
        eprintln!("IO_ERROR: true_id must not be empty");
        return 1;
    }

    // Check if label file already exists
    match storage::load_label_target(name) {
        Ok(existing_true_id) => {
            if existing_true_id != true_id {
                eprintln!("LABEL_EXISTS: label '{}' already points to a different true_id '{}'", name, existing_true_id);
                return 1;
            }
            // Same true_id, allow (idempotent)
        }
        Err(ref msg) if msg.starts_with("IO_ERROR: file not found") => {
            // Label doesn't exist, proceed to create
        }
        Err(ref msg) if msg.starts_with("IO_ERROR:") => {
            eprintln!("{}", msg);
            return 1;
        }
        Err(ref msg) => {
            eprintln!("IO_ERROR: {}", msg);
            return 1;
        }
    }

    // Save the label mapping
    match storage::save_label_mapping(name, true_id) {
        Ok(()) => {
            println!("OK: alias '{}' defined for true_id '{}'", name, true_id);
            0
        }
        Err(ref msg) if msg.starts_with("LABEL_EXISTS:") => {
            eprintln!("{}", msg);
            return 1;
        }
        Err(ref msg) if msg.starts_with("IO_ERROR:") => {
            eprintln!("{}", msg);
            return 1;
        }
        Err(ref msg) => {
            eprintln!("IO_ERROR: {}", msg);
            return 1;
        }
    }
}
