use std::fs;
use crate::storage;
use crate::trueid;

/// Read: locate anchor, print location + hash. Exit 0 on success, 1 on error.
pub fn execute(
    file_path: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
) -> i32 {
    let raw = match fs::read(file_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", crate::map_io_error_read(e));
            return 1;
        }
    };

    // Enforce UTF-8 validity per SPEC
    if std::str::from_utf8(&raw).is_err() {
        eprintln!("IO_ERROR: invalid UTF-8");
        return 1;
    }

    let normalized = crate::matcher::normalize_line_endings(&raw);
    let anchor_bytes = match crate::load_anchor(anchor, anchor_file) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    match crate::matcher::resolve(&normalized, &anchor_bytes) {
        Err(e) => {
            eprintln!("{}", e);
            1
        }
        Ok(m) => {
            let region = &normalized[m.byte_start..m.byte_end];
            let h = crate::hash::compute(region);
            // Output is machine-readable: one key=value per line.
            println!("start_line={}", m.start_line);
            println!("end_line={}", m.end_line);
            println!("hash={}", h);
            println!("content={}", String::from_utf8_lossy(region));
            // Save anchor metadata and output internal label
            let anchor_str = String::from_utf8_lossy(&anchor_bytes).to_string();
            let meta = storage::AnchorMeta {
                file: file_path.to_string(),
                anchor: anchor_str,
                hash: h.clone(),
                line_range: (m.start_line, m.end_line),
            };
            if let Err(e) = storage::save_anchor_metadata(&meta) {
                eprintln!("IO_ERROR: cannot save anchor metadata: {}", e);
                return 1;
            }
            // For v1.2.0: output both label (v1.1.0 compat) and true_id
            // label is the region hash for v1.1.0 compatibility
            // true_id is computed as xxh3_64(file_hash + "_" + region_hash) for root level
            let true_id = trueid::compute(file_path, &h, None);
            println!("label={}", h);
            println!("true_id={}", true_id);
            0
        }
    }
}
