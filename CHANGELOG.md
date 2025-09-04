# Nyash Project – Changelog (Work in progress)

This changelog tracks high‑level milestones while Core MIR and Phase 12 evolve. For detailed per‑file history, see git log and docs under `docs/development/roadmap/`.

## 2025‑09‑04
- Phase 12.7‑A complete: peek, continue, `?` operator, lambda, field type annotations. Language reference updated.
- Phase 12.7‑B (basic) complete: parser‑level desugaring for `|>`, `?.`, `??`, `+=/-=/*=/=`, `..` behind `NYASH_SYNTAX_SUGAR_LEVEL`.
- Docs: language reference and Phase 12.7 README updated to reflect basic completion; extensions tracked under gated plan.
- MIR Core migration: enforcing Core‑15 in code/tests during transition; Core‑13 target defined in docs; final flip planning in progress.

## 2025‑09‑03
- Nyash ABI TypeBox integration stabilized across core boxes; differential tests added; loader defaults adjusted (builtin + plugins).

---

Notes
- “Core‑15 vs Core‑13” migration: Implementation currently enforces 15 for stability; docs include Core‑13 target reference. Final flip (docs/refs/entrypoints) is tracked under `docs/development/roadmap/mir/core-13/step-50/`.
- Phase 12.7‑B desugaring is gated by `NYASH_SYNTAX_SUGAR_LEVEL`; tokenizer additions are non‑breaking.
