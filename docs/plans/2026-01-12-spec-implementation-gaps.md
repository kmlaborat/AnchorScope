# AnchorScope Spec Implementation Gaps - Implementation Plan

> **REQUIRED SUB-SKILL:** Use the executing-plans skill to implement this plan task-by-task.

**Goal:** Fix all identified discrepancies between `docs/SPEC.md` v1.3.0 and current implementation

**Architecture:** Systematic bug fix approach - identify, prioritize, and fix each gap with proper error handling and test coverage

**Tech Stack:** Rust, Clap (CLI), xxh3 (hashing), tempfile (temp files)

---

## Task 1: Verify `label` Command Implementation

**Files:**
- Read: `src/commands/label.rs`
- Read: `src/storage.rs` (for label-related functions)
- Test: N/A

**Step 1: Review label.rs implementation**

Expected findings:
- Check if `label` command creates alias mapping
- Verify `LABEL_EXISTS` error handling
- Confirm multiple aliases per True ID are supported

**Step 2: Check storage functions**

Look for:
- `save_label_mapping()` - creates label alias
- `load_label_target()` - resolves alias to True ID
- Duplicate alias detection logic

**Step 3: Document gaps**

Create list of any missing functionality or incorrect behavior

---

## Task 2: Verify `paths` Command Implementation

**Files:**
- Read: `src/commands/paths.rs`
- Test: N/A

**Step 1: Review paths.rs implementation**

Expected behavior per SPEC:
- Accept True ID or alias
- Return absolute paths of `content` and `replacement`
- Return replacement path even if file doesn't exist

**Step 2: Verify output format**

Check for:
```
content:     /tmp/anchorscope/{file_hash}/{true_id}/content
replacement: /tmp/anchorscope/{file_hash}/{true_id}/replacement
```

**Step 3: Document gaps**

List any missing paths or incorrect path resolutions

---

## Task 3: Fix Error Message Format

**Files:**
- Modify: `src/commands/write.rs:46-52`
- Modify: `src/commands/pipe.rs`
- Modify: Other command files as needed

**Step 1: Update error messages to match SPEC format**

SPEC §6.8 specifies these exact error messages:
```
NO_MATCH
MULTIPLE_MATCHES
HASH_MISMATCH
DUPLICATE_TRUE_ID
LABEL_EXISTS
AMBIGUOUS_REPLACEMENT
NO_REPLACEMENT
IO_ERROR: file not found
IO_ERROR: permission denied
IO_ERROR: invalid UTF-8
IO_ERROR: read failure
IO_ERROR: write failure
```

**Step 2: Current issues to fix**

In `write.rs:46-52`:
```rust
if from_replacement && !replacement.is_empty() {
    eprintln!("IO_ERROR: ambiguous replacement source"); // Should be "AMBIGUOUS_REPLACEMENT"
    return 1;
}
if !from_replacement && replacement.is_empty() {
    eprintln!("IO_ERROR: no replacement provided"); // Should be "NO_REPLACEMENT"
    return 1;
}
```

**Step 3: Update all error messages**

Search for all `eprintln!!` calls and ensure they match SPEC format exactly.

**Step 4: Write tests**

Create unit tests that verify error messages match SPEC format

---

## Task 4: Improve Buffer Deletion Error Handling

**Files:**
- Modify: `src/commands/write.rs:297-313`

**Step 1: Add proper error handling**

Current code silently swallows errors:
```rust
let _ = crate::storage::invalidate_true_id_hierarchy(&file_hash, &true_id);
```

**Step 2: Implement proper cleanup**

Add error logging:
```rust
if let Err(e) = crate::storage::invalidate_true_id_hierarchy(&file_hash, &true_id) {
    eprintln!("WARN: failed to cleanup buffer: {}", e);
}
```

**Step 3: Add cleanup on failure**

Ensure buffer is cleaned up even if write fails partway through

---

## Task 5: Verify True ID Generation Logic

**Files:**
- Read: `src/read.rs:359-385`
- Read: `src/hash.rs`

**Step 1: Review True ID calculation**

Per SPEC §3.2:
```
true_id = xxh3_64(hex(parent_scope_hash) || 0x5F || hex(child_scope_hash))
```

**Step 2: Check implementation**

Current implementation uses `format!("{}_{}", parent_scope_hash, child_scope_hash)`

**Step 3: Verify correctness**

The `0x5F` is the ASCII code for `_`, so the implementation is technically correct. However, add a comment explaining this intentional design choice.

**Step 4: Add unit test**

Create test that verifies True ID generation matches SPEC formula exactly

---

## Task 6: Add Depth Limit Validation for Nested Anchors

**Files:**
- Modify: `src/commands/read.rs`
- Add: `src/config.rs` (if not exists)

**Step 1: Add max_depth configuration**

Define constant or config value for maximum nesting depth

**Step 2: Add depth check in read command**

Before allowing nested read, check if depth exceeds limit

**Step 3: Add error message**

```
IO_ERROR: maximum nesting depth (X) exceeded
```

**Step 4: Write tests**

Test depth validation with various nesting levels

---

## Task 7: Add Missing Unit Tests

**Files:**
- Create: `tests/spec_validation.rs`
- Modify: Existing test files as needed

**Step 1: Test SPEC compliance**

Create comprehensive test suite:
- UTF-8 validation
- CRLF normalization
- Error message formats
- Buffer lifecycle
- True ID generation
- Depth limiting

**Step 2: Run tests**

Verify all tests pass

---

## Task 8: Update Documentation

**Files:**
- Modify: `README.md`
- Modify: Any other relevant docs

**Step 1: Document known limitations**

If any gaps remain, document them clearly

**Step 2: Update version info**

Ensure version numbers are consistent

---

## Execution Order

1. **Priority 1 (Critical):** Task 3 (Error Messages) - breaks user scripts
2. **Priority 2 (High):** Task 1-2 (Verify label/paths) - missing functionality
3. **Priority 3 (Medium):** Task 4 (Error Handling) - robustness
4. **Priority 4 (Low):** Task 5-8 (Testing/Docs) - maintenance

---

## Acceptance Criteria

- [x] All error messages match SPEC §6.8 exactly
- [x] `label` command supports multiple aliases per True ID
- [x] `paths` command returns correct absolute paths
- [x] Buffer cleanup handles errors gracefully
- [x] True ID generation matches SPEC formula
- [x] Depth limiting prevents excessive nesting
- [x] All tests pass
- [x] Documentation updated

---

## Testing Strategy

1. Run existing test suite: `cargo test`
2. Run SPEC compliance tests: `cargo test --test spec_validation`
3. Manual testing with various edge cases
4. Integration tests for `pipe` command

---

## References

- SPEC: `docs/SPEC.md`
- Current Implementation: `src/`
- Error codes: SPEC §6.8
- Buffer structure: SPEC §4.2-4.3
