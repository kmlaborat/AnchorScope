# AnchorScope

Deterministic scoped editing protocol.
Minimal, stateless, byte-level exact matching with hash verification.

- Current: v2.0.0 — [docs/SPEC.md](docs/SPEC.md)
- Legacy: v1.x — [v1/](v1/)

---

## Quick Start

Build the project:

```bash
cargo build --release
```

Or install globally:

```bash
cargo install --path .
```

### Typical Workflow

AnchorScope operates in a simple two-step cycle: `read` to locate and hash a scope, then `write` to replace it with hash verification.

```bash
# 1. Read — locate the anchor and obtain scope_hash
anchorscope read --file src/main.rs --anchor "fn calculate_area"

# Output:
# scope_hash=3a7f1c2d4e5b6f8a
# content=fn calculate_area(...)

# 2. Write — replace the scope with hash verification
anchorscope write \
  --file src/main.rs \
  --anchor "fn calculate_area" \
  --expected-hash 3a7f1c2d4e5b6f8a \
  --replacement "fn calculate_area(...) { /* updated */ }"
```

The agent is responsible for retaining `scope_hash` between `read` and `write`. If the file changes between steps, `write` returns `HASH_MISMATCH` and the agent re-runs `read`.

---

## Commands

### read

Match an anchor, compute and return `scope_hash` and matched content.

```bash
anchorscope read --file <path> --anchor "<string>"
# or
anchorscope read --file <path> --anchor-file <path>
```

| Option | Description |
| ------ | ----------- |
| `--file` | Path to the target file |
| `--anchor` | Anchor string (exact match, multi-line via escape sequences) |
| `--anchor-file` | Path to a file containing the anchor string |

`--anchor` and `--anchor-file` are mutually exclusive.

**Output** (exit 0 on success):

```
scope_hash=<16-char hex string>
content=<matched bytes as UTF-8>
```

The source file is never modified.

### write

Verify the hash and apply a replacement.

```bash
anchorscope write \
  --file <path> \
  --anchor "<string>" \
  --expected-hash <hex> \
  --replacement "<string>"
# or with files
anchorscope write \
  --file <path> \
  --anchor-file <path> \
  --expected-hash <hex> \
  --replacement-file <path>
```

| Option | Description |
| ------ | ----------- |
| `--file` | Path to the target file |
| `--anchor` | Anchor string — must match exactly |
| `--anchor-file` | Path to a file containing the anchor string |
| `--expected-hash` | `scope_hash` obtained from `read` |
| `--replacement` | Replacement string (replaces the entire matched scope) |
| `--replacement-file` | Path to a file containing the replacement content |

`--anchor` / `--anchor-file` are mutually exclusive.
`--replacement` / `--replacement-file` are mutually exclusive.

The replacement is written **as-is** — no normalization is applied. The agent is responsible for the byte content of the replacement.

---

## Error Model

AnchorScope returns a specific error string for each failure mode. Errors are printed to stderr; exit code 1 indicates failure.

| Output | Description |
| ------ | ----------- |
| `NO_MATCH` | Zero occurrences of the anchor found, or empty anchor |
| `MULTIPLE_MATCHES` | Anchor matches at more than one position |
| `HASH_MISMATCH` | Computed hash differs from `--expected-hash` |
| `IO_ERROR: file not found` | Target file or anchor file does not exist |
| `IO_ERROR: permission denied` | Insufficient file system permissions |
| `IO_ERROR: invalid UTF-8` | File or anchor content is not valid UTF-8 |
| `IO_ERROR: read failure` | File could not be read |
| `IO_ERROR: write failure` | File could not be written or offset verification failed |

**Error evaluation order is strict:**

1. Read and validate file content
2. Validate UTF-8
3. Normalize (in-memory) and match anchor
4. If match count ≠ 1, return match-count error
5. If match count = 1, compute and compare hash
6. If hash mismatch, return `HASH_MISMATCH`
7. If hash matches, perform write (may yield `IO_ERROR`)

---

## Anchor Strategies

### Inline Anchor (`--anchor`)

Pass the anchor as a command-line string. Requires proper shell escaping for newlines and special characters:

```bash
anchorscope read --file src.rs --anchor $'fn main() {\n\tprintln!("Hello");\n}'
```

Inline arguments are assumed to be valid UTF-8 by the CLI layer.

### File-Based Anchor (`--anchor-file`) — Recommended

Read the anchor from a file. No escaping required; preserves exact byte content including newlines:

```bash
cat > anchor.txt << 'EOF'
fn main() {
    println!("Hello");
}
EOF
anchorscope read --file src.rs --anchor-file anchor.txt
```

**File-based anchors are recommended for multi-line anchors and agent-generated workflows.**

---

## Determinism Guarantees

AnchorScope provides the following guarantees:

* **Byte-level matching** — only exact byte equality is accepted; no fuzzy matching, no regex, no heuristics
* **Complete search** — all possible starting positions are evaluated; overlapping matches are detected
* **Single-location semantics** — operations succeed only when exactly one match exists
* **Hash determinism** — `xxh3_64` produces identical output for identical byte sequences
* **In-memory normalization** — CRLF→LF normalization is applied solely for matching and hashing; the original file bytes outside the matched scope are never modified
* **Atomic replacement** — the file is reconstructed as `prefix + replacement + suffix` with no modifications to prefix or suffix
* **Stateless design** — no state is held between operations; the agent retains `scope_hash`

---

## Why AnchorScope

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
* **No recovery** — failures are explicit and require agent intervention

The result is a protocol that behaves identically across all compliant implementations, enabling reliable automation and deterministic anchoring of edits.

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

---

## License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for the full text.

---

### Disclaimer

**THE SOFTWARE IS PROVIDED "AS IS"**, without warranty of any kind. As this is a reference implementation of a file-editing protocol, the author is not responsible for any data loss or unintended file modifications resulting from its use. Always use version control and test in a safe environment.

Copyright (c) 2026 kmlaborat
