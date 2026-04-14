# AnchorScope Tutorial

A comprehensive guide to using pi-anchorscope skills with the AnchorScope protocol.

---

## 1. Quick Start

AnchorScope provides deterministic, verifiable code editing through:

1. **Anchored Scopes** - Exact byte-level matching with minimal context
2. **True IDs** - Content-derived identifiers for stable references
3. **Hash Verification** - Integrity checks before every write
4. **Buffer Management** - External state persistence via pipe/paths

**Governing principle: No Match, No Hash, No Write.**

---

## 2. Basic Read Operation

The `anchorscope read` command locates and hashes an anchored scope:

```bash
anchorscope read --file <path> --anchor "<string>"
```

### 2.1 Single-Line Anchor

```bash
anchorscope read --file docs/tutorials/sample.txt --anchor "// This is a comment"
```

Output:
```
start_line=4
end_line=4
hash=5d7008ad1b1478cb
content=/ This is a comment
true_id=445a9ef90dcde6a5
label=5d7008ad1b1478cb
true_id=445a9ef90dcde6a5
```

The output includes:
- `start_line`: 1-based line number of anchor start
- `end_line`: 1-based line number of anchor end
- `hash`: 16-char hex for use with `--expected-hash`
- `true_id`: 16-char hex for buffer operations
- `content`: The matched anchor text
- `label`: Auto-generated label (same as hash)

### 2.2 Multi-Line Anchor

```bash
anchorscope read --file docs/tutorials/sample.txt --anchor $'fn main() {\n    println!("Hello, World!");\n}'
```

Output:
```
start_line=5
end_line=7
hash=22e89e5c1ca0c55d
content=fn main() {
    println!("Hello, World!");
}
true_id=8db42edf7905d28f
label=22e89e5c1ca0c55d
true_id=8db42edf7905d28f
```

For multi-line anchors, consider using `--anchor-file` (see Section 9).

---

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

### 3.1 Write with Inline Replacement

First, get the hash from a read operation:

```bash
# Read to get the hash
anchorscope read --file docs/tutorials/sample.txt --anchor "// This is a comment"
# hash=5d7008ad1b1478cb
```

Then write with the hash:

```bash
anchorscope write \
  --file docs/tutorials/sample.txt \
  --anchor "// This is a comment" \
  --expected-hash 5d7008ad1b1478cb \
  --replacement "// Modified: Comment updated via AnchorScope"
```

Output:
```
OK: written 70 bytes
```

### 3.2 Verify the Write

```bash
cat docs/tutorials/sample.txt
```

Output:
```
# Sample File for pi-anchorscope Tutorial

## Section 1
// Modified: Comment updated via AnchorScope
fn main() {
    println!("Hello, World!");
}

## Section 2
// Another comment
fn helper() {
    println!("Helper function");
}

```

### 3.3 Buffer-Based Write (using --true-id)

For more complex workflows, you can write to the buffer directly using `--true-id`:

```bash
# Get True ID and hash
anchorscope read --file docs/tutorials/sample.txt --anchor "fn main()"
# Copy the true_id and hash from output

# Pipe new content to buffer
anchorscope pipe --true-id <true_id> --out | transform-tool | anchorscope pipe --true-id <true_id> --in

# Write from buffer
anchorscope write --true-id <true_id> --expected-hash <hash> --from-replacement
```

This is useful when:
- You want to modify content before writing
- You need to inspect the buffer before committing
- You want to chain multiple buffer operations

---

## 4. Label Management

Labels provide human-readable aliases for True IDs:

```bash
# Create a label
anchorscope label --name <name> --true-id <hash>

# Use label in read
anchorscope read --file <path> --label <name> --anchor "<string>"

# Use label in write
anchorscope write --label <name> --replacement "<string>"  # or --from-replacement
```

**Note:** When using `--label`, you cannot combine `--expected-hash`. Use `--replacement` or `--from-replacement` instead.

Labels are stored in `{TMPDIR}/anchorscope/labels/`.

### 4.1 Creating a Label

```bash
anchorscope label --name "main_function" --true-id 8db42edf7905d28f
```

Output:
```
OK: label "main_function" -> 8db42edf7905d28f
```

### 4.2 Reading with a Label

```bash
anchorscope read --file docs/tutorials/sample.txt --label "main_function" --anchor "fn main()"
```

Output:
```
start_line=5
end_line=7
hash=22e89e5c1ca0c55d
content=fn main() {
    println!("Hello, World!");
}
true_id=8db42edf7905d28f
label=main_function
```

### 4.3 Writing with a Label

```bash
anchorscope write \
  --file docs/tutorials/sample.txt \
  --label "main_function" \
  --expected-hash 22e89e5c1ca0c55d \
  --replacement "fn main() {\n    println!(\"Hello, AnchorScope!\");\n}"
```

---

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
- `--file-io`: Passes content path to external tool
- `--tool`: External tool command to execute
- `--tool-args`: Arguments to pass to the tool (space-separated)

The `replacement` file is used by `anchorscope write --from-replacement.`

**Note:** The `--tool` and `--tool-args` options may have limited support on Windows.

### 5.1 Pipe Content to External Tool

```bash
anchorscope pipe --label "main_function" --out
```

Output:
```
fn main() {
    println!("Hello, AnchorScope!");
}
```

### 5.2 Pipe with Transformation

```bash
anchorscope pipe --label "main_function" --out | sed 's/Hello/Hello World/' | anchorscope pipe --label "main_function" --in
```

Output: (no output on success)

### 5.3 Verify the Replacement

```bash
cat docs/tutorials/sample.txt
```

Output:
```
# Sample File for pi-anchorscope Tutorial

## Section 1
// Modified: Comment updated via AnchorScope
fn main() {
    println!("Hello World, AnchorScope!");
}

## Section 2
// Another comment
fn helper() {
    println!("Helper function");
}

```

---

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

### 6.1 Get Buffer Paths

```bash
anchorscope paths --label "main_function"
```

Output:
```
content:     C:\Users\MURAMATSU\AppData\Local\Temp\anchorscope\8db42edf7905d28f\content
replacement: C:\Users\MURAMATSU\AppData\Local\Temp\anchorscope\8db42edf7905d28f\replacement
```

### 6.2 Inspect Buffer Files

```bash
# Show content
cat "C:\Users\MURAMATSU\AppData\Local\Temp\anchorscope\8db42edf7905d28f\content"

# Show replacement
cat "C:\Users\MURAMATSU\AppData\Local\Temp\anchorscope\8db42edf7905d28f\replacement"
```

---

## 7. Tree Visualization

Display the current Anchor Buffer structure:

```bash
# Show all buffers (requires --file argument)
anchorscope tree --file <path>

# Filter by file
anchorscope tree --file <path>
```

Output shows:
- True IDs
- Aliases (if any)
- Presence of `replacement` files (✓)

This helps you understand the buffer state and debug issues.

### 7.1 Display Buffer Tree

```bash
anchorscope tree --file docs/tutorials/sample.txt
```

Output:
```
492b5443ae42e164  (docs/tutorials/sample.txt)
└── 21b28ed9d6f89d9b  [main_function]
    └── efc64d6fa480c277  [efc64d6fa480c277]
```

### 7.2 Filter by File

```bash
anchorscope tree --file docs/tutorials/sample.txt
```

Output:
```
8db42edf7905d28f  (docs/tutorials/sample.txt)
└── 445a9ef90dcde6a5  [main_function]
```

---

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

### 8.1 NO_MATCH Error

```bash
anchorscope read --file docs/tutorials/sample.txt --anchor "nonexistent anchor"
```

Output:
```
NO_MATCH
```

### 8.2 HASH_MISMATCH Error

```bash
anchorscope write \
  --file docs/tutorials/sample.txt \
  --anchor "// Modified: Comment updated via AnchorScope" \
  --expected-hash 0000000000000000 \
  --replacement "New content"
```

Output:
```
HASH_MISMATCH: expected=0000000000000000 actual=5d7008ad1b1478cb
```

### 8.3 NO_REPLACEMENT Error

```bash
anchorscope write \
  --file docs/tutorials/sample.txt \
  --anchor "// Modified: Comment updated via AnchorScope" \
  --expected-hash 5d7008ad1b1478cb
```

Output:
```
NO_REPLACEMENT
```

---

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

### 9.1 Create Multi-Line Anchor File

```bash
cat > docs/tutorials/anchor.txt << 'EOF'
fn helper() {
    println!("Helper function");
}
EOF
```

### 9.2 Read Using Anchor File

```bash
anchorscope read --file docs/tutorials/sample.txt --anchor-file docs/tutorials/anchor.txt
```

Output:
```
start_line=13
end_line=15
hash=f8e7d6c5b4a39281
content=fn helper() {
    println!("Helper function");
}
true_id=7c6b5a4938271605
label=f8e7d6c5b4a39281
true_id=7c6b5a4938271605
```

### 9.3 Write Using Anchor File

```bash
anchorscope write \
  --file docs/tutorials/sample.txt \
  --anchor-file docs/tutorials/anchor.txt \
  --expected-hash f8e7d6c5b4a39281 \
  --replacement "fn helper() {\n    println!(\"Helper function updated!\");\n}"
```

Output:
```
OK: written 68 bytes
```

---

## 10. Multi-Level Anchoring

Nested anchoring allows you to target specific patterns within larger scopes. The True ID encodes parent context, making nested anchors uniquely identifiable.

### 10.1 Level 1: Anchor the Outer Scope

```bash
anchorscope read --file demo_target.rs --anchor "fn calculate_area(width: f64, height: f64) -> f64 {\n    // Calculate the area of a rectangle\n    // Formula: width * height\n    width * height\n}"
```

### 10.2 Level 2: Nested Anchor Inside the Buffer

```bash
# Get the True ID from Level 1
TRUE_ID=$(anchorscope read --file demo_target.rs --anchor "fn calculate_area(width: f64, height: f64) -> f64 {\n    // Calculate the area of a rectangle\n    // Formula: width * height\n    width * height\n}" | grep "^true_id=" | cut -d= -f2)

# Anchor a pattern inside the function buffer
anchorscope read --true-id $TRUE_ID --anchor "// Formula: width * height"
```

### 10.3 Why Multi-Level Anchoring?

When the same pattern appears multiple times in a file, nested anchoring makes it uniquely targetable:

```bash
# File has TWO 'for i in range(10):' loops
# Level 1: Anchor the specific function
anchorscope read --file demo_target.py --anchor "def process_data():"

# Level 2: Anchor the loop inside the function buffer
anchorscope read --file demo_target.py --label func_data --anchor "for i in range(10):"
```

---

## 11. Safety Mechanisms

AnchorScope provides several safety mechanisms to prevent unsafe edits:

### 11.1 HASH_MISMATCH

Prevents writes if the file has changed since the read operation.

```bash
# Read with hash
anchorscope read --file demo_target.rs --anchor "fn demo() {"
# Get the hash value

# Modify the file
echo "fn demo() { modified }" > demo_target.rs

# Try to write with old hash
anchorscope write --file demo_target.rs --anchor "fn demo() {" --expected-hash <old_hash> --replacement "new content"
# Output: HASH_MISMATCH
```

### 11.2 AMBIGUOUS_REPLACEMENT

Requires explicit replacement source (`--replacement` or `--from-replacement`).

```bash
# Using both flags causes error
anchorscope write --file demo_target.rs --anchor "fn demo() {" --replacement "new" --from-replacement
# Output: AMBIGUOUS_REPLACEMENT
```

### 11.3 NO_REPLACEMENT

Fails if no replacement is specified.

```bash
anchorscope write --file demo_target.rs --anchor "fn demo() {" --from-replacement
# Output: NO_REPLACEMENT
```

### 11.4 MULTIPLE_MATCHES

Fails if anchor appears multiple times.

```bash
echo -e "// First\n// First" > demo_multi.rs
anchorscope read --file demo_multi.rs --anchor "// First"
# Output: MULTIPLE_MATCHES (2)
```

---

## 12. Complete Showcase: Multi-Step Workflow

This section demonstrates a complete workflow combining all features, similar to the v1.3.0 showcase:

### 12.1 Setup: Create a Working Copy

```bash
# Create demo file
cat > demo_target.rs << 'RUST_CODE'
// Geometry calculator
// This module provides functions for calculating area and perimeter

fn calculate_area(width: f64, height: f64) -> f64 {
    // Calculate the area of a rectangle
    // Formula: width * height
    width * height
}

fn calculate_perimeter(width: f64, height: f64) -> f64 {
    // Calculate the perimeter of a rectangle
    // Formula: 2 * (width + height)
    2.0 * (width + height)
}

fn main() {
    let w = 5.0;
    let h = 3.0;
    println!("Area: {}", calculate_area(w, h));
    println!("Perimeter: {}", calculate_perimeter(w, h));
}
RUST_CODE
```

### 12.2 Step 1: Level 1 - Anchor the Outer Function

```bash
ANCHOR_FUNC='fn calculate_area(width: f64, height: f64) -> f64 {
    // Calculate the area of a rectangle
    // Formula: width * height
    width * height
}'

anchorscope read --file demo_target.rs --anchor "$ANCHOR_FUNC"
```

### 12.3 Step 2: Create a Human-Readable Label

```bash
TRUE_ID_FUNC=$(anchorscope read --file demo_target.rs --anchor "$ANCHOR_FUNC" | grep "^true_id=" | head -1 | cut -d= -f2)
anchorscope label --name "func_area" --true-id "$TRUE_ID_FUNC"
```

### 12.4 Step 3: Level 2 - Nested Anchor

```bash
ANCHOR_FORMULA="// Formula: width * height"
anchorscope read --true-id "$TRUE_ID_FUNC" --anchor "$ANCHOR_FORMULA"

TRUE_ID_NESTED=$(anchorscope read --true-id "$TRUE_ID_FUNC" --anchor "$ANCHOR_FORMULA" | grep "^true_id=" | head -1 | cut -d= -f2)
SCOPE_HASH_NESTED=$(anchorscope read --true-id "$TRUE_ID_FUNC" --anchor "$ANCHOR_FORMULA" | grep "^hash=" | head -1 | cut -d= -f2)
```

### 12.5 Step 4: Create Label for Nested Anchor

```bash
anchorscope label --name "area_formula" --true-id "$TRUE_ID_NESTED"
```

### 12.6 Step 5: Pipe Command - stdout Mode

```bash
anchorscope pipe --true-id "$TRUE_ID_NESTED" --out
```

### 12.7 Step 6: Pipe Command - Write Replacement via stdin

```bash
PIPE_OUT=$(anchorscope pipe --true-id "$TRUE_ID_NESTED" --out)
echo "$PIPE_OUT" | sed 's/width \* height/(width * height) + 1/' | anchorscope pipe --true-id "$TRUE_ID_NESTED" --in
```

### 12.8 Step 7: Write from Replacement to File

```bash
anchorscope write --true-id "$TRUE_ID_NESTED" --anchor "$ANCHOR_FORMULA" --expected-hash "$SCOPE_HASH_NESTED" --from-replacement
```

### 12.9 Step 8: Verify the Change

```bash
cat demo_target.rs
```

---

## 13. Common Workflow

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

---

## 14. Key Commands Reference

| Command | Purpose |
|---------|---------|
| `read` | Locate and hash an anchored scope |
| `write` | Replace scope with hash verification |
| `label` | Assign human-readable alias to True ID |
| `tree` | Visualize buffer structure |
| `pipe` | Bridge with external tools |
| `paths` | Get buffer file paths for debugging |

---

## 15. Summary

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

---
