use std::fs;

/// Execute the read command.
/// Returns 0 on success, 1 on error.
pub fn execute(
    file: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
) -> i32 {
    // Load anchor bytes (raw, not yet normalized)
    let anchor_raw = match load_anchor(anchor, anchor_file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    // Call library API
    match anchorscope::read(file, &anchor_raw) {
        Err(e) => {
            eprintln!("{}", e);
            1
        }
        Ok(result) => {
            let content = std::str::from_utf8(&result.content).unwrap();
            println!("scope_hash={}", result.scope_hash);
            println!("content={}", content);
            0
        }
    }
}

/// Load anchor from either inline or file source.
/// Returns raw anchor bytes (not normalized).
fn load_anchor(anchor: Option<&str>, anchor_file: Option<&str>) -> Result<Vec<u8>, String> {
    match (anchor, anchor_file) {
        (None, None) => Err("ERROR: either --anchor or --anchor-file must be provided".to_string()),
        (Some(a), None) => {
            if a.is_empty() {
                Err("NO_MATCH".to_string())
            } else {
                Ok(a.as_bytes().to_vec())
            }
        }
        (None, Some(path)) => {
            let content =
                fs::read(path).map_err(|e| format!("IO_ERROR: {}", e))?;
            if std::str::from_utf8(&content).is_err() {
                return Err("IO_ERROR: invalid UTF-8".to_string());
            }
            let s = String::from_utf8(content).unwrap();
            if s.is_empty() {
                return Err("NO_MATCH".to_string());
            }
            Ok(s.into_bytes())
        }
        (Some(_), Some(_)) => {
            Err("ERROR: --anchor and --anchor-file are mutually exclusive".to_string())
        }
    }
}
