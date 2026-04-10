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

/// Extract function body from Python file.
/// Returns the full function including the definition line.
pub fn extract_function_body(content: &[u8], anchor_start: usize, anchor_end: usize) -> Vec<u8> {
    let normalized = normalize_line_endings(content);
    let mut start = anchor_start;
    let mut end = anchor_end;
    
    // Find the start of the function definition (look backwards for "def ")
    let search_start = if start >= 10 { start - 10 } else { 0 };
    let search_region = &normalized[search_start..start];
    if let Some(def_pos) = find_reverse(search_region, b"def ") {
        // Find the beginning of the line containing "def "
        let actual_def_pos = search_start + def_pos;
        if actual_def_pos > 0 {
            // Find the previous newline
            let prev_newline = normalized[..actual_def_pos].iter().rposition(|&b| b == b'\n');
            start = prev_newline.map(|p| p + 1).unwrap_or(0);
        } else {
            start = actual_def_pos;
        }
    }
    
    // Find the end of the function (look for next def statement or end of file)
    let mut current_pos = end;
    while current_pos < normalized.len() {
        // Find the next newline
        let next_newline = normalized[current_pos..].iter().position(|&b| b == b'\n');
        if next_newline.is_none() {
            // End of file
            end = normalized.len();
            break;
        }
        let newline_pos = current_pos + next_newline.unwrap();
        let line_end = newline_pos + 1;
        
        // Check if the next line starts with "def " (next function)
        let next_line_start = line_end;
        if next_line_start >= normalized.len() {
            end = normalized.len();
            break;
        }
        
        // Skip blank lines and comments
        let remaining = &normalized[next_line_start..];
        if !remaining.is_empty() {
            // Find first non-whitespace character
            let first_non_ws = remaining.iter().skip_while(|&&b| b == b' ' || b == b'\t' || b == b'\n').position(|&b| b != b'\n');
            if let Some(pos) = first_non_ws {
                let check_pos = next_line_start + pos;
                // Check if line starts with "def "
                if check_pos + 4 <= normalized.len() && &normalized[check_pos..check_pos + 4] == b"def " {
                    // Found next function definition, so this is the end of current function
                    end = current_pos;
                    break;
                }
            }
        }
        
        current_pos = line_end;
    }
    
    normalized[start..end].to_vec()
}

/// Find the last occurrence of `needle` in `haystack`, returning the byte offset.
fn find_reverse(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    let limit = haystack.len() - needle.len();
    let mut i = limit;
    loop {
        if haystack[i..i + needle.len()] == *needle {
            return Some(i);
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    None
}
