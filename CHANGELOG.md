# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- `pipe` command for external tool integration via stdout/stdin or file I/O
- `paths` command for buffer file path resolution
- `replacement` file in Anchor Buffer for pipeline workflows

### Changed
- Version updated to v1.3.0

## [1.2.0] - 2026-04-09

### Added
- **True ID**: hash-based unique identity for every anchor
- **Alias**: optional human-readable name via `label` command
- **Anchor Buffer**: structured temporary directory for multi-level anchoring
- **tree command**: display current buffer structure
- **trueid command**: compute True ID for nested anchoring

### Changed
- `label` command now uses `--true-id` instead of `--internal-label`
- `read` output includes both `label` (v1.1.0 compat) and `true_id` fields
- Buffer storage: `{TMPDIR}/anchorscope/{file_hash}/{true_id}/content`
- Labels storage: `{TMPDIR}/anchorscope/labels/{alias}.json`

### Breaking Changes
- `--internal-label` replaced by `--true-id` in `label` command
- Label metadata format changed from `internal_label` to `true_id`

### Fixed
- Label validation: now checks if True ID exists in buffer before creating alias

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
