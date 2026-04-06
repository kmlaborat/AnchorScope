# AnchorScope Specification v1.1.0 (Updated)

## Deterministic Scoped Editing Protocol

**AnchorScope is a deterministic code editing protocol based on Scope Anchoring.**
It treats code as **immutable UTF-8 byte sequences**, not as text or syntax.

All operations are strictly **byte-level, deterministic, and single-location**.

AnchorScope v1.1.0 focuses on **atomic operations** (`read` / `write` / `label`) and sets the foundation for future snapshot and pipeline management.

---

## 1. Concept: Scoped Editing (Informative)

### 1.1 Problem: Fragility of Global Edits

Full-file rewrites are high-risk, and diff-based patching is fragile.
Even minor contextual changes can invalidate patches.

### 1.2 Solution: Automatic Label Anchors

AnchorScope defines a precise **editing scope** using an exact byte sequence ("anchor"), combined with **hash-based state verification**.

* Every `read` operation automatically generates an internal **label (anchor ID)** for the matched region.
* This internal label allows **subsequent atomic edits** without ambiguity.
* Users can optionally assign a **human-readable name** to this internal label using the `label` command.

This enables edits that are:

* Safe (fail-fast)
* Precise (single-location)
* Idempotent (state-verified)

### 1.3 Layered Model

| Layer              | Name            | Role                                       |
| :----------------- | :-------------- | :----------------------------------------- |
| **Concept**        | Scoped Editing  | Philosophy of local, verifiable mutation   |
| **Protocol**       | Scope Anchoring | Deterministic matching & hashing rules     |
| **Implementation** | AnchorScope     | Reference CLI (`read` / `write` / `label`) |

---

## 2. Protocol: Scope Anchoring (Normative)

### 2.1 Invariants

The following invariants **MUST** hold:

1. Matching **MUST** be exact byte equality after normalization.
2. Matching **MUST** evaluate all possible byte offsets.
3. Exactly one match **MUST** exist to proceed.
4. All operations **MUST** be deterministic.
5. No implicit interpretation (syntax, encoding heuristics) is allowed.
6. Parent-child or multi-layer anchors **MUST NOT** be relied upon for atomic edits.

   * Instead, perform edits recursively on **replacement copies in memory**.

---

### 2.2 Encoding & Validation

All inputs **MUST** be valid UTF-8.

* File content **MUST** be validated immediately after reading.
* `--label-file` content **MUST** be validated.
* Inline arguments (`--label`, `--replacement`) are assumed valid.

#### Error

If invalid UTF-8 is detected:

```
IO_ERROR: invalid UTF-8
```

---

### 2.3 Normalization

```
CRLF (\r\n) → LF (\n)
```

Normalization **MUST** be applied:

* After validation (before matching)
* Before hashing
* Before writing

No other transformations are allowed (trimming, Unicode normalization, whitespace changes).

---

### 2.4 Equality Definition

Two byte sequences are equal **if and only if**:

1. Both are valid UTF-8
2. Both are normalized
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

## 3. Execution Model (Normative)

### 3.1 Processing Pipeline

```
READ → VALIDATE → NORMALIZE → MATCH → HASH → AUTO-LABEL
```

* Any stage failure **MUST terminate immediately**.
* No stage may be skipped or reordered.

- Every `read` automatically generates an internal **label (anchor ID)** for later reference.
- Users may optionally assign a **human-readable label** using the `label` command.

### 3.2 Write Phase

```
HASH_VERIFIED → WRITE → COMPLETE
```

* Compare current file hash with `expected_hash`.
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

---

### 3.3 In-Memory Replacement (New v1.1.0)

* `read` can create **replacement copies in memory**.
* Recursive edits are allowed **only on in-memory copies**, not directly on the original file.
* Final `write` applies all in-memory changes **atomically** to the file.
* After successful write, related in-memory copies **MUST** be invalidated.
* Temporary in-memory copies may be cached for debugging or retry, but **do not alter file state** until atomic write.

---

## 4. Implementation: AnchorScope CLI (Normative)

### 4.1 Commands

* `read` – extract anchor content and auto-generate internal label
* `write` – apply replacement atomically
* `label` – assign a human-readable name to an internal label

---

### 4.2 Read Contract

* Execute pipeline to HASH and AUTO-LABEL

* Return:

  * Line range
  * Hash
  * Matched content (normalized UTF-8)
  * Auto-generated internal label

* **Does not modify file**

* Supports creation of **in-memory replacement copies** for recursive edits

---

### 4.3 Label Command Contract

* Map an internal label to a user-defined, human-readable name
* Optional: multiple labels may point to the same internal anchor
* Enables subsequent `write` or `read` using human-readable label

---

### 4.4 Write Contract

* Verify hash against `expected_hash`
* Replace matched region **only if verified**
* On success, invalidate all related in-memory copies

---

### 4.5 Deterministic Error Handling

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

## 5. Non-Goals

* Multi-file operations
* AST parsing or language awareness
* Regex or fuzzy matching
* Encoding detection or conversion
* Any modification outside matched region
* Multi-layer parent/child anchors for a single file

> **Automatic anchor generation** is now part of `read` (auto-label), so `anchor` command is replaced by `label` for explicit naming.

---

## 6. Guarantees

* Every edit targets exactly one uniquely identified region
* No edit applied if file state changed
* All operations deterministic and reproducible
* Strict byte-level equality
* Fail-fast design
* Zero modification outside matched region
* Normalization consistent
* Recursive in-memory edits allowed, but only **one atomic write per file**
* `read` auto-generates internal label; `label` command is optional for human-readable naming

---

## 7. Summary

AnchorScope v1.1.0 defines **atomic, deterministic file editing** with:

* Safe and precise byte-level edits
* Hash-verified consistency
* Auto-labeled anchors upon `read`
* Optional human-readable labels via `label` command
* Support for in-memory recursive preparation of replacements
* Foundation for snapshots and pipelines in future versions

> **Correctness over convenience
> Synchronization over intelligence
> Failure over ambiguity**
