#!/bin/bash

# AnchorScope v1.3.0 Demo: Multi-Level Anchoring with External Tools
# Demonstrates precise code editing using nested anchors, pipe, and paths commands

set -euo pipefail

echo "=== AnchorScope v1.3.0 Demo ==="
echo "Demonstrating: Multi-level anchoring, pipe/paths commands, and error handling"
echo ""
echo "This demo shows:"
echo "1. Multi-level anchoring for precise code editing"
echo "2. External tool integration via pipe command"
echo "3. Buffer path access via paths command"
echo "4. Safety mechanisms (HASH_MISMATCH, AMBIGUOUS_REPLACEMENT, etc.)"
echo ""

DEMO_FILE="examples/demo_target.rs"
BIN="./target/release/anchorscope"
echo "Target file: $DEMO_FILE"
echo ""

# Show the demo file
echo "=== Step 0: The Demo File ==="
echo "--- File content ---"
cat "$DEMO_FILE"
echo ""

# Step 1: Level 1 - Anchor the outer function
echo "=== Step 1: Level 1 - Anchor the calculate_area function ==="
echo "Command: read --file $DEMO_FILE --anchor \"fn calculate_area(width: f64, height: f64) -> f64 {\""
echo ""
$BIN read --file "$DEMO_FILE" --anchor "fn calculate_area(width: f64, height: f64) -> f64 {"
echo ""

TRUE_ID_FUNC=$($BIN read --file "$DEMO_FILE" --anchor "fn calculate_area(width: f64, height: f64) -> f64 {" | grep "^true_id=" | head -1 | cut -d= -f2)
echo "Function True ID: $TRUE_ID_FUNC"
echo ""

# Step 2: Create a human-readable label
echo "=== Step 2: Create a human-readable label ==="
echo "Command: label --name func_area --true-id $TRUE_ID_FUNC"
echo ""
$BIN label --name "func_area" --true-id "$TRUE_ID_FUNC"
echo ""

# Step 3: View buffer structure
echo "=== Step 3: View buffer structure ==="
echo "Command: tree --file $DEMO_FILE"
echo ""
$BIN tree --file "$DEMO_FILE"
echo ""

# Step 4: Level 2 - Nested anchor using a unique pattern
echo "=== Step 4: Level 2 - Nested anchor ==="
echo "We anchor a pattern inside the calculate_area function."
echo "Using a unique string 'Formula: width' which only appears once."
echo ""

# Use a pattern that's truly unique in the file
$BIN read --file "$DEMO_FILE" --anchor "// Formula: width * height"
echo ""

TRUE_ID_NESTED=$($BIN read --file "$DEMO_FILE" --anchor "// Formula: width * height" | grep "^true_id=" | head -1 | cut -d= -f2)
echo "Nested True ID: $TRUE_ID_NESTED"
echo ""

# Step 5: Create label for the nested anchor
echo "=== Step 5: Create label for nested anchor ==="
echo "Command: label --name area_formula --true-id $TRUE_ID_NESTED"
echo ""
$BIN label --name "area_formula" --true-id "$TRUE_ID_NESTED"
echo ""

# Step 6: Pipe command - stdout mode
echo "=== Step 6: Pipe command - stdout mode ==="
echo "Streaming buffer content to stdout for external tools."
echo "Command: pipe --label area_formula --out"
echo ""
$BIN pipe --label "area_formula" --out
echo ""
echo "→ Buffer content streamed successfully"
echo ""

# Step 7: Pipe command - write replacement via stdin
echo "=== Step 7: Pipe command - write replacement via stdin ==="
echo "Simulating external tool processing via: pipe --out | sed | pipe --in"
echo ""

# Get the content, modify it, and pipe it back
PIPE_OUT=$($BIN pipe --label "area_formula" --out)
echo "$PIPE_OUT" | sed 's/width \* height/(width * height) + 1/' | $BIN pipe --label "area_formula" --in-flag
echo ""
echo "→ Replacement content written via pipe --in"
echo ""

# Step 8: View buffer structure with replacement
echo "=== Step 8: View buffer structure with replacement ==="
echo "Command: tree --file $DEMO_FILE"
echo ""
$BIN tree --file "$DEMO_FILE"
echo ""

# Step 9: Paths command
echo "=== Step 9: Paths command ==="
echo "Get buffer paths for external tools."
echo "Command: paths --label area_formula"
echo ""
$BIN paths --label "area_formula"
echo ""

CONTENT_PATH=$($BIN paths --label "area_formula" | grep "^content:" | cut -d: -f2- | xargs)
REPLACEMENT_PATH=$($BIN paths --label "area_formula" | grep "^replacement:" | cut -d: -f2- | xargs)

echo "Content path: $CONTENT_PATH"
echo "Replacement path: $REPLACEMENT_PATH"
echo ""

# Step 10: Write from replacement
echo "=== Step 10: Write from replacement to file ==="
echo "Command: write --file $DEMO_FILE --label area_formula --replacement 'modified content'"
echo ""
$BIN write --file "$DEMO_FILE" --label "area_formula" --replacement "modified content"
echo ""

# Step 11: Verify the change
echo "=== Step 11: Verify the change ==="
echo ""
echo "--- Updated file ---"
cat "$DEMO_FILE"
echo ""

# Step 12: Demonstrate HASH_MISMATCH safety
echo "=== Step 12: Demonstrate HASH_MISMATCH safety ==="
echo "If we modify the file between read and write, write will fail safely."
echo ""

echo "Creating test file..."
echo "fn demo() { x }" > "examples/demo_hash.rs"

echo "Read the function..."
HASH_OUTPUT=$($BIN read --file "examples/demo_hash.rs" --anchor "fn demo() {")
echo "$HASH_OUTPUT"
ORIGINAL_HASH=$(echo "$HASH_OUTPUT" | grep "^hash=" | cut -d= -f2)
echo "Original hash: $ORIGINAL_HASH"
echo ""

echo "Modifying the file..."
echo "fn demo() { y }" > "examples/demo_hash.rs"

echo "Trying to write with original hash..."
if $BIN write --file "examples/demo_hash.rs" --anchor "fn demo() {" --expected-hash "$ORIGINAL_HASH" --replacement "fn demo() { modified }" 2>&1; then
    echo "Write succeeded"
else
    echo "Write failed (expected HASH_MISMATCH)"
fi

echo ""
echo "Cleaning up..."
rm -f "examples/demo_hash.rs"
echo ""

# Step 13: Demonstrate MULTIPLE_MATCHES
echo "=== Step 13: Demonstrate MULTIPLE_MATCHES ==="
echo "Creating file with duplicate patterns..."
echo -e "// First occurrence\n// First occurrence" > "examples/demo_multi.rs"

echo "File content:"
cat "examples/demo_multi.rs"
echo ""

echo "Attempting to anchor with non-unique pattern..."
if $BIN read --file "examples/demo_multi.rs" --anchor "// First occurrence" 2>&1; then
    echo "Read succeeded"
else
    echo "Read failed (expected MULTIPLE_MATCHES)"
fi

echo ""
echo "Cleaning up..."
rm -f "examples/demo_multi.rs"
echo ""

# Step 14: Demonstrate AMBIGUOUS_REPLACEMENT
echo "=== Step 14: Demonstrate AMBIGUOUS_REPLACEMENT ==="
echo "Creating test file..."
echo "fn demo() { }" > "examples/demo_ambig.rs"

echo "Trying to use both --replacement and --from-replacement..."
if $BIN write --file "examples/demo_ambig.rs" --anchor "fn demo() {" --replacement "new" --from-replacement 2>&1; then
    echo "Write succeeded"
else
    echo "Write failed (expected AMBIGUOUS_REPLACEMENT)"
fi

echo ""
echo "Cleaning up..."
rm -f "examples/demo_ambig.rs"
echo ""

# Step 15: Demonstrate NO_REPLACEMENT
echo "=== Step 15: Demonstrate NO_REPLACEMENT ==="
echo "Creating test file..."
echo "fn demo() { }" > "examples/demo_norep.rs"

echo "Trying to write without specifying replacement..."
if $BIN write --file "examples/demo_norep.rs" --anchor "fn demo() {" --from-replacement 2>&1; then
    echo "Write succeeded"
else
    echo "Write failed (expected NO_REPLACEMENT)"
fi

echo ""
echo "Cleaning up..."
rm -f "examples/demo_norep.rs"
echo ""

echo "=== Demo Complete ==="
echo ""
echo "Key Takeaways:"
echo "1. True ID encodes parent context for unique identification"
echo "2. Nested anchors operate on buffer copies, not original file"
echo "3. Labels provide human-readable references to True IDs"
echo "4. pipe command integrates with external tools (stdout mode)"
echo "5. pipe command with stdin writes replacement content"
echo "6. paths command returns buffer file paths for external tools"
echo "7. HASH_MISMATCH prevents unsafe writes if file changes"
echo "8. AMBIGUOUS_REPLACEMENT prevents conflicting replacement sources"
echo "9. NO_REPLACEMENT prevents writes without replacement source"
echo "10. MULTIPLE_MATCHES prevents ambiguous operations"
echo ""
