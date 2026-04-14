# pi-anchorscope 教科書 Implementation Plan

> **REQUIRED SUB-SKILL:** Use the executing-plans skill to implement this plan task-by-task.

**Goal:** Create a comprehensive tutorial for using pi-anchorscope skills with the AnchorScope protocol, including all command examples with execution results

**Architecture:** The tutorial will cover:
1. Quick start with basic read/write operations
2. Label management for human-readable aliases
3. Pipe mode for external tool integration
4. Paths mode for debugging
5. Tree visualization for buffer structure
6. All error conditions with examples
7. v1.3.0 Showcase examples (v1_3_0_showcase.sh)
8. v1.2.0 Showcase examples (v1_2_0_showcase.sh)
9. v1.1.0 Showcase examples (v1_1_0_showcase.sh)

**Tech Stack:** 
- pi coding agent (v0.22.0+)
- pi-anchorscope skill package
- AnchorScope CLI (v1.3.0)
- Rust implementation with xxh3_64 hashing

---

## Task 1: Setup and Test File Creation

**Files:**
- Create: `docs/tutorials/pi-anchorscope-tutorial.md`
- Create: `docs/tutorials/sample.txt`

**Step 1: Create test directory and sample file**

```bash
mkdir -p docs/tutorials
```

```bash
cat > docs/tutorials/sample.txt << 'EOF'
# Sample File for pi-anchorscope Tutorial

## Section 1
// This is a comment
fn main() {
    println!("Hello, World!");
}

## Section 2
// Another comment
fn helper() {
    println!("Helper function");
}
EOF
```

**Step 2: Verify file creation**

```bash
cat docs/tutorials/sample.txt
```

Expected output:
```
# Sample File for pi-anchorscope Tutorial

## Section 1
// This is a comment
fn main() {
    println!("Hello, World!");
}

## Section 2
// Another comment
fn helper() {
    println!("Helper function");
}
```

---

## Task 2: Basic Read Operation

**Files:**
- Tutorial file: `docs/tutorials/pi-anchorscope-tutorial.md`

**Step 1: Read with single-line anchor**

```bash
anchorscope read --file docs/tutorials/sample.txt --anchor "// This is a comment"
```

Expected output:
```
start_line=4
end_line=4
scope_hash=<16-char hex>
true_id=<16-char hex>
content=// This is a comment
```

**Step 2: Read with multi-line anchor**

```bash
anchorscope read --file docs/tutorials/sample.txt --anchor $'fn main() {\n    println!("Hello, World!");\n}'
```

Expected output:
```
start_line=5
end_line=7
scope_hash=<16-char hex>
true_id=<16-char hex>
content=fn main() {
    println!("Hello, World!");
}
```

**Step 3: Document the read command in tutorial**

Add to `docs/tutorials/pi-anchorscope-tutorial.md`:
```markdown
## 2. Basic Read Operation

The `anchorscope read` command locates and hashes an anchored scope:

```bash
anchorscope read --file <path> --anchor "<string>"
```

Output includes:
- `start_line`: 1-based line number of anchor start
- `end_line`: 1-based line number of anchor end
- `scope_hash`: 16-char hex for use with `--expected-hash`
- `true_id`: 16-char hex for buffer operations
- `content`: The matched anchor text
```

---

## Task 3: Basic Write Operation

**Files:**
- Test file: `docs/tutorials/sample.txt`
- Tutorial file: `docs/tutorials/pi-anchorscope-tutorial.md`

**Step 1: Write with inline replacement**

```bash
anchorscope write \
  --file docs/tutorials/sample.txt \
  --anchor "// This is a comment" \
  --expected-hash <scope_hash_from_read> \
  --replacement "// Modified: Comment updated via AnchorScope"
```

Expected output:
```
OK: written <bytes> bytes
```

**Step 2: Verify the write**

```bash
cat docs/tutorials/sample.txt
```

Expected: The comment line should be updated.

**Step 3: Document the write command in tutorial**

Add to tutorial:
```markdown
## 3. Basic Write Operation

The `anchorscope write` command replaces an anchored scope with verification:

```bash
anchorscope write \
  --file <path> \
  --anchor "<string>" \
  --expected-hash <hex> \
  --replacement "<string>"
```

The `--expected-hash` ensures the anchor hasn't changed since you read it.
```

---

## Task 4: Label Management

**Files:**
- Tutorial file: `docs/tutorials/pi-anchorscope-tutorial.md`

**Step 1: Create a label**

```bash
anchorscope label --name "main_function" --true-id <true_id_from_read>
```

Expected output:
```
OK: label "main_function" -> <true_id>
```

**Step 2: Read using label**

```bash
anchorscope read --file docs/tutorials/sample.txt --label "main_function"
```

**Step 3: Document label commands in tutorial**

Add to tutorial:
```markdown
## 4. Label Management

Labels provide human-readable aliases for True IDs:

```bash
# Create a label
anchorscope label --name <name> --true-id <hash>

# Use label in read
anchorscope read --file <path> --label <name>

# Use label in write (with buffer replacement)
anchorscope write --label <name> --from-replacement
```

Labels are stored in `{TMPDIR}/anchorscope/labels/`.
```

---

## Task 5: Pipe Mode (External Tool Integration)

**Files:**
- Tutorial file: `docs/tutorials/pi-anchorscope-tutorial.md`

**Step 1: Pipe content to external tool**

```bash
anchorscope pipe --label "main_function" --out | cat -e
```

Expected: Shows the anchored content with line endings visible.

**Step 2: Pipe with transformation**

```bash
anchorscope pipe --label "main_function" --out | sed 's/Hello/Hello AnchorScope/' | anchorscope pipe --label "main_function" --in
```

Expected output: (no output on success)

**Step 3: Verify the replacement**

```bash
cat docs/tutorials/sample.txt
```

**Step 4: Document pipe commands in tutorial**

Add to tutorial:
```markdown
## 5. Pipe Mode - External Tool Integration

Pipe mode bridges AnchorScope with external tools:

```bash
# stdout mode (default)
anchorscope pipe --label <name> --out | external-tool | anchorscope pipe --label <name> --in

# file-io mode
anchorscope pipe --label <name> --tool <tool> --file-io --tool-args "<args>"
```

- `--out`: Streams `buffer/{true_id}/content` to stdout
- `--in`: Reads from stdin, writes to `buffer/{true_id}/replacement`

The `replacement` file is used by `anchorscope write --from-replacement`.
```

---

## Task 6: Paths Mode

**Files:**
- Tutorial file: `docs/tutorials/pi-anchorscope-tutorial.md`

**Step 1: Get buffer paths**

```bash
anchorscope paths --label "main_function"
```

Expected output:
```
content:     C:\...\Temp\anchorscope\...\content
replacement: C:\...\Temp\anchorscope\...\replacement
```

**Step 2: Inspect buffer files**

```bash
# Show content
cat "$(anchorscope paths --label "main_function" | grep content | awk '{print $2}')"

# Show replacement
cat "$(anchorscope paths --label "main_function" | grep replacement | awk '{print $2}')"
```

**Step 3: Document paths commands in tutorial**

Add to tutorial:
```markdown
## 6. Paths Mode

Get absolute paths to buffer files for debugging:

```bash
anchorscope paths --label <name>
anchorscope paths --true-id <hash>
```

Output:
- `content`: Path to the anchored scope content
- `replacement`: Path to the proposed replacement (created by pipe)

This is useful for inspecting what AnchorScope has buffered.
```

---

## Task 7: Tree Visualization

**Files:**
- Tutorial file: `docs/tutorials/pi-anchorscope-tutorial.md`

**Step 1: Display buffer tree**

```bash
anchorscope tree
```

Expected output:
```
<file_hash>  (docs/tutorials/sample.txt)
└── <true_id>  [main_function]
    └── replacement ✓
```

**Step 2: Filter by file**

```bash
anchorscope tree --file docs/tutorials/sample.txt
```

**Step 3: Document tree commands in tutorial**

Add to tutorial:
```markdown
## 7. Tree Visualization

Display the current Anchor Buffer structure:

```bash
# Show all buffers
anchorscope tree

# Filter by file
anchorscope tree --file <path>
```

Output shows:
- True IDs
- Aliases (if any)
- Presence of `replacement` files (✓)

This helps you understand the buffer state and debug issues.
```

---

## Task 8: Error Conditions

**Files:**
- Tutorial file: `docs/tutorials/pi-anchorscope-tutorial.md`

**Step 1: NO_MATCH error**

```bash
anchorscope read --file docs/tutorials/sample.txt --anchor "nonexistent anchor"
```

Expected output:
```
NO_MATCH
```

**Step 2: HASH_MISMATCH error**

```bash
anchorscope write \
  --file docs/tutorials/sample.txt \
  --anchor "// Modified: Comment updated via AnchorScope" \
  --expected-hash 0000000000000000 \
  --replacement "New content"
```

Expected output:
```
HASH_MISMATCH: expected=0000000000000000 actual=<actual_hash>
```

**Step 3: NO_REPLACEMENT error**

```bash
anchorscope write \
  --file docs/tutorials/sample.txt \
  --anchor "// Modified: Comment updated via AnchorScope" \
  --expected-hash <actual_hash>
```

Expected output:
```
NO_REPLACEMENT
```

**Step 4: Document error conditions in tutorial**

Add to tutorial:
```markdown
## 8. Error Conditions

AnchorScope returns specific error conditions:

| Error | Description | Example |
|-------|-------------|---------|
| `NO_MATCH` | Zero occurrences of anchor found | Read with non-existent anchor |
| `MULTIPLE_MATCHES (N)` | Anchor appears N>1 times | Ambiguous anchor in file |
| `HASH_MISMATCH` | Matched scope differs from expected | Wrong `--expected-hash` |
| `DUPLICATE_TRUE_ID` | Same True ID at multiple buffer locations | Buffer corruption |
| `LABEL_EXISTS` | Alias already points to different True ID | Duplicate label |
| `AMBIGUOUS_REPLACEMENT` | Both `--replacement` and `--from-replacement` provided | Using both flags |
| `NO_REPLACEMENT` | Neither `--replacement` nor `--from-replacement` given | Missing replacement |
| `IO_ERROR: ...` | File I/O or UTF-8 validation failure | Permission denied, invalid UTF-8 |

All errors print to stderr and exit with code 1.
```

---

## Task 9: Multi-Line Anchor with File

**Files:**
- Test file: `docs/tutorials/sample.txt`
- Tutorial file: `docs/tutorials/pi-anchorscope-tutorial.md`

**Step 1: Create multi-line anchor file**

```bash
cat > docs/tutorials/anchor.txt << 'EOF'
fn helper() {
    println!("Helper function");
}
EOF
```

**Step 2: Read using anchor file**

```bash
anchorscope read --file docs/tutorials/sample.txt --anchor-file docs/tutorials/anchor.txt
```

**Step 3: Write using anchor file**

```bash
anchorscope write \
  --file docs/tutorials/sample.txt \
  --anchor-file docs/tutorials/anchor.txt \
  --expected-hash <scope_hash> \
  --replacement "fn helper() {\n    println!("Helper function updated!");\n}"
```

**Step 4: Document file-based anchors in tutorial**

Add to tutorial:
```markdown
## 9. File-Based Anchors (Recommended for Multi-Line)

For multi-line anchors, use `--anchor-file`:

```bash
# Create anchor file (no escaping needed)
echo 'fn main() {
    println!("Hello");
}' > anchor.txt

# Use anchor file
anchorscope read --file <path> --anchor-file anchor.txt
anchorscope write --file <path> --anchor-file anchor.txt --expected-hash <hash> --replacement "<new_content>"
```

File-based anchors:
- Preserve exact byte content including newlines
- No shell escaping required
- Ideal for agent-generated workflows
```

---

## Task 10: Examples Folder - v1.3.0 Showcase

**Files:**
- Tutorial file: `docs/tutorials/pi-anchorscope-tutorial.md`
- Demo file: `examples/demo_target.rs`

**Step 1: Run v1_3_0_showcase.sh and document results**

```bash
bash examples/v1_3_0_showcase.sh
```

Expected demonstration topics:
1. Multi-level anchoring for precise code editing
2. External tool integration via pipe command
3. Buffer path access via paths command
4. Safety mechanisms (HASH_MISMATCH, AMBIGUOUS_REPLACEMENT, etc.)

**Step 2: Document showcase results in tutorial**

Add to tutorial:
```markdown
## 10. Examples Folder - v1.3.0 Showcase

The `examples/v1_3_0_showcase.sh` script demonstrates:

### 10.1 Multi-Level Anchoring

Level 1 anchors the outer function, Level 2 anchors a pattern inside it:

```bash
# Level 1: Anchor the calculate_area function
anchorscope read --file demo_target.rs --anchor "fn calculate_area(...) -> f64 {...}"

# Level 2: Nested anchor inside the function buffer
anchorscope read --true-id <func_true_id> --anchor "// Formula: ..."
```

### 10.2 External Tool Integration via Pipe

```bash
# Stream content to stdout
anchorscope pipe --true-id <true_id> --out

# Pipe through external tool
anchorscope pipe --true-id <true_id> --out | transform-tool | anchorscope pipe --true-id <true_id> --in
```

### 10.3 Buffer Path Access

```bash
# Get buffer paths for debugging
anchorscope paths --label <name>
```

### 10.4 Safety Mechanisms

- **HASH_MISMATCH**: Prevents writes if file changed since read
- **AMBIGUOUS_REPLACEMENT**: Requires explicit replacement source
- **NO_REPLACEMENT**: Fails if no replacement specified
- **MULTIPLE_MATCHES**: Fails if anchor appears multiple times
```

---

## Task 11: Examples Folder - v1.2.0 Showcase

**Files:**
- Tutorial file: `docs/tutorials/pi-anchorscope-tutorial.md`
- Demo file: `examples/demo_target.py`

**Step 1: Run v1_2_0_showcase.sh and document results**

```bash
bash examples/v1_2_0_showcase.sh
```

Expected demonstration topics:
1. Multi-level anchoring for ambiguous patterns
2. Label management with nested anchors
3. HASH_MISMATCH safety demonstration

**Step 2: Document showcase results in tutorial**

Add to tutorial:
```markdown
## 11. Examples Folder - v1.2.0 Showcase

The `examples/v1_2_0_showcase.sh` script demonstrates:

### 11.1 Multi-Level Anchoring for Ambiguous Patterns

When the same pattern appears multiple times in a file, nested anchoring makes it uniquely targetable:

```bash
# File has TWO 'for i in range(10):' loops
# Level 1: Anchor the specific function
anchorscope read --file demo_target.py --anchor "def process_data():"

# Level 2: Anchor the loop inside the function buffer
anchorscope read --file demo_target.py --label func_data --anchor "for i in range(10):"
```

### 11.2 Buffer Structure

```
{TMPDIR}/anchorscope/{file_hash}/{true_id}/content
```

### 11.3 HASH_MISMATCH Safety

If the file changes between read and write, the write fails safely.

---

## Task 12: Examples Folder - v1.1.0 Showcase

**Files:**
- Tutorial file: `docs/tutorials/pi-anchorscope-tutorial.md`
- Demo file: `examples/demo_target.txt`

**Step 1: Analyze v1_1_0_showcase.sh and document results**

Expected demonstration topics:
1. Auto-labeling (internal label generation)
2. Human-readable label assignment
3. Label-based writes
4. Safety failures (NO_MATCH, HASH_MISMATCH)

**Step 2: Document showcase results in tutorial**

Add to tutorial:
```markdown
## 12. Examples Folder - v1.1.0 Showcase

The `examples/v1_1_0_showcase.sh` script demonstrates:

### 12.1 Auto-Labeling

The `read` command generates an internal label automatically:

```
label=<internal-label>
```

### 12.2 Human-Readable Label Assignment

```bash
anchorscope label --name <name> --internal-label <internal-label>
```

### 12.3 Label-Based Writes

```bash
anchorscope write --file <path> --label <name> --replacement <content>
```

### 12.4 Safety Failures

- **NO_MATCH**: Label's anchor no longer exists after modification
- **HASH_MISMATCH**: File changed between read and write

---

## Task 13: Cleanup and Summary

**Files:**
- Tutorial file: `docs/tutorials/pi-anchorscope-tutorial.md`

**Step 1: Clean up test files**

```bash
rm docs/tutorials/sample.txt docs/tutorials/anchor.txt
```

**Step 2: Add summary to tutorial**

Add to tutorial:
```markdown
## Summary

AnchorScope provides deterministic, verifiable code editing through:

1. **Anchored Scopes** - Exact byte-level matching with minimal context
2. **True IDs** - Content-derived identifiers for stable references
3. **Hash Verification** - Integrity checks before every write
4. **Buffer Management** - External state persistence via pipe/paths

**Governing principle: No Match, No Hash, No Write.**

### When to Use

- LLM-driven code editing where determinism is critical
- Multi-step edits requiring state persistence
- External tool integration with content transformation
- Debugging buffer state with tree/paths commands
- Multi-line anchor matching with exact byte preservation

### Key Commands

| Command | Purpose |
|---------|---------|
| `read` | Locate and hash an anchored scope |
| `write` | Replace scope with hash verification |
| `label` | Assign human-readable alias to True ID |
| `tree` | Visualize buffer structure |
| `pipe` | Bridge with external tools |
| `paths` | Get buffer file paths for debugging |

### Common Workflow

```bash
# 1. Read to get True ID and hash
anchorscope read --file file.rs --anchor "fn main()"

# 2. (Optional) Create label for easier reference
anchorscope label --name "main" --true-id <true_id>

# 3. Prepare replacement via pipe
anchorscope pipe --label "main" --out | transform-tool | anchorscope pipe --label "main" --in

# 4. Write with hash verification
anchorscope write --label "main" --from-replacement
```
```

---

## Task 14: Update README

**Files:**
- Tutorial file: `docs/tutorials/pi-anchorscope-tutorial.md`
- Main README: `README.md`

**Step 1: Add tutorial link to README**

Add to README:
```markdown
## Documentation

- **[pi-anchorscope Tutorial](docs/tutorials/pi-anchorscope-tutorial.md)** - Complete guide to using pi-anchorscope skills
- **[AnchorScope v1.3.0 Showcase](examples/v1_3_0_showcase.sh)** - Live demo of all features
```

---

## Execution Notes

1. Run each step and capture exact output
2. Replace `<scope_hash>`, `<true_id>`, etc. with actual values
3. Include both successful and error outputs
4. Test on Windows (since this is the current environment)
5. Ensure all paths use forward slashes or proper escaping
6. Run examples folder scripts and capture actual output
7. Verify all examples work with the current pi-anchorscope version
