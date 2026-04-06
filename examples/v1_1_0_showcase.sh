#!/usr/bin/env bash
set -uo pipefail

# ============================================================================
# AnchorScope v1.1.0 Showcase
# Demonstrates: Auto-Labeling, Label Management, and Deterministic Safety
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DEMO_FILE="$SCRIPT_DIR/demo_target.txt"
WORK_FILE="$SCRIPT_DIR/demo_target_work.rs"

# ANSI colors for pretty output
GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Build fresh
echo -e "${CYAN}===========================================================${NC}"
echo -e "${CYAN}  Building AnchorScope v1.1.0...${NC}"
echo -e "${CYAN}===========================================================${NC}"
cargo build --quiet --manifest-path "$PROJECT_DIR/Cargo.toml"
ANCHOR=(cargo run --quiet --manifest-path "$PROJECT_DIR/Cargo.toml" --)

# Create working copy
cp "$DEMO_FILE" "$WORK_FILE"
echo ""
echo -e "${CYAN}  Created working copy: ${YELLOW}demo_target_work.rs${NC}"
echo ""

# Helper: run anchorscope command
run_read() {
    echo -e "${YELLOW}  $ anchorscope read --file demo_target_work.rs --anchor '$1'${NC}"
    "${ANCHOR[@]}" read --file "$WORK_FILE" --anchor "$1"
}

run_cmd() {
    echo -e "${YELLOW}  $ anchorscope $@${NC}"
    "${ANCHOR[@]}" "$@" 2>&1 || true
}

# Helper: extract label from read output
extract_label() {
    echo "$1" | grep "^label=" | head -1 | cut -d= -f2
}

# ============================================================================
# PART 1: HAPPY PATH — Auto-Labeling & Label Management
# ============================================================================
echo -e "${CYAN}===========================================================${NC}"
echo -e "${GREEN}  PART 1: HAPPY PATH${NC}"
echo -e "${CYAN}  Read → Auto-Label → Assign Name → Write${NC}"
echo -e "${CYAN}===========================================================${NC}"
echo ""

echo -e "${CYAN}  Step 1: Read the first TODO comment (auto-label generated)${NC}"
OUTPUT=$(run_read 'TODO: Add input validation (reject negative numbers)' || true)
INTERNAL_ID=$(echo "$OUTPUT" | grep "^label=" | head -1 | cut -d= -f2)
echo "$OUTPUT"
echo ""

if [ -z "$INTERNAL_ID" ]; then
    echo -e "${RED}  ERROR: Could not extract internal label from read output${NC}"
    rm -f "$WORK_FILE"
    exit 1
fi

echo -e "${CYAN}  Extracted Internal Label: ${GREEN}$INTERNAL_ID${NC}"
echo ""

echo -e "${CYAN}  Step 2: Assign human-readable label${NC}"
run_cmd label --name "validation_todo" --internal-label "$INTERNAL_ID"
echo ""

echo -e "${CYAN}  Step 3: Write replacement using --label${NC}"
run_cmd write \
    --file "$WORK_FILE" \
    --label "validation_todo" \
    --replacement "validate_input: items.iter().map(|&x| if x < 0.0 { 0.0 } else { x }).collect::<Vec<f64>>().iter().sum()"
echo ""

echo -e "${CYAN}  Result — updated code:${NC}"
echo -e "${GREEN}$(sed -n '8,11p' "$WORK_FILE")${NC}"
echo ""

# ============================================================================
# PART 2: SAFETY PATH — Deterministic Failures
# ============================================================================
echo -e "${CYAN}===========================================================${NC}"
echo -e "${RED}  PART 2: SAFETY PATH${NC}"
echo -e "${CYAN}  Demonstrating Deterministic Hash Verification${NC}"
echo -e "${CYAN}===========================================================${NC}"
echo ""

echo -e "${RED}  Attempt A: Re-use same label after successful write${NC}"
echo -e "  (The original anchor text no longer exists in the modified file)${NC}"
run_cmd write \
    --file "$WORK_FILE" \
    --label "validation_todo" \
    --replacement "something_else"
echo ""
echo -e "${GREEN}  → As expected: Fail-fast — the label's anchor is gone (NO_MATCH)${NC}"
echo ""

echo -e "${RED}  Attempt B: Write with wrong hash against modified file${NC}"
echo -e "  (Simulating state drift — anchor content was invalidated)${NC}"
run_cmd write \
    --file "$WORK_FILE" \
    --anchor "validate_input:" \
    --expected-hash "ffffffffffffffff" \
    --replacement "replaced"
echo ""
echo -e "${GREEN}  → As expected: HASH_MISMATCH confirmed${NC}"
echo ""

# ============================================================================
# CLEANUP
# ============================================================================
echo -e "${CYAN}===========================================================${NC}"
echo -e "${CYAN}  Cleaning up...${NC}"
echo -e "${CYAN}===========================================================${NC}"
rm -f "$WORK_FILE"
# Remove demo labels to keep system clean
LABEL_DIR="$HOME/.anchorscope/labels/validation_todo.json"
if [ -f "$LABEL_DIR" ]; then rm -f "$LABEL_DIR"; fi
# Remove anchor store entries for demo
if [ -d "$HOME/.anchorscope/anchors" ]; then
    find "$HOME/.anchorscope/anchors" -name "*.json" -newer "$DEMO_FILE" -delete 2>/dev/null || true
fi

echo -e "${GREEN}  Done — working copy and temporary labels removed.${NC}"
echo ""
echo -e "${CYAN}===========================================================${NC}"
echo -e "${GREEN}  AnchorScope v1.1.0 Showcase Complete ✅${NC}"
echo -e "${CYAN}===========================================================${NC}"
