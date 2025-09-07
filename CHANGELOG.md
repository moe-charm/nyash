# Nyash Project – Changelog (Work in progress)

This changelog tracks high‑level milestones while Core MIR and Phase 12 evolve. For detailed per‑file history, see git log and docs under `docs/development/roadmap/`.

## 2025‑09‑06
- Core‑13 flip complete: code/tests enforce Core‑13 minimal kernel. Normalizations (Array/Ref→BoxCall, TypeCheck/Cast/Barrier/WeakRef unification) are ON by default via env (NYASH_MIR_CORE13=1). New tests validate normalization.
- Docs synced: step‑50 marked done; DEV quickstart points to Core‑13 reference.

## 2025‑09‑04
- Phase 12.7‑A complete: peek, continue, `?` operator, lambda, field type annotations. Language reference updated.
- Phase 12.7‑B (basic) complete: parser‑level desugaring for `|>`, `?.`, `??`, `+=/-=/*=/=`, `..` behind `NYASH_SYNTAX_SUGAR_LEVEL`.
- Docs: language reference and Phase 12.7 README updated to reflect basic completion; extensions tracked under gated plan.
- MIR Core migration: previously enforcing Core‑15 during transition; superseded by 2025‑09‑06 Core‑13 flip.

## 2025‑09‑03
- Nyash ABI TypeBox integration stabilized across core boxes; differential tests added; loader defaults adjusted (builtin + plugins).

---

Notes
- Core‑13 is canonical minimal kernel. Historical Core‑15 notes remain under `docs/development/roadmap/` for reference.
- Phase 12.7‑B desugaring is gated by `NYASH_SYNTAX_SUGAR_LEVEL`; tokenizer additions are non‑breaking.
