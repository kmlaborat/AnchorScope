/// A single exact match found in the normalized content.
pub struct Match {
    /// Byte offset of the match start (in normalized content).
    pub byte_start: usize,
    /// Byte offset one past the match end (in normalized content).
    pub byte_end: usize,
}

/// Errors returned by the matcher.
#[derive(Debug)]
pub enum MatchError {
    NoMatch,
    MultipleMatches,
}

impl std::fmt::Display for MatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchError::NoMatch => write!(f, "NO_MATCH"),
            MatchError::MultipleMatches => write!(f, "MULTIPLE_MATCHES"),
        }
    }
}

/// Normalize CRLF → LF in a byte slice.
///
/// Returns `(normalized_bytes, offset_map)` where `offset_map[j]` gives the
/// byte offset in the original (pre-normalization) content that corresponds
/// to byte `j` in the normalized content.
pub fn normalize_line_endings(raw: &[u8]) -> (Vec<u8>, Vec<usize>) {
    let mut out = Vec::with_capacity(raw.len());
    let mut offset_map = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'\r' && i + 1 < raw.len() && raw[i + 1] == b'\n' {
            out.push(b'\n');
            offset_map.push(i); // map to the \r position (start of \r\n pair) in original
            i += 2;
        } else {
            out.push(raw[i]);
            offset_map.push(i);
            i += 1;
        }
    }
    (out, offset_map)
}

/// Convert a byte range in normalized content to the corresponding byte range
/// in the original (pre-normalization) content.
///
/// `offset_map` maps each normalized index to the original index.
/// For CRLF→LF, the \n maps to the \r position. When the match includes
/// such a \n at the end, orig_end must be extended past the \r\n pair.
pub fn map_to_original(
    raw: &[u8],
    normalized: &[u8],
    offset_map: &[usize],
    norm_start: usize,
    norm_end: usize,
    raw_len: usize,
) -> (usize, usize) {
    let orig_start = offset_map[norm_start];
    let mut orig_end = if norm_end < offset_map.len() {
        offset_map[norm_end]
    } else {
        raw_len
    };

    // If the last matched byte in normalized is \n, and it came from \r\n,
    // extend orig_end to include the full \r\n pair.
    if norm_end > 0 {
        let last_matched = normalized[norm_end - 1];
        if last_matched == b'\n' && orig_end + 1 < raw.len()
            && raw[orig_end] == b'\r'
            && raw[orig_end + 1] == b'\n'
        {
            orig_end += 2;
        }
    }

    (orig_start, orig_end)
}

/// Verify that the mapped original offset matches the formula:
/// `original_offset == normalized_offset + number_of_CR_before_original_offset`
///
/// Returns true if the verification passes.
pub fn verify_offset(raw: &[u8], norm_offset: usize, orig_offset: usize) -> bool {
    let cr_count = raw[..orig_offset].iter().filter(|&&b| b == b'\r').count();
    orig_offset == norm_offset + cr_count
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
        i += 1;
    }
    positions
}

/// Resolve anchor to a single unique match in the given content, or return an error.
/// Counts ALL exact byte-level matches. Overlapping matches are included.
pub fn resolve(haystack: &[u8], anchor: &[u8]) -> Result<Match, MatchError> {
    let positions = find_all(haystack, anchor);
    match positions.len() {
        0 => Err(MatchError::NoMatch),
        1 => {
            let byte_start = positions[0];
            let byte_end = byte_start + anchor.len();
            Ok(Match { byte_start, byte_end })
        }
        _ => Err(MatchError::MultipleMatches),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_lf_only() {
        let raw = b"hello\nworld";
        let (normalized, offset_map) = normalize_line_endings(raw);
        assert_eq!(normalized.as_slice(), b"hello\nworld");
        // offset_map should be identity for LF-only content (11 bytes)
        assert_eq!(offset_map, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn normalize_crlf() {
        let raw = b"hello\r\nworld\r\n";
        let (normalized, offset_map) = normalize_line_endings(raw);
        assert_eq!(normalized.as_slice(), b"hello\nworld\n");
        // \n maps to \r position (start of \r\n pair)
        assert_eq!(offset_map, vec![0, 1, 2, 3, 4, 5, 7, 8, 9, 10, 11, 12]);
    }

    #[test]
    fn resolve_single_match() {
        let haystack = b"AAABBBCCC";
        let anchor = b"BBB";
        let m = resolve(haystack, anchor).unwrap();
        assert_eq!(m.byte_start, 3);
        assert_eq!(m.byte_end, 6);
    }

    #[test]
    fn resolve_no_match() {
        let haystack = b"AAABBBCCC";
        let anchor = b"XXX";
        assert!(matches!(resolve(haystack, anchor), Err(MatchError::NoMatch)));
    }

    #[test]
    fn resolve_multiple_matches() {
        let haystack = b"AAA BBB AAA";
        let anchor = b"AAA";
        assert!(matches!(
            resolve(haystack, anchor),
            Err(MatchError::MultipleMatches)
        ));
    }

    #[test]
    fn map_to_original_crlf_no_newline() {
        // "BBB" does NOT include \n
        let raw = b"AAA\r\nBBB\r\nCCC\r\n";
        let (normalized, offset_map) = normalize_line_endings(raw);
        // offset_map: [0,1,2,3,5,6,7,8,10,11,12,13]
        let (orig_start, orig_end) =
            map_to_original(raw, &normalized, &offset_map, 4, 7, raw.len());
        // orig_end = offset_map[7] = 8 (\r position)
        // last_matched = norm[6] = 'B' != \n → no adjustment
        // raw[5..8] = "BBB" ✅
        assert_eq!(orig_start, 5);
        assert_eq!(orig_end, 8);
        assert_eq!(&raw[orig_start..orig_end], b"BBB");
    }

    #[test]
    fn map_to_original_crlf_with_newline() {
        // "BBB\n" INCLUDES the \n (came from \r\n)
        let raw = b"AAA\r\nBBB\r\nCCC\r\n";
        let (normalized, offset_map) = normalize_line_endings(raw);
        let (orig_start, orig_end) =
            map_to_original(raw, &normalized, &offset_map, 4, 8, raw.len());
        // orig_end = offset_map[8] = 10
        // last_matched = norm[7] = \n
        // offset_map[7] = 8, raw[8]=\r, raw[9]=\n → extend: 8+2=10
        // But orig_end was already 10 (offset_map[8]). So no change needed.
        // raw[5..10] = "BBB\r\n" ✅
        assert_eq!(orig_start, 5);
        assert_eq!(orig_end, 10);
        assert_eq!(&raw[orig_start..orig_end], b"BBB\r\n");
    }

    #[test]
    fn verify_offset_formula_crlf() {
        // raw: A(0) A(1) A(2) \r(3) \n(4) B(5) B(6) B(7) \r(8) \n(9) C(10) C(11) C(12) \r(13) \n(14)
        let raw = b"AAA\r\nBBB\r\nCCC\r\n";

        // "BBB" start: orig=5, norm=4, CR before 5: 1 → 4+1=5 ✅
        assert!(verify_offset(raw, 4, 5));

        // "BBB" end: orig=8, norm=7, CR before 8: 1 → 7+1=8 ✅
        assert!(verify_offset(raw, 7, 8));

        // "CCC" start: orig=10, norm=8, CR before 10: 2 → 8+2=10 ✅
        assert!(verify_offset(raw, 8, 10));

        // "CCC" end: orig=13, norm=11, CR before 13: 2 → 11+2=13 ✅
        assert!(verify_offset(raw, 11, 13));
    }

    #[test]
    fn end_to_end_crlf_single_line() {
        let raw = b"AAA\r\nBBB\r\nCCC\r\n";
        let anchor = b"BBB";
        let (normalized, offset_map) = normalize_line_endings(raw);
        let anchor_norm = normalize_line_endings(anchor).0;
        let m = resolve(&normalized, &anchor_norm).unwrap();
        assert_eq!(m.byte_start, 4);
        assert_eq!(m.byte_end, 7);

        let (orig_start, orig_end) =
            map_to_original(raw, &normalized, &offset_map, m.byte_start, m.byte_end, raw.len());
        assert_eq!(orig_start, 5);
        assert_eq!(orig_end, 8);
        assert_eq!(&raw[orig_start..orig_end], b"BBB");
    }

    #[test]
    fn end_to_end_crlf_multiline() {
        let raw = b"HEADER\r\nfn foo() {\r\n    return 1;\r\n}\r\nFOOTER\r\n";
        let anchor = b"fn foo() {\n    return 1;\n}";
        let (normalized, offset_map) = normalize_line_endings(raw);
        let anchor_norm = normalize_line_endings(anchor).0;
        let m = resolve(&normalized, &anchor_norm).unwrap();
        // "fn foo() {\n    return 1;\n}" (26 bytes) at normalized 7..33
        assert_eq!(m.byte_start, 7);
        assert_eq!(m.byte_end, 33);

        let (orig_start, orig_end) =
            map_to_original(raw, &normalized, &offset_map, m.byte_start, m.byte_end, raw.len());
        // orig_start = offset_map[7] = 8
        // orig_end = offset_map[33] = 36
        // last_matched = norm[32] = '}'
        // raw[36] != \r (raw[36]=\n) → no adjustment
        // Wait, offset_map[33] for norm[33]=\n → maps to \r at 36.
        // Actually, let me check: norm[32]='}' → offset_map[32]=35.
        // norm[33]=\n → offset_map[33]=36 (\r position).
        // orig_end = 36. last_matched = norm[32] = '}' != \n → no adjustment.
        // raw[8..36] = "fn foo() {\r\n    return 1;\r\n}" ✅
        assert_eq!(orig_start, 8);
        assert_eq!(orig_end, 36);
        assert_eq!(&raw[orig_start..orig_end], b"fn foo() {\r\n    return 1;\r\n}");
    }

    #[test]
    fn end_to_end_crlf_multiline_with_trailing_newline() {
        // Same as above but anchor includes trailing \n
        let raw = b"HEADER\r\nfn foo() {\r\n    return 1;\r\n}\r\nFOOTER\r\n";
        let anchor = b"fn foo() {\n    return 1;\n}\n";
        let (normalized, offset_map) = normalize_line_endings(raw);
        let anchor_norm = normalize_line_endings(anchor).0;
        let m = resolve(&normalized, &anchor_norm).unwrap();
        // "fn foo() {\n    return 1;\n}\n" (27 bytes) at normalized 7..34
        assert_eq!(m.byte_start, 7);
        assert_eq!(m.byte_end, 34);

        let (orig_start, orig_end) =
            map_to_original(raw, &normalized, &offset_map, m.byte_start, m.byte_end, raw.len());
        // orig_end = offset_map[34] = 38
        // last_matched = norm[33] = \n
        // offset_map[33] = 36, raw[36]=\r, raw[37]=\n → extend: 36+2=38
        // orig_end was 38, adjusted to 38 (same) → raw[8..38] = "fn foo() {\r\n    return 1;\r\n}\r\n"
        assert_eq!(orig_start, 8);
        assert_eq!(orig_end, 38);
        assert_eq!(
            &raw[orig_start..orig_end],
            b"fn foo() {\r\n    return 1;\r\n}\r\n"
        );
    }

    #[test]
    fn end_to_end_mixed_line_endings() {
        // Mixed: "AAA\r\nBBB\nCCC\r\n"
        // raw: A(0)A(1)A(2)\r(3)\n(4)B(5)B(6)B(7)\n(8)C(9)C(10)C(11)\r(12)\n(13)
        let raw = b"AAA\r\nBBB\nCCC\r\n";
        let anchor = b"BBB";
        let (normalized, offset_map) = normalize_line_endings(raw);
        assert_eq!(normalized.as_slice(), b"AAA\nBBB\nCCC\n");

        let anchor_norm = normalize_line_endings(anchor).0;
        let m = resolve(&normalized, &anchor_norm).unwrap();
        assert_eq!(m.byte_start, 4);
        assert_eq!(m.byte_end, 7);

        let (orig_start, orig_end) =
            map_to_original(raw, &normalized, &offset_map, m.byte_start, m.byte_end, raw.len());
        assert_eq!(orig_start, 5);
        assert_eq!(orig_end, 8);
        assert_eq!(&raw[orig_start..orig_end], b"BBB");
    }

    #[test]
    fn end_to_end_mixed_multiline_anchor() {
        // Mixed: "HDR\r\nAAA\r\nBBB\nCCC\nFOOTER\r\n"
        // Anchor spans CRLF and LF: "AAA\nBBB\nCCC"
        let raw = b"HDR\r\nAAA\r\nBBB\nCCC\nFOOTER\r\n";
        let anchor = b"AAA\nBBB\nCCC";
        let (normalized, offset_map) = normalize_line_endings(raw);
        // normalized: "HDR\nAAA\nBBB\nCCC\nFOOTER\n"
        assert_eq!(normalized.as_slice(), b"HDR\nAAA\nBBB\nCCC\nFOOTER\n");

        let anchor_norm = normalize_line_endings(anchor).0;
        let m = resolve(&normalized, &anchor_norm).unwrap();
        // "AAA\nBBB\nCCC" at normalized 4..15
        assert_eq!(m.byte_start, 4);

        let (orig_start, orig_end) =
            map_to_original(raw, &normalized, &offset_map, m.byte_start, m.byte_end, raw.len());
        // The match includes \n at norm[6] (from \r\n) and \n at norm[10] (from LF only).
        // orig_start = offset_map[4] → position of 'A' in original
        // orig_end should be just after 'C' of CCC
        // raw[orig_start..orig_end] should be "AAA\r\nBBB\nCCC"
        assert_eq!(&raw[orig_start..orig_end], b"AAA\r\nBBB\nCCC");
    }
}
