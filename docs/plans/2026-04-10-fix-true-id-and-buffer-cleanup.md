# Fix AnchorScope True ID & Buffer Cleanup Implementation Plan

> **REQUIRED SUB‑SKILL:** Use the `executing-plans` skill to implement this plan task‑by‑task.

**Goal:** Make AnchorScope’s True ID generation conform to the v1.2.0 specification and ensure that successful `write` operations clean up all related buffer artefacts.

**Architecture:**
- `read` will compute True ID using the *parent region hash* (or file hash for level‑1) and the *child region hash* as defined in SPEC §3.1.
- `write` will, after a successful replace, locate the buffer directory that corresponds to the used True ID and delete it together with any nested children.
- Helper utilities will expose `file_hash_for_true_id` and clean up label files.

**Tech Stack:** Rust 1.78+, `serde_json`, `xxhash-rust`, `clap`, standard library I/O.

---

### Task 1: Add Utility to Resolve Parent Region Hash & File‑Hash

**Files:**
- Modify `src/storage.rs` – add `fn file_hash_for_true_id(true_id: &str) -> Result<String, String>`
- Modify `src/buffer_path.rs` – no changes needed, just used by the new function.

**Test:** `tests/integration/true_id_resolution_tests.rs` (new).

**Step 1: Write the failing test**
```rust
#[test]
fn resolves_file_hash_for_true_id() {
    // Setup a fake buffer hierarchy using storage helpers
    // (save a file, then a nested read to generate true_id)
    let true_id = /* result of nested read */;

    let file_hash = storage::file_hash_for_true_id(&true_id).unwrap();
    assert!(!file_hash.is_empty());
}
```

**Step 2: Run test to verify it fails** – `cargo test --test true_id_resolution_tests`.

**Step 3: Implement the minimal code**
- Scan all `{TMPDIR}/anchorscope` directories (as already done in `find_file_hash_for_true_id`).
- When a matching `true_id` is found, return the parent directory name (the file hash).
- Return an `IO_ERROR` if not found.

**Step 4: Run test to verify it passes** – `cargo test`.

**Step 5: Commit**
```bash
git add src/storage.rs tests/integration/true_id_resolution_tests.rs
git commit -m "feat: utility to map True ID → file hash"
```

---

### Task 2: Correct True ID Generation for Nested Anchors

**Files:**
- `src/commands/read.rs` – replace the current True ID calculation with the spec‑compliant formula.
- `src/matcher.rs` – no changes required.

**Test:** Extend `tests/integration/true_id_generation_tests.rs` (new) to cover level‑1 and level‑2 anchors.

**Step 1: Write the failing test**
```rust
#[test]
fn true_id_is_computed_from_parent_and_child_hashes() {
    // 1️⃣ Read a file → get first true_id (level‑1)
    let level1 = run_read(...);
    // 2️⃣ Read using label/parent true_id → get level2 true_id
    let level2 = run_read_with_parent(level1.true_id.clone(), ...);
    // Expected: level2 != level1 and matches spec formula
    assert_ne!(level1.true_id, level2.true_id);
}
```

**Step 2: Run test – it will panic because current code hashes the *entire parent buffer* instead of the parent region hash.**

**Step 3: Implement the minimal code**
- After a successful match, compute `region_hash = hash(region_bytes)`.
- Retrieve `parent_region_hash`:
  - For level‑1: `parent_region_hash = file_hash` (already computed).
  - For nested reads: load parent buffer’s `metadata.json` via `storage::load_buffer_metadata` and use its `region_hash`.
- Compute `true_id = hash(format!("{}_{}", parent_region_hash, region_hash).as_bytes())`.
- Store the new `true_id` and its `parent_true_id` in buffer metadata (`save_buffer_metadata`).

**Step 4: Run test – should now pass.**

**Step 5: Commit**
```bash
git add src/commands/read.rs tests/integration/true_id_generation_tests.rs
git commit -m "fix: True ID follows SPEC §3.1 for nested anchors"
```

---

### Task 3: Make `write` Clean Up All Buffer artefacts

**Files:**
- `src/commands/write.rs` – replace the current `invalidate_anchor` call with proper buffer cleanup using the new utility from Task 1.
- `src/storage.rs` – add `fn invalidate_true_id_hierarchy(file_hash: &str, true_id: &str) -> Result<(), String>` that recursively removes the directory `{file_hash}/{true_id}` and all descendants.

**Test:** `tests/integration/write_cleanup_tests.rs` (new).

**Step 1: Write the failing test**
```rust
#[test]
fn write_deletes_buffer_hierarchy_and_label() {
    // 1️⃣ Perform a read to generate a true_id and a label.
    let true_id = run_read(...).true_id;
    storage::save_label_mapping("tmp_label", &true_id).unwrap();

    // 2️⃣ Run write using the label.
    let exit = run_write_with_label("tmp_label", ...);
    assert_eq!(exit, 0);

    // 3️⃣ Verify buffer directory is gone.
    let file_hash = storage::file_hash_for_true_id(&true_id).unwrap();
    let dir = buffer_path::true_id_dir(&file_hash, &true_id);
    assert!(!dir.exists());

    // 4️⃣ Verify label file is removed.
    let label_path = buffer_path::labels_dir().join("tmp_label.json");
    assert!(!label_path.exists());
}
```

**Step 2: Run test – it will fail because `write` only removes the old anchor JSON.**

**Step 3: Implement the minimal code**
- Resolve the True ID (via label or expected hash).
- Call `storage::file_hash_for_true_id` to obtain `file_hash`.
- Invoke `storage::invalidate_true_id_hierarchy(&file_hash, &true_id)`.
- After that, call `storage::invalidate_label(label_name)` if a label was used.

**Step 4: Run test – should now pass.**

**Step 5: Commit**
```bash
git add src/commands/write.rs src/storage.rs tests/integration/write_cleanup_tests.rs
git commit -m "fix: write now removes buffer hierarchy and associated label"
```

---

### Task 4: Strip Debug `eprintln!` Calls from Production Code

**Files:**
- `src/commands/read.rs` – remove all `eprintln!("DEBUG …")`.
- `src/storage.rs` – remove debug prints in `load_anchor_metadata_by_true_id` and `print_all_buffers`.
- `src/commands/write.rs` – remove debug prints around label resolution.

**Test:** No new test needed; compilation must succeed and CI must not see stray debug output.

**Step 1: Write the failing test** – not needed; we’ll rely on `cargo test` output (no unexpected `stderr` during normal runs).

**Step 2: Run `cargo test` – currently prints debug lines, causing noisy test output.

**Step 3: Implement the minimal code** – delete the `eprintln!` statements or guard them with `#[cfg(debug_assertions)]`.

**Step 4: Run `cargo test` – output should be clean.

**Step 5: Commit**
```bash
git add src/commands/read.rs src/storage.rs src/commands/write.rs
git commit -m "chore: remove debug prints from production code"
```

---

### Task 5: Add Comprehensive Integration Tests for Nested Anchors

**Files:**
- `tests/integration/nested_anchor_tests.rs` (new).

**Step 1: Write the failing test** – create a temporary file with a large outer region and a unique inner region, perform two successive `read` calls (outer then inner), assert that the second True ID matches the spec formula and that `write` on the inner anchor succeeds and cleans up correctly.

**Step 2: Run test – fails because True ID and cleanup are still buggy.

**Step 3: After completing Tasks 1‑4, re‑run the test – it should now pass.

**Step 4: Commit**
```bash
git add tests/integration/nested_anchor_tests.rs
git commit -m "test: full integration test for multi‑level anchoring"
```

---

### Task 6: Update Documentation

**Files:**
- `docs/SPEC.md` – add a short note referencing the new buffer‑cleanup behaviour.
- `docs/plans/2026-04-10-fix-true-id-and-buffer-cleanup.md` – this plan file itself (already saved).

**Step 1: Write the changes** – edit the two markdown files.

**Step 2: Run `cargo test` to ensure no breakage.**

**Step 3: Commit**
```bash
git add docs/SPEC.md docs/plans/2026-04-10-fix-true-id-and-buffer-cleanup.md
git commit -m "docs: note buffer cleanup and True ID generation details"
```
