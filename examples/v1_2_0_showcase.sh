#!/bin/bash

# AnchorScope v1.2.0 Demo: Multi-Level Anchoring
# Demonstrates precise code editing using nested anchors with True IDs

set -e

echo "=== AnchorScope v1.2.0 Demo ==="
echo "Demonstrating: Multi-level anchoring for precise code editing"
echo ""
echo "This demo shows how to edit specific code sections when the same pattern"
echo "appears multiple times in a file."
echo ""

DEMO_FILE="demo_target.py"
BIN="anchorscope"
echo "Target file: $DEMO_FILE"
echo ""

# Show the demo file
echo "=== Step 0: The Problem ==="
echo "This file has TWO 'for i in range(10):' loops"
echo "A simple global search would match BOTH, making edits ambiguous."
echo ""
echo "--- File content (relevant sections) ---"
grep -n "for i in range(10):" "$DEMO_FILE"
echo ""

echo "=== Cleanup: Remove any existing labels ==="
echo "Removing previous labels..."
rm -rf "${TMPDIR:-/tmp}/anchorscope/labels" 2>/dev/null || true
echo ""

echo "=== Step 1: Level 1 - Anchor the outer scope ==="
echo "We first anchor the specific function we want to edit."
echo "Command: read --file $DEMO_FILE --anchor \"def process_data():\""
echo ""
$BIN read --file "$DEMO_FILE" --anchor "def process_data():"
echo ""

TRUE_ID_FUNC=$($BIN read --file "$DEMO_FILE" --anchor "def process_data():" | grep "^true_id=" | head -1 | cut -d= -f2)
echo "Function True ID: $TRUE_ID_FUNC"
echo ""

echo "=== Step 2: Create a human-readable label ==="
echo "Command: label --name func_data --true-id $TRUE_ID_FUNC"
echo ""
$BIN label --name "func_data" --true-id "$TRUE_ID_FUNC"
echo ""

echo "=== Step 3: Level 2 - Nested anchor (inside the function buffer) ==="
echo "Now we anchor the loop INSIDE the 'func_data' buffer."
echo "Even though there are TWO 'for i in range(10):' in the file,"
echo "there's only ONE inside the 'process_data' function buffer."
echo ""
echo "Command: read --file $DEMO_FILE --label func_data --anchor \"for i in range(10):\""
echo ""
$BIN read --file "$DEMO_FILE" --label func_data --anchor "for i in range(10):"
echo ""

TRUE_ID_LOOP=$($BIN read --file "$DEMO_FILE" --label func_data --anchor "for i in range(10):" | grep "^true_id=" | head -1 | cut -d= -f2)
echo "Loop True ID: $TRUE_ID_LOOP"
echo ""

echo "=== Step 4: Create label for the loop ==="
echo "Command: label --name target_loop --true-id $TRUE_ID_LOOP"
echo ""
$BIN label --name "target_loop" --true-id "$TRUE_ID_LOOP"
echo ""

echo "=== Step 5: View buffer structure ==="
echo "Command: tree --file $DEMO_FILE"
echo ""
$BIN tree --file "$DEMO_FILE"
echo ""

echo "=== Step 6: Deterministic write using nested label ==="
echo "We'll replace 'for i in range(10):' with 'for i in range(20):'"
echo "using the nested label to target the specific loop."
echo ""
echo "Command: write --file \"$DEMO_FILE\" --label target_loop --replacement \"for i in range(20):\""
echo ""
$BIN write --file "$DEMO_FILE" --label target_loop --replacement "for i in range(20):"
echo ""

echo "=== Step 7: Verify the change ==="
echo ""
echo "--- Updated loop ---"
grep -A1 "for i in range(20):" "$DEMO_FILE"
echo ""

echo "=== Step 8: Demonstrate HASH_MISMATCH safety ==="
echo "If we modify the file between read and write, write will fail safely."
echo ""

echo "Creating backup..."
cp "$DEMO_FILE" "$DEMO_FILE.backup"

echo "Modifying file (changing 'Logging' to 'Processing')..."
sed -i 's/Processing/Transforming/' "$DEMO_FILE"

echo "Trying to write with stale buffer..."
if $BIN write --file "$DEMO_FILE" --label target_loop --replacement "for i in range(30):" 2>&1; then
    echo "Write succeeded (buffer is up to date)"
else
    echo "Write failed (buffer is stale or error occurred)"
fi

echo ""
echo "Restoring original file..."
mv "$DEMO_FILE.backup" "$DEMO_FILE"

echo ""
echo "=== Demo Complete ==="
echo ""
echo "Key Takeaways:"
echo "1. True ID encodes parent context: xxh3_64(file_hash + \"_\" + region_hash)"
echo "2. Nested anchors operate on buffer copies, not original file"
echo "3. Labels provide human-readable references to True IDs"
echo "4. HASH_MISMATCH prevents unsafe writes if file changes"
echo "5. Buffer structure: {TMPDIR}/anchorscope/{file_hash}/{true_id}/content"
echo "6. Ambiguous patterns become uniquely targetable with nested anchoring"
echo ""
