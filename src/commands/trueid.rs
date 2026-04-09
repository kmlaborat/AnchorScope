use crate::trueid;

/// TrueId: Compute and output True ID for an anchor.
/// Used for nested anchoring where parent context is needed.
pub fn execute(
    file_path: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    parent_true_id: Option<&str>,
) -> i32 {
    // Load anchor bytes
    let anchor_bytes = match crate::load_anchor(anchor, anchor_file) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    // Read and normalize file content
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

    // Resolve anchor
    let m = match crate::matcher::resolve(&normalized, &anchor_bytes) {
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
        Ok(m) => m,
    };

    // Compute region hash
    let region = &normalized[m.byte_start..m.byte_end];
    let region_hash = crate::hash::compute(region);

    // Compute True ID
    let true_id = trueid::compute(file_path, &region_hash, parent_true_id);

    // Output
    println!("true_id={}", true_id);
    println!("region_hash={}", region_hash);
    println!("start_line={}", m.start_line);
    println!("end_line={}", m.end_line);
    
    0
}
