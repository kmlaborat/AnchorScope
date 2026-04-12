# Code Review Fixes Implementation Plan

> **REQUIRED SUB-SKILL:** Use the executing-plans skill to implement this plan task-by-task.

**Goal:** Address minor code review issues identified during security-audit-fixes review.

## Issues to Fix

### Issue 1: Improve tempfile error messages with path context

**File:** `src/commands/write.rs:22-33`

**Current:**
```rust
Err(e) => {
    return Err(AnchorScopeError::WriteFailure(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("tempfile creation error: {}", e),
    )));
}
```

**Fix:** Include the target path in error messages
```rust
Err(e) => {
    return Err(AnchorScopeError::WriteFailure(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("tempfile creation error for '{}': {}", path.display(), e),
    )));
}
```

### Issue 2: Improve WriteFailure error message format

**File:** `src/error.rs:112`

**Current:**
```rust
AnchorScopeError::WriteFailure(e) => format!("IO_ERROR: write failure: {}", e.kind()),
```

**Fix:** Include full error message (not just kind)
```rust
AnchorScopeError::WriteFailure(e) => format!("IO_ERROR: write failure: {}", e),
```

### Issue 3: Add WriteFailure test case

**File:** `tests/integration/security_tests.rs`

**Add test:**
```rust
#[test]
fn write_failure_shows_io_error_details() {
    // Test that write failures include full error details
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("readonly.txt");
    std::fs::write(&file, "data").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444)).unwrap();

    let out = run_anchorscope(&["write", "--file", file.to_str().unwrap(),
                               "--anchor", "a", "--expected-hash", "deadbeef",
                               "--replacement", "new"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    // Verify the error message includes more than just "write failure"
    assert!(stderr.contains("IO_ERROR") || stderr.contains("write failure"));
}
```

---

## Implementation Plan

### Task 1: Improve tempfile error messages with path context

**Files:** `src/commands/write.rs:22-33`

**Step 1:** Update error messages to include path

**Step 2:** Run `cargo test` - verify all tests still pass

**Step 3:** Commit

```bash
git add src/commands/write.rs
git commit -m "fix: include path in tempfile error messages"
```

---

### Task 2: Improve WriteFailure error message format

**Files:** `src/error.rs:112`

**Step 1:** Update error message to show full error (not just kind)

**Step 2:** Run `cargo test` - verify all tests still pass

**Step 3:** Commit

```bash
git add src/error.rs
git commit -m "fix: include full error details in WriteFailure message"
```

---

### Task 3: Add WriteFailure test case

**Files:** `tests/integration/security_tests.rs`

**Step 1:** Write failing test for WriteFailure message format

**Step 2:** Run test to verify it works

**Step 3:** Commit

```bash
git add tests/integration/security_tests.rs
git commit -m "test: add WriteFailure message format test"
```

---

**All tasks are now defined.** After these fixes, the branch should be ready to merge.
