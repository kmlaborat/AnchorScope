# Refactor Storage: System Temp + Virtual tmpfs + Lifecycle Management

> **REQUIRED SUB-SKILL:** Use the executing-plans skill to implement this plan task-by-task.

**Goal:** Migrate anchor/label storage from `~/.anchorscope/` to `std::env::temp_dir()/anchorscope/`, implement virtual tmpfs structure with automatic file cleanup after successful writes.

**Architecture:** Replace `dirs::home_dir()` with `std::env::temp_dir()` as the base path. Create a clean hierarchical `anchorscope/anchors/` and `anchorscope/labels/` structure. Add `invalidate_anchor(hash)` and `invalidate_label(name)` functions to delete ephemeral files after successful `write`, preventing disk "memory leaks."

**Tech Stack:** Rust (edition 2021), `std::env::temp_dir()`, `std::fs`.

---

## Task 1: Refactor `src/storage.rs` — System Temp + Lifecycle Functions

**Files:**
- Modify: `src/storage.rs`

**Step 1:** Replace `home_dir()` with `temp_dir()` and remove `dirs` dependency.

Replace `use dirs;` and the two `ensure_*_dir()` functions with:

```rust
fn anchorscope_temp_dir() -> Result<PathBuf, String> {
    let base = std::env::temp_dir().join("anchorscope");
    fs::create_dir_all(&base).map_err(|e| format!("IO_ERROR: cannot create temp dir: {}", e))?;
    Ok(base)
}

fn ensure_anchor_dir() -> Result<PathBuf, String> {
    let base = anchorscope_temp_dir()?;
    let dir = base.join("anchors");
    fs::create_dir_all(&dir).map_err(|e| format!("IO_ERROR: cannot create anchor dir: {}", e))?;
    Ok(dir)
}

fn ensure_label_dir() -> Result<PathBuf, String> {
    let base = anchorscope_temp_dir()?;
    let dir = base.join("labels");
    fs::create_dir_all(&dir).map_err(|e| format!("IO_ERROR: cannot create label dir: {}", e))?;
    Ok(dir)
}
```

**Step 2:** Add lifecycle cleanup functions. Append at end of `storage.rs`:

```rust
pub fn invalidate_anchor(hash: &str) {
    if let Ok(dir) = ensure_anchor_dir() {
        let path = dir.join(format!("{}.json", hash));
        let _ = fs::remove_file(path);
    }
}

pub fn invalidate_label(name: &str) {
    if let Ok(dir) = ensure_label_dir() {
        let path = dir.join(format!("{}.json", name));
        let _ = fs::remove_file(path);
    }
}
```

**Step 3:** Simplify `save_label_mapping` — remove legacy AnchorMeta fallback.

```rust
pub fn save_label_mapping(name: &str, internal_label: &str) -> Result<(), String> {
    let dir = ensure_label_dir()?;
    let path = dir.join(format!("{}.json", name));
    if path.exists() {
        let existing = fs::read_to_string(&path)
            .map_err(|e| format!("IO_ERROR: cannot read existing label: {}", e))?;
        let existing_meta: LabelMeta = serde_json::from_str(&existing)
            .map_err(|e| format!("IO_ERROR: existing label corrupted: {}", e))?;
        if existing_meta.internal_label != internal_label {
            return Err(format!("LABEL_EXISTS: label '{}' already points to a different internal label", name));
        }
    }
    let meta = LabelMeta { internal_label: internal_label.to_string() };
    let json = serde_json::to_string_pretty(&meta).map_err(|e| format!("IO_ERROR: JSON serialization failed: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("IO_ERROR: cannot write label mapping: {}", e))?;
    Ok(())
}
```

**Step 4:** Write the entire new file:

```rust
use std::path::PathBuf;
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnchorMeta {
    pub file: String,
    pub anchor: String,
    pub hash: String,
    pub line_range: (usize, usize),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LabelMeta {
    pub internal_label: String,
}

fn anchorscope_temp_dir() -> Result<PathBuf, String> {
    let base = std::env::temp_dir().join("anchorscope");
    fs::create_dir_all(&base).map_err(|e| format!("IO_ERROR: cannot create temp dir: {}", e))?;
    Ok(base)
}

fn ensure_anchor_dir() -> Result<PathBuf, String> {
    let base = anchorscope_temp_dir()?;
    let dir = base.join("anchors");
    fs::create_dir_all(&dir).map_err(|e| format!("IO_ERROR: cannot create anchor dir: {}", e))?;
    Ok(dir)
}

fn ensure_label_dir() -> Result<PathBuf, String> {
    let base = anchorscope_temp_dir()?;
    let dir = base.join("labels");
    fs::create_dir_all(&dir).map_err(|e| format!("IO_ERROR: cannot create label dir: {}", e))?;
    Ok(dir)
}

pub fn save_anchor_metadata(meta: &AnchorMeta) -> Result<(), String> {
    let dir = ensure_anchor_dir()?;
    let path = dir.join(format!("{}.json", meta.hash));
    let json = serde_json::to_string_pretty(meta).map_err(|e| format!("IO_ERROR: JSON serialization failed: {}", e))?;
    fs::write(path, json).map_err(|e| format!("IO_ERROR: cannot write anchor metadata: {}", e))?;
    Ok(())
}

pub fn load_anchor_metadata(hash: &str) -> Result<AnchorMeta, String> {
    let dir = ensure_anchor_dir()?;
    let path = dir.join(format!("{}.json", hash));
    let content = fs::read_to_string(&path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => format!("IO_ERROR: anchor metadata not found"),
        _ => format!("IO_ERROR: cannot read anchor metadata: {}", e),
    })?;
    serde_json::from_str(&content).map_err(|e| format!("IO_ERROR: anchor metadata corrupted: {}", e))
}

pub fn save_label_mapping(name: &str, internal_label: &str) -> Result<(), String> {
    let dir = ensure_label_dir()?;
    let path = dir.join(format!("{}.json", name));
    if path.exists() {
        let existing = fs::read_to_string(&path)
            .map_err(|e| format!("IO_ERROR: cannot read existing label: {}", e))?;
        let existing_meta: LabelMeta = serde_json::from_str(&existing)
            .map_err(|e| format!("IO_ERROR: existing label corrupted: {}", e))?;
        if existing_meta.internal_label != internal_label {
            return Err(format!("LABEL_EXISTS: label '{}' already points to a different internal label", name));
        }
    }
    let meta = LabelMeta { internal_label: internal_label.to_string() };
    let json = serde_json::to_string_pretty(&meta).map_err(|e| format!("IO_ERROR: JSON serialization failed: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("IO_ERROR: cannot write label mapping: {}", e))?;
    Ok(())
}

pub fn load_label_target(name: &str) -> Result<String, String> {
    let dir = ensure_label_dir()?;
    let path = dir.join(format!("{}.json", name));
    let content = fs::read_to_string(&path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => format!("IO_ERROR: label not found"),
        _ => format!("IO_ERROR: cannot read label mapping: {}", e),
    })?;
    let meta: LabelMeta = serde_json::from_str(&content).map_err(|e| format!("IO_ERROR: label mapping corrupted: {}", e))?;
    Ok(meta.internal_label)
}

pub fn invalidate_anchor(hash: &str) {
    if let Ok(dir) = ensure_anchor_dir() {
        let path = dir.join(format!("{}.json", hash));
        let _ = fs::remove_file(path);
    }
}

pub fn invalidate_label(name: &str) {
    if let Ok(dir) = ensure_label_dir() {
        let path = dir.join(format!("{}.json", name));
        let _ = fs::remove_file(path);
    }
}
```

**Step 5:** Run `cargo check --quiet`.

Expected: Clean compilation (may warn about unused `dirs` in Cargo.toml — that gets removed in Task 4).

**Step 6:** Commit

```bash
git add src/storage.rs
git commit -m "refactor: migrate storage to system temp dir with lifecycle cleanup"
```

---

## Task 2: Update `src/commands/write.rs` — Invalidate After Success

**Files:**
- Modify: `src/commands/write.rs`

**Step 1:** Capture the label name for later cleanup. Refactor the resolution block to also return a `used_label: Option<String>`:

```rust
pub fn execute(
    file_path: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    expected_hash: Option<&str>,
    label: Option<&str>,
    replacement: &str,
) -> i32 {
    let (target_file, anchor_bytes, expected_hash, used_label) = if let Some(label_name) = label {
        let internal_label = match crate::storage::load_label_target(label_name) {
            Ok(l) => l,
            Err(e) => { eprintln!("{}", e); return 1; }
        };
        let meta = match crate::storage::load_anchor_metadata(&internal_label) {
            Ok(m) => m,
            Err(e) => { eprintln!("{}", e); return 1; }
        };
        (meta.file, meta.anchor.into_bytes(), meta.hash, Some(label_name.to_string()))
    } else {
        let anchor_bytes = match crate::load_anchor(anchor, anchor_file) {
            Ok(a) => a,
            Err(e) => { eprintln!("{}", e); return 1; }
        };
        let expected_hash = match expected_hash {
            Some(h) => h.to_string(),
            None => {
                eprintln!("ERROR: expected-hash required when not using label");
                return 1;
            }
        };
        (file_path.to_string(), anchor_bytes, expected_hash, None)
    };
```

**Step 2:** After successful write, invalidate ephemeral files:

```rust
    match fs::write(&target_file, &result) {
        Ok(_) => {
            // Clean up ephemeral files after successful write (SPEC §3.3)
            if let Some(ref lname) = used_label {
                crate::storage::invalidate_label(lname);
            }
            crate::storage::invalidate_anchor(&expected_hash);
            println!("OK: written {} bytes", result.len());
            0
        }
        Err(e) => {
            eprintln!("{}", crate::map_io_error_write(e));
            1
        }
    }
```

**Step 3:** Run `cargo check --quiet` to verify.

**Step 4:** Commit

```bash
git add src/commands/write.rs
git commit -m "feat: invalidate ephemeral files after successful write"
```

---

## Task 3: Add Storage Lifecycle Tests

**Files:**
- Create: `tests/integration/storage_lifecycle_tests.rs`
- Modify: `tests/integration/mod.rs`

**Step 1:** Create `tests/integration/storage_lifecycle_tests.rs`:

```rust
use crate::test_helpers::*;
use std::path::PathBuf;

fn anchorscope_temp_dir() -> PathBuf {
    std::env::temp_dir().join("anchorscope")
}

#[test]
fn test_anchor_and_label_files_created() {
    let (_temp_dir, file_path) = create_temp_file("Hello\nWorld\n");

    let output = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(), "--anchor", "Hello"
    ]);
    assert!(output.status.success());
    let result = parse_output(&String::from_utf8_lossy(&output.stdout));
    let label_hash = result.get("label").unwrap().clone();

    let anchor_file = anchorscope_temp_dir().join("anchors").join(format!("{}.json", label_hash));
    assert!(anchor_file.exists(), "anchor metadata should exist after read");

    run_anchorscope(&[
        "label", "--name", "greeting", "--internal-label", &label_hash
    ]);

    let label_file = anchorscope_temp_dir().join("labels").join("greeting.json");
    assert!(label_file.exists(), "label mapping should exist after label command");
}

#[test]
fn test_write_using_label_invalidates_files() {
    let (_temp_dir, file_path) = create_temp_file("Hello\nWorld\n");

    let out = run_anchorscope(&[
        "read", "--file", file_path.to_str().unwrap(), "--anchor", "Hello"
    ]);
    assert!(out.status.success());
    let result = parse_output(&String::from_utf8_lossy(&out.stdout));
    let internal_label = result.get("label").unwrap().clone();

    let label_out = run_anchorscope(&[
        "label", "--name", "greet", "--internal-label", &internal_label
    ]);
    assert!(label_out.status.success());

    let anchor_file = anchorscope_temp_dir().join("anchors").join(format!("{}.json", internal_label));
    let label_file = anchorscope_temp_dir().join("labels").join("greet.json");

    assert!(anchor_file.exists());
    assert!(label_file.exists());

    let write_out = run_anchorscope(&[
        "write", "--label", "greet",
        "--replacement", "Hi",
        "--file", file_path.to_str().unwrap()
    ]);
    assert!(write_out.status.success());

    assert!(!anchor_file.exists(), "anchor file should be invalidated after write");
    assert!(!label_file.exists(), "label file should be invalidated after write");
}
```

**Step 2:** Add to `tests/integration/mod.rs`:

```rust
#[cfg(test)]
mod storage_lifecycle_tests;
```

**Step 3:** Run tests:
```bash
cargo test storage_lifecycle_tests
```

Expected: 2 tests pass.

**Step 4:** Commit

```bash
git add tests/integration/storage_lifecycle_tests.rs tests/integration/mod.rs
git commit -m "test: verify ephemeral file lifecycle (create on read/label, delete on write)"
```

---

## Task 4: Remove `dirs` Dependency & Update README

**Files:**
- Modify: `Cargo.toml`
- Modify: `README.md`

**Step 1:** Remove `dirs = "5"` from `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
xxhash-rust = { version = "0.8", features = ["xxh3"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**Step 2:** Run `cargo check --quiet`.

**Step 3:** Append to README.md after the Label section:

```markdown
### Storage (Ephemeral)

AnchorScope uses the system temporary directory (`std::env::temp_dir()/anchorscope/`)
for storing auto-generated label metadata. These files are ephemeral and are
automatically cleaned up after a successful `write` operation.

```
${TMPDIR}/anchorscope/
├── anchors/
│   └── {hash}.json    ← Auto-generated by `read`
└── labels/
    └── {name}.json    ← Created by `label`
```

You can inspect these files for debugging with `tree $(echo %TEMP%/anchorscope)` (Windows) or `tree $TMPDIR/anchorscope/` (Unix).
```

**Step 4:** Run `cargo test --quiet` — all tests must pass.

**Step 5:** Commit

```bash
git add Cargo.toml README.md
git commit -m "chore: remove dirs dependency, document ephemeral storage"
```

---

## Task 5: Final Verification

**Steps:**

1. `cargo test --quiet` — all tests pass (45+)
2. `bash examples/v1_1_0_showcase.sh` — end-to-end demo still works
3. Commit any remaining tweaks

```bash
git add -A
git commit -m "chore: final verification — all tests passing, showcase confirmed"
```

---

## Expected Outcome

| Before | After |
|--------|-------|
| `~/.anchorscope/` (home dir) | `${TMP}/anchorscope/` (system temp) |
| Files persist forever | Files deleted after successful `write` |
| `dirs` crate dependency | Only std library |
| No lifecycle management | `invalidate_anchor()` + `invalidate_label()` |

**SPEC §3.3 Compliant:** "Temporary in-memory copies may be cached for debugging or retry, but **do not alter file state** until atomic write" — now implemented as ephemeral temp files with automatic cleanup.

---

Plan complete and saved to `docs/plans/2025-04-07-temp-dir-ephemeral-storage.md`. **Which execution approach?**

**1. Subagent-Driven** (this session) **2. Parallel Session** (separate)
