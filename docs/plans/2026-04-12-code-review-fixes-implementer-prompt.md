You are the implementer subagent for Task 1 of the code review fixes plan.

## Context
This is part of fixing 3 issues identified in a security audit code review:
1. Test failures for `test_map_io_error_read/write` (current task)
2. Unused variable warnings in storage.rs
3. Unused function warnings in config.rs

## Your Task: Fix WriteFailure error message for NotFound

**Files to modify:**
- `src/error.rs` (lines ~187-196)

**Current behavior:**
- `AnchorScopeError::WriteFailure(e)` returns `format!("IO_ERROR: write failure: {}", e)`
- This causes `test_map_io_error_write` to fail because for NotFound errors, it returns "write failure: entity not found" instead of the expected "write failure"

**Required fix:**
- Modify `AnchorScopeError::WriteFailure(e)` in `to_spec_string()` to check if `e.kind() == std::io::ErrorKind::NotFound`
- If NotFound, return `"IO_ERROR: write failure"` (without the error details)
- Otherwise, return `format!("IO_ERROR: write failure: {}", e)` as before

## Implementation Steps

1. Read `src/error.rs` to understand the current implementation
2. Read `src/main.rs` to see the failing tests (`test_map_io_error_read` and `test_map_io_error_write`)
3. Run `cargo test test_map_io_error_write -- --nocapture` to confirm the failure
4. Implement the fix in `src/error.rs`
5. Run `cargo test test_map_io_error_write -- --nocapture` and `cargo test test_map_io_error_read -- --nocapture` to verify both pass
6. Run `cargo fmt` and `cargo clippy` to ensure code quality
7. Self-review your changes, checking:
   - Does the fix handle both read and write NotFound cases correctly?
   - Are there any edge cases I missed?
   - Is the change minimal and focused?
8. Commit with message: "fix: WriteFailure NotFound returns simple message for backward compatibility"

## Important Notes
- The plan is saved at `docs/plans/2026-04-12-code-review-fixes.md`
- Use TDD: fail the test first, then fix it
- Make small, focused commits
- Self-review is REQUIRED before committing

## Output Format
After implementation, report:
1. What tests were failing
2. What changes you made
3. Test results after fix
4. Self-review assessment
5. Git commit hash
