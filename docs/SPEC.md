# AnchorScope Specification v1.0.1

## Deterministic Scoped Editing Protocol

**AnchorScope is a deterministic code editing protocol based on Scope Anchoring.**
It treats code as **immutable UTF-8 byte sequences**, not as text or syntax.

All operations are strictly **byte-level, deterministic, and single-location**.

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

### 1.3 Layered Model

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

### 2.2 Encoding & Validation (CRITICAL)

All inputs **MUST** be valid UTF-8.

#### Validation Scope

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
* Validation **MUST occur before normalization**

---

### 2.3 Normalization

#### Rule

```
CRLF (\r\n) → LF (\n)
```

#### Application

Normalization **MUST** be applied:

* After validation (before matching)
* Before hashing
* Before writing

#### Symmetry

Normalization applies identically to:

* File content
* Anchor
* Replacement

#### Constraint

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

#### Search Procedure

* Evaluate **every possible starting byte position**
* Increment by exactly **1 byte**
* Perform exact byte comparison

#### Constraints

* Regex **MUST NOT** be used
* Fuzzy matching **MUST NOT** be used
* Heuristics **MUST NOT** be used

#### Implementation Note

Implementations **MAY** use optimized algorithms (e.g., KMP),
provided that the result is identical to a full byte-by-byte scan.

#### Overlapping Matches

Overlapping matches **MUST** be detected and counted.

#### Empty Anchor

An empty anchor is considered invalid.

For simplicity and determinism, it **MUST** be treated as:

```
NO_MATCH
```

This avoids introducing a separate error class and keeps matching semantics uniform.

#### Outcomes

| Match Count | Result             |
| ----------- | ------------------ |
| 0           | `NO_MATCH`         |
| 1           | Success            |
| >1          | `MULTIPLE_MATCHES` |

---

### 2.6 Hashing

#### Algorithm

```
xxh3_64
```

#### Input

* Normalized matched byte region

#### Output

* Lowercase 16-character hex string

#### Execution

* Performed **only if exactly one match exists**
* Performed **before any write**

---

### 2.7 Line Numbering

Line numbers **MUST** be:

* 1-based
* Based on normalized content (LF only)
* Inclusive range `[start, end]`

---

## 3. Execution Model (Normative)

### 3.1 Processing Pipeline

All operations **MUST** execute the following pipeline:

```
READ → VALIDATE → NORMALIZE → MATCH → HASH
```

At each stage:

* If the stage fails, the process **MUST terminate immediately** with a defined error.
* No stage may be skipped or reordered.

---

### 3.2 Write Phase

If and only if all prior stages succeed, the `write` command proceeds:

```
HASH_VERIFIED → WRITE → COMPLETE
```

* The current file state **MUST** be hashed and compared with `expected_hash`
* If the hash does not match:

```
HASH_MISMATCH
```

* The WRITE step **MUST**:

  * replace only the matched region
  * either succeed or terminate with:

```
IO_ERROR: write failure
```

---

## 4. Implementation: AnchorScope CLI (Normative)

### 4.1 Overview

The reference implementation exposes:

* `read`
* `write`

Both commands **MUST** follow the protocol strictly.

---

### 4.2 Read Contract

The `read` command **MUST**:

1. Execute the pipeline up to HASH
2. Return:

   * Line range (1-based, inclusive)
   * Hash
   * Matched content (normalized UTF-8)
3. NOT modify the file

The returned content **MUST exactly correspond** to the hashed byte region.

---

### 4.3 Write Contract

The `write` command **MUST**:

1. Compute the hash from current file state
2. Compare with `expected_hash`
3. Perform replacement **only if equal**
4. Otherwise return:

```
HASH_MISMATCH
```

---

### 4.4 Write Semantics

* Only the matched byte range is replaced
* Prefix and suffix **MUST remain unchanged**
* Output file **MUST be normalized (LF only)**

---

### 4.5 Typical Workflow (Informative)

A typical usage flow is:

1. Execute `read` to obtain:

   * line range
   * hash
   * content
2. Modify the content externally
3. Execute `write` with:

   * original anchor
   * replacement
   * `expected_hash` from step 1

The write operation will only succeed if the file state has not changed.

---

### 4.6 Deterministic Error Handling

Implementations **MUST NOT** expose raw OS errors.

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

The following are explicitly out of scope:

* Multi-file operations
* Automatic anchor generation
* AST parsing or language awareness
* Regex or fuzzy matching
* Encoding detection or conversion
* Any modification outside the matched region

---

## 6. Guarantees

This protocol guarantees that:

1. Every edit targets exactly one uniquely identified region.
2. No edit is applied if the file state has changed.
3. All operations are deterministic and reproducible.
4. Equality is strictly defined at the byte level.
5. No implicit interpretation is performed.
6. The system is fail-fast by design.
7. Zero modification occurs outside the matched region.
8. Normalization is consistent and persistent.
9. Invalid input states are always fatal, including:

   * invalid UTF-8
   * empty anchor
   * multiple matches

---

## 7. Summary

AnchorScope defines a **minimal, strict, and deterministic editing protocol**.

By eliminating ambiguity and interpretation, it enables:

* Reliable LLM-driven code modification
* Safe automated refactoring
* Reproducible editing pipelines

The protocol prioritizes:

> **Correctness over convenience
> Synchronization over intelligence
> Failure over ambiguity**