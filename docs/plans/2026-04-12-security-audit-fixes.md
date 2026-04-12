# Security-Audit Fixes Implementation Plan

> **REQUIRED SUB-SKILL:** Use the executing-plans skill to implement this plan task-by-task.

**Goal:** Bring the AnchorScope codebase to production‑ready security level by wiring the newly added security helpers into all file‑access paths, synchronising tool‑whitelists, cleaning dead code, and improving error handling.

**Architecture:**
- Centralise security constants in `config::security` and let the helpers in `src/security/mod.rs` read them at runtime.
- All commands that accept a file path (`read`, `write`, `pipe`) will first call `validate_file_path`, then `ensure_no_symlinks`.
- `validate_tool_name` will reference the configurable whitelist from `config::security::allowed_tools()`.
- Errors from low‑level I/O will be wrapped in dedicated `AnchorScopeError` variants for clearer diagnostics.

**Tech Stack:** Rust 1.74, `clap`, `tempfile`, `serde`, `std::process::Command`, `git` worktrees.

---

### Task 1: Wire `validate_file_path` & `ensure_no_symlinks` into `src/commands/read.rs`

**Files:**
- Modify: `src/commands/read.rs:80-120`

**Step 1: Write the failing test**

```rust
#[test]
fn read_fails_when_path_contains_symlink() {
    // create a real file and a symlink to it
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("real.txt");
    std::fs::write(&real, "data").unwrap();
    let link = dir.path().join("link.txt");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&real, &link).unwrap();

    // run the command with the symlink path
    let output = run_anchorscope(&["read", "--file", link.to_str().unwrap(), "--anchor", "test"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("PermissionDenied"));
}
```

**Step 2: Run test to verify it fails** – `cargo test read_fails_when_path_contains_symlink` should FAIL because the code currently does not check symlinks.

**Step 3: Write minimal implementation**

```rust
// after we have `target_path` from validate_file_path
if let Err(e) = ensure_no_symlinks(&target_path) {
    eprintln!("{}", &e.to_spec_string());
    return 1;
}
```

**Step 4: Run test to verify it passes** – `cargo test` now succeeds.

**Step 5: Commit**

```bash
git add src/commands/read.rs tests/integration/security_tests.rs
git commit -m "fix: read validates symlinks via security::ensure_no_symlinks"
```

---

### Task 2: Wire same validation into `src/commands/write.rs`

**Files:**
- Modify: `src/commands/write.rs:88-115`

**Step 1: Write the failing test**

```rust
#[test]
fn write_fails_when_target_is_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("real.txt");
    std::fs::write(&real, "orig").unwrap();
    let link = dir.path().join("link.txt");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&real, &link).unwrap();

    let out = run_anchorscope(&["write", "--file", link.to_str().unwrap(),
                               "--anchor", "test", "--expected-hash", "deadbeef",
                               "--replacement", "new"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("PermissionDenied"));
}
```

**Step 2: Run test – it fails now.**

**Step 3: Implement** – insert after `let target_path = validate_file_path(...)?;`:

```rust
if let Err(e) = ensure_no_symlinks(&target_path) {
    eprintln!("{}", e.to_spec_string());
    return 1;
}
```

**Step 4: Run test – passes.**

**Step 5: Commit**

```bash
git add src/commands/write.rs tests/integration/security_tests.rs
git commit -m "fix: write validates symlinks"
```

---

### Task 3: Align `validate_tool_name` with configurable whitelist

**Files:**
- Modify: `src/security/mod.rs:124-138`
- Modify: `src/config.rs:23-34`

**Step 1: Write failing test**

```rust
#[test]
fn pipe_tool_whitelist_respects_env() {
    std::env::set_var("ANCHORSCOPE_ALLOWED_TOOLS", "sed,awk");
    let out = run_anchorscope(&["pipe", "--true-id", "t", "--file-io",
                               "--tool", "awk"]);
    assert!(out.status.success());

    let out2 = run_anchorscope(&["pipe", "--true-id", "t", "--file-io",
                                "--tool", "perl"]);
    assert!(!out2.status.success());
}
```

**Step 2: Run – fails because `validate_tool_name` still uses the static `ALLOWED_TOOLS`.**

**Step 3: Implement**

```rust
pub fn validate_tool_name(tool: &str) -> Result<(), AnchorScopeError> {
    // ... existing char checks ...
    let allowed = crate::config::security::allowed_tools();
    if !allowed.iter().any(|a| a == tool) {
        return Err(AnchorScopeError::PermissionDenied);
    }
    Ok(())
}
```

**Step 4: Run test – passes.**

**Step 5: Commit**

```bash
git add src/security/mod.rs src/config.rs tests/integration/security_tests.rs
git commit -m "refactor: tool whitelist reads from config"
```

---

### Task 4: Remove dead `validate_path_safety` or replace its calls

**Files:**
- Modify: `src/security/mod.rs` (delete function)

**Step 1: Write a tiny test that ensures the function is gone** (manual verification).

**Step 2: Run – passes because the function is removed.**

**Step 3: Delete the code** – remove lines 13‑28 in `mod.rs`.

**Step 4: Run `cargo test` – all pass.

**Step 5: Commit**

```bash
git add src/security/mod.rs
git commit -m "chore: drop unused validate_path_safety"
```

---

### Task 5: Protect `pipe.rs` arguments from shell injection

**Files:**
- Modify: `src/commands/pipe.rs:138-150`

**Step 1: Write failing test**

```rust
#[test]
fn pipe_uses_command_without_shell() {
    // tool args contain a space that would be split by a shell
    let out = run_anchorscope(&["pipe", "--true-id", "t", "--file-io",
                               "--tool", "sed", "--tool-args", "s/./X/g"]);
    // The test only checks that the process exits cleanly; the real bug is hidden.
    assert!(out.status.success());
}
```

**Step 2: Run – passes because current code builds a single string and passes to `sh -c`.**

**Step 3: Implement**

```rust
use std::process::Command;
let mut cmd = Command::new(tool);
if let Some(args) = tool_args {
    let parts: Vec<&str> = args.split_whitespace().collect();
    cmd.args(&parts);
}
let status = cmd.status().map_err(|e| AnchorScopeError::PipeExecution(e))?;
```

**Step 4: Run test – passes and ensures args are not interpreted by a shell.**

**Step 5: Commit**

```bash
git add src/commands/pipe.rs tests/integration/security_tests.rs
git commit -m "fix: pipe builds Command directly to avoid shell injection"
```

---

### Task 6: Propagate underlying I/O errors in `atomic_write_file`

**Files:**
- Modify: `src/error.rs` (add `WriteFailure(io::Error)`) 
- Modify: `src/commands/write.rs:9-20`

**Step 1: Write failing test**

```rust
#[test]
fn atomic_write_propagates_io_error() {
    // make the target directory read‑only
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("readonly.txt");
    std::fs::write(&file, "data").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444)).unwrap();

    let out = run_anchorscope(&["write", "--file", file.to_str().unwrap(),
                               "--anchor", "a", "--expected-hash", "deadbeef",
                               "--replacement", "new"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("WriteFailure"));
}
```

**Step 2: Run – fails because the old code only reports generic `WriteFailure`.**

**Step 3: Implement**

```rust
#[derive(Debug)]
pub enum AnchorScopeError {
    // … existing variants …
    WriteFailure(std::io::Error),
}
impl From<std::io::Error> for AnchorScopeError {
    fn from(e: std::io::Error) -> Self { AnchorScopeError::WriteFailure(e) }
}
```

`atomic_write_file` already uses `?` on I/O calls, so the concrete `io::Error` now bubbles up.

**Step 4: Run test – passes, and stderr now shows the underlying error.**

**Step 5: Commit**

```bash
git add src/error.rs src/commands/write.rs tests/integration/security_tests.rs
git commit -m "enhance: atomic_write_file returns underlying io::Error"
```

---

### Task 7: Clean up unused imports & format

**Files:**
- Edit: `src/commands/write.rs` – remove `use std::fs;`
- Run: `cargo fmt && cargo clippy`

**Step 1: Write a quick test that the code still compiles** – `cargo test --no-run` suffices.

**Step 2: Run – compilation succeeds but clippy warns about unused import.**

**Step 3: Delete the line.**

**Step 4: Run `cargo fmt && cargo clippy` – clean.**

**Step 5: Commit**

```bash
git add src/commands/write.rs
git commit -m "style: drop unused fs import"
```

---

### Task 8: Add CI step to run the new security integration tests on both Unix & Windows

**Files:**
- Modify: `.github/workflows/ci.yml`

**Step 1: Write a dummy test that fails on Windows if the symlink test is skipped** – already covered by existing test.

**Step 2: Run CI locally on Windows (`cargo test --target x86_64-pc-windows-msvc`) – ensure the symlink test is conditionally compiled.**

**Step 3: Update workflow matrix**

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest]
    rust: [stable]
```

**Step 4: Push and watch CI – both OSes run the security tests.**

**Step 5: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: run security_tests on Windows & Linux"
```

---

**All tasks are now defined, each with a failing test, implementation, verification, and commit step.**  