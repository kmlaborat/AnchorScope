# AnchorScope

**Deterministic code editing protocol — reference implementation of the Scope Anchoring standard.**

---

## What is AnchorScope

AnchorScope is a minimal reference implementation of the Scope Anchoring protocol, a deterministic code editing protocol that guarantees exact region identification and safe replacement through deterministic hash verification.

Unlike traditional search-replace tools that rely on heuristics or fuzzy matching, AnchorScope enforces byte-level exactness, making it suitable for agent-based systems where correctness and reproducibility are paramount.

See the full specification: [docs/SPEC.md](docs/SPEC.md)

---

## Core Model: READ → MATCH → HASH → WRITE

AnchorScope operations follow a strict four-phase pipeline:

1. **READ** — Load file content, normalize line endings (CRLF → LF), validate UTF-8 (for file-based inputs)
2. **MATCH** — Find all exact byte-level occurrences of the anchor; count results
3. **HASH** — When exactly one match exists, compute xxh3_64 hash of the matched region
4. **WRITE** — Replace the matched region if hash verification succeeds

All phases execute on normalized byte sequences. No transformations other than CRLF→LF normalization are permitted.

**AnchorScope enforces correctness by design: it does not attempt to resolve ambiguity.**

---

## CLI Usage

### Read: Locate and Hash

```bash
anchorscope read --file <path> --anchor "<string>"
# or
anchorscope read --file <path> --anchor-file <path>
```

Output (exit 0 on success):

```
start_line=<1-based line>
end_line=<1-based line>
hash=<16-char hex string>
content=<matched bytes as UTF-8>
```

### Write: Replace with Verification

```bash
anchorscope write \
  --file <path> \
  --anchor "<string>" \
  --expected-hash <hex> \
  --replacement "<string>"
# or
anchorscope write \
  --file <path> \
  --anchor-file <path> \
  --expected-hash <hex> \
  --replacement "<string>"
```

Exit 0 on success, 1 on any error condition.

### Anchor: Define Labeled Region

```bash
anchorscope anchor \
  --file <path> \
  --label <name> \
  --anchor "<string>" \
  --expected-hash <hex>
# or
anchorscope anchor \
  --file <path> \
  --label <name> \
  --anchor-file <path> \
  --expected-hash <hex>
```

Stores a mapping from label to file, anchor, and hash in `~/.anchorscope/labels/<label>.json` for later reference.

Exit 0 on success, 1 on any error.

---

## Anchor Strategies

### Inline Anchor (`--anchor`)

Pass the anchor as a command-line string. Requires proper shell escaping for newlines and special characters:

```bash
anchorscope read --file src.rs --anchor $'fn main() {\n\tprintln!("Hello");\n}'
```

Inline arguments are assumed to be valid UTF-8 by the CLI layer and are not validated by AnchorScope itself.

### File-Based Anchor (`--anchor-file`) — Recommended

Read the anchor from a file. No escaping required; preserves exact byte content including newlines:

```bash
echo 'fn main() {
    println!("Hello");
}' > anchor.txt
anchorscope read --file src.rs --anchor-file anchor.txt
```

**File-based anchors are recommended for multi-line anchors and agent-generated workflows.**

---

## Determinism Guarantees

AnchorScope provides the following guarantees:

* **Byte-level matching**: Only exact byte equality is accepted. No character-level logic, no Unicode normalization.
* **Complete search**: All possible starting positions are evaluated; overlapping matches are detected and counted.
* **Symmetric normalization**: CRLF→LF normalization applies identically to file content, anchor, and replacement.
* **Hash determinism**: xxh3_64 produces identical output for identical byte sequences.
* **Single-location semantics**: Operations succeed only when exactly one match exists.
* **Atomic replacement**: The entire file is reconstructed as `prefix + replacement + suffix` with no modifications to prefix or suffix.
* **Persistent normalization**: Written files are stored in normalized form (LF only).

---

## Error Model

AnchorScope returns a specific error condition for each failure mode. Errors are printed to stderr; exit code 1 indicates failure.

| Condition              | Output                                   | Description                                       |
| ---------------------- | ---------------------------------------- | ------------------------------------------------- |
| `NO_MATCH`             | `NO_MATCH`                               | Zero occurrences of anchor found                  |
| `MULTIPLE_MATCHES (N)` | `MULTIPLE_MATCHES (N)`                   | Anchor appears at N>1 positions                   |
| `HASH_MISMATCH`        | `HASH_MISMATCH: expected=... actual=...` | Matched region differs from expected              |
| `IO_ERROR: ...`        | `IO_ERROR: <type>`                       | File I/O, permission, or UTF-8 validation failure |

**Error evaluation order is strict:**

1. Count matches (must be exactly 1)
2. If count ≠ 1, return match-count error
3. If count = 1, compute and compare hash
4. If hash mismatch, return `HASH_MISMATCH`
5. If hash matches, perform write (may yield `IO_ERROR`)

---

## Why AnchorScope?

Traditional search-replace tools make implicit assumptions:

* Fuzzy matching tolerates minor variations
* Whitespace trimming "corrects" formatting
* Heuristics guess intent when patterns are ambiguous
* Multi-line overlaps are partially matched

These conveniences introduce **non-determinism**. The same anchor may match different regions across implementations or after minor edits. This breaks agent-based workflows where reproducibility is essential.

AnchorScope eliminates all guesswork:

* **No fuzzy matching** — only exact byte sequences
* **No trimming** — whitespace is significant
* **No early termination** — all candidates evaluated
* **No recovery** — failures are explicit and require human or orchestration-layer intervention

The result is a protocol that behaves identically across all compliant implementations, enabling reliable automation and deterministic anchoring of edits.

---

## Reference Implementation

This repository is the reference implementation of the AnchorScope protocol as defined in the Scope Anchoring specification. It implements the full MVP scope:

* Single-file operations
* Exact multi-line anchor matching
* xxh3_64 hash verification
* Deterministic error handling
* UTF-8 validation for file-based inputs
* CRLF→LF normalization

The implementation is deliberately minimal and strict. No optional features, no compatibility modes, no heuristics.

---

## Project Status & Maintenance

**AnchorScope is a reference implementation provided "as-is".**

As the founder of the Scope Anchoring protocol, my primary focus is on the specification itself and higher-level tools.

The goal of this repository is to keep the implementation minimal, stable, and strictly aligned with the specification. 

- **Maintenance:** Active feature development and rapid Pull Request responses are not guaranteed.
- **Forks:** Independent implementations and forks are highly encouraged.
- **Bugs:** Only critical bugs affecting the deterministic nature of the protocol will be prioritized.

---

## Credits & Dependencies

AnchorScope is built upon these excellent open-source libraries:

- **[xxhash-rust](https://github.com/DoumanAsh/xxhash-rust)** (BSL-1.0) — High-performance XXH3 implementation.
- **[clap](https://github.com/clap-rs/clap)** (MIT/Apache-2.0) — Flexible Command Line Argument Parser.
- **[tempfile](https://github.com/Stebalien/tempfile)** (MIT/Apache-2.0) — Robust temporary file management for testing.

---

## License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for the full text.

---

### Disclaimer

**THE SOFTWARE IS PROVIDED "AS IS"**, without warranty of any kind. As this is a reference implementation of a file-editing protocol, the author is not responsible for any data loss or unintended file modifications resulting from its use. Always use version control and test in a safe environment.

Copyright (c) 2026 kmlaborat
