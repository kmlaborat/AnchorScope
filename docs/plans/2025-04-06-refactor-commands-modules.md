# Commands Module Refactoring Plan

> **REQUIRED SUB-SKILL:** Use the executing-plans skill to implement this plan task-by-task.

**Goal:** Refactor main.rs by extracting command logic into separate `commands/` modules, improving maintainability and separation of concerns.

**Architecture:**
- Create `src/commands/` directory with `mod.rs`, `read.rs`, `write.rs`, `anchor.rs`
- Move `cmd_read`, `cmd_write`, `cmd_anchor` functions to respective modules
- Keep shared utilities (`map_io_error_read`, `map_io_error_write`, `validate_utf8`) in `main.rs` or create `src/utils.rs`
- Simplify `main.rs` to only handle CLI parsing and command dispatch
- Ensure all tests continue to pass

**Tech Stack:** Rust (edition 2021)

---

## Task 1: Create commands/ Module Structure

**Files:**
- Create: `src/commands/mod.rs`
- Create: `src/commands/read.rs`
- Create: `src/commands/write.rs`
- Create: `src/commands/anchor.rs`

**Step 1: Create src/commands/mod.rs**

```rust
pub mod read;
pub mod write;
pub mod anchor;
```

**Step 2: Create src/commands/read.rs**

Copy the entire `cmd_read` function from `main.rs`:

```rust
use crate::matcher;
use crate::hash;

/// Read: locate anchor, print location + hash. Exit 0 on success, 1 on error.
pub fn execute(
    file_path: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
) -> i32 {
    // ... (copy entire cmd_read body from main.rs)
}
```

**Step 3: Create src/commands/write.rs**

Copy `cmd_write` similarly:

```rust
use crate::matcher;
use crate::hash;

/// Write: locate anchor, verify hash, replace, write back. Exit 0 or 1.
pub fn execute(
    file_path: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    expected_hash: &str,
    replacement: &str,
) -> i32 {
    // ... (copy entire cmd_write body from main.rs)
}
```

**Step 4: Create src/commands/anchor.rs**

Copy `cmd_anchor`:

```rust
use crate::matcher;
use crate::hash;
use dirs;
use serde_json;

/// Anchor: verify anchor matches expected_hash, then store label mapping.
pub fn execute(
    file_path: &str,
    label: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    expected_hash: &str,
) -> i32 {
    // ... (copy entire cmd_anchor body from main.rs)
}
```

**Step 5: Verify compilation**

```bash
cargo check --quiet
```

Expected: Compilation errors (functions still in main.rs, not yet called from new modules)

**Step 6: Update main.rs to use new modules**

- Add `mod commands;` at top
- Update `use` statements if needed
- Replace `cmd_read` with `commands::read::execute`
- Replace `cmd_write` with `commands::write::execute`
- Replace `cmd_anchor` with `commands::anchor::execute`

**Step 7: Remove old function definitions from main.rs**

Delete `fn cmd_read(...)`, `fn cmd_write(...)`, `fn cmd_anchor(...)` entirely from main.rs.

**Step 8: Run tests**

```bash
cargo test --quiet
```

Expected: All tests pass (44 tests)

**Step 9: Commit**

```bash
git add src/commands/ src/main.rs
git commit -m "refactor: extract command logic into separate modules"
```

---

## Task 2: Handle Shared Utilities (Optional cleanup)

**Files:**
- `src/main.rs` (keep or move utils)
- Consider creating `src/utils.rs`

**Analysis:** After extraction, remaining in `main.rs`:
- `map_io_error_read`
- `map_io_error_write`
- `validate_utf8`
- `load_anchor`
- `normalize_line_endings` (already in `matcher.rs`)

`validate_utf8` is only used by `anchor` command? Actually check:
- `cmd_read`: uses `validate_utf8`? No
- `cmd_write`: uses `validate_utf8` on `replacement`? Only `validate_utf8` is defined but not used currently (see main.rs line 78-81). It's defined but unused in v1.1.0? Let's check.

**Decision:** Leave utilities in `main.rs` for now (they are small). Future refactor can extract if needed.

**No changes needed in this task** – skip.

---

## Task 3: Update Tests if Needed

**Files:**
- `tests/integration/anchor_command_tests.rs`
- `tests/integration/validation_order_tests.rs`

**Check:** Tests call `anchorscope` binary, not internal functions. No changes needed.

Run full test suite to confirm:

```bash
cargo test --quiet
```

**Step: Run and verify all 44 tests pass**

Expected: All pass.

**Step: Commit if any test fixes needed** (likely none)

---

## Task 4: Final Verification

**Steps:**
1. `cargo check --quiet`
2. `cargo test --quiet` (all 44 tests)
3. `cargo clippy --quiet` (if available)
4. Verify help output unchanged:
   ```bash
   cargo run -- --help
   ```

**Step: Commit any final tweaks**

---

## Task 5: Complete Development

After all tasks complete:
- Announce: "I'm using the finishing-a-development-branch skill to complete this work."
- Use `/skill:finishing-a-development-branch` to merge or create PR

---

## Execution Notes

- Do NOT modify `src/cli.rs` (CLI definitions stay)
- Do NOT modify `src/matcher.rs` or `src/hash.rs`
- Keep the same function signatures; only move bodies
- Ensure imports are correct after move
- Preserve all error messages exactly

---

## Expected Outcome

- `src/main.rs` reduced from ~267 lines to ~50 lines
- `src/commands/` directory with 4 files (~30-100 lines each)
- All existing functionality preserved
- All tests passing
- Clean git commit history

---

## Ready to Execute

Plan saved to `docs/plans/2025-04-06-refactor-commands-modules.md`.

**Two execution options:**

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task

**2. Parallel Session (separate)** - Open new session with executing-plans

Which approach would you like?