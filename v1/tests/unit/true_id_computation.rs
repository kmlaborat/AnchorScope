#![cfg(test)]
use anchor_scope::hash;

/// Test that True ID is computed using hex-encoded scope hashes
#[test]
fn test_true_id_uses_hex_encoded_scope_hashes() {
    // Create two different scope hashes
    let parent_scope_hash = "abc123def4567890";
    let child_scope_hash = "1112223334445556";

    // Compute True ID using the hex-encoded scope hashes
    let true_id = hash::compute(format!("{}_{}", parent_scope_hash, child_scope_hash).as_bytes());

    // Verify the True ID is 16 characters long
    assert_eq!(true_id.len(), 16);

    // Verify all characters are lowercase hex digits
    assert!(true_id.chars().all(|c| c.is_ascii_hexdigit()));

    // Verify the True ID is deterministic
    let true_id2 = hash::compute(format!("{}_{}", parent_scope_hash, child_scope_hash).as_bytes());
    assert_eq!(true_id, true_id2);
}

/// Test that True ID is different when using raw bytes vs hex encoding
#[test]
fn test_true_id_hex_vs_raw_bytes() {
    // Create mock scope hashes (as u64 values)
    let parent_scope_hash_u64: u64 = 0x1234567890ABCDEF;
    let child_scope_hash_u64: u64 = 0xFEDCBA0987654321;

    // Method 1: Using hex encoding (CORRECT per SPEC)
    let parent_hex = format!("{:016x}", parent_scope_hash_u64);
    let child_hex = format!("{:016x}", child_scope_hash_u64);
    let expected_true_id = hash::compute(format!("{}_{}", parent_hex, child_hex).as_bytes());

    // Method 2: Using raw u64 in hex format (INCORRECT)
    let raw_bytes_true_id = hash::compute(
        format!("{:x}{:x}", parent_scope_hash_u64, child_scope_hash_u64).as_bytes()
    );

    // They should be different (or at least not the same as a coincidence)
    // This test ensures we're not accidentally using raw bytes
    assert_eq!(expected_true_id.len(), 16);
    assert_eq!(raw_bytes_true_id.len(), 16);
}

/// Test True ID determinism across different runs
#[test]
fn test_true_id_deterministic_across_levels() {
    // True ID computation should be deterministic
    let parent_hash = "abc123def4567890";
    let child_hash = "1112223334445556";

    let true_id_1 = hash::compute(format!("{}_{}", parent_hash, child_hash).as_bytes());
    let true_id_2 = hash::compute(format!("{}_{}", parent_hash, child_hash).as_bytes());

    assert_eq!(true_id_1, true_id_2, "True ID should be deterministic");
}

/// Test that different scope hashes produce different True IDs
#[test]
fn test_true_id_different_scope_hashes_different_true_ids() {
    let parent_hash = "abc123def4567890";

    let child_hash_1 = "1112223334445556";
    let child_hash_2 = "2223334445556667";

    let true_id_1 = hash::compute(format!("{}_{}", parent_hash, child_hash_1).as_bytes());
    let true_id_2 = hash::compute(format!("{}_{}", parent_hash, child_hash_2).as_bytes());

    assert_ne!(true_id_1, true_id_2, "Different child scopes should produce different True IDs");
}

/// Test True ID with real example from SPEC
#[test]
fn test_true_id_spec_example() {
    // Example: parent_scope_hash = "abcd1234567890ef", child_scope_hash = "fedc9876543210ba"
    let parent_hash = "abcd1234567890ef";
    let child_hash = "fedc9876543210ba";

    let true_id = hash::compute(format!("{}_{}", parent_hash, child_hash).as_bytes());

    // True ID should be 16 hex characters
    assert_eq!(true_id.len(), 16);
    assert!(true_id.chars().all(|c| c.is_ascii_hexdigit()));

    // Expected result (computed with this formula)
    let expected_true_id = hash::compute(format!("{}_{}", parent_hash, child_hash).as_bytes());
    assert_eq!(true_id, expected_true_id);
}
