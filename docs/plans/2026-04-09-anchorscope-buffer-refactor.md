# AnchorScope Buffer Implementation Plan

> **REQUIRED SUB-SKILL:** Use the executing-plans skill to implement this plan task-by-task.

**Goal:** Implement the Anchor Scope Anchoring protocol v1.2.0 with True IDs, nested anchoring, and structured buffer storage as defined in SPEC §3-§6.

**Architecture:** 
- Anchor Buffer: `{TMPDIR}/anchorscope/{file_hash}/{true_id}/content` for nested levels
- Labels: `{TMPDIR}/anchorscope/labels/{alias}.json` for human-readable names  
- True ID computation: `xxh3_64(parent_region_hash + "_" + child_region_hash)` for nested, `xxh3_64(file_hash + "_" + region_hash)` for root
- Multi-level anchoring: Each `read` creates a buffer copy; subsequent reads operate on buffer copies only

**Tech Stack:** Rust, xxh3_64, serde, clap

---

## Task 1: Create anchor buffer directory structure (SPEC §4.2)

**Files:**
- Create: `src/buffer_path.rs` - Path construction utilities
- Modify: `src/storage.rs` - Refactor existing functions to use new structure

**Step 1: Create buffer_path.rs with path utilities**

```rust
use std::path::{Path, PathBuf};

/// Returns {TMPDIR}/anchorscope base path
pub fn anchorscope_temp_dir() -> PathBuf {
    std::env::temp_dir().join("anchorscope")
}

/// Returns {TMPDIR}/anchorscope/anchors for v1.1.0 compatibility
pub fn anchors_dir() -> PathBuf {
    anchorscope_temp_dir().join("anchors")
}

/// Returns {TMPDIR}/anchorscope/labels for alias storage
pub fn labels_dir() -> PathBuf {
    anchorscope_temp_dir().join("labels")
}

/// Returns {TMPDIR}/anchorscope/{file_hash}
pub fn file_dir(file_hash: &str) -> PathBuf {
    anchorscope_temp_dir().join(file_hash)
}

/// Returns {TMPDIR}/anchorscope/{file_hash}/{true_id}
pub fn true_id_dir(file_hash: &str, true_id: &str) -> PathBuf {
    file_dir(file_hash).join(true_id)
}

/// Returns {TMPDIR}/anchorscope/{file_hash}/{parent_true_id}/{true_id} for nested
pub fn nested_true_id_dir(file_hash: &str, parent_true_id: &str, true_id: &str) -> PathBuf {
    true_id_dir(file_hash, parent_true_id).join(true_id)
}
```

**Step 2: Update storage.rs to use new path utilities**

```rust
// Replace old path functions with imports
use crate::buffer_path::{anchorscope_temp_dir, anchors_dir, labels_dir, file_dir, true_id_dir, nested_true_id_dir};

// Update ensure_* functions to use new paths
fn ensure_file_dir(file_hash: &str) -> Result<PathBuf, String> { ... }
fn ensure_true_id_dir(file_hash: &str, true_id: &str) -> Result<PathBuf, String> { ... }
```

**Step 3: Run tests to verify no regression**

```bash
cargo test --lib
```

Expected: All library tests pass

**Step 4: Commit**

```bash
git add src/buffer_path.rs src/storage.rs
git commit -m "refactor: create buffer_path module for v1.2.0 structure"
```

---

## Task 2: Implement nested buffer content storage (SPEC §4.3)

**Files:**
- Modify: `src/storage.rs` - Add nested content storage functions

**Step 1: Add save/load for nested buffer content**

```rust
/// Save buffer content for nested level: {file_hash}/{parent_true_id}/{true_id}/content
pub fn save_nested_buffer_content(file_hash: &str, parent_true_id: &str, true_id: &str, content: &[u8]) -> Result<(), String> {
    let dir = nested_true_id_dir(file_hash, parent_true_id, true_id);
    ensure_dir(&dir)?;
    let path = dir.join("content");
    fs::write(&path, content)
        .map_err(|e| io_error_to_spec(e, "write failure"))
}

/// Load buffer content from nested level
pub fn load_nested_buffer_content(file_hash: &str, parent_true_id: &str, true_id: &str) -> Result<Vec<u8>, String> {
    let path = nested_true_id_dir(file_hash, parent_true_id, true_id).join("content");
    fs::read(&path)
        .map_err(|e| io_error_to_spec(e, "read failure"))
}
```

**Step 2: Update invalidate functions for nested structure**

```rust
/// Delete nested buffer directory and all descendants
pub fn invalidate_nested_true_id(file_hash: &str, parent_true_id: &str, true_id: &str) {
    if let Ok(parent_dir) = nested_true_id_dir(file_hash, parent_true_id, true_id).parent() {
        let path = parent_dir.join(true_id);
        let _ = fs::remove_dir_all(path);
    }
}
```

**Step 3: Write failing test for nested buffer storage**

Create `tests/integration/nested_buffer_tests.rs`:

```rust
#[test]
fn test_nested_buffer_content_storage() {
    let (_temp_dir, file_path) = create_temp_file("Hello\nWorld\n");
    
    // First read creates root level
    let out = run_anchorscope(&["read", "--file", file_path.to_str().unwrap(), "--anchor", "Hello"]);
    assert!(out.status.success());
    let result = parse_output(&String::from_utf8_lossy(&out.stdout));
    let root_true_id = result.get("true_id").unwrap().clone();
    
    // Second read on buffer should create nested level
    // (This test will be updated when read command supports nested mode)
}
```

**Step 4: Run test to verify it fails**

```bash
cargo test --test nested_buffer_tests
```

Expected: Test fails (nested buffer not yet implemented)

**Step 5: Run full test suite to ensure no regression**

```bash
cargo test
```

Expected: All existing tests pass

**Step 6: Commit**

```bash
git add src/storage.rs tests/integration/nested_buffer_tests.rs
git commit -m "feat: add nested buffer content storage functions"
```

---

## Task 3: Update read command to output True ID properly (SPEC §3.2)

**Files:**
- Modify: `src/commands/read.rs`

**Step 1: Update read to compute and output True ID**

```rust
// After getting region_hash, compute true_id
let true_id = crate::trueid::compute_for_read(file_path, &h, None);

println!("true_id={}", true_id);
```

**Step 2: Implement compute_for_read in trueid.rs**

```rust
/// Compute True ID for a read operation (root level)
pub fn compute_for_read(file_path: &str, region_hash: &str) -> String {
    // Read file, compute file_hash
    // Return xxh3_64(file_hash + "_" + region_hash)
}
```

**Step 3: Write failing test**

```rust
#[test]
fn test_read_outputs_true_id() {
    let (_temp_dir, file_path) = create_temp_file("Hello\nWorld\n");
    
    let out = run_anchorscope(&["read", "--file", file_path.to_str().unwrap(), "--anchor", "Hello"]);
    assert!(out.status.success());
    
    let result = parse_output(&String::from_utf8_lossy(&out.stdout));
    let true_id = result.get("true_id").unwrap();
    
    // True ID should be 16-char hex
    assert_eq!(true_id.len(), 16);
    assert!(true_id.chars().all(|c| c.is_ascii_hexdigit()));
}
```

**Step 4: Run test to verify failure**

```bash
cargo test --test read_outputs_true_id
```

Expected: Test fails (true_id not computed)

**Step 5: Implement and verify**

Run `cargo test` - all tests should pass

**Step 6: Commit**

```bash
git add src/commands/read.rs src/trueid.rs tests/integration/read_outputs_true_id.rs
git commit -m "feat: read command now outputs True ID properly"
```

---

## Task 4: Update write command to support label-based True ID lookup (SPEC §6.3)

**Files:**
- Modify: `src/commands/write.rs`

**Step 1: Update write command to resolve label to True ID**

```rust
// In label mode, load label mapping to get true_id
let true_id = crate::storage::load_label_target(label_name)?;
let buffer_meta = crate::storage::load_buffer_metadata_for_true_id(&true_id)?;
```

**Step 2: Add helper function in storage.rs**

```rust
/// Load buffer metadata by True ID (searches all file_hash directories)
pub fn load_buffer_metadata_for_true_id(true_id: &str) -> Result<BufferMeta, String> {
    // Search through all {file_hash}/{true_id}/metadata.json
}
```

**Step 3: Write failing test**

```rust
#[test]
fn test_write_using_label_with_true_id() {
    // Test write using label that points to True ID
}
```

**Step 4: Implement and verify**

**Step 5: Commit**

```bash
git add src/commands/write.rs src/storage.rs
git commit -m "feat: write command supports label-based True ID lookup"
```

---

## Task 5: Implement tree command to show nested buffer structure (SPEC §6.5)

**Files:**
- Modify: `src/commands/tree.rs`

**Step 1: Display True IDs with aliases**

```rust
// For each file_hash directory, show:
// - Root level True IDs
// - Nested True IDs
// - Aliases for each True ID
```

**Step 2: Write failing test**

```rust
#[test]
fn test_tree_shows_buffer_structure() {
    // Test tree output format
}
```

**Step 3: Implement and verify**

**Step 4: Commit**

```bash
git add src/commands/tree.rs
git commit -m "feat: tree command displays nested buffer structure"
```

---

## Task 6: Update label command to validate True ID exists in buffer (SPEC §6.4)

**Files:**
- Modify: `src/commands/label.rs`

**Step 1: Update label command to check True ID in buffer**

```rust
// Check if true_id exists in either:
// 1. Old location: anchors/{true_id}.json
// 2. New location: any file_hash/{true_id}/content
```

**Step 2: Write failing test**

```rust
#[test]
fn test_label_rejects_nonexistent_true_id() {
    // Test label command with non-existent True ID
}
```

**Step 3: Implement and verify**

**Step 4: Commit**

```bash
git add src/commands/label.rs
git commit -m "feat: label command validates True ID exists in buffer"
```

---

## Task 7: Implement True ID computation for nested anchors (SPEC §3.2)

**Files:**
- Modify: `src/trueid.rs`

**Step 1: Update compute function for nested anchors**

```rust
/// Compute True ID for nested anchoring
/// parent_true_id is Some for levels 2+, None for root level
pub fn compute(file_path: &str, region_hash: &str, parent_true_id: Option<&str>) -> String {
    // If parent_true_id is Some:
    //   xxh3_64(parent_true_id + "_" + region_hash)
    // If parent_true_id is None:
    //   xxh3_64(file_hash + "_" + region_hash)
}
```

**Step 2: Write failing test for nested True ID**

```rust
#[test]
fn test_nested_true_id computation() {
    // Test parent_region_hash + "_" + child_region_hash
}
```

**Step 3: Implement and verify**

**Step 4: Commit**

```bash
git add src/trueid.rs
git commit -m "feat: True ID computation for nested anchors"
```

---

## Task 8: Update read to save nested buffer on buffer input (SPEC §4.3)

**Files:**
- Modify: `src/commands/read.rs`

**Step 1: Check if input is file or buffer reference**

```rust
// If file_path points to buffer content file:
//   - Load parent buffer content
//   - Read anchor from parent buffer
//   - Save nested buffer: file_hash/parent_true_id/true_id/content
```

**Step 2: Write failing test**

```rust
#[test]
fn test_read_on_buffer_creates_nested_level() {
    // Test reading from a buffer copy creates nested structure
}
```

**Step 3: Implement and verify**

**Step 4: Commit**

```bash
git add src/commands/read.rs
git commit -m "feat: read on buffer creates nested buffer level"
```

---

## Task 9: Implement invalidate_label function (SPEC §4.4)

**Files:**
- Modify: `src/storage.rs`

**Step 1: Add invalidate_label function**

```rust
/// Delete ephemeral label mapping after successful write (SPEC §4.4)
pub fn invalidate_label(name: &str) {
    let path = labels_dir().join(format!("{}.json", name));
    let _ = fs::remove_file(path);
}
```

**Step 2: Update write command to call invalidate_label**

**Step 3: Write failing test**

```rust
#[test]
fn test_write_invalidates_label() {
    // Test label file deleted after successful write
}
```

**Step 4: Implement and verify**

**Step 5: Commit**

```bash
git add src/storage.rs src/commands/write.rs
git commit -m "feat: invalidate_label function for write cleanup"
```

---

## Task 10: Update save_buffer_content to use correct path structure

**Files:**
- Modify: `src/storage.rs`

**Step 1: Fix save_buffer_content path**

```rust
// Root level: file_hash/content (not file_hash/true_id/content)
pub fn save_buffer_content(file_hash: &str, content: &[u8]) -> Result<(), String> {
    let dir = file_dir(file_hash);
    ensure_dir(&dir)?;
    let path = dir.join("content");
    fs::write(&path, content)
        .map_err(|e| io_error_to_spec(e, "write failure"))
}
```

**Step 2: Update all callers**

**Step 3: Write failing test**

```rust
#[test]
fn test_root_buffer_content_location() {
    // Verify content stored at file_hash/content
}
```

**Step 4: Implement and verify**

**Step 5: Commit**

```bash
git add src/storage.rs
git commit -m "refactor: correct buffer content path structure"
```

---

## Execution Handoff

Plan complete and saved to `docs/plans/2026-04-09-anchorscope-buffer-refactor.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session in worktree with executing-plans, batch execution with checkpoints

**Which approach?**
