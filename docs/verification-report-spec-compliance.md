# SPEC Compliance Verification Report

> **Date:** 2026-04-11
> **Version:** AnchorScope v1.3.0
> **Status:** All 68 tests pass

## Executive Summary

This report documents SPEC compliance issues identified in the AnchorScope implementation and the fixes applied to ensure full compliance with the AnchorScope Specification v1.3.0.

## Test Results

```
Unit tests:        21 passed, 0 failed
Integration tests: 47 passed, 0 failed
Total:             68 passed, 0 failed
```

## Issues Found and Fixed

### Issue 1: `write.rs --from-replacement` uses file path instead of file_hash

**Location:** `src/commands/write.rs:65-78`

**Original Code:**
```rust
let meta = match crate::storage::load_anchor_metadata_by_true_id(&true_id) {
    Ok(m) => m,
    Err(e) => {
        eprintln!("{}", e);
        return 1;
    }
};
let rep_bytes = if from_replacement {
    match crate::storage::load_replacement_content(&meta.file, &true_id) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    }
}
```

**Problem:**
- `meta.file` is an absolute file path (from `source_path`), not a `file_hash`
- `load_replacement_content` expects `file_hash` as the first parameter
- The replacement file is stored at `{file_hash}/{true_id}/replacement`, not using the file path

**SPEC Requirement:**
Per SPEC §4.2 Directory Structure:
```
{TMPDIR}/anchorscope/
└── {file_hash}/
    └── {true_id}/
        └── replacement  ← output from external tool (created by `pipe`, consumed by `write`)
```

**Fix Applied:**
```rust
let meta = match crate::storage::load_anchor_metadata_by_true_id(&true_id) {
    Ok(m) => m,
    Err(e) => {
        eprintln!("{}", e);
        return 1;
    }
};

// Get file_hash for this true_id (required for loading replacement content)
let file_hash_or_error = storage::file_hash_for_true_id(&true_id);

let rep_bytes = if from_replacement {
    match file_hash_or_error {
        Ok(file_hash) => {
            match crate::storage::load_replacement_content(&file_hash, &true_id) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
            }
        }
        Err(_) => {
            // v1.1.0 format anchor - no replacement file exists
            eprintln!("IO_ERROR: --from-replacement not supported for v1.1.0 format anchors");
            return 1;
        }
    }
} else {
    crate::matcher::normalize_line_endings(replacement.as_bytes())
};
```

**Impact:**
- The `--from-replacement` flag now correctly loads replacement content from the buffer
- Properly handles v1.1.0 format anchors (stored in `anchors/` directory) by rejecting `--from-replacement` since those anchors don't have replacement files

---

### Issue 2: DUPLICATE_TRUE_ID check inefficient

**Location:** `src/commands/write.rs:36-61`

**Original Code:**
```rust
// Find all file_hash directories where this true_id exists, then check each for duplicates
let temp_dir = std::env::temp_dir();
let anchorscope_dir = temp_dir.join("anchorscope");
let mut duplicate_check_error: Option<String> = None;

if let Ok(entries) = std::fs::read_dir(&anchorscope_dir) {
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            let file_hash = entry.file_name();
            let file_hash_str = file_hash.to_string_lossy();
            
            if let Ok(entries) = std::fs::read_dir(&anchorscope_dir) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        let file_hash = entry.file_name();
                        let file_hash_str = file_hash.to_string_lossy();
                        
                        if file_hash_str == "anchors" || file_hash_str == "labels" {
                            continue;
                        }
                        
                        let (found, count) = crate::storage::file_hash_exists_in_dir_with_count(
                            &buffer_path::file_dir(&file_hash_str),
                            &true_id
                        );
                        if found && count > 1 {
                            match storage::check_duplicate_true_id_in_file_hash(&file_hash_str, &true_id) {
                                Ok(_) => {}
                                Err(_) => {
                                    duplicate_check_error = Some("DUPLICATE_TRUE_ID".to_string());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

**Problem:**
- The code iterated over ALL file_hash directories
- This was inefficient and could produce false positives
- Per SPEC §3.2 Duplicate True ID: "Detection scope is limited to the `{file_hash}` directory of the current operation"

**SPEC Requirement:**
Per SPEC §3.2:
> Duplicate True ID
> Although True ID collisions are statistically rare (xxh3_64 is 64-bit), they are
> theoretically possible. If the same True ID is found at multiple locations within
> the **same `{file_hash}` directory**, the system **MUST** terminate immediately with:
> ```
> DUPLICATE_TRUE_ID
> ```
> Detection scope is limited to the `{file_hash}` directory of the current operation.

**Fix Applied:**
```rust
// Check for DUPLICATE_TRUE_ID per SPEC: same true_id in multiple locations within the same file_hash directory
// Only check if the true_id exists in the buffer (not for old-format v1.1.0 anchors)
if let Ok(file_hash) = storage::file_hash_for_true_id(&true_id) {
    // Only check for duplicates within this file_hash
    match storage::check_duplicate_true_id_in_file_hash(&file_hash, &true_id) {
        Ok(_) => {
            // Single location - OK
        }
        Err(_) => {
            eprintln!("DUPLICATE_TRUE_ID");
            return 1;
        }
    }
}
// If file_hash_for_true_id fails, the true_id doesn't exist in the buffer
// (e.g., old v1.1.0 format), so skip the duplicate check
```

**Impact:**
- More efficient - only checks the relevant file_hash directory
- Correctly implements SPEC requirement that detection is limited to the current file_hash

---

### Issue 3: Unused variables in read.rs

**Location:** Multiple locations in `src/commands/read.rs`

**Warnings:**
```
warning: unused variable: `child_name`
   --> src\commands\read.rs:532:21
    |
532 |                 let child_name = entry.file_name();

warning: unused variable: `inner_file_hash`
   --> src\commands\read.rs:459:13
    |
459 |         let inner_file_hash = file_hash.clone();
```

**Fix Applied:**
```rust
// Changed from:
let child_name = entry.file_name();
// To:
let _ = entry.file_name();

// Changed from:
let inner_file_hash = file_hash.clone();
// To:
let _inner_file_hash = file_hash.clone();
```

---

## Verification Steps

1. **Run all tests:**
   ```bash
   cargo test
   ```
   Result: All 68 tests pass

2. **Build without warnings:**
   ```bash
   cargo build --release
   ```
   Result: No warnings (except for test warnings that are acceptable)

3. **Manual verification of fixes:**
   - Verified that `--from-replacement` now correctly loads from `{file_hash}/{true_id}/replacement`
   - Verified that DUPLICATE_TRUE_ID check only examines the current file_hash
   - Verified backward compatibility with v1.1.0 format anchors

---

## Conclusion

All identified SPEC compliance issues have been fixed and verified:

1. ✅ `write.rs --from-replacement` now correctly uses `file_hash` to locate replacement files
2. ✅ DUPLICATE_TRUE_ID check is efficient and limited to the current file_hash directory
3. ✅ Unused variables in `read.rs` have been fixed
4. ✅ All 68 tests pass
5. ✅ Backward compatibility with v1.1.0 format anchors is maintained

The implementation now fully complies with the AnchorScope Specification v1.3.0.
