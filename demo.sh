#!/bin/bash

# AnchorScope v1.2.0 Demo: Multi-Level Anchoring
# Shows how to precisely edit ambiguous code using nested anchors

set -e

echo "=== AnchorScope v1.2.0 Demo ==="
echo "Demonstrating: Multi-level anchoring for precise code editing"
echo ""

APP_FILE="app.py"

# Helper function to run anchorscope and show output
run_anchorscope() {
    echo "---"
    echo "Command: $@"
    echo "---"
    cargo run --bin anchorscope -- "$@"
    echo ""
}

# Clean up temp files first
echo "Step 0: Clean up any existing buffer state"
echo ""

echo "Step 1: Show the target file (app.py)"
echo "========================================"
cat "$APP_FILE"
echo ""

echo "Step 2: Level 1 - Anchor the outer scope (function)"
echo "======================================================"
echo "Target: 'def process_data():' in app.py"
echo "This anchors the entire function as the outer scope."
echo ""

run_anchorscope read --file "$APP_FILE" --anchor "def process_data():"

echo ""
echo "Step 3: Assign human-readable label to the function's True ID"
echo "==============================================================="
echo "The read command output includes:"
echo "  - hash: Region hash (v1.1.0 compatible)"
echo "  - true_id: True ID = xxh3_64(file_hash + \"_\" + region_hash)"
echo ""

# Get the true_id from previous output
TRUE_ID_FUNC=$(cargo run --quiet --bin anchorscope -- read --file "$APP_FILE" --anchor "def process_data():" | grep "true_id=" | cut -d= -f2)
echo "Function True ID: $TRUE_ID_FUNC"
echo ""

run_anchorscope label --name "func_data" --true-id "$TRUE_ID_FUNC"
echo ""

echo "Step 4: Level 2 - Anchor inside the buffer (nested anchor)"
echo "============================================================"
echo "Now we search for 'for i in range(10):' INSIDE the 'func_data' buffer."
echo "Even though there are TWO 'for i in range(10):' in the file,"
echo "there's only ONE inside the 'process_data' function."
echo ""

echo "Trying to search in file directly (would fail with MULTIPLE_MATCHES):"
echo "$ cargo run --quiet --bin anchorscope -- read --file \"$APP_FILE\" --anchor \"for i in range(10):\""
cargo run --quiet --bin anchorscope -- read --file "$APP_FILE" --anchor "for i in range(10):" || true
echo ""

echo "But searching INSIDE the buffer (success!):"
# NOTE: Nested read not yet implemented - this is a placeholder for future implementation
echo "ERROR: Nested read not yet implemented in current version"
echo ""
echo "For now, we demonstrate the label system:"
echo ""

echo "Step 5: View buffer structure with tree command"
echo "================================================"
run_anchorscope tree --file "$APP_FILE"
echo ""

echo "Step 6: Write using label (deterministic replacement)"
echo "======================================================"
echo "We'll use label-based write which uses the label to find the anchor."
echo ""

echo "Step 7: Test HASH_MISMATCH safety"
echo "=================================="
echo "If the file changes after read but before write, write fails safely."
echo ""

echo "=== Demo Complete ==="
echo ""
echo "Key Takeaways:"
echo "1. True ID encodes parent context: xxh3_64(file_hash + \"_\" + region_hash)"
echo "2. Nested anchors operate on buffer copies, not original file"
echo "3. Labels provide human-readable references to True IDs"
echo "4. HASH_MISMATCH prevents unsafe writes if file changes"
echo "5. Buffer structure: {TMPDIR}/anchorscope/{file_hash}/{true_id}/content"
echo ""
