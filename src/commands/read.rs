use std::fs;

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
            0
        }
    }
}
