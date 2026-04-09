# AnchorScope v1.2.0 Integration Plan

> **REQUIRED SUB-SKILL:** Use the executing-plans skill to implement this plan task-by-task.

**Goal:** Complete the v1.2.0 implementation by integrating the existing True ID and nested buffer infrastructure into the main commands, and ensure all v1.2.0 features are properly exposed through the CLI.

**Architecture:** 
- Complete the integration of v1.2.0 features that are partially implemented:
  - `tree` command - display buffer structure
  - `trueid` command - compute True ID for nested anchoring
- Ensure True ID computation is properly integrated into read/write operations
- Fix buffer structure to match SPEC §4.2 exactly (root content at file_hash/content)

**Tech Stack:** Rust, xxh3_64, serde, clap

---

## Task 1: Export tree and trueid commands in commands module

**Files:**
- Modify: `src/commands/mod.rs`

**Step 1: Add tree and trueid to commands module**

```rust
pub mod read;
pub mod write;
pub mod label;
pub mod tree;
pub mod trueid;
```

**Step 2: Run build to verify compilation**

```bash
cargo build
```

Expected: No compilation errors

**Step 3: Commit**

```bash
git add src/commands/mod.rs
git commit -m "feat: export tree and trueid commands"
```

---

## Task 2: Update main.rs to use tree and trueid commands

**Files:**
- Modify: `src/main.rs`

**Step 1: Update main function to handle tree and trueid commands**

```rust
Command::Tree { file } => commands::tree::execute(&file),
Command::TrueId { ... } => commands::trueid::execute(...),
```

**Step 2: Run build to verify**

```bash
cargo build
```

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: integrate tree and trueid commands into main"
```

---

## Task 3: Fix True ID computation to match SPEC §3.2

**Files:**
- Modify: `src/trueid.rs`
- Modify: `src/commands/read.rs`

**Step 1: Implement True ID computation per SPEC §3.2**

Root level:
```
true_id = xxh3_64(file_hash + "_" + region_hash)
```

Nested level:
```
true_id = xxh3_64(parent_true_id + "_" + region_hash)
```

**Step 2: Update read command to use proper True ID**

Current: `println!("true_id={}", h);` (just uses region hash)
Should use: `trueid::compute(file_path, &region_hash, None)`

**Step 3: Write failing test for True ID computation**

```rust
#[test]
fn test_true_id_root_level_computation() {
    let (_temp_dir, file_path) = create_temp_file("Hello\nWorld\n");
    
    let out = run_anchorscope(&["read", "--file", file_path.to_str().unwrap(), "--anchor", "Hello"]);
    let result = parse_output(&String::from_utf8_lossy(&out.stdout));
    let true_id = result.get("true_id").unwrap();
    let hash = result.get("hash").unwrap();
    
    // Compute expected true_id
    let file_raw = fs::read(&file_path).unwrap();
    let normalized = crate::matcher::normalize_line_endings(&file_raw);
    let file_hash = crate::hash::compute(&normalized);
    let expected_true_id = crate::hash::compute(format!("{}_{}", file_hash, hash).as_bytes());
    
    assert_eq!(true_id, &expected_true_id);
}
```

**Step 4: Run test to verify failure**

```bash
cargo test --test true_id_computation
```

**Step 5: Implement and verify**

**Step 6: Commit**

```bash
git add src/trueid.rs src/commands/read.rs
git commit -m "feat: fix True ID computation to match SPEC §3.2"
```

---

## Task 4: Fix buffer structure per SPEC §4.2

**Files:**
- Modify: `src/storage.rs` - fix save_anchor_metadata_with_true_id

**Step 1: Root level content should be stored at file_hash/content, NOT file_hash/{true_id}/content**

Currently the code saves to `{file_hash}/{true_id}/content` but SPEC says root level should be at `{file_hash}/content`.

**Step 2: Update save_anchor_metadata_with_true_id**

```rust
// Root level: save content to file_hash/content
let content_path = buffer_path::file_dir(&file_hash).join("content");
fs::write(&content_path, &normalized)?;

// Then save True ID level content
save_buffer_content(&file_hash, true_id, &normalized)?;
```

**Step 3: Write failing test**

```rust
#[test]
fn test_root_buffer_content_at_file_hash_content() {
    let (_temp_dir, file_path) = create_temp_file("Hello\nWorld\n");
    
    // After read, content should be at file_hash/content
    // and True ID content should be at file_hash/true_id/content
}
```

**Step 4: Implement and verify**

**Step 5: Commit**

```bash
git add src/storage.rs
git commit -m "refactor: fix buffer structure per SPEC §4.2"
```

---

## Task 5: Implement nested anchor support

**Files:**
- Modify: `src/commands/read.rs` - support reading from buffer copies
- Modify: `src/storage.rs` - add save_nested_buffer_content

**Step 1: Add nested buffer content functions**

```rust
/// Save buffer content for nested level: {file_hash}/{parent_true_id}/{true_id}/content
pub fn save_nested_buffer_content(file_hash: &str, parent_true_id: &str, true_id: &str, content: &[u8]) -> Result<(), String> {
    let dir = buffer_path::nested_true_id_dir(file_hash, parent_true_id, true_id);
    ensure_dir(&dir)?;
    let path = dir.join("content");
    fs::write(&path, content)
        .map_err(|e| io_error_to_spec(e, "write failure"))
}
```

**Step 2: Update read command to detect buffer input and save nested level**

When the input file_path points to a buffer content file (file_hash/content or file_hash/true_id/content):
- Extract parent_true_id from path
- Save nested buffer with parent context

**Step 3: Write failing test**

```rust
#[test]
fn test_nested_anchor_storage() {
    // Test reading from a buffer copy creates nested structure
}
```

**Step 4: Implement and verify**

**Step 5: Commit**

```bash
git add src/commands/read.rs src/storage.rs
git commit -m "feat: implement nested anchor support"
```

---

## Task 6: Add tree command display nested structure

**Files:**
- Modify: `src/commands/tree.rs`

**Step 1: Update tree command to display full buffer hierarchy**

Current: Only shows aliases
Should: Show file_hash → True IDs → nested True IDs with indentation

```rust
fn show_buffer_hierarchy(dir: &Path, prefix: &str) {
    // Recursively show directory contents
    // Show True IDs with their aliases
    // Show nested levels with proper indentation
}
```

**Step 2: Write failing test**

```rust
#[test]
fn test_tree_shows_nested_structure() {
    // Create nested anchors and verify tree displays them correctly
}
```

**Step 3: Implement and verify**

**Step 4: Commit**

```bash
git add src/commands/tree.rs
git commit -m "feat: tree command displays nested buffer structure"
```

---

## Task 7: Add tests for v1.2.0 features

**Files:**
- Create: `tests/integration/v1_2_0_tests.rs`

**Step 1: Test True ID computation**

```rust
#[test]
fn test_true_id_root_level() { ... }

#[test]
fn test_true_id_nested_level() { ... }
```

**Step 2: Test label with True ID**

```rust
#[test]
fn test_label_with_true_id() { ... }
```

**Step 3: Test tree command**

```rust
#[test]
fn test_tree_command() { ... }
```

**Step 4: Run all tests**

```bash
cargo test
```

Expected: All 45+ tests pass

**Step 5: Commit**

```bash
git add tests/integration/v1_2_0_tests.rs
git commit -m "test: add v1.2.0 feature tests"
```

---

## Execution Handoff

Plan complete and saved to `docs/plans/2026-04-09-anchorscope-v1-2-0-integration.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session in worktree with executing-plans, batch execution with checkpoints

**Which approach?**
