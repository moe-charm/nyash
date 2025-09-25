# Strings Blueprint — UTF‑8 First, Bytes Separate

Status: active (Feature‑Pause compatible)
Updated: 2025-09-21

Purpose
- Unify string semantics by delegating StringBox public APIs to dedicated cursor boxes.
- Keep behavior stable while making codepoint vs byte decisions explicit and testable.

Pillars
- Utf8CursorBox (codepoint-oriented)
  - length/indexOf/substring operate on UTF‑8 codepoints.
  - Intended as the default delegate for StringBox public APIs.
- ByteCursorBox (byte-oriented)
  - length/indexOf/substring operate on raw bytes.
  - Use explicitly for byte-level parsing or binary protocols.

Delegation Strategy
- StringBox delegates to Utf8CursorBox for core methods: length/indexOf/substring.
- Provide conversion helpers: toUtf8Cursor(), toByteCursor() (thin wrappers).
- Prefer delegation over inheritance; keep “from” minimal to avoid API ambiguity.

API Semantics
- indexOf: define two flavors via the box boundary.
  - StringBox.indexOf → Utf8CursorBox.indexOf (CP-based; canonical)
  - ByteCursorBox.indexOf → byte-based; opt‑in only
- substring: follow the same split (CP vs Byte). Do not mix semantics.
  - Document preconditions for indices (out‑of‑range clamped/errored per guide).

Implementation Plan (staged, non‑breaking)
1) Provide MVP cursor boxes (done)
   - apps/libs/utf8_cursor.nyash
   - apps/libs/byte_cursor.nyash
2) Delegate StringBox public methods to Utf8CursorBox (internal only; behavior unchanged)
   - Start with length → indexOf → substring
   - Add targeted smokes for edge cases (multi‑byte CP, boundaries)
3) Replace ad‑hoc scans in Nyash scripts with cursor usage (Mini‑VM/macros)
   - Migrate internal scanners (no external behavior change)
4) Introduce ByteCursorBox only where byte‑level semantics are required
   - Keep call sites explicit to avoid ambiguity

Transition Gate (Rust dev only)
- Env `NYASH_STR_CP=1` enables CP semantics for legacy byte-based paths in Rust runtime (e.g., StringBox.length/indexOf/lastIndexOf).
- Default remains byte in Rust during the feature‑pause; PyVM follows CP semantics. CI smokes validate CP behavior via PyVM.

Related Docs
- reference/language/strings.md — policy & scope
- guides/language-core-and-sugar.md — core minimal + sugar
- reference/language/EBNF.md — operators (! adopted; do‑while not adopted)
- guides/loopform.md — loop normalization policy

Box Foundations (string-related)
- Utf8CursorBox, ByteCursorBox
- StringExtBox (trim/startsWith/endsWith/replace/split)
- StringBuilderBox (append/toString)
- JsonCursorBox (lightweight JSON scanning helpers)

Testing Notes
- Keep PyVM as the reference execution path.
- Add smokes: CP boundaries, mixed ASCII/non‑ASCII, indexOf not found, substring slices.
- Avoid perf work; focus on semantics + observability.
