use crate::storage;

/// Label: assign a human-readable name to an internal label (from read output).
/// Verifies that the internal label exists in the anchor store, then saves the mapping.
pub fn execute(name: &str, internal_label: &str) -> i32 {
    // Verify internal label exists in anchors store
    if storage::load_anchor_metadata(internal_label).is_err() {
        eprintln!("IO_ERROR: unknown internal label: {}", internal_label);
        return 1;
    }
    // Save label mapping (name -> internal_label)
    if let Err(e) = storage::save_label_mapping(name, internal_label) {
        eprintln!("{}", e);
        return 1;
    }
    println!("OK: label '{}' defined", name);
    0
}
