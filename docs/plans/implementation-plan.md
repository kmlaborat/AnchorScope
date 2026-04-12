# Implementation Plan: Fix Critical Specification Gaps

**Target Version:** v1.3.0  
**Timeline:** 10 hours estimated  
**Priority:** Block release

---

## Overview

This plan addresses 3 critical gaps between the AnchorScope implementation and SPEC v1.3.0 compliance:

1. **Missing `--from-replacement` support** in write command
2. **Missing `DUPLICATE_TRUE_ID` error handling**
3. **Missing `AMBIGUOUS_REPLACEMENT` validation**

All fixes are backward compatible and add new functionality without breaking existing behavior.

---

## Phase 1: Add `--from-replacement` Support

**Time Estimate:** 3 hours  
**Blocks:** Pipeline workflow usability

### Step 1.1: Update CLI Definition

**File:** `src/cli.rs`

Add `--from-replacement` flag to `Write` command:

```rust
#[derive(Subcommand)]
pub enum Command {
    Write {
        // ... existing fields ...
        
        /// Path to the target file.
        #[arg(long)]
        file: String,

        // ... anchor fields ...
        
        /// Expected xxh3 hash (hex) of the matched region.
        #[arg(long, conflicts_with = "label")]
        expected_hash: Option<String>,

        /// Use a human-readable label to identify the anchor.
        #[arg(long, conflicts_with_all = ["anchor", "anchor_file", "expected_hash"])]
        label: Option<String>,

        /// Replacement string (replaces the entire anchor region).
        #[arg(long)]
        replacement: String,

        /// Use buffer's replacement file as replacement content.
        /// Cannot be used with --replacement.
        #[arg(long)]
        from_replacement: bool,
    },
}
```

**Task Checklist:**
- [ ] Add `from_replacement: bool` field to `Write` command
- [ ] Add CLI help text
- [ ] Add conflict constraints (can't use with `--replacement`)

---

### Step 1.2: Update Write Command Logic

**File:** `src/commands/write.rs`

Add logic to handle `--from-replacement`:

```rust
pub fn execute(
    file_path: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    expected_hash: Option<&str>,
    label: Option<&str>,
    replacement: &str,
    from_replacement: bool,  // NEW PARAMETER
) -> i32 {
    // ... existing code ...
    
    // NEW: Validate replacement source
    if from_replacement && !replacement.is_empty() {
        eprintln!("AMBIGUOUS_REPLACEMENT");
        return 1;
    }
    
    // Resolve replacement content
    let replacement_bytes = if from_replacement {
        // Read from buffer's replacement file
        // ... NEW LOGIC (see 1.3) ...
    } else {
        // Existing: use inline replacement
        crate::matcher::normalize_line_endings(replacement.as_bytes())
    };
    
    // ... rest of execute logic (unchanged) ...
}
```

**Task Checklist:**
- [ ] Add `from_replacement` parameter to `execute` function
- [ ] Add conflict check with `AMBIGUOUS_REPLACEMENT` error
- [ ] Implement replacement source resolution logic

---

### Step 1.3: Add Storage Helper for Replacement

**File:** `src/storage.rs`

Add new function to load replacement content:

```rust
/// Load replacement content from buffer for a True ID.
/// Returns Err if replacement file doesn't exist or invalid UTF-8.
pub fn load_replacement_content(file_hash: &str, true_id: &str) -> Result<Vec<u8>, String> {
    let replacement_path = buffer_path::true_id_dir(file_hash, true_id).join("replacement");
    
    if !replacement_path.exists() {
        return Err("IO_ERROR: file not found".to_string());
    }
    
    let content = fs::read(&replacement_path)
        .map_err(|e| io_error_to_spec(e, "read failure"))?;
    
    // Validate UTF-8 per SPEC §2.2
    if std::str::from_utf8(&content).is_err() {
        return Err("IO_ERROR: invalid UTF-8".to_string());
    }
    
    Ok(content)
}
```

**Task Checklist:**
- [ ] Implement `load_replacement_content` function
- [ ] Add UTF-8 validation
- [ ] Return proper error messages

---

### Step 1.4: Update Write Execution Branch

**File:** `src/main.rs`

Pass `from_replacement` flag to write command:

```rust
Command::Write {
    file,
    anchor,
    anchor_file,
    expected_hash,
    label,
    replacement,
    from_replacement,  // NEW
} => commands::write::execute(
    &file,
    anchor.as_deref(),
    anchor_file.as_deref(),
    expected_hash.as_deref(),
    label.as_deref(),
    &replacement,
    from_replacement,  // NEW
),
```

**Task Checklist:**
- [ ] Extract `from_replacement` from CLI
- [ ] Pass to `commands::write::execute`

---

### Step 1.5: Update Write Function Signature

**File:** `src/commands/write.rs`

Update `execute` function signature:

```rust
pub fn execute(
    file_path: &str,
    anchor: Option<&str>,
    anchor_file: Option<&str>,
    expected_hash: Option<&str>,
    label: Option<&str>,
    replacement: &str,
    from_replacement: bool,  // NEW
) -> i32 {
    // ... existing implementation ...
}
```

**Task Checklist:**
- [ ] Add parameter to function signature
- [ ] Add conflict validation
- [ ] Implement replacement resolution logic

---

### Step 1.6: Add Integration Test

**File:** `tests/integration/write_from_replacement_tests.rs` (new)

```rust
mod write_from_replacement_tests {
    use serial_test::serial;
    
    #[test]
    #[serial]
    fn test_write_from_replacement_uses_buffer_content() {
        // Setup: Create buffer with replacement file
        let content = b"def foo():\n    pass";
        let file_hash = crate::hash::compute(content);
        let true_id = "test_write_from_replacement";
        
        // Save file content and source path
        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_source_path(&file_hash, "/tmp/test.py").unwrap();
        storage::save_buffer_content(&file_hash, &true_id, content).unwrap();
        storage::save_buffer_metadata(&file_hash, &true_id, &storage::BufferMeta {
            true_id: true_id.to_string(),
            parent_true_id: None,
            region_hash: crate::hash::compute(content),
            anchor: "def foo()".to_string(),
        }).unwrap();
        
        // Create replacement file (simulating pipe output)
        let replacement = b"def foo():\n    return 42\n";
        let replacement_path = buffer_path::true_id_dir(&file_hash, &true_id).join("replacement");
        std::fs::write(&replacement_path, replacement).unwrap();
        
        // Write using --from-replacement
        let exit_code = commands::write::execute(
            "/tmp/test.py",
            Some("def foo()"),
            None,
            Some(&crate::hash::compute(content)),
            None,
            "",  // replacement ignored
            true,  // from_replacement = true
        );
        
        assert_eq!(exit_code, 0);
        
        // Verify file was replaced with replacement content
        let final_content = std::fs::read_to_string("/tmp/test.py").unwrap();
        assert_eq!(final_content, "def foo():\n    return 42\n");
        
        // Cleanup
        storage::invalidate_true_id_hierarchy(&file_hash, &true_id).unwrap();
        let _ = std::fs::remove_file("/tmp/test.py");
    }
    
    #[test]
    #[serial]
    fn test_write_conflict_returns_ambiguous_replacement() {
        // Same setup as above
        
        let exit_code = commands::write::execute(
            "/tmp/test.py",
            Some("def foo()"),
            None,
            Some(&crate::hash::compute(content)),
            None,
            "CONFLICT",  // replacement provided
            true,  // from_replacement also true
        );
        
        assert_eq!(exit_code, 1);
        // Error output should be "AMBIGUOUS_REPLACEMENT"
        
        // Cleanup
    }
}
```

**Task Checklist:**
- [ ] Create test file
- [ ] Add successful `--from-replacement` test
- [ ] Add conflict detection test

---

## Phase 2: Implement `DUPLICATE_TRUE_ID` Handling

**Time Estimate:** 4 hours  
**Blocks:** SPEC compliance and determinism guarantee

### Step 2.1: Create Helper Function for Duplicate Detection

**File:** `src/storage.rs`

Enhance existing `file_hash_exists_in_dir_with_count` to return locations:

```rust
/// Find all locations of a true_id within a file_hash directory tree.
/// Returns (found, count, locations).
fn find_true_id_locations(file_dir: &Path, true_id: &str) -> (bool, usize, Vec<PathBuf>) {
    use std::collections::VecDeque;
    
    let mut count = 0;
    let mut locations: Vec<PathBuf> = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(file_dir.to_path_buf());
    
    while let Some(current_dir) = queue.pop_front() {
        let target_dir = current_dir.join(true_id);
        
        if target_dir.join("content").exists() || target_dir.join("metadata.json").exists() {
            count += 1;
            locations.push(target_dir.clone());
            
            if count > 1 {
                return (true, count, locations);  // Found duplicates
            }
        }
        
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    queue.push_back(entry.path());
                }
            }
        }
    }
    
    (count > 0, count, locations)
}
```

**Task Checklist:**
- [ ] Implement duplicate location finder
- [ ] Collect all paths for error reporting

---

### Step 2.2: Update `find_true_id_dir` to Detect Duplicates

**File:** `src/storage.rs`

Modify existing function to check for duplicates:

```rust
pub fn find_true_id_dir(file_hash: &str, true_id: &str) -> Result<Option<PathBuf>, AmbiguousAnchorError> {
    use std::collections::VecDeque;
    
    let mut found_paths: Vec<PathBuf> = Vec::new();
    let file_dir = buffer_path::file_dir(file_hash);
    
    let mut queue = VecDeque::new();
    queue.push_back(file_dir.clone());
    
    while let Some(current_dir) = queue.pop_front() {
        let target_dir = current_dir.join(true_id);
        
        if target_dir.join("content").exists() || target_dir.join("metadata.json").exists() {
            found_paths.push(target_dir.clone());
            
            if found_paths.len() > 1 {
                return Err(AmbiguousAnchorError {
                    true_id: true_id.to_string(),
                    locations: found_paths,
                });
            }
        }
        
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    queue.push_back(entry.path());
                }
            }
        }
    }
    
    Ok(if found_paths.is_empty() { None } else { Some(found_paths[0].clone()) })
}
```

**Task Checklist:**
- [ ] Verify duplicate detection logic
- [ ] Return `Err(AmbiguousAnchorError)` when duplicates found

---

### Step 2.3: Update `load_buffer_metadata` to Handle Duplicates

**File:** `src/storage.rs`

Modify existing function:

```rust
pub fn load_buffer_metadata(file_hash: &str, true_id: &str) -> Result<BufferMeta, String> {
    match find_true_id_dir(file_hash, true_id) {
        Ok(Some(dir_path)) => {
            // ... existing logic ...
        }
        Ok(None) => {
            Err(io_error_to_spec(std::io::Error::new(std::io::ErrorKind::NotFound, "metadata.json"), "file not found"))
        }
        Err(AmbiguousAnchorError { true_id: tid, locations }) => {
            let locations_str: Vec<String> = locations.iter().map(|p| p.display().to_string()).collect();
            Err(format!("DUPLICATE_TRUE_ID: same true_id '{}' found at multiple locations: {}", tid, locations_str.join(", ")))
        }
    }
}
```

**Task Checklist:**
- [ ] Add duplicate handling branch
- [ ] Format error message per SPEC

---

### Step 2.4: Update All Commands to Handle `AmbiguousAnchorError`

**Files:** `src/commands/read.rs`, `src/commands/write.rs`, `src/commands/label.rs`, `src/commands/paths.rs`

Add error handling in each command:

```rust
// In read.rs - when loading buffer content
match storage::load_buffer_metadata(&file_hash, &true_id) {
    Ok(meta) => { /* proceed */ }
    Err(ref msg) if msg.starts_with("DUPLICATE_TRUE_ID") => {
        eprintln!("DUPLICATE_TRUE_ID");
        return 1;
    }
    Err(ref msg) if msg.starts_with("IO_ERROR:") => {
        eprintln!("{}", msg);
        return 1;
    }
    Err(msg) => {
        eprintln!("IO_ERROR: {}", msg);
        return 1;
    }
}
```

**Task Checklist:**
- [ ] Update `read` command
- [ ] Update `write` command (via label)
- [ ] Update `label` command
- [ ] Update `paths` command

---

### Step 2.5: Add Integration Tests

**File:** `tests/integration/error_duplicate_true_id_tests.rs` (new)

```rust
mod error_duplicate_true_id_tests {
    use serial_test::serial;
    
    #[test]
    #[serial]
    fn test_duplicate_true_id_detected_and_rejected() {
        // Setup: Create two buffers with same true_id
        let content = b"test content";
        let file_hash = crate::hash::compute(content);
        let true_id = "test_duplicate";
        
        // Save first buffer
        storage::save_file_content(&file_hash, content).unwrap();
        storage::save_source_path(&file_hash, "/tmp/test1.txt").unwrap();
        storage::save_buffer_content(&file_hash, &true_id, content).unwrap();
        storage::save_buffer_metadata(&file_hash, &true_id, &storage::BufferMeta {
            true_id: true_id.to_string(),
            parent_true_id: None,
            region_hash: crate::hash::compute(content),
            anchor: "test".to_string(),
        }).unwrap();
        
        // Create second buffer with same true_id
        // (In real scenario, this would be from a bug or race condition)
        let nested_dir = buffer_path::file_dir(&file_hash).join("parent1").join(&true_id);
        std::fs::create_dir_all(&nested_dir).unwrap();
        std::fs::write(nested_dir.join("content"), content).unwrap();
        
        let nested_meta = storage::BufferMeta {
            true_id: true_id.to_string(),
            parent_true_id: Some("parent1".to_string()),
            region_hash: crate::hash::compute(content),
            anchor: "test".to_string(),
        };
        std::fs::write(nested_dir.join("metadata.json"), serde_json::to_string_pretty(&nested_meta).unwrap()).unwrap();
        
        // Attempt read should fail with DUPLICATE_TRUE_ID
        let exit_code = commands::read::execute(
            "/tmp/test1.txt",
            Some("test"),
            None,
            None,
        );
        
        assert_eq!(exit_code, 1);
        // Error output should contain "DUPLICATE_TRUE_ID"
        
        // Cleanup
        storage::invalidate_true_id_hierarchy(&file_hash, &true_id).unwrap();
        let _ = std::fs::remove_file("/tmp/test1.txt");
    }
    
    #[test]
    #[serial]
    fn test_duplicate_true_id_in_write_via_label() {
        // Similar setup but test write via label
        // ... test implementation ...
    }
}
```

**Task Checklist:**
- [ ] Create test file
- [ ] Add duplicate detection test
- [ ] Add write via label test

---

## Phase 3: Add `AMBIGUOUS_REPLACEMENT` Validation

**Time Estimate:** 0.5 hours  
**Priority:** High

### Step 3.1: Add Conflict Check in Write Command

**File:** `src/commands/write.rs`

Already implemented in Phase 1, Step 1.2:

```rust
// NEW: Validate replacement source
if from_replacement && !replacement.is_empty() {
    eprintln!("AMBIGUOUS_REPLACEMENT");
    return 1;
}
```

**Task Checklist:**
- [ ] Verify conflict check is in place
- [ ] Verify error message format

---

### Step 3.2: Add Test

**File:** `tests/integration/write_from_replacement_tests.rs` (already created in Phase 1)

Add conflict detection test (already included in Phase 1, Step 1.6).

**Task Checklist:**
- [ ] Verify test exists
- [ ] Verify test passes

---

## Implementation Order Summary

```
Day 1:
  1.1 CLI Update (0.5h)
  1.2 Write Command Logic (1h)
  1.3 Storage Helper (0.5h)
  1.4 Main.rs Update (0.25h)
  
Day 2:
  1.5 Write Function Signature (0.25h)
  1.6 Integration Tests (2h)
  
Day 3:
  2.1 Duplicate Detection Helper (1h)
  2.2 Update find_true_id_dir (0.5h)
  2.3 Update load_buffer_metadata (0.5h)
  2.4 Update All Commands (1h)
  2.5 Integration Tests (2h)
  
Day 4:
  3.1 Verify AMBIGUOUS_REPLACEMENT (0.25h)
  3.2 Verify Test (0.25h)
  Run cargo test (1h)
  cargo fix warnings (0.5h)
```

---

## Validation Checklist

After implementation, verify:

- [ ] All existing tests still pass (21 unit + 47 integration)
- [ ] New tests added and passing (3 new integration tests)
- [ ] `cargo build` succeeds without warnings
- [ ] SPEC compliance verified for:
  - [ ] §6.3 Write Contract (`--from-replacement` support)
  - [ ] §3.2 True ID (DUPLICATE_TRUE_ID handling)
  - [ ] §6.3 Write Contract (AMBIGUOUS_REPLACEMENT validation)
- [ ] Pipeline workflow example works end-to-end
- [ ] Determinism guarantee maintained

---

## Risk Assessment

| Risk | Impact | Mitigation |
| :--- | :----- | :--------- |
| Breaking existing write usage | Medium | All changes backward compatible |
| Duplicate detection false positive | High | Thorough testing with nested structures |
| Performance regression | Low | No new expensive operations |

---

## Success Criteria

1. All 70 tests pass (21 unit + 47 integration)
2. 3 new integration tests pass
3. SPEC §6.3 fully implemented
4. SPEC §3.2 DUPLICATE_TRUE_ID handled
5. No compiler warnings
6. Pipeline example works

---

## Notes

- All changes are additive; no breaking changes to existing functionality
- Error messages follow SPEC §4.5 format exactly
- Buffer structure remains unchanged; only behavior improvements
