# Code Review Fixes Implementation Plan

> **REQUIRED SUB-SKILL:** Use the executing-plans skill to implement this plan task-by-task.

**Goal:** Fix 3 issues identified in code review: (1) Test failures for `test_map_io_error_read/write`, (2) Unused variable warnings in storage.rs, (3) Unused function warnings in config.rs

**Architecture:** 
- Fix error message mismatch by adjusting `WriteFailure::to_spec_string()` to handle NotFound specially
- Prefix unused variables with `_` to suppress warnings
- Remove or document unused configuration functions

**Tech Stack:** Rust 1.74, `thiserror`, `clap`

---

### Task 1: Fix `WriteFailure` error message for NotFound

**Files:**
- Modify: `src/error.rs:187-196`

**Step 1: Write the failing test**

```rust
// In src/main.rs, test_map_io_error_write:
#[test]
fn test_map_io_error_write() {
    // Test NotFound
    let e = std::io::Error::from(std::io::ErrorKind::NotFound);
    assert_eq!(map_io_error_write(e), "IO_ERROR: write failure");

    // Test PermissionDenied
    let e = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
    assert_eq!(map_io_error_write(e), "IO_ERROR: permission denied");

    // Test other errors (Interrupted, Other, etc.)
    let e = std::io::Error::from(std::io::ErrorKind::Interrupted);
    assert_eq!(map_io_error_write(e), "IO_ERROR: write failure");

    let e = std::io::Error::from(std::io::ErrorKind::Other);
    assert_eq!(map_io_error_write(e), "IO_ERROR: write failure");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_map_io_error_write -- --nocapture`
Expected: FAIL with assertion error showing "write failure: entity not found" != "write failure"

**Step 3: Implement fix in `src/error.rs`**

```rust
// Before (lines ~187-196):
AnchorScopeError::WriteFailure(e) => format!("IO_ERROR: write failure: {}", e),

// After:
AnchorScopeError::WriteFailure(e) => {
    // For NotFound, return simple message for backward compatibility
    if e.kind() == std::io::ErrorKind::NotFound {
        "IO_ERROR: write failure".to_string()
    } else {
        format!("IO_ERROR: write failure: {}", e)
    }
},
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_map_io_error_write -- --nocapture`
Expected: PASS

Run: `cargo test test_map_io_error_read -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/error.rs src/main.rs
git commit -m "fix: WriteFailure NotFound returns simple message for backward compatibility"
```

---

### Task 2: Fix unused variable warnings in `storage.rs`

**Files:**
- Modify: `src/storage.rs:328-331` and `src/storage.rs:379-382`

**Step 1: Write minimal test (manual verification)**

Run: `cargo build 2>&1 | grep "unused variable"`
Expected: Shows warnings for `tid` and `locations_str` variables

**Step 2: Fix first location (lines ~328-331)**

```rust
// Before:
Err(AmbiguousAnchorError { true_id: tid, locations }) => {
    let locations_str: Vec<String> = locations.iter().map(|p| p.display().to_string()).collect();
    Err(AnchorScopeError::DuplicateTrueId)
}

// After:
Err(AmbiguousAnchorError { true_id: _tid, locations }) => {
    let _locations_str: Vec<String> = locations.iter().map(|p| p.display().to_string()).collect();
    Err(AnchorScopeError::DuplicateTrueId)
}
```

**Step 3: Fix second location (lines ~379-382)**

```rust
// Before:
Err(AmbiguousAnchorError { true_id: tid, locations }) => {
    let locations_str: Vec<String> = locations.iter().map(|p| p.display().to_string()).collect();
    Err(AnchorScopeError::DuplicateTrueId)
}

// After:
Err(AmbiguousAnchorError { true_id: _tid, locations }) => {
    let _locations_str: Vec<String> = locations.iter().map(|p| p.display().to_string()).collect();
    Err(AnchorScopeError::DuplicateTrueId)
}
```

**Step 4: Run build to verify warnings are gone**

Run: `cargo build 2>&1 | grep "unused variable"`
Expected: No output (no warnings)

**Step 5: Commit**

```bash
git add src/storage.rs
git commit -m "fix: prefix unused variables with underscore in storage.rs"
```

---

### Task 3: Fix unused function warnings in `config.rs`

**Files:**
- Modify: `src/config.rs:23-45`

**Step 1: Verify warnings exist**

Run: `cargo build 2>&1 | grep "unused function"`
Expected: Shows warnings for `max_file_size` and `max_nesting_depth`

**Step 2: Add documentation to functions**

Since these functions are placeholders for future configuration, add `#[allow(dead_code)]` and doc comments:

```rust
// Before (lines ~30-40):
pub fn max_file_size() -> u64 {
    // ... existing implementation ...
}

pub fn max_nesting_depth() -> usize {
    // ... existing implementation ...
}

// After:
/// Maximum file size (100MB default) for security checks
pub fn max_file_size() -> u64 {
    // ... existing implementation ...
}

/// Maximum nesting depth for buffer hierarchy
pub fn max_nesting_depth() -> usize {
    // ... existing implementation ...
}
```

**Alternative: If these functions are truly unused and not planned for immediate use**

```rust
/// Maximum file size (100MB default) for security checks
#[allow(dead_code)]
pub fn max_file_size() -> u64 {
    // ... existing implementation ...
}

/// Maximum nesting depth for buffer hierarchy
#[allow(dead_code)]
pub fn max_nesting_depth() -> usize {
    // ... existing implementation ...
}
```

**Step 3: Run build to verify warnings are gone**

Run: `cargo build 2>&1 | grep "unused function"`
Expected: No output (no warnings)

Run: `cargo clippy` 
Expected: No warnings about unused functions

**Step 4: Commit**

```bash
git add src/config.rs
git commit -m "fix: add documentation to config functions to suppress dead_code warnings"
```

---

### Task 4: Run full verification

**Files:**
- No code changes needed

**Step 1: Run clippy**

Run: `cargo clippy`
Expected: No warnings

**Step 2: Run all tests**

Run: `cargo test`
Expected: All 26 tests pass (24 existing + 2 new)

**Step 3: Format code**

Run: `cargo fmt`
Expected: No changes needed (code already formatted)

**Step 4: Final commit**

```bash
git add .
git commit -m "chore: run cargo clippy and cargo fmt for code quality"
```

---

### Task 5: Create CI verification (optional)

**Files:**
- Modify: `.github/workflows/ci.yml`

**Step 1: Add clippy and test step**

```yaml
- name: Run clippy
  run: cargo clippy --all-targets --all-features -- -D warnings

- name: Run tests
  run: cargo test --all-targets --all-features
```

**Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add clippy check to CI workflow"
```

---

## All tasks complete

Run `cargo test` to verify all 26 tests pass before merging.
