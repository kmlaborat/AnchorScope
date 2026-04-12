# AnchorScope SPEC Compliance Fixes

> **REQUIRED SUB-SKILL:** Use the executing-plans skill to implement this plan task-by-task.

**Goal:** Fix critical SPEC violations in the AnchorScope implementation to ensure deterministic behavior and cross-compatibility

**Architecture:** 
- Fix DUPLICATE_TRUE_ID check to only check within the current file_hash
- Remove duplicate metadata load in write command
- Ensure all SPEC requirements are met

**Tech Stack:** Rust, xxhash_rust, serde, clap

---

## Task 1: Add DUPLICATE_TRUE_ID test case

**Files:**
- Create: `tests/integration/duplicate_true_id_detection.rs`

**Step 1: Write the failing test**

This test should verify that:
1. When a True ID exists in multiple locations within the same file_hash directory
2. The write command with --label returns DUPLICATE_TRUE_ID

```rust
use crate::integration::test_helpers::{create_temp_file, save_buffer_structure};
use anchorscope::storage;

#[test]
fn write_duplicate_true_id_returns_error() {
    // Setup: Create a file with content
    let content = b"1234567890";
    let (temp_dir, file_path) = create_temp_file(content);
    let file_hash = crate::hash::compute(content);

    // Save file content to buffer
    storage::save_file_content(&file_hash, content).unwrap();
    storage::save_source_path(&file_hash, &file_path).unwrap();

    // Create two separate True ID directories with the same content
    // This simulates a bug where the same True ID was saved twice
    let true_id = "aaaaaaaaaaaaaaaa"; // Hardcoded for test
    
    // Create first directory
    let dir1 = crate::buffer_path::file_dir(&file_hash).join(true_id);
    std::fs::create_dir_all(&dir1).unwrap();
    std::fs::write(dir1.join("content"), content).unwrap();
    std::fs::write(
        dir1.join("metadata.json"),
        serde_json::json!({
            "true_id": true_id,
            "region_hash": crate::hash::compute(content),
            "anchor": "123",
            "parent_true_id": null
        }).to_string(),
    ).unwrap();

    // Create second directory (duplicate)
    let dir2 = crate::buffer_path::file_dir(&file_hash).join("temp_dup").join(true_id);
    std::fs::create_dir_all(&dir2).unwrap();
    std::fs::write(dir2.join("content"), content).unwrap();
    std::fs::write(
        dir2.join("metadata.json"),
        serde_json::json!({
            "true_id": true_id,
            "region_hash": crate::hash::compute(content),
            "anchor": "123",
            "parent_true_id": "temp_dup"
        }).to_string(),
    ).unwrap();

    // Save a label pointing to the duplicate True ID
    storage::save_label_mapping("test_label", true_id).unwrap();

    // Run write command with --label
    let result = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "write",
            &file_path,
            "--label",
            "test_label",
            "--replacement",
            "REPLACED",
        ])
        .current_dir(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .output()
        .unwrap();

    // Cleanup
    let _ = std::fs::remove_dir_all(temp_dir);
    storage::invalidate_label("test_label");
    storage::invalidate_true_id_hierarchy(&file_hash, true_id).unwrap();

    // Verify DUPLICATE_TRUE_ID error
    assert!(String::from_utf8_lossy(&result.stderr).contains("DUPLICATE_TRUE_ID"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test mod write_duplicate_true_id_returns_error -v`
Expected: Test fails because the DUPLICATE_TRUE_ID check hasn't been implemented yet

**Step 3: Commit**

```bash
git add tests/integration/duplicate_true_id_detection.rs
git commit -m "test: add DUPLICATE_TRUE_ID detection test"
```

---

## Task 2: Fix DUPLICATE_TRUE_ID check in write command

**Files:**
- Modify: `src/commands/write.rs:40-73`

**Step 1: Understand the current implementation**

The current implementation iterates over all file_hash directories, which is inefficient and may detect false positives.

**Step 2: Implement the fix**

```rust
// Replace lines 40-73 with:
// Get the file_hash for this True ID
let file_hash = match storage::file_hash_for_true_id(&true_id) {
    Ok(h) => h,
    Err(e) => {
        eprintln!("{}", e);
        return 1;
    }
};

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
```

**Step 3: Run test to verify it passes**

Run: `cargo test --test mod write_duplicate_true_id_returns_error -v`
Expected: Test passes

**Step 4: Commit**

```bash
git add src/commands/write.rs
git commit -m "fix: simplify DUPLICATE_TRUE_ID check to only current file_hash"
```

---

## Task 3: Remove duplicate metadata load in write command

**Files:**
- Modify: `src/commands/write.rs:78-86`

**Step 1: Identify the duplicate code**

Lines 78-86 show `load_anchor_metadata_by_true_id` called twice with the same arguments.

**Step 2: Remove the duplicate**

```rust
// Before (lines 74-86):
let meta = match crate::storage::load_anchor_metadata_by_true_id(&true_id) {
    Ok(m) => m,
    Err(e) => {
        eprintln!("{}", e);
        return 1;
    }
};

let meta = match crate::storage::load_anchor_metadata_by_true_id(&true_id) {
    Ok(m) => m,
    Err(e) => {
        eprintln!("{}", e);
        return 1;
    }
};

// After:
let meta = match crate::storage::load_anchor_metadata_by_true_id(&true_id) {
    Ok(m) => m,
    Err(e) => {
        eprintln!("{}", e);
        return 1;
    }
};
```

**Step 3: Run all tests to verify no regressions**

Run: `cargo test`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/commands/write.rs
git commit -m "fix: remove duplicate load_anchor_metadata_by_true_id call"
```

---

## Task 4: Verify True ID hash algorithm

**Files:**
- Review: `src/commands/read.rs:183-187`

**Step 1: Confirm the implementation matches SPEC**

The SPEC requires:
```
true_id = xxh3_64(hex(parent_scope_hash) || 0x5F || hex(child_scope_hash))
```

The current implementation uses:
```rust
crate::hash::compute(format!("{}_{}", parent_region_hash, region_hash).as_bytes())
```

The `format!("{}_{}", a, b)` produces a string where `_` is the underscore character (ASCII 0x5F).
When `.as_bytes()` is called, the underscore becomes byte 0x5F.

**Verification:** This is correct - the byte sequence produced is identical to `hex(parent_hash) || 0x5F || hex(child_hash)`.

**Step 2: Add comment to clarify**

Add a comment to explain that the format produces the correct byte sequence per SPEC:

```rust
// True ID per SPEC §3.2: xxh3_64(hex(parent_hash) || 0x5F || hex(child_hash))
// format! with "{}_{}" produces the same byte sequence as byte concatenation with underscore (0x5F)
let true_id = crate::hash::compute(format!("{}_{}", parent_region_hash, region_hash).as_bytes());
```

**Step 3: Commit**

```bash
git add src/commands/read.rs
git commit -m "docs: clarify True ID hash algorithm per SPEC"
```

---

## Task 5: Fix unused variable warnings

**Files:**
- Modify: `src/commands/read.rs:532`
- Modify: `src/commands/write.rs:78`
- Modify: `src/commands/read.rs:459`

**Step 1: Fix child_name warning**

```rust
// Before:
let child_name = entry.file_name();

// After:
let _child_name = entry.file_name();
```

**Step 2: Fix meta warning**

```rust
// Before:
let meta = match crate::storage::load_anchor_metadata_by_true_id(&true_id) {
    Ok(m) => m,
    Err(e) => {
        eprintln!("{}", e);
        return 1;
    }
};

// After:
let _meta = match crate::storage::load_anchor_metadata_by_true_id(&true_id) {
    Ok(m) => m,
    Err(e) => {
        eprintln!("{}", e);
        return 1;
    }
};
```

Wait, this is the duplicate code we're removing. After removing the duplicate, there should be no warning.

**Step 3: Fix inner_file_hash warning**

```rust
// Before:
let inner_file_hash = file_hash.clone(); // same file_hash used

// After:
let _inner_file_hash = file_hash.clone(); // same file_hash used
```

**Step 4: Run cargo fix**

```bash
cargo fix --bin anchorscope --allow-dirty
```

**Step 5: Commit**

```bash
git add src/commands/read.rs src/commands/write.rs
git commit -m "fix: address unused variable warnings"
```

---

## Task 6: Run final verification

**Files:**
- None

**Step 1: Run all tests**

```bash
cargo test
```

Expected: All 68 tests pass

**Step 2: Build without warnings**

```bash
cargo build --release
```

Expected: No warnings

**Step 3: Commit final changes**

```bash
git add .
git commit -m "fix: SPEC compliance fixes"
```

---

## Summary

This plan fixes the following issues:

| Issue | File | Fix |
|-------|------|-----|
| DUPLICATE_TRUE_ID check inefficient | `src/commands/write.rs:40-73` | Only check current file_hash |
| Duplicate metadata load | `src/commands/write.rs:78-86` | Remove duplicate |
| Unused variables | Multiple | Add underscore prefix or remove |

All tests should pass after these fixes.
