use xxhash_rust::xxh3::xxh3_64;

/// Compute xxh3_64 hash of bytes, return as lowercase hex string.
/// Deterministic: same bytes -> same output, always.
pub fn compute(bytes: &[u8]) -> String {
    let h = xxh3_64(bytes);
    format!("{:016x}", h)
}
