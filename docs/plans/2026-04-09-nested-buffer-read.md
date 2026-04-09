# Nested Buffer Read Implementation Plan

> **REQUIRED SUB-SKILL:** Use the executing-plans skill to implement this plan task-by-task.

**Goal:** Implement support for reading anchors from buffer copies (nested anchoring) per SPEC §4.3, enabling multi-level anchoring within already-matched regions.

**Architecture:** 
- The `read` command will accept a `--label` or `--true-id` parameter to specify a parent anchor in the buffer
- When specified, the command reads from `{TMPDIR}/anchorscope/{file_hash}/{parent_true_id}/content` instead of the original file
- New True ID is computed as `xxh3_64(parent_true_id + "_" + region_hash)` per SPEC §3.2
- Buffer structure: `{file_hash}/{parent_true_id}/{true_id}/content` for nested levels

**Tech Stack:** Rust, xxh3_64, serde, clap

---

## Task 1: Add `--label` parameter to `read` command

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/commands/read.rs`
- Modify: `src/main.rs`

**Step 1: Add label parameter to CLI**

```rust
Read {
    /// Path to the target file.
    #[arg(long)]
    file: String,

    /// Anchor string.
    #[arg(long)]
    anchor: Option<String>,

    /// Path to a file containing the anchor string.
    #[arg(long)]
    anchor_file: Option<String>,

    /// Use a human-readable label to identify the parent buffer anchor.
    #[arg(long, conflicts_with_all = ["anchor_file"])]
    label: Option<String>,
}
```

**Step 2: Update read command signature to accept label**

```rust
pub fn execute(
    file_path: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    label: Option<&str>,
) -> i32 {
```

**Step 3: Update main.rs to pass label to read**

```rust
Command::Read { file, anchor, anchor_file, label } => {
    commands::read::execute(&file, anchor.as_deref(), anchor_file.as_deref(), label.as_deref())
}
```

**Step 4: Run build to verify**

```bash
cargo build
```

Expected: No errors

**Step 5: Commit**

```bash
git add src/cli.rs src/commands/read.rs src/main.rs
git commit -m "feat: add --label parameter to read command"
```

---

## Task 2: Implement label-to-buffer-path resolution

**Files:**
- Modify: `src/commands/read.rs`

**Step 1: Add label resolution logic**

```rust
// If label is provided, resolve to buffer content path
let (target_file, buffer_source) = if let Some(label_name) = label {
    // Load label mapping to get true_id
    let true_id = storage::load_label_target(label_name)?;
    
    // Load source path from file buffer
    let file_hash = compute_file_hash(&file_path); // Need to compute from file
    let source_path = storage::load_source_path(&file_hash)?;
    
    // Determine if we're reading from root buffer or nested buffer
    let buffer_path = if is_root_level(&true_id, &file_hash) {
        buffer_path::file_dir(&file_hash).join("content")
    } else {
        buffer_path::true_id_dir(&file_hash, &true_id).join("content")
    };
    
    (buffer_path, true)
} else {
    (file_path.to_string(), false)
};
```

**Step 2: Update buffer reading to use resolved path**

**Step 3: Write failing test**

```rust
#[test]
fn test_read_with_label() {
    // Create file and read with label
    let (_temp_dir, file_path) = create_temp_file("Hello\nWorld\n");
    
    let read_out = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(), "--anchor", "Hello"
    ]);
    assert!(read_out.status.success());
    
    let result = parse_output(&String::from_utf8_lossy(&read_out.stdout));
    let true_id = result.get("true_id").unwrap().clone();
    
    let label_out = run_anchorscope(&[
        "label", "--name", "test", "--true-id", &true_id
    ]);
    assert!(label_out.status.success());
    
    // Read using label (nested mode)
    let nested_out = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(), "--label", "test", "--anchor", "Hello"
    ]);
    assert!(nested_out.status.success());
}
```

**Step 4: Run test to verify failure**

```bash
cargo test --test nested_buffer_tests test_read_with_label
```

Expected: Test fails (label support not implemented)

**Step 5: Implement and verify**

```bash
cargo test
```

Expected: All tests pass

**Step 6: Commit**

```bash
git add src/commands/read.rs tests/integration/nested_buffer_tests.rs
git commit -m "feat: implement label-based nested buffer read"
```

---

## Task 3: Implement nested True ID computation

**Files:**
- Modify: `src/trueid.rs`

**Step 1: Update compute function to accept buffer context**

```rust
/// Compute True ID for a region from buffer content
/// When parent_true_id is Some, computes xxh3_64(parent_true_id + "_" + region_hash)
/// When parent_true_id is None, computes xxh3_64(file_hash + "_" + region_hash)
pub fn compute_from_buffer(
    file_hash: &str,
    region_hash: &str,
    parent_true_id: Option<&str>,
) -> String {
    if let Some(parent) = parent_true_id {
        hash::compute(format!("{}_{}", parent, region_hash).as_bytes())
    } else {
        hash::compute(format!("{}_{}", file_hash, region_hash).as_bytes())
    }
}
```

**Step 2: Update read command to use new function**

**Step 3: Write failing test**

```rust
#[test]
fn test_nested_true_id_computation() {
    // Verify True ID is computed with parent context
    // True ID = xxh3_64(parent_true_id + "_" + region_hash)
}
```

**Step 4: Run test to verify failure**

**Step 5: Implement and verify**

**Step 6: Commit**

```bash
git add src/trueid.rs
git commit -m "feat: implement nested True ID computation"
```

---

## Task 4: Implement buffer path detection

**Files:**
- Modify: `src/commands/read.rs`

**Step 1: Detect if target is a buffer file**

```rust
/// Check if file_path points to a buffer content file
/// Returns (file_hash, true_id) if it's a buffer file, None otherwise
fn is_buffer_file(file_path: &str) -> Option<(String, String)> {
    // Check if path matches {TMPDIR}/anchorscope/{file_hash}/content
    // Or {TMPDIR}/anchorscope/{file_hash}/{true_id}/content
}
```

**Step 2: Extract parent_true_id from buffer path**

**Step 3: Write failing test**

```rust
#[test]
fn test_buffer_path_detection() {
    // Verify buffer paths are correctly identified
}
```

**Step 4: Implement and verify**

**Step 5: Commit**

```bash
git add src/commands/read.rs
git commit -m "feat: implement buffer path detection"
```

---

## Task 5: Update tree command to show nested structure

**Files:**
- Modify: `src/commands/tree.rs`

**Step 1: Recursively display buffer structure**

```rust
fn show_buffer_hierarchy(dir: &Path, prefix: &str, depth: usize) {
    // Show True IDs with proper indentation
    // Show aliases for each True ID
}
```

**Step 2: Display full hierarchy with nesting depth**

**Step 3: Write failing test**

```rust
#[test]
fn test_tree_shows_nested_hierarchy() {
    // Create nested anchors and verify tree shows them with indentation
}
```

**Step 4: Implement and verify**

**Step 5: Commit**

```bash
git add src/commands/tree.rs
git commit -m "feat: tree command displays nested buffer hierarchy"
```

---

## Task 6: Add integration tests for nested anchoring

**Files:**
- Modify: `tests/integration/nested_buffer_tests.rs`

**Step 1: Test full nested flow**

```rust
#[test]
fn test_nested_anchoring_full_flow() {
    // 1. Read root level
    // 2. Create label
    // 3. Read nested level using label
    // 4. Verify nested True ID
    // 5. Write using nested label
    // 6. Verify buffer cleanup
}
```

**Step 2: Test HASH_MISMATCH with nested anchors**

**Step 3: Run all tests**

```bash
cargo test
```

Expected: All tests pass

**Step 4: Commit**

```bash
git add tests/integration/nested_buffer_tests.rs
git commit -m "test: add nested anchoring integration tests"
```

---

## Execution Handoff

Plan complete and saved to `docs/plans/2026-04-09-nested-buffer-read.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session in worktree with executing-plans, batch execution with checkpoints

**Which approach?**
