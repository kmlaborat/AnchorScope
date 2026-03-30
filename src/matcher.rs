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
