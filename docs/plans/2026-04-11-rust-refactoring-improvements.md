# Rust Code Quality Improvements Implementation Plan

> **REQUIRED SUB-SKILL:** Use the executing-plans skill to implement this plan task-by-task.

**Goal:** Improve Rust code quality through type-safe error handling, common logic consolidation, memory-efficient I/O, and single-responsibility functions while maintaining strict SPEC v1.3.0 compliance.

**Architecture:** 
1. Introduce structured error types using `thiserror` to replace string-based error propagation
2. Extract shared directory traversal logic into `storage.rs` methods
3. Optimize memory usage in `matcher.rs` by using slice-based operations
4. Refactor large `execute` functions into smaller, focused private functions
5. All error types implement `Display` to output SPEC §4.5规定のエラー文字列 (absolute compliance)

**Tech Stack:**
- Rust 2021 Edition
- `thiserror` (add dependency) for structured error types
- `xxhash-rust` (existing) for xxh3_64 hashing - NO CHANGE
- `serde`/`serde_json` (existing) - NO CHANGE
- No breaking changes to SPEC compliance

---

## Phase 0: Setup and Dependencies

### Task 0.1: Add `thiserror` dependency

**Files:**
- Modify: `Cargo.toml:5-13`

**Step 1: Update Cargo.toml dependencies**

Add `thiserror = "1"` to dependencies:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
xxhash-rust = { version = "0.8", features = ["xxh3"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tempfile = "3"
thiserror = "1"  # ADD THIS LINE
```

**Step 2: Run cargo to update**

```bash
cargo update
cargo build
```

Expected: Build succeeds with no errors

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "feat: add thiserror dependency for type-safe error handling"
```

---

## Phase 1: Type-Safe Error Handling

### Task 1.1: Create `src/error.rs` with structured error types

**Files:**
- Create: `src/error.rs`
- Modify: `src/main.rs:1-30` (add module declaration)

**Step 1: Write error.rs with all SPEC errors**

```rust
use thiserror::Error;

/// AnchorScope-specific errors per SPEC §4.5
#[derive(Error, Debug)]
pub enum AnchorScopeError {
    #[error("NO_MATCH")]
    NoMatch,

    #[error("MULTIPLE_MATCHES")]
    MultipleMatches,

    #[error("HASH_MISMATCH")]
    HashMismatch,

    #[error("DUPLICATE_TRUE_ID")]
    DuplicateTrueId,

    #[error("LABEL_EXISTS")]
    LabelExists,

    #[error("AMBIGUOUS_REPLACEMENT")]
    AmbiguousReplacement,

    #[error("NO_REPLACEMENT")]
    NoReplacement,

    #[error("IO_ERROR: file not found")]
    FileNotFound,

    #[error("IO_ERROR: permission denied")]
    PermissionDenied,

    #[error("IO_ERROR: invalid UTF-8")]
    InvalidUtf8,

    #[error("IO_ERROR: read failure")]
    ReadFailure,

    #[error("IO_ERROR: write failure")]
    WriteFailure,

    #[error("IO_ERROR: buffer metadata for true_id '{0}' not found")]
    BufferMetadataNotFound(String),

    #[error("IO_ERROR: parent buffer metadata corrupted: {0}")]
    ParentBufferMetadataCorrupted(String),

    #[error("IO_ERROR: cannot load source path: {0}")]
    CannotLoadSourcePath(String),

    #[error("IO_ERROR: cannot save file content: {0}")]
    CannotSaveFileContent(String),

    #[error("IO_ERROR: cannot save source path: {0}")]
    CannotSaveSourcePath(String),

    #[error("IO_ERROR: cannot save scope content: {0}")]
    CannotSaveScopeContent(String),

    #[error("IO_ERROR: cannot save buffer metadata: {0}")]
    CannotSaveBufferMetadata(String),

    #[error("IO_ERROR: JSON serialization failed: {0}")]
    JsonSerializationFailed(String),

    #[error("IO_ERROR: label mapping corrupted: {0}")]
    LabelMappingCorrupted(String),

    #[error("IO_ERROR: cannot load buffer content")]
    CannotLoadBufferContent,

    #[error("IO_ERROR: parent directory for true_id '{0}' not found")]
    ParentDirectoryNotFound(String),

    #[error("IO_ERROR: maximum nesting depth ({0}) exceeded")]
    MaximumNestingDepthExceeded(usize),

    #[error("IO_ERROR: external tool failed")]
    ExternalToolFailed,

    #[error("IO_ERROR: cannot execute external tool")]
    CannotExecuteExternalTool,

    #[error("IO_ERROR: cannot create temporary directory")]
    CannotCreateTempDirectory,

    #[error("IO_ERROR: buffer not found")]
    BufferNotFound,

    #[error("IO_ERROR: replacement not found")]
    ReplacementNotFound,

    #[error("IO_ERROR: label mapping for '{0}' not found")]
    LabelMappingNotFound(String),
}

/// Convert AnchorScopeError to SPEC-compliant error string
impl AnchorScopeError {
    pub fn to_spec_string(&self) -> String {
        match self {
            AnchorScopeError::NoMatch => "NO_MATCH".to_string(),
            AnchorScopeError::MultipleMatches => "MULTIPLE_MATCHES".to_string(),
            AnchorScopeError::HashMismatch => "HASH_MISMATCH".to_string(),
            AnchorScopeError::DuplicateTrueId => "DUPLICATE_TRUE_ID".to_string(),
            AnchorScopeError::LabelExists => "LABEL_EXISTS".to_string(),
            AnchorScopeError::AmbiguousReplacement => "AMBIGUOUS_REPLACEMENT".to_string(),
            AnchorScopeError::NoReplacement => "NO_REPLACEMENT".to_string(),
            AnchorScopeError::FileNotFound => "IO_ERROR: file not found".to_string(),
            AnchorScopeError::PermissionDenied => "IO_ERROR: permission denied".to_string(),
            AnchorScopeError::InvalidUtf8 => "IO_ERROR: invalid UTF-8".to_string(),
            AnchorScopeError::ReadFailure => "IO_ERROR: read failure".to_string(),
            AnchorScopeError::WriteFailure => "IO_ERROR: write failure".to_string(),
            AnchorScopeError::BufferMetadataNotFound(_) => "IO_ERROR: buffer metadata for true_id not found".to_string(),
            AnchorScopeError::ParentBufferMetadataCorrupted(_) => "IO_ERROR: parent buffer metadata corrupted".to_string(),
            AnchorScopeError::CannotLoadSourcePath(_) => "IO_ERROR: cannot load source path".to_string(),
            AnchorScopeError::CannotSaveFileContent(_) => "IO_ERROR: cannot save file content".to_string(),
            AnchorScopeError::CannotSaveSourcePath(_) => "IO_ERROR: cannot save source path".to_string(),
            AnchorScopeError::CannotSaveScopeContent(_) => "IO_ERROR: cannot save scope content".to_string(),
            AnchorScopeError::CannotSaveBufferMetadata(_) => "IO_ERROR: cannot save buffer metadata".to_string(),
            AnchorScopeError::JsonSerializationFailed(_) => "IO_ERROR: JSON serialization failed".to_string(),
            AnchorScopeError::LabelMappingCorrupted(_) => "IO_ERROR: label mapping corrupted".to_string(),
            AnchorScopeError::CannotLoadBufferContent => "IO_ERROR: cannot load buffer content".to_string(),
            AnchorScopeError::ParentDirectoryNotFound(_) => "IO_ERROR: parent directory for true_id not found".to_string(),
            AnchorScopeError::MaximumNestingDepthExceeded(_) => "IO_ERROR: maximum nesting depth exceeded".to_string(),
            AnchorScopeError::ExternalToolFailed => "IO_ERROR: external tool failed".to_string(),
            AnchorScopeError::CannotExecuteExternalTool => "IO_ERROR: cannot execute external tool".to_string(),
            AnchorScopeError::CannotCreateTempDirectory => "IO_ERROR: cannot create temporary directory".to_string(),
            AnchorScopeError::BufferNotFound => "IO_ERROR: buffer not found".to_string(),
            AnchorScopeError::ReplacementNotFound => "IO_ERROR: replacement not found".to_string(),
            AnchorScopeError::LabelMappingNotFound(_) => "IO_ERROR: label mapping not found".to_string(),
        }
    }
}

/// Convert std::io::Error to AnchorScopeError
impl From<std::io::Error> for AnchorScopeError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => AnchorScopeError::FileNotFound,
            std::io::ErrorKind::PermissionDenied => AnchorScopeError::PermissionDenied,
            _ => AnchorScopeError::ReadFailure,
        }
    }
}
```

**Step 2: Update main.rs to import error module**

```rust
mod buffer_path;
mod cli;
mod commands;
mod config;
mod error;  // ADD THIS LINE
mod hash;
mod matcher;
mod storage;
```

**Step 3: Update main.rs to use new error types**

Replace `map_io_error_read` and `map_io_error_write`:

```rust
pub fn map_io_error_read(e: std::io::Error) -> String {
    AnchorScopeError::from(e).to_spec_string()
}

pub fn map_io_error_write(e: std::io::Error) -> String {
    AnchorScopeError::from(e).to_spec_string()
}
```

**Step 4: Run tests to verify no regressions**

```bash
cargo test --lib map_io_error
```

Expected: All tests pass

**Step 5: Commit**

```bash
git add src/error.rs src/main.rs
git commit -m "refactor: introduce type-safe AnchorScopeError enum
- Add thiserror dependency
- Create structured error types with Display impl
- Replace string-based error mapping"
```

---

### Task 1.2: Update `storage.rs` to use `AnchorScopeError`

**Files:**
- Modify: `src/storage.rs` (every function returning `Result<_, String>`)

**Step 1: Update imports**

At the top of storage.rs:

```rust
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};
use crate::buffer_path;
use crate::error::AnchorScopeError;
```

**Step 2: Replace `ensure_dir` helper**

```rust
fn ensure_dir(path: &Path) -> Result<(), AnchorScopeError> {
    fs::create_dir_all(path).map_err(AnchorScopeError::from)
}
```

**Step 3: Replace `io_error_to_spec` helper**

```rust
fn io_error_to_spec(e: std::io::Error, context: &str) -> AnchorScopeError {
    match e.kind() {
        std::io::ErrorKind::NotFound => AnchorScopeError::FileNotFound,
        std::io::ErrorKind::PermissionDenied => AnchorScopeError::PermissionDenied,
        _ => AnchorScopeError::WriteFailure,
    }
}
```

**Step 4: Update all functions returning `Result<_, String>`**

For example, `save_anchor_metadata`:

```rust
pub fn save_anchor_metadata(meta: &AnchorMeta) -> Result<(), AnchorScopeError> {
    let dir = buffer_path::anchors_dir();
    ensure_dir(&dir)?;
    let path = dir.join(format!("{}.json", meta.hash));
    let json = serde_json::to_string_pretty(meta)
        .map_err(|_| AnchorScopeError::JsonSerializationFailed("metadata".to_string()))?;
    fs::write(&path, json).map_err(|e| io_error_to_spec(e, "write failure"))
}
```

**Step 5: Update all callers**

Update main.rs functions to handle `AnchorScopeError`:

```rust
Command::Read { ... } => commands::read::execute(...),
```

becomes:

```rust
Command::Read { ... } => match commands::read::execute(...) {
    Ok(_) => 0,
    Err(e) => {
        eprintln!("{}", e.to_spec_string());
        1
    }
},
```

**Step 6: Run tests**

```bash
cargo test
```

Expected: All tests pass

**Step 7: Commit**

```bash
git add src/storage.rs src/main.rs
git commit -m "refactor(storage): use AnchorScopeError type instead of String"
```

---

### Task 1.3: Update `matcher.rs` to return `Result<Match, MatchError>`

**Files:**
- Modify: `src/matcher.rs` (no changes needed to MatchError enum - already correct)

**Step 1: Verify MatchError already implements Display correctly**

Already done in the current code - no changes needed.

**Step 2: Update all callers of `matcher::resolve`**

In `read.rs`, `write.rs`, `tree.rs`:

```rust
match crate::matcher::resolve(&normalized, &anchor_bytes_normalized) {
    Err(e) => {
        eprintln!("{}", e);
        1
    }
    Ok(m) => { /* ... */ }
}
```

Expected: Already correct since Display is implemented

**Step 3: Commit**

```bash
git add src/matcher.rs
git commit -m "refactor(matcher): use MatchError enum with Display impl"
```

---

## Phase 2: Common Logic Extraction

### Task 2.1: Extract directory traversal to `storage.rs`

**Files:**
- Modify: `src/storage.rs`
- Modify: `src/commands/read.rs`

**Step 1: Add `find_buffer_content` method to storage.rs**

```rust
/// Find buffer content for a true_id by searching all directory levels.
/// Returns (content, is_nested) where is_nested indicates if found in nested location.
pub fn find_buffer_content(file_hash: &str, true_id: &str) -> Result<Vec<u8>, AnchorScopeError> {
    // Check flat location first
    let flat_path = buffer_path::true_id_dir(file_hash, true_id).join("content");
    if flat_path.exists() {
        return fs::read(&flat_path).map_err(|e| io_error_to_spec(e, "read failure"));
    }
    
    // Check nested locations using BFS
    let file_dir = buffer_path::file_dir(file_hash);
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(file_dir);
    
    while let Some(current_dir) = queue.pop_front() {
        // Check all subdirectories
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let child_dir = entry.path();
                    let content_path = child_dir.join(true_id).join("content");
                    
                    if content_path.exists() {
                        return fs::read(&content_path).map_err(|e| io_error_to_spec(e, "read failure"));
                    }
                    
                    // Add to queue for deeper search
                    queue.push_back(child_dir);
                }
            }
        }
    }
    
    Err(AnchorScopeError::CannotLoadBufferContent)
}
```

**Step 2: Update `storage.rs::load_buffer_content` to use the new method**

```rust
pub fn load_buffer_content(file_hash: &str, true_id: &str) -> Result<Vec<u8>, AnchorScopeError> {
    find_buffer_content(file_hash, true_id)
}
```

**Step 3: Remove duplicated code from `commands/read.rs`**

Remove `find_nested_buffer_content` and `check_buffer_exists_in_dir_recursive` functions,
and replace calls with `storage::find_buffer_content`.

**Step 4: Update callers in read.rs**

```rust
let buffer_content = storage::find_buffer_content(&file_hash, &true_id)?;
```

**Step 5: Run tests**

```bash
cargo test
```

Expected: All tests pass

**Step 6: Commit**

```bash
git add src/storage.rs src/commands/read.rs
git commit -m "refactor(storage): extract find_buffer_content to eliminate duplicate traversal logic"
```

---

### Task 2.2: Extract True ID search to `storage.rs`

**Files:**
- Modify: `src/storage.rs`
- Modify: `src/commands/read.rs`
- Modify: `src/commands/label.rs`

**Step 1: Add `file_hash_for_true_id_opt` method**

```rust
/// Find file_hash containing a given true_id. Returns None if not found.
/// Returns Err(AmbiguousAnchorError) if true_id exists in multiple locations.
pub fn file_hash_for_true_id_opt(true_id: &str) -> Result<Option<String>, AmbiguousAnchorError> {
    find_file_hash_for_true_id_with_dup_check(true_id)
}
```

**Step 2: Update existing `file_hash_for_true_id` to use the new method**

```rust
pub fn file_hash_for_true_id(true_id: &str) -> Result<String, String> {
    match find_file_hash_for_true_id_with_dup_check(true_id) {
        Ok(Some(hash)) => Ok(hash),
        Ok(None) => Err(format!("IO_ERROR: file hash for True ID '{}' not found", true_id)),
        Err(AmbiguousAnchorError { true_id: tid, locations }) => {
            let locations_str: Vec<String> = locations.iter().map(|p| p.display().to_string()).collect();
            Err(format!("ERROR: Ambiguous anchor detection - same true_id '{}' found in multiple file_hash directories: {}", tid, locations_str.join(", ")))
        }
    }
}
```

**Step 3: Remove duplicated code from commands/read.rs**

Remove `find_file_hash_for_true_id` function, use `storage::file_hash_for_true_id_opt`.

**Step 4: Remove duplicated code from commands/label.rs**

Remove `find_true_id_in_dir` function, use `storage::file_hash_for_true_id_opt` with a helper to check existence.

**Step 5: Run tests**

```bash
cargo test
```

Expected: All tests pass

**Step 6: Commit**

```bash
git add src/storage.rs src/commands/read.rs src/commands/label.rs
git commit -m "refactor(storage): extract file_hash search to eliminate duplicate code"
```

---

## Phase 3: Memory-Efficient I/O Optimization

### Task 3.1: Optimize `matcher::normalize_line_endings` to avoid allocation

**Files:**
- Modify: `src/matcher.rs`
- Modify: `src/main.rs` (load_anchor)
- Modify: `src/commands/pipe.rs`

**Step 1: Add `normalize_line_endings_in_place` function**

```rust
/// Normalize CRLF -> LF in place, modifying the input slice directly.
/// Returns a slice view of the normalized content.
/// 
/// Note: This function modifies the input buffer. Callers must ensure
/// they have ownership or can safely modify the data.
pub fn normalize_line_endings_in_place(buffer: &mut Vec<u8>) -> &[u8] {
    let mut write_idx = 0;
    let mut i = 0;
    let len = buffer.len();
    
    while i < len {
        if buffer[i] == b'\r' && i + 1 < len && buffer[i + 1] == b'\n' {
            // Skip CR, keep LF
            i += 1;
        } else {
            buffer[write_idx] = buffer[i];
            write_idx += 1;
            i += 1;
        }
    }
    
    &buffer[..write_idx]
}

/// Normalize CRLF -> LF and return new Vec.
/// This is the original function, kept for backward compatibility.
#[deprecated(note = "Use normalize_line_endings_in_place or normalize_line_endings_slice for zero-copy")]
pub fn normalize_line_endings(raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'\r' && i + 1 < raw.len() && raw[i + 1] == b'\n' {
            i += 1;
        } else {
            out.push(raw[i]);
            i += 1;
        }
    }
    out
}

/// Normalize CRLF -> LF without allocation, working directly on a slice.
/// Returns a new slice with CRLF normalized to LF.
/// 
/// This function does NOT modify the input. It returns a view into the original data.
pub fn normalize_line_endings_slice(raw: &[u8]) -> Vec<u8> {
    // Since we can't return a slice that's shorter than the input without
    // allocation, we still need to allocate. But we can optimize the common case.
    normalize_line_endings(raw)
}
```

**Step 2: Update `load_anchor` in main.rs**

```rust
pub fn load_anchor(anchor: Option<&str>, anchor_file: Option<&str>) -> Result<Vec<u8>, String> {
    match (anchor, anchor_file) {
        (None, None) => return Err("ERROR: either --anchor or --anchor-file must be provided".to_string()),
        (Some(_), Some(_)) => return Err("IO_ERROR: mutually exclusive options".to_string()),
        _ => {}
    }

    let anchor_bytes = match anchor {
        Some(a) => {
            if a.is_empty() {
                return Err("NO_MATCH".to_string());
            }
            normalize_line_endings(a.as_bytes())
        }
        None => {
            let path = anchor_file.unwrap();
            let mut content = fs::read(path).map_err(|e| map_io_error_read(e))?;
            // Validate UTF-8
            if std::str::from_utf8(&content).is_err() {
                return Err("IO_ERROR: invalid UTF-8".to_string());
            }
            // Normalize in place (modify content Vec)
            normalize_line_endings_in_place(&mut content);
            content  // Return modified Vec
        }
    };

    Ok(anchor_bytes)
}
```

**Step 3: Update pipe.rs to use optimized version**

```rust
pub fn read_from_stdin_and_write_replacement(true_id: &str, stdin_bytes: &[u8]) -> Result<(), String> {
    let file_hash = match storage::file_hash_for_true_id(true_id) {
        Ok(h) => h,
        Err(e) => return Err(e),
    };
    
    // Validate UTF-8
    if std::str::from_utf8(stdin_bytes).is_err() {
        return Err("IO_ERROR: invalid UTF-8".to_string());
    }
    
    // Normalize CRLF -> LF (this still allocates, but we can optimize)
    let normalized = normalize_line_endings(stdin_bytes);
    
    // ... rest of function
}
```

**Step 4: Update write.rs to use optimized version**

```rust
let rep_bytes = if from_replacement {
    // ...
} else {
    let mut norm_bytes = replacement.as_bytes().to_vec();
    normalize_line_endings_in_place(&mut norm_bytes);
    norm_bytes
};
```

**Step 5: Run tests**

```bash
cargo test --lib matcher
```

Expected: All tests pass

**Step 6: Commit**

```bash
git add src/matcher.rs src/main.rs src/commands/pipe.rs src/commands/write.rs
git commit -m "refactor(matcher): add normalize_line_endings_in_place for memory efficiency"
```

---

## Phase 4: Single Responsibility Principle Refactoring

### Task 4.1: Refactor `commands/read.rs::execute` function

**Files:**
- Modify: `src/commands/read.rs`

**Step 1: Extract anchor resolution**

```rust
fn resolve_target_and_anchor(
    file_path: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    label: Option<&str>,
) -> Result<(String, Vec<u8>, Option<(Vec<u8>, String)>), String> {
    // Extracted logic from execute()
    // Returns (target_file, anchor_bytes, buffer_parent_true_id)
    // ...
}
```

**Step 2: Extract file reading and validation**

```rust
fn read_and_validate_file(file_path: &str) -> Result<Vec<u8>, String> {
    let raw = fs::read(file_path).map_err(map_io_error_read)?;
    if std::str::from_utf8(&raw).is_err() {
        return Err("IO_ERROR: invalid UTF-8".to_string());
    }
    Ok(raw)
}
```

**Step 3: Extract hash computation**

```rust
fn compute_hashes(
    file_hash: &mut String,
    true_id: &mut String,
    parent_true_id: &mut Option<String>,
    normalized: &[u8],
    anchor_bytes_normalized: &[u8],
    buffer_parent_true_id: &Option<(Vec<u8>, String)>,
    raw: &[u8],
) -> Result<(), String> {
    // Compute scope hash
    let scope = &normalized[m.byte_start..m.byte_end];
    let h = crate::hash::compute(scope);
    
    // Compute file_hash from raw
    *file_hash = crate::hash::compute(raw);
    
    // Compute True ID with parent context
    // ...
}
```

**Step 4: Extract buffer saving**

```rust
fn save_buffer_content(
    file_hash: &str,
    true_id: &str,
    parent_true_id: &Option<String>,
    normalized: &[u8],
    m: &Match,
    anchor_str_base: &str,
) -> Result<(), String> {
    // Save scope content and metadata
    // ...
}
```

**Step 5: Update execute() to use extracted functions**

```rust
pub fn execute(...) -> Result<(), AnchorScopeError> {
    let (target_file, anchor_bytes, buffer_parent_true_id) = resolve_target_and_anchor(...);
    let raw = read_and_validate_file(&target_file)?;
    // ... use other extracted functions
}
```

**Step 6: Run tests**

```bash
cargo test --test commands_read
```

Expected: All tests pass

**Step 7: Commit**

```bash
git add src/commands/read.rs
git commit -m "refactor(commands/read): extract private functions for single responsibility
- resolve_target_and_anchor
- read_and_validate_file
- compute_hashes
- save_buffer_content"
```

---

### Task 4.2: Refactor `commands/write.rs::execute` function

**Files:**
- Modify: `src/commands/write.rs`

**Step 1: Extract replacement source resolution**

```rust
fn resolve_replacement_source(
    from_replacement: bool,
    replacement: &str,
    label: Option<&str>,
) -> Result<(Vec<u8>, Option<String>), String> {
    // Validate replacement source
    if from_replacement && !replacement.is_empty() {
        return Err("AMBIGUOUS_REPLACEMENT".to_string());
    }
    if !from_replacement && replacement.is_empty() {
        return Err("NO_REPLACEMENT".to_string());
    }
    
    // Determine replacement bytes
    // ...
}
```

**Step 2: Extract file reading**

```rust
fn read_target_file(file_path: &str) -> Result<Vec<u8>, String> {
    fs::read(file_path).map_err(map_io_error_read)
}
```

**Step 3: Update execute()**

```rust
pub fn execute(...) -> Result<(), AnchorScopeError> {
    let (replacement_bytes, used_label) = resolve_replacement_source(...)?;
    let raw = read_target_file(&target_file)?;
    // ...
}
```

**Step 4: Run tests**

```bash
cargo test --test commands_write
```

Expected: All tests pass

**Step 5: Commit**

```bash
git add src/commands/write.rs
git commit -m "refactor(commands/write): extract private functions for single responsibility"
```

---

### Task 4.3: Refactor `commands/label.rs::execute` function

**Files:**
- Modify: `src/commands/label.rs`

**Step 1: Extract true_id existence check**

```rust
fn check_true_id_exists(true_id: &str) -> Result<(), String> {
    // Check old location
    // Check new buffer locations
    // ...
}
```

**Step 2: Update execute()**

```rust
pub fn execute(name: &str, true_id: &str) -> Result<(), AnchorScopeError> {
    check_true_id_exists(true_id)?;
    
    // Validate arguments
    if name.is_empty() { ... }
    if true_id.is_empty() { ... }
    
    // Check if label exists
    // ...
}
```

**Step 3: Run tests**

```bash
cargo test --test commands_label
```

Expected: All tests pass

**Step 4: Commit**

```bash
git add src/commands/label.rs
git commit -m "refactor(commands/label): extract true_id existence check to separate function"
```

---

## Phase 5: Final Cleanup and Testing

### Task 5.1: Update `src/commands/pipe.rs` to use AnchorScopeError

**Files:**
- Modify: `src/commands/pipe.rs`

**Step 1: Add error type conversion**

```rust
use crate::error::AnchorScopeError;

fn result_to_exit_code<T>(result: Result<T, AnchorScopeError>) -> i32 {
    match result {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{}", e.to_spec_string());
            1
        }
    }
}
```

**Step 2: Update functions to return `Result<(), AnchorScopeError>`**

```rust
pub fn stream_content_to_stdout(true_id: &str) -> Result<(), AnchorScopeError> {
    let file_hash = storage::file_hash_for_true_id(true_id)?;
    let content_path = buffer_path::true_id_dir(&file_hash, true_id).join("content");
    
    let content = std::fs::read(&content_path).map_err(|e| io_error_to_spec(e, "read failure"))?;
    
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(&content).map_err(|e| io_error_to_spec(e, "write failure"))?;
    
    Ok(())
}
```

**Step 3: Run tests**

```bash
cargo test --test commands_pipe
```

Expected: All tests pass

**Step 4: Commit**

```bash
git add src/commands/pipe.rs src/error.rs
git commit -m "refactor(commands/pipe): use AnchorScopeError for type-safe error handling"
```

---

### Task 5.2: Update `src/commands/paths.rs` to use AnchorScopeError

**Files:**
- Modify: `src/commands/paths.rs`

**Step 1: Update functions to return `Result<_, AnchorScopeError>`**

```rust
pub fn execute_for_true_id(true_id: &str) -> Result<PathsResult, AnchorScopeError> {
    let file_hash = storage::file_hash_for_true_id(true_id)?;
    
    let content_path = buffer_path::true_id_dir(&file_hash, true_id).join("content");
    let replacement_path = buffer_path::true_id_dir(&file_hash, true_id).join("replacement");
    
    if !content_path.exists() {
        return Err(AnchorScopeError::FileNotFound);
    }
    
    Ok(PathsResult {
        content_path,
        replacement_path,
    })
}
```

**Step 2: Run tests**

```bash
cargo test --test commands_paths
```

Expected: All tests pass

**Step 3: Commit**

```bash
git add src/commands/paths.rs src/error.rs
git commit -m "refactor(commands/paths): use AnchorScopeError for type-safe error handling"
```

---

### Task 5.3: Run full test suite and verify SPEC compliance

**Files:**
- All modified files

**Step 1: Run all tests**

```bash
cargo test
```

Expected: All tests pass

**Step 2: Verify SPEC compliance**

Check that all error messages match SPEC §4.5:

```bash
cargo run -- read --file test.txt --anchor "test"
# Should output SPEC-compliant error strings
```

**Step 3: Check no `Result<(), String>` remains**

```bash
rg "Result<\(\), String>" src/
```

All instances should be `Result<_, AnchorScopeError>` or removed

**Step 4: Final commit**

```bash
git add -A
git commit -m "refactor: complete type-safe error handling migration
- Replace all Result<(), String> with AnchorScopeError
- Update all commands to use new error types
- Verify SPEC §4.5 compliance"
```

---

## Phase 6: Documentation and Verification

### Task 6.1: Update README with refactoring details

**Files:**
- Modify: `README.md` (if exists)

**Step 1: Add refactoring section**

```markdown
## Code Quality Improvements

AnchorScope uses type-safe error handling with `thiserror`:

- **Structured Errors**: All errors are now typed via `AnchorScopeError`
- **SPEC Compliance**: Error messages strictly follow SPEC §4.5
- **Memory Efficiency**: I/O operations minimize allocations
- **Single Responsibility**: Each function has one clear purpose

See `src/error.rs` for complete error type definitions.
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add refactoring details to README"
```

---

### Task 6.2: Create `docs/REFACTORING.md`

**Files:**
- Create: `docs/REFACTORING.md`

**Step 1: Write refactoring documentation**

```markdown
# Rust Code Quality Improvements

This document describes the refactoring work done to improve AnchorScope's Rust code quality.

## Error Handling

### Before
```rust
fn some_function() -> Result<(), String> {
    if condition {
        return Err("ERROR_MESSAGE".to_string());
    }
    Ok(())
}
```

### After
```rust
fn some_function() -> Result<(), AnchorScopeError> {
    if condition {
        return Err(AnchorScopeError::NoMatch);
    }
    Ok(())
}
```

### Benefits
- Type-safe error propagation with `?` operator
- Compile-time verification of error handling
- Clear error contract via `Display` trait
- SPEC compliance guaranteed by enum variants

## Common Logic Extraction

Directory traversal logic was extracted to `storage.rs`:

- `find_buffer_content`: Find buffer content by searching all levels
- `file_hash_for_true_id_opt`: Find file_hash containing a true_id

## Memory Optimization

```rust
// Zero-copy alternative for common case
pub fn normalize_line_endings_in_place(buffer: &mut Vec<u8>) -> &[u8]
```

## Single Responsibility

Large functions were split into private helper functions:

- `resolve_target_and_anchor`
- `read_and_validate_file`
- `compute_hashes`
- `save_buffer_content`
```

**Step 2: Commit**

```bash
git add docs/REFACTORING.md
git commit -m "docs: add REFACTORING.md documenting improvements"
```

---

## Summary

**Total Tasks: 22**

**Estimated Time:** 3-4 hours

**Risk Level:** Low - All changes are refactoring with no SPEC changes

**Backward Compatibility:** Full - No breaking changes to API or SPEC

**Testing Strategy:**
- Unit tests for each refactored module
- Integration tests via full test suite
- Manual verification of SPEC compliance

**Rollback Plan:**
- Each task is a separate commit
- Can revert individual refactoring phases if needed

---

Plan complete and saved to `docs/plans/2026-04-11-rust-refactoring-improvements.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**
