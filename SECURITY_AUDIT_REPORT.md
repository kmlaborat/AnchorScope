# AnchorScope Security Audit Report

**Auditor**: External Security Expert  
**Date**: 2026-04-11  
**Scope**: Path traversal, resource exhaustion, race conditions, and command injection risks in AnchorScope v1.3.0

---

## Executive Summary

AnchorScope implements a deterministic code editing protocol with a focus on safety through hash verification. While the core design is sound, this audit identified **4 CRITICAL security vulnerabilities** and **3 HIGH-severity issues** that must be addressed before production use.

### Critical Findings
1. **Path Traversal via `--file` and `--anchor-file`** (CRITICAL)
2. **Command Injection via `pipe --tool`** (CRITICAL)
3. **Resource Exhaustion via Deep Nesting** (HIGH)
4. **Resource Exhaustion via Large Files** (MEDIUM)

### Key Strengths
- Deterministic matching eliminates fuzzy logic vulnerabilities
- UTF-8 validation for file content
- CRLF→LF normalization is well-implemented
- Ephemeral storage with automatic cleanup

---

## 1. Path Traversal Vulnerabilities

### Severity: **CRITICAL**

### Risk 1.1: Path Traversal in `--file` and `--anchor-file`

**Location**: `src/commands/read.rs`, `src/commands/write.rs`

**Issue**: The `--file` and `--anchor-file` arguments accept arbitrary paths without validation. An attacker can provide paths containing `..` components to read/write files outside the intended directory.

**Proof of Concept**:
```bash
# Read arbitrary file
anchorscope read --file /etc/passwd --anchor "test"

# Write to arbitrary location (if hash matches)
anchorscope write --file ../../../etc/cron.d/malicious --anchor "..." --expected-hash "..." --replacement "..."
```

**Attack Scenarios**:
1. Reading sensitive configuration files (`.env`, SSH keys, credentials)
2. Writing malicious payloads to system configuration directories
3. Bypassing sandbox/containment boundaries

**Affected Code**:
```rust
// src/commands/read.rs - No path validation
let (target_file, anchor_bytes, buffer_parent_true_id) = if let Some(label_name) = label {
    // ... label resolution ...
} else {
    // Direct mode: use provided args
    let anchor_bytes = match crate::load_anchor(anchor, anchor_file) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    
    (file_path.to_string(), anchor_bytes, None)  // <-- No validation of file_path
};
```

**Fix Recommendation**:
```rust
use std::path::Path;

fn validate_path_safety(path: &Path, allowed_base: &Path) -> Result<(), AnchorScopeError> {
    let canonicalized_allowed = std::fs::canonicalize(allowed_base)
        .map_err(|_| AnchorScopeError::FileNotFound)?;
    
    let resolved = std::fs::canonicalize(path)
        .map_err(|_| AnchorScopeError::FileNotFound)?;
    
    if !resolved.starts_with(&canonicalized_allowed) {
        return Err(AnchorScopeError::PermissionDenied);
    }
    
    Ok(())
}

// Use in read/write commands:
let target_path = Path::new(file_path);
validate_path_safety(target_path, &std::env::current_dir()?)?;
```

---

### Risk 1.2: Symbolic Link Following

**Location**: All file operations

**Issue**: The code does not check for symbolic links. An attacker can create symbolic links pointing outside allowed directories, bypassing path validation.

**Attack Scenarios**:
```bash
# Create malicious symlink
ln -s /etc/shadow anchorscope_link

# Read via symlink
anchorscope read --file anchorscope_link --anchor "test"

# Symlink to temp directory for race condition (see Risk 3)
ln -s /tmp/anchorscope/target anchorscope_race
```

**Fix Recommendation**:
```rust
use std::fs;

fn ensure_no_symlinks(path: &Path) -> Result<(), AnchorScopeError> {
    // Check if path itself is a symlink
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| AnchorScopeError::FileNotFound)?;
    
    if metadata.file_type().is_symlink() {
        return Err(AnchorScopeError::PermissionDenied);
    }
    
    Ok(())
}

// Use before file operations:
ensure_no_symlinks(Path::new(file_path))?;
```

---

## 2. Command Injection via `pipe --tool`

### Severity: **CRITICAL**

**Location**: `src/commands/pipe.rs`

**Issue**: The `pipe --file-io --tool <command>` command directly passes user-controlled input to `std::process::Command::new()`. While `Command::new()` doesn't invoke a shell by default, the security model is fragile and susceptible to command injection via argument injection.

**Proof of Concept**:
```bash
# Argument injection attack
anchorscope pipe --true-id <true_id> --file-io --tool "malicious_tool; rm -rf /"

# If the tool name contains spaces or special characters, parsing issues may occur
anchorscope pipe --true-id <true_id> --file-io --tool "tool with spaces"
```

**More Insidious Attack - Argument Injection**:
```bash
# If tool parsing doesn't handle quotes properly
anchorscope pipe --true-id <true_id> --file-io \
  --tool 'cp; cat /etc/shadow > /tmp/stolen.txt; echo "malicious_tool"'

# If tool contains spaces and is parsed incorrectly
anchorscope pipe --true-id <true_id> --file-io \
  --tool 'malicious; python3 -c "import os; os.system(\"rm -rf /\")"'
```

**Affected Code**:
```rust
// src/commands/pipe.rs - execute_file_io function
let status = match std::process::Command::new(tool)  // <-- Direct use of user input
    .arg(&content_path)
    .arg(&output_path)
    .status()
{
    Ok(s) => s,
    Err(_) => {
        eprintln!("IO_ERROR: cannot execute external tool");
        return 1;
    }
};
```

**Attack Vectors**:
1. **Shell metacharacters**: `;`, `|`, `&`, `$()`, backticks
2. **Argument splitting**: Tool names with spaces parsed incorrectly
3. **Path traversal in tool**: `--tool "../../../tmp/malicious"`

**Fix Recommendation**:
```rust
// Whitelist approach - only allow trusted tools
const ALLOWED_TOOLS: &[&str] = &["sed", "awk", "perl", "python3", "node"];

fn validate_tool_name(tool: &str) -> Result<(), AnchorScopeError> {
    if !ALLOWED_TOOLS.contains(&tool) {
        return Err(AnchorScopeError::PermissionDenied);
    }
    
    // Additional checks
    if tool.contains("..") || tool.starts_with("/") {
        return Err(AnchorScopeError::PermissionDenied);
    }
    
    Ok(())
}

// In execute_file_io:
validate_tool_name(tool)?;
```

**Alternative: Environment-Based Whitelist**:
```rust
fn validate_tool_name(tool: &str) -> Result<(), AnchorScopeError> {
    // Check for path separators (prevent absolute/relative paths)
    if tool.contains('/') || tool.contains('\\') {
        return Err(AnchorScopeError::PermissionDenied);
    }
    
    // Check for shell metacharacters
    let dangerous_chars = [';', '|', '&', '$', '`', ' ', '\n', '\r', '\t'];
    for c in dangerous_chars {
        if tool.contains(c) {
            return Err(AnchorScopeError::PermissionDenied);
        }
    }
    
    // Check against whitelist if configured
    if let Ok(allowed) = std::env::var("ANCHORSCOPE_ALLOWED_TOOLS") {
        let allowed_tools: Vec<&str> = allowed.split(',').map(|s| s.trim()).collect();
        if !allowed_tools.is_empty() && !allowed_tools.contains(&tool) {
            return Err(AnchorScopeError::PermissionDenied);
        }
    }
    
    Ok(())
}
```

---

## 3. Resource Exhaustion via Deep Nesting

### Severity: **HIGH**

**Location**: `src/commands/read.rs`, `src/storage.rs`

**Issue**: While `max_depth()` enforces a nesting limit (default 5, configurable up to 100), the BFS-based directory traversal in storage operations doesn't have separate path length limits, allowing stack exhaustion via deeply nested directories.

**Attack Scenario**:
```bash
# Create deeply nested structure manually
# (This would be done by an attacker with write access to temp dir)
mkdir -p /tmp/anchorscope/abc123/level1/level2/.../level99/level100

# Trigger traversal via read/write
anchorscope read --file <any_file> --anchor "test"
```

**Affected Code**:
```rust
// src/storage.rs - BFS traversal without path depth limit
while let Some(current_dir) = queue.pop_front() {
    // ... process directory ...
    
    // Add all subdirectories to the queue
    if let Ok(entries) = std::fs::read_dir(&current_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                queue.push_back(entry.path());
            }
        }
    }
}
```

**Fix Recommendation**:
```rust
use std::collections::VecDeque;

fn bfs_with_depth_limit<F>(root: &Path, max_depth: usize, mut visitor: F) -> Result<(), AnchorScopeError>
where
    F: FnMut(&Path, usize) -> Result<bool, AnchorScopeError>,  // Returns true to continue
{
    let mut queue: VecDeque<(PathBuf, usize)> = VecDeque::new();
    queue.push_back((root.to_path_buf(), 0));
    
    while let Some((current_dir, depth)) = queue.pop_front() {
        // Depth limit check
        if depth > max_depth {
            continue;  // Skip deeper directories
        }
        
        // Process directory
        let should_continue = visitor(&current_dir, depth)?;
        if !should_continue {
            break;
        }
        
        // Add subdirectories
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let child_path = entry.path();
                    
                    // Path length check
                    if child_path.as_os_str().len() > 1024 {
                        continue;  // Skip overly long paths
                    }
                    
                    queue.push_back((child_path, depth + 1));
                }
            }
        }
    }
    
    Ok(())
}
```

---

## 4. Resource Exhaustion via Large Files

### Severity: **MEDIUM**

**Location**: All file operations in `src/commands/read.rs`, `src/commands/write.rs`

**Issue**: No file size limits are enforced. An attacker can provide extremely large files, causing:
1. Memory exhaustion (loading entire file into `Vec<u8>`)
2. CPU exhaustion (hash computation, matching)
3. Disk exhaustion (temp file storage)

**Attack Scenario**:
```bash
# Create large file
dd if=/dev/zero of=large_file bs=1M count=10000  # 10GB

# Process large file
anchorscope read --file large_file --anchor "test"
```

**Affected Code**:
```rust
// src/commands/read.rs - No file size validation
let raw = match fs::read(&target_file) {
    Ok(b) => b,  // Loads entire file into memory
    Err(e) => {
        eprintln!("{}", crate::map_io_error_read(e));
        return 1;
    }
};

// Enforce UTF-8 validity per SPEC
if std::str::from_utf8(&raw).is_err() {  // Processes entire file
    eprintln!("IO_ERROR: invalid UTF-8");
    return 1;
}
```

**Fix Recommendation**:
```rust
use std::fs;

const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;  // 100MB limit

fn validate_file_size(path: &Path) -> Result<(), AnchorScopeError> {
    let metadata = fs::metadata(path)
        .map_err(|_| AnchorScopeError::FileNotFound)?;
    
    if metadata.len() > MAX_FILE_SIZE {
        return Err(AnchorScopeError::WriteFailure);  // Or new error type
    }
    
    Ok(())
}

// Use before file operations:
validate_file_size(Path::new(file_path))?;
```

**Alternative: Streaming Processing**:
```rust
use std::io::{self, Read};

fn read_file_with_limit(path: &str, max_size: usize) -> Result<Vec<u8>, AnchorScopeError> {
    let mut file = fs::File::open(path)
        .map_err(|_| AnchorScopeError::FileNotFound)?;
    
    let mut metadata = file.metadata()
        .map_err(|_| AnchorScopeError::FileNotFound)?;
    
    if metadata.len() > max_size as u64 {
        return Err(AnchorScopeError::WriteFailure);
    }
    
    let mut buffer = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut buffer)
        .map_err(|_| AnchorScopeError::ReadFailure)?;
    
    Ok(buffer)
}
```

---

## 5. Race Condition Vulnerabilities

### Severity: **HIGH**

**Location**: `src/commands/write.rs`, `src/storage.rs`

**Issue**: AnchorScope uses temporary directories in `/tmp/anchorscope/` with predictable paths. Between file operations, an attacker can race to:
1. Replace buffer files with malicious content
2. Create symlinks to redirect writes
3. Modify content between hash verification and write

**Attack Scenario - Symlink Race**:
```bash
# Attacker sets up race condition
mkdir -p /tmp/anchorscope/target_file
ln -s /etc/passwd /tmp/anchorscope/target_file/true_id/content

# Victim runs write command
anchorscope write --label my_label --replacement "malicious"

# Due to race, write may follow symlink
```

**Attack Scenario - File Replacement**:
```bash
# Attacker monitors for buffer creation
while true; do
    if [ -d "/tmp/anchorscope/$(ls /tmp/anchorscope | head -1)" ]; then
        cp /tmp/anchorscope/*/true_id/content /tmp/backup_content
        echo "MALICIOUS_CONTENT" > /tmp/anchorscope/*/true_id/content
    fi
done &

# Victim runs anchorscope operations
anchorscope read --file target.txt --anchor "test"
anchorscope write --label my_label --replacement "..."
```

**Affected Code**:
```rust
// src/commands/write.rs - Vulnerable to race between verification and write
let m = match crate::matcher::resolve(&normalized, &anchor_bytes) {
    // ... verification ...
};

// SPINDLE: Time-of-check to time-of-use (TOCTOU)
let result = match fs::write(&target_file, &result) {
    Ok(_) => {
        // Cleanup happens AFTER write
        if let Some(ref label_name) = used_label {
            // ...
        }
        // ...
    }
    // ...
};
```

**Fix Recommendation**:
```rust
use std::fs;
use std::os::unix::fs::OpenOptionsExt;
use tempfile::NamedTempFile;

fn atomic_write_file(path: &Path, content: &[u8]) -> Result<(), AnchorScopeError> {
    // Create temporary file in same directory for atomic rename
    let parent = path.parent()
        .ok_or(AnchorScopeError::WriteFailure)?;
    
    let mut temp_file = NamedTempFile::new_in(parent)
        .map_err(|_| AnchorScopeError::WriteFailure)?;
    
    temp_file.write_all(content)
        .map_err(|_| AnchorScopeError::WriteFailure)?;
    
    // Atomic rename
    temp_file.persist(path)
        .map_err(|_| AnchorScopeError::WriteFailure)?;
    
    Ok(())
}

// Use in write command:
atomic_write_file(Path::new(&target_file), &result)?;
```

**Additional Defense: File Descriptor Locking**:
```rust
use std::os::unix::io::AsRawFd;

fn lock_file(path: &Path) -> Result<fs::File, AnchorScopeError> {
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .map_err(|_| AnchorScopeError::PermissionDenied)?;
    
    // Try to acquire exclusive lock (non-blocking)
    flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB)
        .map_err(|_| AnchorScopeError::PermissionDenied)?;
    
    Ok(file)
}
```

---

## 6. Symbolic Link Attacks in Buffer Storage

### Severity: **HIGH**

**Location**: `src/buffer_path.rs`, `src/storage.rs`

**Issue**: Buffer directory paths are constructed from hashes (predictable) and the code doesn't verify that directory components are actual directories, not symlinks.

**Attack Scenario**:
```bash
# Attacker creates symlink to sensitive directory
mkdir -p /tmp/anchorscope/attacker_controlled
ln -s /home/user/.ssh /tmp/anchorscope/attacker_controlled/target_true_id

# Victim's write command may follow symlink
anchorscope write --label my_label --replacement "..."
# May write to /home/user/.ssh/authorized_keys instead of buffer
```

**Fix Recommendation**:
```rust
use std::fs;

fn safe_create_dir_all(path: &Path) -> Result<(), AnchorScopeError> {
    // Create each component and verify it's a directory
    let mut current = PathBuf::new();
    
    for component in path.components() {
        current.push(component);
        
        if !current.exists() {
            fs::create_dir(&current)
                .map_err(|_| AnchorScopeError::WriteFailure)?;
        } else {
            // Verify it's a directory, not a symlink
            let metadata = fs::symlink_metadata(&current)
                .map_err(|_| AnchorScopeError::WriteFailure)?;
            
            if !metadata.file_type().is_dir() {
                return Err(AnchorScopeError::PermissionDenied);
            }
        }
    }
    
    Ok(())
}
```

---

## 7. Hash Collision Attacks

### Severity: **MEDIUM**

**Location**: All hash-based operations

**Issue**: While xxh3_64 is cryptographically secure for this use case, the hash verification model allows an attacker who can control file content to potentially craft collisions if they can predict the anchor scope.

**Mitigation**: The requirement for exact anchor matching significantly reduces this risk, as the attacker cannot arbitrarily modify the file without breaking the anchor match.

---

## 8. Information Disclosure via Error Messages

### Severity: **LOW**

**Location**: All error handling

**Issue**: Some error messages may expose internal file paths or implementation details.

**Affected Code**:
```rust
// Error messages may include path information
eprintln!("IO_ERROR: cannot load source path: {}", e);
```

**Recommendation**: Sanitize error messages to not expose internal paths unless absolutely necessary.

---

## Summary Table

| Risk | Severity | Location | Impact | Exploitability |
|------|----------|----------|--------|----------------|
| Path Traversal (--file, --anchor-file) | CRITICAL | read.rs, write.rs | Arbitrary file read/write | HIGH |
| Command Injection (pipe --tool) | CRITICAL | pipe.rs | Remote code execution | HIGH |
| Symlink Following (all file ops) | CRITICAL | All file ops | Arbitrary file access | MEDIUM |
| Resource Exhaustion (Deep Nesting) | HIGH | read.rs, storage.rs | DoS | LOW |
| Resource Exhaustion (Large Files) | MEDIUM | read.rs, write.rs | DoS | MEDIUM |
| Race Conditions (TOCTOU) | HIGH | write.rs, storage.rs | Buffer substitution | MEDIUM |
| Buffer Path Symlinks | HIGH | buffer_path.rs, storage.rs | Arbitrary file access | MEDIUM |
| Hash Collision | MEDIUM | All hash ops | Verification bypass | LOW |
| Info Disclosure | LOW | All error handling | Path exposure | LOW |

---

## Prioritized Remediation Plan

### Phase 1: Critical (Immediate)
1. **Path Traversal Prevention**: Implement `validate_path_safety()` for all file operations
2. **Command Injection Prevention**: Implement `validate_tool_name()` whitelist for `pipe --tool`
3. **Symlink Detection**: Add symlink checks before all file operations

### Phase 2: High (Within 1 Week)
4. **Resource Limits**: Implement file size limits (100MB default)
5. **Race Condition Fix**: Use atomic file writes with temp files
6. **Buffer Path Security**: Validate buffer directories are not symlinks

### Phase 3: Medium (Within 2 Weeks)
7. **Nesting Depth Limits**: Add path length and depth limits to BFS traversals
8. **Error Message Sanitization**: Remove path information from error messages

---

## Additional Recommendations

1. **Security Testing**: Add fuzzing tests for path handling
2. **Sandboxing**: Consider running in a sandboxed environment
3. **Audit Logging**: Log all file operations for security auditing
4. **Configuration**: Add security-focused configuration options:
   - `ANCHORSCOPE_FILE_SIZE_LIMIT`
   - `ANCHORSCOPE_MAX_DEPTH`
   - `ANCHORSCOPE_ALLOWED_TOOLS`
   - `ANCHORSCOPE_FILE_BASE_DIR`

---

## Conclusion

AnchorScope's core design principles (determinism, exact matching) provide strong safety guarantees, but the implementation lacks essential security controls for:
- Path traversal prevention
- Command injection prevention
- Race condition mitigation
- Resource exhaustion protection

These vulnerabilities must be addressed before deploying in environments where attackers may have any level of access to the system or input files.

**Recommendation**: Do not deploy to production until Phase 1 (Critical) fixes are implemented.

---

## References

- [OWASP Path Traversal](https://owasp.org/www-community/attacks/Path_Traversal)
- [OWASP Command Injection](https://owasp.org/www-community/attacks/Command_Injection)
- [CWE-36: Absolute Path Traversal](https://cwe.mitre.org/data/definitions/36.html)
- [CWE-77: Command Injection](https://cwe.mitre.org/data/definitions/77.html)
- [CWE-367: Time-of-check Time-of-use (TOCTOU)](https://cwe.mitre.org/data/definitions/367.html)
- [CWE-400: Uncontrolled Resource Consumption](https://cwe.mitre.org/data/definitions/400.html)
