# Spec Compliance Fixes - UTF-8 Validation & Error Message Alignment

> **REQUIRED SUB-SKILL:** Use the executing-plans skill to implement this plan task-by-task.

**Goal:** Fix two specification gaps: (1) add UTF-8 validation for buffer content loaded in label-mode, and (2) align custom error messages with SPEC §4.5 naming conventions.

**Architecture:** Two minimal, independent fixes:
1. In `src/commands/read.rs`, after loading buffer content for label-mode, validate UTF-8 before use.
2. In `src/commands/write.rs`, rename `AMBIGUOUS_REPLACEMENT` and `NO_REPLACEMENT` to match SPEC §4.5 format.

**Tech Stack:** Rust, AnchorScope v1.3.0 spec, existing test infrastructure.

---

## Task 1: Add UTF-8 validation for buffer content in label-mode

**Files:**
- Modify: `src/commands/read.rs:110-115` (add validation after loading buffer_content)
- Test: `tests/unit/true_id_computation.rs` (add test for invalid UTF-8 in label mode)

**Step 1: Write the failing test**

Add a test case that creates a buffer with invalid UTF-8 bytes and attempts to read it in label-mode.

```rust
#[test]
fn read_label_mode_rejects_invalid_utf8_buffer() {
    use crate::hash;
    use crate::storage;

    // Create a buffer with invalid UTF-8
    let raw_content = b"valid content";
    let file_hash = hash::compute(raw_content);
    let true_id = "invalid_utf8_test";

    // Save valid file content
    storage::save_file_content(&file_hash, raw_content).unwrap();
    storage::save_source_path(&file_hash, "/tmp/test.txt").unwrap();

    // Save a buffer with INVALID UTF-8 bytes
    let invalid_buffer = vec![0xFF, 0xFE, 0x00, 0x01];
    storage::save_buffer_content(&file_hash, true_id, &invalid_buffer).unwrap();

    let buffer_meta = storage::BufferMeta {
        true_id: true_id.to_string(),
        parent_true_id: None,
        scope_hash: hash::compute(&invalid_buffer),
        anchor: "test".to_string(),
    };
    storage::save_buffer_metadata(&file_hash, true_id, &buffer_meta).unwrap();

    // Save a valid label pointing to the invalid buffer
    storage::save_label_mapping("bad_utf8_label", true_id).unwrap();

    // Execute read with label pointing to invalid UTF-8 buffer
    let exit_code = crate::commands::read::execute("/tmp/test.txt", None, None, Some("bad_utf8_label"));

    // Cleanup
    storage::invalidate_label("bad_utf8_label");
    storage::invalidate_true_id_hierarchy(&file_hash, true_id).unwrap();

    // Should fail with IO_ERROR: invalid UTF-8
    assert_eq!(exit_code, 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test unit read_label_mode_rejects_invalid_utf8_buffer -v`
Expected: `test read_label_mode_rejects_invalid_utf8_buffer ... FAILED` (because validation is missing)

**Step 3: Implement UTF-8 validation in read.rs**

Add validation right after loading buffer content for label-mode:

```rust
// After line ~110 (after loading buffer_content for label mode)
let buffer_content = match storage::load_buffer_content(&file_hash, &true_id) {
    Ok(c) => c,
    Err(_) => {
        // Try nested location - find the parent that contains this true_id
        let content = match find_nested_buffer_content(&file_hash, &true_id) {
            Some(c) => c,
            None => {
                eprintln!("IO_ERROR: cannot load buffer content");
                return 1;
            }
        };
        content
    }
};

// Validate UTF-8 for buffer content per SPEC §2.2
if std::str::from_utf8(&buffer_content).is_err() {
    eprintln!("IO_ERROR: invalid UTF-8");
    return 1;
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test unit read_label_mode_rejects_invalid_utf8_buffer -v`
Expected: `test read_label_mode_rejects_invalid_utf8_buffer ... ok`

**Step 5: Run all tests to ensure no regressions**

Run: `cargo test --test unit -v`
Expected: All unit tests pass

Run: `cargo test --test integration -v`
Expected: All integration tests pass

**Step 6: Commit**

```bash
git add src/commands/read.rs tests/unit/true_id_computation.rs
git commit -m "fix: validate UTF-8 for buffer content in label-mode (SPEC §2.2)"
```

---

## Task 2: Align error messages with SPEC §4.5 naming conventions

**Files:**
- Modify: `src/commands/write.rs:55-57` (rename error messages)
- Test: `tests/integration/error_no_replacement_tests.rs` (update test expectations)

**Step 1: Update error message names**

Replace `AMBIGUOUS_REPLACEMENT` and `NO_REPLACEMENT` with SPEC-compliant names:

```rust
// Line ~55-57
// OLD:
if from_replacement && !replacement.is_empty() {
    eprintln!("AMBIGUOUS_REPLACEMENT");
    return 1;
}
if !from_replacement && replacement.is_empty() {
    eprintln!("NO_REPLACEMENT");
    return 1;
}

// NEW:
if from_replacement && !replacement.is_empty() {
    eprintln!("IO_ERROR: ambiguous replacement source");
    return 1;
}
if !from_replacement && replacement.is_empty() {
    eprintln!("IO_ERROR: no replacement provided");
    return 1;
}
```

**Step 2: Update tests to expect new error messages**

In `tests/integration/error_no_replacement_tests.rs`:

```rust
// Find where NO_REPLACEMENT is checked and update
assert!(output.contains("IO_ERROR: no replacement provided"));
```

In `tests/integration/error_multiple_matches_tests.rs` (if AMBIGUOUS_REPLACEMENT is tested):

```rust
assert!(output.contains("IO_ERROR: ambiguous replacement source"));
```

**Step 3: Run tests to verify error message changes**

Run: `cargo test --test integration error_no_replacement -v`
Expected: Tests pass with new error message format

Run: `cargo test --test integration error_ambiguous -v`
Expected: Tests pass (if ambiguous test exists)

**Step 4: Run all integration tests**

Run: `cargo test --test integration -v`
Expected: All integration tests pass

**Step 5: Commit**

```bash
git add src/commands/write.rs tests/integration/error_no_replacement_tests.rs
git commit -m "fix: align error messages with SPEC §4.5 naming conventions"
```

---

## Task 3: Verify spec compliance after fixes

**Files:**
- Review: `docs/SPEC.md` sections 2.2, 4.5
- Review: `src/commands/read.rs` and `src/commands/write.rs`

**Step 1: Manual spec review**

Review SPEC §2.2: "All inputs MUST be valid UTF-8"
- ✅ File content validation exists
- ✅ Anchor file content validation exists
- ✅ Inline replacement validation exists
- ✅ **Buffer content validation added** (Task 1)
- ✅ Replacement buffer content validation exists (pipe.rs)

Review SPEC §4.5: "Error message format"
- ✅ `IO_ERROR: invalid UTF-8`
- ✅ `IO_ERROR: ambiguous replacement source`
- ✅ `IO_ERROR: no replacement provided`
- ✅ `NO_MATCH`
- ✅ `MULTIPLE_MATCHES`
- ✅ `HASH_MISMATCH`
- ✅ `DUPLICATE_TRUE_ID`

**Step 2: Final test run**

Run: `cargo test -v`
Expected: All tests pass

**Step 3: Commit final review**

```bash
git add docs/SPEC.md
git commit -m "docs: verify spec compliance after UTF-8 validation fix"
```

---

## Summary of Changes

| Issue | Status | Impact |
|-------|--------|--------|
| Missing UTF-8 validation for buffer content | **FIXED** | Prevents invalid UTF-8 from propagating through label-mode operations |
| Non-spec-compliant error messages | **FIXED** | Aligns with SPEC §4.5 format requirements |
| Test coverage | **ADDED** | New unit test for invalid UTF-8 buffer content |

**Verification:** After both fixes, AnchorScope will be fully compliant with SPEC §2.2 and §4.5.

---

Plan complete and saved to `docs/plans/2026-04-12-spec-compliance-fixes.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**
