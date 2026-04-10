# AnchorScope Specification v1.2.0

## Deterministic Scoped Editing Protocol

**AnchorScope is a deterministic code editing protocol based on Scope Anchoring.**
It treats code as **immutable UTF-8 byte sequences**, not as text or syntax.

All operations are strictly **byte-level, deterministic, and single-location**.

AnchorScope v1.2.0 extends v1.1.0 by introducing:

* **True ID**: hash-based unique identity for every anchor
* **Alias**: optional human-readable name via `label` command
* **Anchor Buffer**: a structured temporary directory enabling multi-level anchoring
* **Determinism Enforcement**: fail-fast behavior when duplicate True IDs are detected
* **Physical Constraints**: maximum nesting depth of 5 levels (configurable via `ANCHORSCOPE_MAX_DEPTH` environment variable)

---

## 1. Concept: Scoped Editing (Informative)

### 1.1 Problem: Fragility of Global Edits

Full-file rewrites are high-risk, and diff-based patching is fragile.
Even minor contextual changes can invalidate patches.

### 1.2 Solution: Anchor and Scope

AnchorScope defines a precise **editing scope** using an exact byte sequence ("Anchor"),
combined with **hash-based state verification**.

This enables edits that are:

* Safe (fail-fast)
* Precise (single-location)
* Idempotent (state-verified)

### 1.3 Multi-Level Anchoring

Uniqueness of an anchor is required within its source scope.
When a target region is large, uniqueness is harder to achieve.

Multi-level anchoring solves this:

1. Set a broad outer anchor (easy to make unique in the full file)
2. Set an inner anchor within the outer anchor's copy (easy to make unique in the smaller scope)
3. Edit the innermost target

Each level operates on a **buffer copy** of the parent's matched region,
not on the original file. This prevents a write at any level from invalidating
anchors at other levels.

### 1.4 Layered Model

| Layer              | Name            | Role                                       |
| :----------------- | :-------------- | :----------------------------------------- |
| **Concept**        | Scoped Editing  | Philosophy of local, verifiable mutation   |
| **Protocol**       | Scope Anchoring | Deterministic matching & hashing rules     |
| **Implementation** | AnchorScope     | Reference CLI (`read` / `write` / `label` / `tree`) |

---

## 2. Protocol: Scope Anchoring (Normative)

### 2.1 Invariants

The following invariants **MUST** hold:

1. Matching **MUST** be exact byte equality after normalization.
2. Matching **MUST** evaluate all possible byte offsets.
3. Exactly one match **MUST** exist to proceed.
4. All operations **MUST** be deterministic.
5. No implicit interpretation (syntax, encoding heuristics) is allowed.
6. Multi-level anchors **MUST NOT** operate directly on the original file after the first level.
   Subsequent levels operate on **buffer copies** only.

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

Normalization **MUST** be applied:

* After validation (before matching)
* Before hashing
* Before writing

Normalization applies identically to file content, anchor, and replacement.

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
* Input: normalized matched byte region
* Output: lowercase 16-character hex string
* Executed only if exactly one match exists, **before write**

---

### 2.7 Line Numbering

* 1-based
* Based on normalized content (LF only)
* Inclusive range `[start, end]`

---

## 3. Anchor Identity (Normative)

### 3.1 Region Hash

The **region hash** is computed from the matched byte region:

```
region_hash = xxh3_64(normalized matched bytes)
```

This is the hash returned by `read` and used as `expected_hash` in `write`.

---

### 3.2 True ID

The **True ID** uniquely identifies an anchor within its parent scope.

```
true_id = xxh3_64(parent_region_hash + "_" + child_region_hash)
```

For the first level (anchored directly into the original file):

```
true_id = xxh3_64(file_hash + "_" + region_hash)
```

where `file_hash = xxh3_64(normalized full file bytes)`.

Properties:

* Always 16 lowercase hex characters
* Encodes both parent context and matched region
* Two anchors with identical content but different parents have different True IDs
* Determined solely by hash values; no file path or anchor string is included

---

### 3.3 Alias

An **alias** is an optional human-readable name assigned to a True ID via the `label` command.

* Multiple aliases may point to the same True ID
* Aliases do not replace True IDs; they coexist
* An alias is a convenience reference only; all protocol operations use True IDs

---

## 4. Anchor Buffer (Normative)

### 4.1 Purpose

The **Anchor Buffer** is a structured temporary directory that stores:

* A copy of the original file (root)
* Copies of each matched region (one per `read`)

Buffer copies serve as the source for multi-level anchoring.
They are **not** a snapshot or version history. They exist solely to enable
recursive editing without modifying the original file.

---

### 4.2 Directory Structure

```
{TMPDIR}/anchorscope/
└── {file_hash}/
    ├── content          ← normalized copy of the original file
    ├── source_path      ← absolute path to the original file (plain text)
    └── {true_id}/
        ├── content      ← normalized copy of the matched region
        └── {true_id}/
            ├── content
            └── {true_id}/
                └── content

{TMPDIR}/anchorscope/labels/
└── {alias}.json         ← alias → true_id mapping
```

* `{TMPDIR}` is the OS temporary directory (`$TMPDIR` on Unix/macOS, `%TEMP%` on Windows)
* `file_hash` identifies the root (original file)
* `true_id` identifies each anchor level
* `source_path` is stored **only at the root level**
* `content` files contain normalized UTF-8 text

---

### 4.3 Lifecycle

| Event | Effect on Buffer |
| :---- | :--------------- |
| `read` on original file | Creates `{file_hash}/content`, `{file_hash}/source_path`, `{file_hash}/{true_id}/content` |
| `read` on buffer copy | Creates `{file_hash}/{true_id}/{true_id}/content` (nested) |
| `write` success | Deletes the written anchor's True ID directory and all its descendants |
| `write` failure | Buffer is retained for retry or inspection |
| Process exit / error | Buffer is retained (OS temp cleanup handles eventual removal) |

**Note:** The `write` command deletes the buffer directory corresponding to the True ID used for the write operation, which may be different from the file_hash of the original file. The buffer hierarchy is cleaned up recursively to ensure all related artefacts are removed.

---

### 4.4 Labels File

```json
{ "true_id": "a1b2c3d4e5f6a7b8" }
```

* Stored at `{TMPDIR}/anchorscope/labels/{alias}.json`
* Deleted when the referenced True ID's directory is deleted

---

## 5. Execution Model (Normative)

### 5.1 Processing Pipeline

```
READ → VALIDATE → NORMALIZE → MATCH → HASH → BUFFER_WRITE
```

* Any stage failure **MUST terminate immediately**
* No stage may be skipped or reordered

---

### 5.2 Write Phase

```
HASH_VERIFIED → WRITE → BUFFER_INVALIDATE → COMPLETE
```

* Compare current content hash with `expected_hash`
* If mismatch:

```
HASH_MISMATCH
```

* WRITE **MUST**:
  * Replace only the matched region
  * Succeed or terminate with:

```
IO_ERROR: write failure
```

* On success, delete the anchor's buffer directory and all descendants

---

## 6. Implementation: AnchorScope CLI (Normative)

### 6.1 Commands

* `read` – match anchor, compute hash and True ID, write buffer copy
* `write` – verify hash, apply replacement, invalidate buffer
* `label` – assign alias to a True ID
* `tree` – display current buffer structure

---

### 6.2 Read Contract

The `read` command **MUST**:

1. Execute the full pipeline through BUFFER_WRITE
2. Return:
   * Line range (1-based, inclusive)
   * Region hash
   * True ID
   * Matched content (normalized UTF-8)
3. **NOT** modify the source file or any parent buffer

Target of `read` is either:

* The original file (level 1)
* A buffer `content` file referenced by True ID or alias (level 2+)

---

### 6.3 Write Contract

The `write` command **MUST**:

1. Compute hash from current content of the target (file or buffer)
2. Compare with `expected_hash`
3. Perform replacement **only if equal**
4. On success, delete the anchor's buffer directory and all descendants
5. Otherwise return `HASH_MISMATCH`

---

### 6.4 Label Contract

The `label` command **MUST**:

* Accept a True ID and a human-readable alias
* Create `labels/{alias}.json` mapping alias to True ID
* Verify the True ID exists in the buffer before creating the alias
* Allow multiple aliases per True ID
* Reject alias reuse pointing to a different True ID:

```
LABEL_EXISTS
```

---

### 6.5 Tree Contract

The `tree` command **MUST**:

* Display the current buffer structure rooted at `{file_hash}`
* Show True IDs, aliases (if any), and nesting depth
* Reflect the actual state of the buffer directory

Example output:

```
{file_hash}  (/path/to/original.rs)
└── {true_id}  [my_function]
    └── {true_id}
        └── {true_id}  [inner_loop]
```

---

### 6.6 Nesting Depth Limitation

To ensure cross-platform compatibility and prevent platform-specific issues (particularly Windows MAX_PATH limits), the maximum nesting depth is **5 levels** by default.

**Environment Variable Override:**

The limit can be overridden via the `ANCHORSCOPE_MAX_DEPTH` environment variable:

* If unset or invalid: defaults to 5
* If set: uses the specified value (clamped to range [1, 100])

**Warning:**

Using a depth limit greater than 5 levels may cause portability issues:
* **Windows:** May exceed MAX_PATH limits (260 characters)
* **Linux/macOS:** May exceed filesystem path limits
* **Docker/containers:** May have different path resolution
* **Cross-platform scripts:** May behave differently on different systems

Users who override the default limit assume full responsibility for ensuring determinism and compatibility across target environments.

---

### 6.7 Deterministic Error Handling

Allowed outputs:

```
NO_MATCH
MULTIPLE_MATCHES
HASH_MISMATCH
LABEL_EXISTS
AMBIGUOUS_ANCHOR
IO_ERROR: file not found
IO_ERROR: permission denied
IO_ERROR: invalid UTF-8
IO_ERROR: read failure
IO_ERROR: write failure
```

---

### 6.7 Determinism Guarantees

To maintain determinism and prevent ambiguous anchor resolution, AnchorScope enforces:

1. **Duplicate True ID Detection**: If the same True ID exists in multiple buffer locations (e.g., due to incomplete cleanup from previous operations), the system **MUST** fail fast with `AMBIGUOUS_ANCHOR` error. This prevents non-deterministic behavior where operations might operate on the wrong buffer.

2. **Maximum Nesting Depth**: To ensure portability across platforms (especially Windows with MAX_PATH limits), the maximum nesting depth for anchors is **5 levels**. Attempting to create a 6th level **MUST** be rejected with `IO_ERROR: maximum nesting depth exceeded`. This constraint ensures the same scripts work identically on all platforms.

3. **Parent Region Hash**: True IDs **MUST** be computed using `parent_region_hash` (the hash of the matched region at the parent level), not `parent_true_id` or file paths. This ensures True IDs are truly derived from content state, not implementation artifacts.

---

## 7. Non-Goals

* Snapshot or version history (that is git's responsibility)
* Multi-file operations
* AST parsing or language awareness
* Regex or fuzzy matching
* Encoding detection or conversion
* Any modification outside the matched region
* Automatic propagation of writes across anchor levels (reserved for v1.3.0 pipelines)

---

## 8. Guarantees

1. Every edit targets exactly one uniquely identified region
2. No edit is applied if the content state has changed
3. All operations are deterministic and reproducible
4. Equality is strictly defined at the byte level
5. True IDs are derived solely from hash values; no path or string metadata is included
6. Buffer copies isolate levels; a write at any level does not invalidate unrelated anchors
7. The system is fail-fast by design
8. Zero modification occurs outside the matched region
9. Normalization is consistent and persistent
10. Duplicate True IDs trigger immediate failure with `AMBIGUOUS_ANCHOR` error
11. Maximum nesting depth is 5 levels for cross-platform portability

---

## 9. Summary

AnchorScope v1.2.0 defines **atomic, deterministic, multi-level file editing** with:

* Hash-verified consistency at every level
* True IDs derived from `xxh3_64(parent_region_hash + "_" + child_region_hash)`
* Optional human-readable aliases via `label`
* A structured Anchor Buffer enabling recursive editing without touching the original file
* `tree` command for buffer visualization
* No snapshot, no mutable state, no version history

> **Correctness over convenience
> Determinism over mutability
> Hash as the sole source of truth**
