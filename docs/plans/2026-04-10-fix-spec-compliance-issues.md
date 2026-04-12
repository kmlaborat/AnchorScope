# Fix SPEC v1.2.0 Compliance Issues

> **REQUIRED SUB-SKILL:** Use the `executing-plans` skill to implement this plan task-by-task.

**Goal:** Fix 4 critical/important issues to ensure AnchorScope fully complies with SPEC §3.2 (True ID), §4.2 (Directory Structure), §4.3 (Lifecycle), and §5.2 (Write Phase).

**Architecture:**
- Implement recursive buffer cleanup in write phase
- Change from dual flat+nested buffer storage to single hierarchical chain
- Update metadata loading to traverse parent-child hierarchy correctly
- Fix documentation comments and test file cleanup
- Add tests for 3+ level nesting and orphan prevention

**Tech Stack:** Rust 1.78+, `serde_json`, `xxhash-rust`, `clap`, standard library I/O.

---

### Task 1: Implement Recursive Buffer Cleanup (`invalidate_true_id_hierarchy`)

**Files:**
- Modify `src/storage.rs` – add `fn invalidate_true_id_hierarchy(file_hash: &str, true_id: &str) -> Result<(), String>`
- Modify `src/commands/write.rs` – replace `invalidate_true_id` with `invalidate_true_id_hierarchy`

**Step 1: Write the function signature and documentation**

Add after `invalidate_nested_true_id`:
```rust
/// Delete buffer directory and all descendants for a True ID hierarchy.
/// This recursively removes the directory {file_hash}/{true_id} and all nested children.
/// SPEC §4.3 requires that write operations delete the anchor's directory "and all its descendants".
pub fn invalidate_true_id_hierarchy(file_hash: &str, true_id: &str) -> Result<(), String> {
    // Implementation will traverse the hierarchy and delete all descendants
}
```

**Step 2: Run compilation to verify signature**

```bash
cargo build 2>&1 | grep "error\[E"
```

Expected: No errors (just warnings about unused function)

**Step 3: Implement the recursive deletion**

```rust
pub fn invalidate_true_id_hierarchy(file_hash: &str, true_id: &str) -> Result<(), String> {
    let base_path = buffer_path::true_id_dir(file_hash, true_id);
    
    // Delete the immediate directory
    if base_path.exists() {
        std::fs::remove_dir_all(&base_path)
            .map_err(|e| format!("IO_ERROR: cannot delete buffer {}: {}", base_path.display(), e))?;
    }
    
    // Search for nested children and delete them too
    let file_dir = buffer_path::file_dir(file_hash);
    if let Ok(entries) = std::fs::read_dir(&file_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let parent_id = entry.file_name();
                let parent_id_str = parent_id.to_string_lossy();
                let nested_path = buffer_path::nested_true_id_dir(file_hash, &parent_id_str, true_id);
                
                if nested_path.exists() {
                    std::fs::remove_dir_all(&nested_path)
                        .map_err(|e| format!("IO_ERROR: cannot delete nested buffer {}: {}", nested_path.display(), e))?;
                }
            }
        }
    }
    
    Ok(())
}
```

**Step 4: Run compilation to verify implementation**

```bash
cargo build
```

Expected: Build succeeds with no new errors

**Step 5: Update write.rs to use the new function**

In `src/commands/write.rs`, replace the cleanup block:
```rust
// Clean up buffer artifacts
if let Some(ref label_name) = used_label {
    if let Ok(true_id) = crate::storage::load_label_target(label_name) {
        if let Ok(file_hash) = crate::storage::file_hash_for_true_id(&true_id) {
            crate::storage::invalidate_true_id_hierarchy(&file_hash, &true_id)?;
        }
    }
}
```

**Step 6: Run compilation to verify write.rs changes**

```bash
cargo build
```

Expected: Build succeeds

**Step 7: Commit**

```bash
git add src/storage.rs src/commands/write.rs
git commit -m "feat: implement recursive buffer cleanup per SPEC §4.3"
```

---

### Task 2: Fix Buffer Storage to Single Hierarchical Chain

**Files:**
- Modify `src/commands/read.rs` – change buffer save logic to only use nested structure for multi-level anchors
- Modify `src/storage.rs` – adjust helper functions if needed

**Step 1: Write the failing test**

Create a new test file `tests/integration/single_chain_buffer_structure.rs`:

```rust
use crate::test_helpers::*;

#[test]
fn nested_read_creates_single_hierarchical_chain() {
    // Create a file with nested anchors
    let content = "fn outer() {\n    fn inner() {}\n}\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    
    // Level 1: read outer anchor "fn outer"
    let out1 = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(),
        "--anchor", "fn outer"
    ]);
    assert!(out1.status.success());
    let res1 = parse_output(&String::from_utf8_lossy(&out1.stdout));
    let level1_true_id = res1.get("true_id").unwrap().clone();
    
    // Level 2: read inner anchor using label pointing to level 1
    let label_out = run_anchorscope(&[
        "label", "--name", "outer_anchor", "--true-id", &level1_true_id
    ]);
    assert!(label_out.status.success());
    
    let out2 = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(),
        "--anchor", "fn inner",
        "--label", "outer_anchor"
    ]);
    assert!(out2.status.success());
    let res2 = parse_output(&String::from_utf8_lossy(&out2.stdout));
    let level2_true_id = res2.get("true_id").unwrap().clone();
    
    // Verify directory structure:
    // {file_hash}/{level1_true_id}/content (level 1 buffer)
    // {file_hash}/{level1_true_id}/{level2_true_id}/content (level 2 buffer)
    // There should NOT be a flat {file_hash}/{level2_true_id}/ directory
    
    // Load file_hash from buffer metadata
    let file_hash = storage::file_hash_for_true_id(&level1_true_id).unwrap();
    
    let file_dir = buffer_path::file_dir(&file_hash);
    let level1_dir = buffer_path::true_id_dir(&file_hash, &level1_true_id);
    let flat_level2_dir = buffer_path::true_id_dir(&file_hash, &level2_true_id);
    let nested_level2_dir = buffer_path::nested_true_id_dir(&file_hash, &level1_true_id, &level2_true_id);
    
    // Level 1 should exist
    assert!(level1_dir.join("content").exists(), "level 1 buffer should exist");
    assert!(level1_dir.join("metadata.json").exists(), "level 1 metadata should exist");
    
    // Level 2 should exist ONLY as nested under level 1
    assert!(nested_level2_dir.join("content").exists(), "level 2 nested buffer should exist");
    assert!(nested_level2_dir.join("metadata.json").exists(), "level 2 nested metadata should exist");
    
    // Level 2 should NOT exist as flat directory
    assert!(!flat_level2_dir.exists(), "level 2 should NOT exist as flat directory (orphan prevention)");
    
    // Cleanup
    storage::invalidate_true_id_hierarchy(&file_hash, &level1_true_id).unwrap();
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test --test mod nested_read_creates_single_hierarchical_chain
```

Expected: Test fails because flat level2 directory exists (current buggy behavior)

**Step 3: Implement the fix in read.rs**

In `src/commands/read.rs`, modify the buffer save logic. Find the section around line 180 where buffers are saved:

**Current (buggy) code:**
```rust
// Save matched region content to {file_hash}/{true_id}/content
let buffer_to_save = if anchor_str.starts_with("def ") {
    crate::matcher::extract_function_body(&normalized, m.byte_start, m.byte_end)
} else {
    region.to_vec()
};
storage::save_region_content(&file_hash, &true_id, &buffer_to_save)?;
```

**Fix:** Only save flat for level-1, save nested for multi-level:

```rust
// Save matched region content
let buffer_to_save = if anchor_str.starts_with("def ") {
    crate::matcher::extract_function_body(&normalized, m.byte_start, m.byte_end)
} else {
    region.to_vec()
};

if parent_true_id.is_some() {
    // Multi-level: save ONLY to nested location
    storage::save_nested_buffer_content(&file_hash, parent_true_id.as_ref().unwrap(), &true_id, &buffer_to_save)?;
} else {
    // Level-1: save to flat location
    storage::save_region_content(&file_hash, &true_id, &buffer_to_save)?;
}
```

Also update metadata save to use same logic for parent context.

**Step 4: Run test to verify it passes**

```bash
cargo test --test mod nested_read_creates_single_hierarchical_chain
```

Expected: Test passes - buffer structure is now a single chain

**Step 5: Commit**

```bash
git add src/commands/read.rs tests/integration/single_chain_buffer_structure.rs
git commit -m "fix: single hierarchical buffer chain per SPEC §4.2"
```

---

### Task 3: Update `load_anchor_metadata_by_true_id` for Hierarchical Traversal

**Files:**
- Modify `src/storage.rs` – update `load_anchor_metadata_by_true_id` to traverse parent-child chain

**Step 1: Write the failing test**

In `tests/integration/single_chain_buffer_structure.rs`, add:

```rust
#[test]
fn write_uses_correct_buffer_for_nested_anchors() {
    // Setup 2-level nesting
    let content = "fn outer() {\n    fn inner() {}\n}\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    
    // Read level 1
    let out1 = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(),
        "--anchor", "fn outer"
    ]);
    assert!(out1.status.success());
    let res1 = parse_output(&String::from_utf8_lossy(&out1.stdout));
    let level1_true_id = res1.get("true_id").unwrap().clone();
    let level1_hash = res1.get("hash").unwrap().clone();
    
    // Label level 1
    let label_out = run_anchorscope(&[
        "label", "--name", "outer", "--true-id", &level1_true_id
    ]);
    assert!(label_out.status.success());
    
    // Read level 2
    let out2 = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(),
        "--anchor", "fn inner",
        "--label", "outer"
    ]);
    assert!(out2.status.success());
    let res2 = parse_output(&String::from_utf8_lossy(&out2.stdout));
    let level2_true_id = res2.get("true_id").unwrap().clone();
    
    // Write using label pointing to level 2 (this is the fix test)
    let write_out = run_anchorscope(&[
        "write", "--file", file_path.to_str().unwrap(),
        "--anchor", "fn inner",
        "--expected-hash", &level1_hash,  // Use level 1's hash since we're editing the outer function
        "--replacement", "fn outer() { println!(\"modified\"); }"
    ]);
    assert!(write_out.status.success(), "write failed: {}", String::from_utf8_lossy(&write_out.stderr));
    
    // Verify buffer was cleaned up correctly
    let file_hash = storage::file_hash_for_true_id(&level1_true_id).unwrap();
    let level2_flat = buffer_path::true_id_dir(&file_hash, &level2_true_id);
    assert!(!level2_flat.exists(), "level 2 flat buffer should be deleted");
    
    // Cleanup
    storage::invalidate_true_id_hierarchy(&file_hash, &level1_true_id).unwrap();
}
```

**Step 2: Run test to verify it fails (if hierarchical search not implemented)**

```bash
cargo test --test mod write_uses_correct_buffer_for_nested_anchors 2>&1 | grep -A 5 "FAILED\|IO_ERROR"
```

Expected: May fail if hierarchical search not working properly

**Step 3: Update `load_anchor_metadata_by_true_id` to search in parent's directory first**

In `src/storage.rs`, modify the nested search section:

**Current code searches ALL subdirectories:**
```rust
if let Ok(dir_entries) = std::fs::read_dir(buffer_path::file_dir(&file_hash_str)) {
    for dir_entry in dir_entries.flatten() {
        // ... checks every subdirectory
    }
}
```

**Fix:** Search hierarchically - first check if this true_id exists under a known parent:
```rust
// Search for nested children that match this true_id
if let Ok(dir_entries) = std::fs::read_dir(buffer_path::file_dir(&file_hash_str)) {
    for dir_entry in dir_entries.flatten() {
        if dir_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            let parent_id = dir_entry.file_name();
            let parent_id_str = parent_id.to_string_lossy();
            
            let nested_content_path = buffer_path::nested_true_id_dir(&file_hash_str, &parent_id_str, true_id).join("content");
            let nested_metadata_path = buffer_path::nested_true_id_dir(&file_hash_str, &parent_id_str, true_id).join("metadata.json");
            
            if nested_metadata_path.exists() {
                let content = fs::read_to_string(&nested_metadata_path)
                    .map_err(|e| io_error_to_spec(e, "read failure"))?;
                let buffer_meta: BufferMeta = serde_json::from_str(&content)
                    .map_err(|e| format!("IO_ERROR: buffer metadata corrupted: {}", e))?;
                
                let match_result = buffer_meta.true_id == true_id || buffer_meta.region_hash == true_id;
                if match_result {
                    // Load source path and return
                    let source_path = buffer_path::file_dir(&file_hash_str).join("source_path");
                    let file = fs::read_to_string(&source_path)
                        .map_err(|e| io_error_to_spec(e, "read failure"))?;
                    
                    let region_hash = buffer_meta.region_hash.clone();
                    let anchor = buffer_meta.anchor.clone();
                    return Ok(AnchorMeta {
                        file,
                        anchor,
                        hash: region_hash,
                        line_range: (0, 0),
                    });
                }
            }
        }
    }
}
```

**Step 4: Run test to verify it passes**

```bash
cargo test --test mod write_uses_correct_buffer_for_nested_anchors
```

Expected: Test passes

**Step 5: Commit**

```bash
git add src/storage.rs tests/integration/single_chain_buffer_structure.rs
git commit -m "fix: hierarchical buffer search in load_anchor_metadata_by_true_id"
```

---

### Task 4: Add 3+ Level Nesting Test

**Files:**
- Modify `tests/integration/single_chain_buffer_structure.rs`

**Step 1: Add the test**

```rust
#[test]
fn three_level_nesting_write_cleans_up_correctly() {
    let content = "fn a() {\n    fn b() {\n        fn c() {}\n    }\n}\n";
    let (_temp_dir, file_path) = create_temp_file(content);
    
    // Level 1: read "fn a"
    let out1 = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(),
        "--anchor", "fn a"
    ]);
    assert!(out1.status.success());
    let res1 = parse_output(&String::from_utf8_lossy(&out1.stdout));
    let level1_true_id = res1.get("true_id").unwrap().clone();
    
    // Label level 1
    run_anchorscope(&[
        "label", "--name", "level1", "--true-id", &level1_true_id
    ]).expect("label failed");
    
    // Level 2: read "fn b" using level1 label
    let out2 = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(),
        "--anchor", "fn b",
        "--label", "level1"
    ]);
    assert!(out2.status.success());
    let res2 = parse_output(&String::from_utf8_lossy(&out2.stdout));
    let level2_true_id = res2.get("true_id").unwrap().clone();
    
    // Label level 2
    run_anchorscope(&[
        "label", "--name", "level2", "--true-id", &level2_true_id
    ]).expect("label failed");
    
    // Level 3: read "fn c" using level2 label
    let out3 = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(),
        "--anchor", "fn c",
        "--label", "level2"
    ]);
    assert!(out3.status.success());
    let res3 = parse_output(&String::from_utf8_lossy(&out3.stdout));
    let level3_true_id = res3.get("true_id").unwrap().clone();
    
    // Verify 3-level structure exists:
    // file_hash/level1/content (level 1)
    // file_hash/level1/level2/content (level 2)
    // file_hash/level1/level2/level3/content (level 3)
    
    let file_hash = storage::file_hash_for_true_id(&level1_true_id).unwrap();
    
    let level1_path = buffer_path::true_id_dir(&file_hash, &level1_true_id);
    let level2_path = buffer_path::nested_true_id_dir(&file_hash, &level1_true_id, &level2_true_id);
    let level3_path = buffer_path::nested_true_id_dir(&file_hash, &level1_true_id, &level3_true_id);
    
    assert!(level1_path.join("content").exists(), "level 1 should exist");
    assert!(level2_path.join("content").exists(), "level 2 should exist as nested");
    assert!(level3_path.join("content").exists(), "level 3 should exist as nested");
    
    // Write at level 3 - should clean up level 3 and any children
    let write_out = run_anchorscope(&[
        "write", "--file", file_path.to_str().unwrap(),
        "--label", "level3",
        "--replacement", "fn c() { println!(\"c modified\"); }"
    ]);
    assert!(write_out.status.success(), "write failed: {}", String::from_utf8_lossy(&write_out.stderr));
    
    // Level 3 should be deleted
    assert!(!level3_path.exists(), "level 3 should be deleted after write");
    
    // Level 1 and 2 should still exist (they weren't the target of the write)
    assert!(level1_path.join("content").exists(), "level 1 should still exist");
    assert!(level2_path.join("content").exists(), "level 2 should still exist");
    
    // Cleanup
    storage::invalidate_true_id_hierarchy(&file_hash, &level1_true_id).unwrap();
}
```

**Step 2: Run test to verify it passes**

```bash
cargo test --test mod three_level_nesting_write_cleans_up_correctly
```

Expected: Test passes

**Step 3: Commit**

```bash
git add tests/integration/single_chain_buffer_structure.rs
git commit -m "test: 3+ level nesting write cleanup"
```

---

### Task 5: Fix Minor Issues

**Files:**
- `src/commands/read.rs`
- `src/commands/write.rs`
- `src/storage.rs`

**Step 1: Fix comment in read.rs**

Find the line with:
```rust
// Compute file_hash from the raw file content (not buffer content)
```

Replace with:
```rust
// Compute file_hash from raw file content to ensure consistency across read modes.
// The file_hash represents the original file state, not the matched region.
```

**Step 2: Clean up test temp file**

In `src/commands/read.rs` test, add cleanup:
```rust
let tmp_file_path = std::env::temp_dir().join("tmp_anchor_file.txt");
std::fs::write(&tmp_file_path, content).expect("write tmp file");
// ... rest of test ...
// Cleanup
let _ = std::fs::remove_file(&tmp_file_path);
```

**Step 3: Move unused import**

In `src/commands/read.rs`, move `use std::fs;` inside `#[cfg(test)]` module.

**Step 4: Run tests to verify minor fixes**

```bash
cargo test
```

Expected: All tests pass

**Step 5: Commit**

```bash
git add src/commands/read.rs src/storage.rs
git commit -m "chore: fix minor issues (comments, cleanup, imports)"
```

---

### Task 6: Final Verification

**Step 1: Run all tests**

```bash
cargo test 2>&1 | tail -20
```

Expected: All tests pass, no failures

**Step 2: Verify SPEC compliance**

- [ ] True ID formula: `hash(parent_region_hash + "_" + region_hash)` ✅
- [ ] Directory structure: single hierarchical chain ✅
- [ ] Write cleanup: recursive deletion of all descendants ✅
- [ ] Buffer metadata: hierarchical search ✅
- [ ] Level-1: flat `{file_hash}/{true_id}/` ✅
- [ ] Level 2+: nested `{file_hash}/{parent}/{child}/` ✅

**Step 3: Clean up temp directory and run final test**

```bash
rm -rf "/c/Users/MURAMATSU/AppData/Local/Temp/anchorscope"
cargo test
```

Expected: All tests pass with clean slate

**Step 4: Commit all changes**

```bash
git add .
git commit -m "fix: fully compliant SPEC v1.2.0 implementation"
```

---

Plan complete and saved to `docs/plans/2026-04-10-fix-spec-compliance-issues.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?
