Layer Guard — json_native

Scope and responsibility
- This layer implements a minimal native JSON library in Ny.
- Responsibilities: scanning, tokenizing, and parsing JSON; building node structures.
- Forbidden: runtime/VM specifics, code generation, non‑JSON language concerns.

Imports policy (SSOT)
- Dev/CI: file-using allowed for development convenience.
- Prod: use only `nyash.toml` using entries (no ad‑hoc file imports).

Notes
- Error messages aim to include: “Error at line X, column Y: …”.
- Unterminated string → tokenizer emits "Unterminated string literal" (locked by quick smoke).
