use crate::storage;

/// Label: assign a human-readable name to an internal label (from read output).
///
/// Creates `{TEMP}/anchorscope/labels/<name>.json` containing the internal_label hash.
/// This file points to the auto-generated anchor metadata at
/// `{TEMP}/anchorscope/anchors/<hash>.json` created by `read`.
///
/// Uses strict match-based error handling (no ? operator).
/// Error messages conform to SPEC §4.5.
pub fn execute(name: &str, internal_label: &str) -> i32 {
    // Validate arguments per SPEC §2.5: empty anchors are invalid
    if name.is_empty() {
        eprintln!("IO_ERROR: label name must not be empty");
        return 1;
    }

    if internal_label.is_empty() {
        eprintln!("IO_ERROR: internal label must not be empty");
        return 1;
    }

    // Verify internal label exists in the anchor store
    match storage::load_anchor_metadata(internal_label) {
        Ok(_) => {
            // Anchor metadata found — proceed to create the human-readable mapping
        }
        Err(ref msg) if msg.starts_with("IO_ERROR: anchor metadata not found") => {
            let actual_hash = internal_label;
            eprintln!("IO_ERROR: anchor metadata for hash '{}' not found. Run 'read' first.", actual_hash);
            return 1;
        }
        Err(ref msg) if msg.starts_with("IO_ERROR:") => {
            // Map to SPEC §4.5 deterministic error format
            eprintln!("{}", msg);
            return 1;
        }
        Err(ref msg) => {
            eprintln!("IO_ERROR: {}", msg);
            return 1;
        }
    }

    // Save the label mapping in {TEMP}/anchorscope/labels/<name>.json
    match storage::save_label_mapping(name, internal_label) {
        Ok(()) => {
            println!("OK: label '{}' defined", name);
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
