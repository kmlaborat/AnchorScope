# AnchorScope Specification v2.0.0

## Deterministic Scoped Editing Protocol

**AnchorScope is a deterministic code editing protocol based on Scope Anchoring.**
It treats code as **immutable UTF-8 byte sequences**, not as text or syntax.

All operations are strictly **byte-level, deterministic, and single-location**.

The key words "MUST", "MUST NOT", "SHOULD", and "MAY" in this document are to
be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

AnchorScope v2.0.0 is a simplification of v1.x. It removes:

* **Anchor Buffer** — no temporary copies; every operation reads from the source file directly
* **True ID** — no multi-level anchor identity; scope_hash is sufficient
* **Alias / label** — no human-readable name management
* **pipe / paths** — no external tool pipeline commands
* **Line numbering** — no line-based output

The result is a minimal, stateless protocol: `read` and `write`.

---

## 1. Concept: Scoped Editing (Informative)

### 1.1 Problem: Fragility of Global Edits

Full-file rewrites are high-risk, and diff-based patching is fragile.
Even minor contextual changes can invalidate patches.

### 1.2 Solution: Anchor and Scope

AnchorScope defines a precise **editing scope** using an exact byte sequence
("Anchor"), combined with **hash-based state verification**.

This enables edits that are:

* Safe (fail-fast)
* Precise (single-location)
* Idempotent (state-verified)

### 1.3 Stateless Design

AnchorScope v2.0.0 holds no state between operations.

Every `read` operates directly on the source file.
Every `write` re-reads and re-verifies the source file before applying changes.
No temporary files, no buffers, no identity tracking.

The agent is responsible for:

* Choosing the anchor string
* Retaining the `scope_hash` between `read` and `write`
* Constructing the replacement content

### 1.4 Scope Localization

AnchorScope does not prescribe how an agent locates a target scope within a
file. One effective strategy is **Sliding Bisection** (defined separately in
AnchorEdit), which narrows a target region through repeated 3-choice selections
without semantic analysis.

AnchorScope operates after localization is complete: once the agent has chosen
an anchor string, `read` confirms the match and `write` applies the change.

### 1.5 Layered Model

| Layer              | Name            | Role                                     |
| :----------------- | :-------------- | :--------------------------------------- |
| **Concept**        | Scoped Editing  | Philosophy of local, verifiable mutation |
| **Protocol**       | Scope Anchoring | Deterministic matching & hashing rules   |
| **Implementation** | AnchorScope     | Reference CLI (`read` / `write`)         |

---

## 2. Protocol: Scope Anchoring (Normative)

### 2.1 Invariants

The following invariants **MUST** hold:

1. Matching **MUST** be exact byte equality after normalization.
2. Matching **MUST** evaluate all possible byte offsets.
3. Exactly one match **MUST** exist to proceed.
4. All operations **MUST** be deterministic.
5. No implicit interpretation (syntax, encoding heuristics) is allowed.

---

### 2.2 Encoding & Validation

All inputs **MUST** be valid UTF-8.

* File content **MUST** be validated immediately after reading.
* `--anchor-file` content **MUST** be validated.
* Inline arguments (`--anchor`, `--replacement`) are assumed valid.

#### Error

If invalid UTF-8 is detected:

```
IO_ERROR: invalid UTF-8
```

#### Constraints

* No partial decoding
* No lossy conversion
* Validation **MUST** occur before normalization

---

### 2.3 Normalization

```
CRLF (\r\n) → LF (\n)
```

Normalization is an **in-memory operation** applied solely for matching and
hashing. It **MUST NOT** affect the bytes written to the file.

Normalization **MUST** be applied in memory:

* After validation (before matching)
* Before hashing

Normalization applies identically to file content and anchor string, enabling
consistent comparison regardless of the line endings present in the file or
the anchor argument.

The `replacement` content is written to the file **as-is**, without
normalization. The agent is responsible for the byte content of the replacement.

No other transformations are allowed:

* ❌ Trimming
* ❌ Unicode normalization
* ❌ Whitespace changes

---

### 2.4 Equality Definition

Two byte sequences are equal **if and only if**:

1. Both are valid UTF-8
2. Both are normalized using the same rule
3. Their byte sequences are identical

No other notion of equality is permitted.

---

### 2.5 Matching & Identification

* Evaluate **every possible starting byte position** (increment by 1 byte)
* Perform exact byte comparison
* Regex, fuzzy matching, heuristics **MUST NOT** be used
* Empty anchors are invalid and treated as `NO_MATCH`

#### Outcomes

| Match Count | Result             |
| ----------- | ------------------ |
| 0           | `NO_MATCH`         |
| 1           | Success            |
| >1          | `MULTIPLE_MATCHES` |

---

### 2.6 Hashing

* Algorithm: `xxh3_64`
* Input: normalized matched byte sequence of the anchored scope
* Output: lowercase 16-character hex string
* Executed only if exactly one match exists, **before write**

---

## 3. Scope Hash (Normative)

The **scope hash** is computed from the matched byte sequence:

```
scope_hash = xxh3_64(normalized matched bytes)
```

This is the hash returned by `read` and used as `--expected-hash` in `write`.

The scope hash serves as the sole state passed between `read` and `write`.
The agent retains it; AnchorScope does not.

---

## 4. Execution Model (Normative)

### 4.1 Read Pipeline

```
READ → VALIDATE → NORMALIZE → MATCH → HASH
```

* Any stage failure **MUST terminate immediately**
* No stage may be skipped or reordered
* The source file is **not modified**

---

### 4.2 Write Phase

```
READ → VALIDATE → [NORMALIZE] → MATCH → HASH → VERIFY → WRITE → COMPLETE
```

* Re-reads and re-validates the source file from scratch
* Applies normalization **in memory** for matching and hash computation only
* Compares computed hash with `--expected-hash`
* If mismatch:

```
HASH_MISMATCH
```

* WRITE **MUST**:
  * Locate the matched byte range in the **original** (non-normalized) file
  * Replace only that byte range with the replacement content
  * Leave all bytes outside the matched range unchanged
  * Succeed or terminate with:

```
IO_ERROR: write failure
```

`[NORMALIZE]` denotes an in-memory step that does not modify the file.

> **Note:** When the source file contains CRLF sequences, the byte offsets
> obtained from matching against the normalized (LF-only) content **MUST** be
> mapped back to the corresponding offsets in the original file before writing.
> Implementations that ignore this mapping will write to incorrect byte ranges.

---

## 5. Implementation: AnchorScope CLI (Normative)

### 5.1 Commands

* `read` — match anchor, compute and return scope hash and matched content
* `write` — verify hash, apply replacement

---

### 5.2 Read Contract

The `read` command **MUST**:

1. Execute the full read pipeline through HASH
2. Return:
   * `scope_hash`: 16-character lowercase hex string
   * `content`: matched bytes as normalized UTF-8
3. **NOT** modify the source file

```bash
as read --file <path> --anchor "<string>"
# or
as read --file <path> --anchor-file <path>
```

Output (exit 0 on success):

```
scope_hash=<16-char hex>
content=<matched bytes as UTF-8>
```

---

### 5.3 Write Contract

The `write` command **MUST**:

1. Re-read and re-validate the source file
2. Match the anchor
3. Compute hash of matched scope
4. Compare with `--expected-hash`
5. Perform replacement **only if equal**
6. Otherwise return `HASH_MISMATCH`

```bash
as write \
  --file <path> \
  --anchor "<string>" \
  --expected-hash <scope_hash> \
  --replacement "<string>"
# or with anchor/replacement files
as write \
  --file <path> \
  --anchor-file <path> \
  --expected-hash <scope_hash> \
  --replacement-file <path>
```

---

### 5.4 Typical Workflow

```bash
# 1. Read: confirm match and obtain scope_hash
as read --file src/main.rs --anchor "fn calculate_area"

# Output:
# scope_hash=3a7f1c2d4e5b6f8a
# content=...

# 2. Agent constructs replacement content

# 3. Write: apply with hash verification
as write \
  --file src/main.rs \
  --anchor "fn calculate_area" \
  --expected-hash 3a7f1c2d4e5b6f8a \
  --replacement "fn calculate_area..."
```

The agent retains `scope_hash` between step 1 and step 3.
If the file changes between steps, `write` returns `HASH_MISMATCH` and
the agent re-runs `read` to obtain a fresh hash.

---

### 5.5 Deterministic Error Handling

Allowed outputs:

```
NO_MATCH
MULTIPLE_MATCHES
HASH_MISMATCH
IO_ERROR: file not found
IO_ERROR: permission denied
IO_ERROR: invalid UTF-8
IO_ERROR: read failure
IO_ERROR: write failure
```

---

## 6. Non-Goals

* Snapshot or version history (that is git's responsibility)
* Multi-file operations
* AST parsing or language awareness
* Regex or fuzzy matching
* Encoding detection or conversion
* Any modification outside the matched anchored scope
* Scope localization (that is the agent's or AnchorEdit's responsibility)
* External tool pipeline management
* Concurrent execution safety (AnchorScope is designed for single-process use)

---

## 7. Guarantees

1. Every edit targets exactly one uniquely identified anchored scope
2. No edit is applied if the file state has changed since `read`
3. All operations are deterministic and reproducible
4. Equality is strictly defined at the byte level
5. No state is held between operations; the agent retains `scope_hash`
6. The system is fail-fast by design
7. Zero modification occurs outside the matched anchored scope
8. Normalization is in-memory only; file bytes outside the matched scope are never modified

---

## 8. Summary

AnchorScope v2.0.0 defines a **minimal, stateless, deterministic editing protocol**:

* Two commands: `read` and `write`
* One state token: `scope_hash`, retained by the agent
* No buffers, no identity tracking, no pipelines
* Hash-verified safety on every write
* Fail-fast on ambiguity or state change

> **Correctness over convenience
> Stateless over stateful
> Hash as the sole source of truth**
