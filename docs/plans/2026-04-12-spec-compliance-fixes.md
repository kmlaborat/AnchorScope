# SPEC Compliance Fixes Implementation Plan

> **REQUIRED SUB-SKILL:** Use the executing-plans skill to implement this plan task-by-task.

**Goal:** Fix SPEC compliance gaps identified in code review: (1) Register missing write_from_replacement_tests, (2) Clean up unused code causing build warnings, (3) Add NO_REPLACEMENT integration test

**Architecture:** 
- Register the existing write_from_replacement_tests module in tests/integration/mod.rs
- Add #[allow(dead_code)] or document unused functions in config.rs and matcher.rs
- Add integration test for NO_REPLACEMENT error case

**Tech Stack:** Rust 1.74, `thiserror`, `clap`, `serial_test`

---

### Task 1: Register missing write_from_replacement_tests

**Files:**
- Modify: `tests/integration/mod.rs`

**Step 1: Add the missing module declaration**

```rust
// Current mod.rs ends at line 20 with write_success_tests
// Add after write_success_tests (before closing bracket):

#[cfg(test)]
mod write_from_replacement_tests;
```

**Step 2: Run tests to verify they are now discovered**

Run: `cargo test write_from_replacement -- --list`
Expected: 3 tests found:
- `integration::write_from_replacement_tests::test_write_from_replacement_uses_buffer_content`
- `integration::write_from_replacement_tests::test_write_from_replacement_fails_without_label`
- `integration::write_from_replacement_tests::test_write_replacement_conflict_returns_ambiguous_replacement`

**Step 3: Run the new tests**

Run: `cargo test write_from_replacement -- --nocapture`
Expected: 3 tests pass

**Step 4: Verify all tests still pass**

Run: `cargo test`
Expected: 84 tests pass (81 existing + 3 new)

**Step 5: Commit**

```bash
git add tests/integration/mod.rs
git commit -m "feat: register write_from_replacement_tests module"
```

---

### Task 2: Clean up unused code warnings in config.rs

**Files:**
- Modify: `src/config.rs`

**Step 1: Verify warnings exist**

Run: `cargo build 2>&1 | grep "unused function" | grep "config"`
Expected: Shows warnings for `max_file_size` and `max_nesting_depth`

**Step 2: Add #[allow(dead_code)] to unused security functions**

In `src/config.rs`, the security module already has `#[allow(dead_code)]` on both functions. Verify it's present:

```rust
// Line ~23-45 should have:
pub mod security {
    use std::env;

    /// Maximum file size (default 100MB)
    /// Note: Currently unused but kept for future security configuration
    #[allow(dead_code)]  // ADD if missing
    pub fn max_file_size() -> u64 {
        // ... existing implementation ...
    }

    /// Maximum nesting depth (default 100)
    /// Note: Currently unused but kept for future security configuration
    #[allow(dead_code)]  // ADD if missing
    pub fn max_nesting_depth() -> usize {
        // ... existing implementation ...
    }
    // ... rest of file ...
}
```

**Step 3: Run build to verify warnings are gone**

Run: `cargo build 2>&1 | grep "unused function" | grep "config"`
Expected: No output (no warnings)

**Step 4: Run clippy to verify**

Run: `cargo clippy --package anchorscope --message-format short 2>&1 | grep config`
Expected: No dead_code warnings

**Step 5: Commit**

```bash
git add src/config.rs
git commit -m "chore: add #[allow(dead_code)] to security config functions"
```

---

### Task 3: Clean up unused code warnings in matcher.rs

**Files:**
- Modify: `src/matcher.rs`

**Step 1: Verify warnings exist**

Run: `cargo build 2>&1 | grep "unused function" | grep matcher`
Expected: Shows warning for `normalize_line_endings_in_place`

**Step 2: Add #[allow(dead_code)] to unused function**

```rust
// Line ~35 in matcher.rs:
// Before:
pub fn normalize_line_endings_in_place(buffer: &mut Vec<u8>) -> &[u8] {

// After:
/// Normalize CRLF -> LF in place (used by some tests)
#[allow(dead_code)]
pub fn normalize_line_endings_in_place(buffer: &mut Vec<u8>) -> &[u8] {
```

**Step 3: Run build to verify warnings are gone**

Run: `cargo build 2>&1 | grep "unused function" | grep matcher`
Expected: No output (no warnings)

**Step 4: Run clippy to verify**

Run: `cargo clippy --package anchorscope --message-format short 2>&1 | grep matcher`
Expected: No dead_code warnings

**Step 5: Commit**

```bash
git add src/matcher.rs
git commit -m "chore: add #[allow(dead_code)] to normalize_line_endings_in_place"
```

---

### Task 4: Clean up unused code warnings in storage.rs

**Files:**
- Modify: `src/storage.rs`

**Step 1: Verify warnings exist**

Run: `cargo build 2>&1 | grep "unused function" | grep storage`
Expected: Shows warnings for `find_buffer_content`, `file_hash_for_true_id_opt`, `true_id_exists`

**Step 2: Add #[allow(dead_code)] to unused public functions**

```rust
// Line ~199 in storage.rs:
// Before:
pub fn find_buffer_content(file_hash: &str, true_id: &str) -> Result<Vec<u8>, AnchorScopeError> {

// After:
/// Find buffer content for a true_id by searching all directory levels.
/// Public for external use; currently unused in codebase.
#[allow(dead_code)]
pub fn find_buffer_content(file_hash: &str, true_id: &str) -> Result<Vec<u8>, AnchorScopeError> {
```

```rust
// Line ~238 in storage.rs:
// Before:
pub fn file_hash_for_true_id_opt(true_id: &str) -> Result<Option<String>, AmbiguousAnchorError> {

// After:
/// Find file_hash containing a given true_id, returning None if not found.
/// Public for external use; currently unused in codebase.
#[allow(dead_code)]
pub fn file_hash_for_true_id_opt(true_id: &str) -> Result<Option<String>, AmbiguousAnchorError> {
```

```rust
// Line ~619 in storage.rs:
// Before:
pub fn true_id_exists(file_hash: &str, true_id: &str) -> bool {

// After:
/// Check if a true_id exists in the buffer (flat or nested locations).
/// Public for external use; currently unused in codebase.
#[allow(dead_code)]
pub fn true_id_exists(file_hash: &str, true_id: &str) -> bool {
```

**Step 3: Run build to verify warnings are gone**

Run: `cargo build 2>&1 | grep "unused function" | grep storage`
Expected: No output (no warnings)

**Step 4: Run clippy to verify**

Run: `cargo clippy --package anchorscope --message-format short 2>&1 | grep storage`
Expected: No dead_code warnings

**Step 5: Commit**

```bash
git add src/storage.rs
git commit -m "chore: add #[allow(dead_code)] to unused storage functions"
```

---

### Task 5: Add NO_REPLACEMENT integration test

**Files:**
- Create: `tests/integration/error_no_replacement_tests.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod error_no_replacement_tests {
    use serial_test::serial;
    use crate::storage;
    use crate::hash;

    #[test]
    #[serial]
    fn write_no_replacement_returns_error() {
        // Setup: Create a test file
        let content = b"test content";
        let file_hash = hash::compute(content);
        let source_path = std::env::temp_dir().join("test_no_replacement.txt");

        // Save file content
        std::fs::write(&source_path, content).unwrap();
        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_source_path(&file_hash, source_path.to_str().unwrap()).unwrap();

        // Try to write without --replacement and without --from-replacement
        // This should fail with NO_REPLACEMENT error
        let exit_code = crate::commands::write::execute(
            source_path.to_str().unwrap(),
            Some("test"),
            None,
            Some(&hash::compute(content)),
            None,
            "",  // empty replacement
            false,  // from_replacement = false
        );

        assert_eq!(exit_code, 1, "write should fail without replacement");
        
        // Cleanup
        storage::invalidate_true_id_hierarchy(&file_hash, &file_hash).unwrap();
        let _ = std::fs::remove_file(&source_path);
    }

    #[test]
    #[serial]
    fn write_no_replacement_without_anchor_returns_error() {
        // Setup: Create a test file
        let content = b"test content";
        let file_hash = hash::compute(content);
        let source_path = std::env::temp_dir().join("test_no_replacement2.txt");

        // Save file content
        std::fs::write(&source_path, content).unwrap();
        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_source_path(&file_hash, source_path.to_str().unwrap()).unwrap();

        // Try to write without anchor (which implies no --replacement)
        // This should fail with NO_REPLACEMENT error
        let exit_code = crate::commands::write::execute(
            source_path.to_str().unwrap(),
            None,  // no anchor
            None,
            Some(&hash::compute(content)),
            None,
            "",  // empty replacement
            false,  // from_replacement = false
        );

        assert_eq!(exit_code, 1, "write should fail without anchor and replacement");
        
        // Cleanup
        storage::invalidate_true_id_hierarchy(&file_hash, &file_hash).unwrap();
        let _ = std::fs::remove_file(&source_path);
    }

    #[test]
    #[serial]
    fn write_with_from_replacement_without_label_returns_error() {
        // Setup: Create a test file
        let content = b"test content";
        let file_hash = hash::compute(content);
        let source_path = std::env::temp_dir().join("test_from_replacement_no_label.txt");

        // Save file content
        std::fs::write(&source_path, content).unwrap();
        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_source_path(&file_hash, source_path.to_str().unwrap()).unwrap();

        // Try to use --from-replacement without --label (should fail)
        let exit_code = crate::commands::write::execute(
            source_path.to_str().unwrap(),
            Some("test"),
            None,
            Some(&hash::compute(content)),
            None,  // no label
            "",  // replacement ignored
            true,  // from_replacement = true
        );

        assert_eq!(exit_code, 1, "write should fail with from_replacement but no label");
        
        // Cleanup
        storage::invalidate_true_id_hierarchy(&file_hash, &file_hash).unwrap();
        let _ = std::fs::remove_file(&source_path);
    }
}
```

**Step 2: Register the test module**

```rust
// In tests/integration/mod.rs, add after error_nomatch_tests:

#[cfg(test)]
mod error_no_replacement_tests;
```

**Step 3: Run tests to verify they fail initially**

Run: `cargo test write_no_replacement -- --nocapture`
Expected: Tests fail because the error handling may not output the expected message

**Step 4: Implement error handling (if needed)**

The error handling is already in `src/commands/write.rs`:
```rust
// Line ~48-50 in write.rs:
if !from_replacement && replacement.is_empty() {
    eprintln!("NO_REPLACEMENT");
    return 1;
}
```

Verify this is working by checking the output.

**Step 5: Run tests again to verify they pass**

Run: `cargo test write_no_replacement -- --nocapture`
Expected: 3 tests pass

**Step 6: Run all tests to verify no regressions**

Run: `cargo test`
Expected: 87 tests pass (84 existing + 3 new)

**Step 7: Commit**

```bash
git add tests/integration/error_no_replacement_tests.rs tests/integration/mod.rs
git commit -m "test: add NO_REPLACEMENT error integration tests"
```

---

### Task 6: Final verification

**Files:**
- No code changes needed

**Step 1: Run clippy**

Run: `cargo clippy --package anchorscope --message-format short`
Expected: No warnings

**Step 2: Run all tests**

Run: `cargo test`
Expected: 87 tests pass

**Step 3: Build with verbose warnings**

Run: `cargo build --all-targets 2>&1 | grep -E "warning:" | grep -v "unused variable: _"`
Expected: No warnings (or only expected unused variable warnings with underscore prefix)

**Step 4: Format code**

Run: `cargo fmt`
Expected: No changes needed

**Step 5: Final commit**

```bash
git add .
git commit -m "chore: run cargo clippy and cargo fmt for code quality"
```

---

## All tasks complete

Run `cargo test` to verify all 87 tests pass before merging.
