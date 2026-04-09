/// A single exact match found in the file.
pub struct Match {
    /// 1-based line number of the first byte of the anchor.
    pub start_line: usize,
    /// 1-based line number of the last byte of the anchor.
    pub end_line: usize,
    /// Byte offset of the match start in the normalized file.
    pub byte_start: usize,
    /// Byte offset one past the match end in the normalized file.
    pub byte_end: usize,
}

/// Errors returned by the matcher.
#[derive(Debug)]
pub enum MatchError {
    NoMatch,
    MultipleMatches(usize),
}

impl std::fmt::Display for MatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchError::NoMatch => write!(f, "NO_MATCH"),
            MatchError::MultipleMatches(n) => write!(f, "MULTIPLE_MATCHES ({})", n),
        }
    }
}

/// Normalize CRLF -> LF.
/// This is the ONLY implicit normalization permitted by the spec.
pub fn normalize_line_endings(raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'\r' && i + 1 < raw.len() && raw[i + 1] == b'\n' {
            // skip CR; LF will be copied on the next iteration
            i += 1;
        } else {
            out.push(raw[i]);
            i += 1;
        }
    }
    out
}

/// Count the 1-based line number at byte offset `pos` in `haystack`.
/// Lines are delimited by LF (normalization must have been applied first).
fn line_at(haystack: &[u8], pos: usize) -> usize {
    haystack[..pos].iter().filter(|&&b| b == b'\n').count() + 1
}

/// Find ALL exact (byte-level) occurrences of `needle` in `haystack`.
/// Includes overlapping matches.
fn find_all(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return vec![];
    }
    let mut positions = Vec::new();
    let limit = haystack.len() - needle.len();
    let mut i = 0;
    while i <= limit {
        if haystack[i..i + needle.len()] == *needle {
            positions.push(i);
        }
        i += 1; // always advance by 1 to allow overlapping matches
    }
    positions
}

/// Resolve anchor to a single unique match, or return an error.
/// Counts ALL exact byte-level matches. Overlapping matches are included.
pub fn resolve(haystack: &[u8], anchor: &[u8]) -> Result<Match, MatchError> {
    let positions = find_all(haystack, anchor);
    match positions.len() {
        0 => Err(MatchError::NoMatch),
        1 => {
            let byte_start = positions[0];
            let byte_end = byte_start + anchor.len();
            let start_line = line_at(haystack, byte_start);
            let end_line = line_at(haystack, byte_end.saturating_sub(1));
            Ok(Match {
                start_line,
                end_line,
                byte_start,
                byte_end,
            })
        }
        n => Err(MatchError::MultipleMatches(n)),
    }
}

/// Extract function body starting from function definition line.
/// Returns the byte range of the function body (including the function definition line).
/// This is used for nested anchors to limit search scope to the function.
pub fn extract_function_body(content: &[u8], start_line: usize) -> Option<std::ops::Range<usize>> {
    // Check if it's a function definition
    let normalized = normalize_line_endings(content);
    let lines: Vec<&[u8]> = normalized.split(|b| *b == b'\n').collect();
    
    if start_line == 0 || start_line > lines.len() {
        return None;
    }
    
    // Get the function definition line (0-indexed: start_line - 1)
    let func_def_line = lines.get(start_line - 1)?;
    
    // Only extract if it's a function definition
    let func_def_str = String::from_utf8_lossy(func_def_line);
    if !func_def_str.trim().starts_with("def ") {
        return None;
    }
    
    // Calculate the indentation level of the function definition
    let func_indent = count_leading_spaces(func_def_line);
    
    // Find the end of the function by looking for lines with < indentation
    let func_start_byte: usize = lines[..start_line - 1].iter().map(|l| l.len() + 1).sum();
    
    // Start from the next line after function definition
    let mut func_end_byte = func_start_byte;
    for i in start_line..lines.len() {
        let line = lines.get(i)?;
        let line_indent = count_leading_spaces(line);
        
        // If line is empty, always include it (it's part of the function body)
        // If line has < function indentation (and is not empty), function ended
        if line.is_empty() {
            // Empty line, include it
        } else if line_indent < func_indent {
            // Non-empty line with less indentation, function ended
            break;
        }
        
        func_end_byte += line.len() + 1;
    }
    
    // Clamp the range to the actual content length
    let content_len = content.len();
    let end = func_end_byte.min(content_len);
    
    eprintln!("DEBUG extract: func_start_byte={}, end={}, content_len={}, normalized_len={}", func_start_byte, end, content_len, normalized.len());
    
    if func_start_byte < end {
        Some(func_start_byte..end)
    } else {
        None
    }
}

/// Count leading spaces in a line
fn count_leading_spaces(line: &[u8]) -> usize {
    line.iter().take_while(|&&b| b == b' ' || b == b'\t').count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_function_body_simple() {
        let content = b"# Comment\n\ndef process_data():\n    for i in range(10):\n        print(i)\n    print(\"Done\")\n\ndef other():\n    pass\n";
        
        // Function starts at line 3 (1-indexed)
        let result = extract_function_body(content, 3);
        assert!(result.is_some(), "Should extract function body");
        
        let range = result.unwrap();
        let func_body = &content[range.clone()];
        let func_str = String::from_utf8_lossy(func_body);
        
        // Should include function definition and loop
        assert!(func_str.contains("def process_data()"), "Should contain function definition");
        assert!(func_str.contains("for i in range(10)"), "Should contain loop");
        assert!(func_str.contains("print(i)"), "Should contain print statement");
        assert!(func_str.contains("print(\"Done\")"), "Should contain final print");
        
        // Should NOT include the next function
        eprintln!("Extracted function body:\n{}", func_str);
        assert!(!func_str.contains("def other()"), "Should not contain next function");
    }

    #[test]
    fn test_extract_function_body_no_extraction_for_non_function() {
        let content = b"x = 1\nfor i in range(10):\n    print(i)\n";
        
        let result = extract_function_body(content, 2);
        assert!(result.is_none(), "Should return None for non-function anchor");
    }

    #[test]
    fn test_count_leading_spaces() {
        assert_eq!(count_leading_spaces(b"    indented"), 4);
        assert_eq!(count_leading_spaces(b"\tindented"), 1);
        assert_eq!(count_leading_spaces(b"no indent"), 0);
    }
}
