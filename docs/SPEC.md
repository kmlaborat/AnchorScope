# AnchorScope Spec v1.0

**AnchorScope is a deterministic code editing protocol based on Scope Anchoring.**

All operations are strictly **byte-level, deterministic, and single-location**.

---

## 1. Layers

| Layer   | Name            | Role                              |
| ------- | --------------- | --------------------------------- |
| Concept | Scope Anchoring | Algorithm for identifying regions |
| Core    | AnchorScope     | Protocol enforcing correctness    |
| Tool    | AnchorEdit      | Writer                            |
| Tool    | CLI             | Reader                            |

---

## 2. Core Principles

1. **Never guess. Fail instead.**
2. **Synchronization over intelligence:** The tool enforces correctness; the agent adapts.
3. **Single Source of Truth:** All operations work on normalized byte sequences.
4. **Determinism:** Same input h same output.

---

## 3. Normalization

**Rule:** CRLF (`\r\n`) h LF (`\n`)

**Application:**

* Immediately after reading file content
* Before matching
* Before hashing
* Before writing replacement

**Symmetry:** Normalization applied to:

* File content
* Anchor
* Replacement

**Constraint:**

* No other transformations (trimming, Unicode normalization) are allowed.

---

## 4. Encoding (CRITICAL)

AnchorScope operates on **UTF-8 encoded byte sequences**.

### 4.1 Validation Scope

The following inputs **MUST be validated by the implementation**:

* File content
* Anchor file content (`--anchor-file`)

If any of the above inputs are not valid UTF-8:

```text
IO_ERROR: invalid UTF-8
```

MUST be returned.

* No partial decoding or lossy conversion is allowed.
* Validation MUST occur immediately after file read and before normalization.

---

### 4.2 Inline CLI Arguments

Inline CLI arguments:

```bash
--anchor "<string>"
--replacement "<string>"
```

* Are assumed to be valid UTF-8 due to command-line argument constraints
* Are **outside the validation scope of AnchorScope**
* Do not affect the deterministic behavior of the protocol

---

### 4.3 Rationale

Deterministic behavior requires a single, unambiguous byte representation within the scope of the protocol.

---

## 5. Matching

### 5.1 Search Procedure

1. Convert both haystack (file) and anchor to normalized byte sequences.
2. Evaluate **all starting byte positions**, incrementing by exactly **1 byte** per step.
3. Compare **raw bytes only**.
4. Overlapping matches must be detected and included in the count.

---

### 5.2 Outcomes

| Matches | Result             |
| ------- | ------------------ |
| 0       | `NO_MATCH`         |
| 1       | Success            |
| >1      | `MULTIPLE_MATCHES` |

---

### 5.3 Constraints

* Only contiguous byte sequence matches are valid.
* The **entire anchor byte sequence MUST match exactly**.
* Matching a fragment of the anchor (prefix/suffix/partial) is forbidden.
* Matching is purely byte-based:

  * If the full anchor byte sequence appears within a larger sequence, it is considered a valid match.
* No regex, heuristic, or fuzzy matching allowed.

---

### 5.4 Matching Algorithm Constraint

The matching result MUST be identical to a **full byte-by-byte scan** that evaluates every possible starting position.

Implementations MAY use optimized algorithms (e.g., Boyer-Moore, KMP), provided that:

* All valid matches are detected
* Overlapping matches are preserved
* No matches are skipped

---

### 5.5 Match Evaluation Order

1. Count matches.
2. If exactly 1 match h compute hash.
3. If hash matches expected h apply replacement.
4. Otherwise h `HASH_MISMATCH`.

---

## 6. Hashing

* Algorithm: `xxh3_64`
* Input: normalized byte region of exact match
* Output: lowercase 16-character hex string
* Deterministic: identical bytes h identical hash
* Computed **after matching only**
* Hash comparison occurs **only if exactly one match exists**

---

## 7. Write Semantics

* Only the matched byte range is replaced.
* Prefix and suffix bytes remain unchanged.
* Replacement bytes are normalized before writing.
* File is written in normalized form (LF only).
* No backup or atomic replace required.

---

### 7.1 Failure Conditions

| Condition        | Result             |
| ---------------- | ------------------ |
| No match         | `NO_MATCH`         |
| Multiple matches | `MULTIPLE_MATCHES` |
| Hash mismatch    | `HASH_MISMATCH`    |
| IO error         | `IO_ERROR: <type>` |

---

### 7.2 Deterministic IO Errors

IO errors MUST be **deterministic and implementation-defined**.

Implementations MUST NOT expose raw OS error messages.

Allowed IO error outputs:

```text
IO_ERROR: file not found
IO_ERROR: permission denied
IO_ERROR: invalid UTF-8
IO_ERROR: read failure
IO_ERROR: write failure
```

---

## 8. Forbidden Operations

```text
- Fuzzy matching
- Partial matching of the anchor (matching only a fragment of the anchor itself)
- Regex matching
- Whitespace trimming for matching
- Skipping overlapping matches
- Asymmetric normalization
- Implicit correction or guessing
- Encoding detection or conversion
- Any operation that modifies file outside matched region
```

---

## 9. Diagnostics & Reporting

* Only **metadata** allowed (position, byte-level diff, optional similarity metrics)
* **Diagnostics MUST NOT influence matching, hash verification, or writing**

---

## 10. CLI Interface

### 10.1 Read

```bash
anchorscope read --file <path> --anchor "<string>"
```

---

### 10.2 Write

```bash
anchorscope write \
  --file <path> \
  --anchor "<string>" \
  --expected-hash <hex> \
  --replacement "<string>"
```

---

### 10.3 Anchor Input Modes

Anchors can be provided in two ways:

#### 1. Inline (default)

```bash
--anchor "<string>"
```

* Requires proper shell escaping for newlines and special characters.

#### 2. File-based (RECOMMENDED)

```bash
--anchor-file <path>
```

* File content is used as the anchor byte sequence.
* No escaping required.
* Recommended for multi-line anchors and agent usage.

---

### 10.4 Constraints

* Anchor MUST be a non-empty byte sequence.

An empty anchor is considered **invalid as a meaningful query** and cannot produce a valid match.

However, for deterministic execution and consistent state flow, implementations:

* MUST treat an empty anchor as producing **zero matches**
* MUST return:

```text
NO_MATCH
```

* MUST NOT introduce a separate error condition for empty anchors

---

## 11. State Model

```text
READ h MATCH h HASH h WRITE
```

* Retry or orchestration is external responsibility
* All operations are atomic within the matched byte region

---

## 12. Non-Goals

```text
- Multi-file operations
- Automatic anchor generation
- AST parsing
- Heuristic or fuzzy matching
- Orchestrator logic
- Advanced recovery
- Full-file snapshot or replacement operations
```

---

## 13. Summary Guarantees

1. Deterministic single-location matching
2. Exact byte equality (normalized, UTF-8 within validated scope)
3. Overlap detection included
4. Only matched region is replaced
5. Normalization is persistent and symmetric
6. Encoding is fixed and validated within protocol scope
7. IO errors are deterministic and environment-independent
8. Diagnostics are metadata only
9. Forbidden operations strictly prohibited

---