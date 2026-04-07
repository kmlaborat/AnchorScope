# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

## [1.1.0] - 2026-04-07

### Added
- Auto-labeling: `read` command now auto-generates an internal label for matched regions.
- `label` command to map internal labels to human-readable names.
- `write --label` support to perform replacements using human-readable labels.
- Ephemeral storage using system temp directory (`%TEMP%/anchorscope/`).
- Automatic cleanup of anchor/label files after successful `write` (SPEC §3.3 compliance).
- `examples/` demo suite with `v1_1_0_showcase.sh` script.

### Changed
- Refactored codebase into `src/commands/{read,write,label}.rs` modules.
- Storage backend migrated from `~/.anchorscope/` to `std::env::temp_dir()/anchorscope/`.
- `dirs` dependency removed (replaced by `std::env::temp_dir()`).

### Removed
- `anchorscope anchor` command (replaced by `label` command).
- Permanent `~/.anchorscope/` directory (replaced by ephemeral temp dir).

### Fixed
- Deterministic error mapping to conform to SPEC §4.5 (`IO_ERROR: file not found`, `permission denied`, etc.).
- Strict `match`-based error handling for all i32-returning functions (no `?` operator).

## [1.0.1] - 2026-04-03
- Initial stable release.
