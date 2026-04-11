# Implementation Timeline

## Overview
Fix 3 critical SPEC compliance gaps in AnchorScope v1.3.0

## Schedule

### Day 1: Core Replacement Support
- [ ] 9:00-9:30 - Review verification report
- [ ] 9:30-10:00 - Implement CLI changes (Phase 1.1)
- [ ] 10:00-11:00 - Update write command (Phase 1.2)
- [ ] 11:00-11:30 - Add storage helper (Phase 1.3)
- [ ] 11:30-12:00 - Update main.rs (Phase 1.4)

### Day 2: Advanced Write Logic
- [ ] 9:00-9:30 - Finish write function (Phase 1.5)
- [ ] 9:30-11:30 - Add integration tests (Phase 1.6)
- [ ] 11:30-12:00 - Run tests, verify Phase 1 complete

### Day 3: Duplicate Detection
- [ ] 9:00-10:00 - Implement duplicate detection (Phase 2.1)
- [ ] 10:00-10:30 - Update find_true_id_dir (Phase 2.2)
- [ ] 10:30-11:00 - Update load_buffer_metadata (Phase 2.3)
- [ ] 11:00-12:00 - Update all commands (Phase 2.4)

### Day 4: Tests and Polish
- [ ] 9:00-10:00 - Add duplicate ID tests (Phase 2.5)
- [ ] 10:00-10:30 - Verify AMBIGUOUS_REPLACEMENT (Phase 3)
- [ ] 10:30-12:00 - Final testing and cleanup

## Deliverables

### Code Changes
- `src/cli.rs` - Add `--from-replacement` flag
- `src/commands/write.rs` - Support replacement file
- `src/storage.rs` - Add `load_replacement_content`, duplicate detection
- `src/main.rs` - Pass new flag to write command

### Tests
- `tests/integration/write_from_replacement_tests.rs` (new)
- `tests/integration/error_duplicate_true_id_tests.rs` (new)

### Documentation
- `docs/plans/verification-report.md` - Analysis
- `docs/plans/implementation-plan.md` - Detailed plan
- This file - Timeline

## Status Tracker

| Phase | Status | Hours Used |
| :---- | :----- | :--------- |
| 1. CLI Update | Pending | 0 |
| 2. Write Logic | Pending | 0 |
| 3. Storage Helper | Pending | 0 |
| 4. Main Update | Pending | 0 |
| 5. Integration Tests | Pending | 0 |
| 6. Duplicate Detection | Pending | 0 |
| 7. Command Updates | Pending | 0 |
| 8. Final Tests | Pending | 0 |
| **Total** | - | **~10 hours** |

## Blocks

- **No other tasks should be started** until this is complete
- **All changes are critical** for SPEC compliance
- **No feature additions** - strictly bug fixes

## Exit Criteria

- [ ] All 70+ tests pass
- [ ] No compiler warnings
- [ ] SPEC §6.3 fully implemented
- [ ] SPEC §3.2 DUPLICATE_TRUE_ID handled
- [ ] Pipeline workflow verified
