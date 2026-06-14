#!/bin/bash

# AnchorScope v2.0.0 Demo: Deterministic Scoped Editing
# Demonstrates precise code editing using read/write with hash verification

set -uo pipefail

# ── Colors ──────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${BOLD}=== AnchorScope v2.0.0 Demo ===${NC}"
echo -e "Demonstrating: read/write workflow, hash verification, and error handling"
echo ""
echo -e "This demo shows:"
echo -e "  1. ${CYAN}read${NC} — locate an anchor and obtain scope_hash"
echo -e "  2. ${CYAN}write${NC} — replace the scope with hash verification"
echo -e "  3. ${YELLOW}HASH_MISMATCH${NC} — file changed between read and write"
echo -e "  4. ${YELLOW}MULTIPLE_MATCHES${NC} — anchor is not unique"
echo -e "  5. ${YELLOW}NO_MATCH${NC} — anchor does not exist"
echo ""

DEMO_FILE="demo_target.rs"
BIN="./target/release/anchorscope"

echo -e "Binary: ${BOLD}$BIN${NC}"
echo -e "Target file: ${BOLD}$DEMO_FILE${NC}"
echo ""

# ── Helper: create demo file ────────────────────────────────────
create_demo_file() {
    cat > "$DEMO_FILE" << 'RUST_CODE'
fn calculate_area(width: f64, height: f64) -> f64 {
    // Calculate the area of a rectangle
    width * height
}

fn calculate_perimeter(width: f64, height: f64) -> f64 {
    // Calculate the perimeter of a rectangle
    2.0 * (width + height)
}

fn main() {
    println!("Area: {}", calculate_area(5.0, 3.0));
    println!("Perimeter: {}", calculate_perimeter(5.0, 3.0));
}
RUST_CODE
}

# Create demo file
create_demo_file

# ── Step 0: Show the demo file ──────────────────────────────────
echo -e "${BOLD}=== Step 0: The Demo File ===${NC}"
echo "--- File content ---"
cat "$DEMO_FILE"
echo ""

# ── Step 1: read — locate and hash ──────────────────────────────
echo -e "${BOLD}=== Step 1: read — locate the calculate_area function ===${NC}"
echo -e "Command: ${CYAN}read${NC} with multiline anchor capturing the function body"
echo ""

ANCHOR_AREA='fn calculate_area(width: f64, height: f64) -> f64 {
    // Calculate the area of a rectangle
    width * height
}'

echo -e "${BOLD}Running:${NC}"
echo -e "${CYAN}$BIN${NC} read --file $DEMO_FILE --anchor \"<function body>\""
echo ""

READ_OUTPUT=$($BIN read --file "$DEMO_FILE" --anchor "$ANCHOR_AREA")
echo "$READ_OUTPUT"
echo ""

SCOPE_HASH=$(echo "$READ_OUTPUT" | grep "^scope_hash=" | cut -d= -f2)
echo -e "scope_hash: ${GREEN}${SCOPE_HASH}${NC}"
echo ""

# ── Step 2: write — replace with hash verification ──────────────
echo -e "${BOLD}=== Step 2: write — replace the function body ===${NC}"
echo -e "Replace calculate_area with an optimized version."
echo -e "Hash verification ensures the file has not changed since ${CYAN}read${NC}."
echo ""

REPLACEMENT='fn calculate_area(width: f64, height: f64) -> f64 {
    // Optimized: use multiplication directly
    width * height
}'

echo -e "${BOLD}Running:${NC}"
echo -e "${CYAN}$BIN${NC} write --file $DEMO_FILE --expected-hash $SCOPE_HASH"
echo ""

WRITE_OUTPUT=$($BIN write \
    --file "$DEMO_FILE" \
    --anchor "$ANCHOR_AREA" \
    --expected-hash "$SCOPE_HASH" \
    --replacement "$REPLACEMENT" 2>&1)
echo "$WRITE_OUTPUT"
echo ""

# ── Step 3: verify the change ───────────────────────────────────
echo -e "${BOLD}=== Step 3: Verify the change ===${NC}"
echo "--- Updated file ---"
cat "$DEMO_FILE"
echo ""

# Verify calculate_perimeter was NOT modified
if grep -q "2.0 \* (width + height)" "$DEMO_FILE"; then
    echo -e "${GREEN}✓ calculate_perimeter is unchanged${NC}"
else
    echo -e "${RED}✗ calculate_perimeter was modified (unexpected)${NC}"
fi

# Verify main was NOT modified
if grep -q 'println!("Area: {}"' "$DEMO_FILE"; then
    echo -e "${GREEN}✓ main is unchanged${NC}"
else
    echo -e "${RED}✗ main was modified (unexpected)${NC}"
fi

# Verify calculate_area WAS modified
if grep -q "Optimized:" "$DEMO_FILE"; then
    echo -e "${GREEN}✓ calculate_area was updated${NC}"
else
    echo -e "${RED}✗ calculate_area was not updated (unexpected)${NC}"
fi
echo ""

# ── Step 4: HASH_MISMATCH demo ──────────────────────────────────
echo -e "${BOLD}=== Step 4: HASH_MISMATCH — hash verification fails ===${NC}"
echo -e "When the expected hash does not match the actual scope hash,"
echo -e "the write is rejected. This prevents applying stale replacements."
echo ""

# Recreate the demo file for a clean state
create_demo_file

# Read to get the hash
HASH_OUTPUT=$($BIN read --file "$DEMO_FILE" --anchor "$ANCHOR_AREA")
REAL_HASH=$(echo "$HASH_OUTPUT" | grep "^scope_hash=" | cut -d= -f2)
echo -e "Real scope_hash:     ${GREEN}${REAL_HASH}${NC}"

# Use a deliberately wrong hash to simulate a stale/out-of-date hash
FAKE_HASH="0000000000000000"
echo -e "Fake scope_hash:     ${RED}${FAKE_HASH}${NC}"
echo ""

# Try to write with the wrong hash
echo -e "${BOLD}Running:${NC} write with wrong hash..."
MISMATCH_OUTPUT=$($BIN write \
    --file "$DEMO_FILE" \
    --anchor "$ANCHOR_AREA" \
    --expected-hash "$FAKE_HASH" \
    --replacement "$REPLACEMENT" 2>&1)
echo "$MISMATCH_OUTPUT"
echo ""

if echo "$MISMATCH_OUTPUT" | grep -q "HASH_MISMATCH"; then
    echo -e "${GREEN}✓ Write correctly rejected with HASH_MISMATCH${NC}"
else
    echo -e "${RED}✗ Expected HASH_MISMATCH${NC}"
fi
echo ""

# ── Step 5: MULTIPLE_MATCHES demo ───────────────────────────────
echo -e "${BOLD}=== Step 5: MULTIPLE_MATCHES — anchor is not unique ===${NC}"
echo -e "When the anchor matches at more than one position,"
echo -e "AnchorScope rejects the operation instead of guessing."
echo ""

# Recreate for clean state
create_demo_file

# Use a non-unique anchor: the parameter list appears in both functions
MULTI_ANCHOR="(width: f64, height: f64) -> f64"

echo -e "${BOLD}Running:${NC} read with non-unique anchor"
MULTI_OUTPUT=$($BIN read --file "$DEMO_FILE" --anchor "$MULTI_ANCHOR" 2>&1)
echo "$MULTI_OUTPUT"
echo ""

if echo "$MULTI_OUTPUT" | grep -q "MULTIPLE_MATCHES"; then
    echo -e "${GREEN}✓ Correctly rejected with MULTIPLE_MATCHES${NC}"
else
    echo -e "${RED}✗ Expected MULTIPLE_MATCHES${NC}"
fi
echo ""

# ── Step 6: NO_MATCH demo ───────────────────────────────────────
echo -e "${BOLD}=== Step 6: NO_MATCH — anchor does not exist ===${NC}"
echo -e "When the anchor does not match any position in the file,"
echo -e "AnchorScope returns NO_MATCH."
echo ""

NO_MATCH_ANCHOR="fn compute_volume(length: f64, width: f64, height: f64)"

echo -e "${BOLD}Running:${NC} read with non-existent anchor"
NO_MATCH_OUTPUT=$($BIN read --file "$DEMO_FILE" --anchor "$NO_MATCH_ANCHOR" 2>&1)
echo "$NO_MATCH_OUTPUT"
echo ""

if echo "$NO_MATCH_OUTPUT" | grep -q "NO_MATCH"; then
    echo -e "${GREEN}✓ Correctly returned NO_MATCH${NC}"
else
    echo -e "${RED}✗ Expected NO_MATCH${NC}"
fi
echo ""

# ── Cleanup ─────────────────────────────────────────────────────
echo -e "${BOLD}=== Cleanup ===${NC}"
rm -f "$DEMO_FILE"
echo -e "Removed ${BOLD}$DEMO_FILE${NC}"
echo ""

echo -e "${BOLD}=== Demo Complete ===${NC}"
echo -e "All scenarios demonstrated successfully."
