# Runtime Architecture — Unified VM Engine (Phase‑A)

Status: Adopted (entry unified). Default engine = Fallback (MIR Interpreter).

## Overview

Runner pipeline (common across backends):
- PreLex normalization (raw strings `r"…"`, numeric separators `_`, line‑head `@local` preexpand)
- Using resolver (strip + AST prelude merge; dev/ci ON, prod OFF)
- Macro normalization (child‑safe)
- MIR compile
- VM/LLVM dispatch

## VM Engines

Trait
```rust
pub trait VmEngine {
    fn execute(&mut self, module: &MirModule) -> Result<i32, String>;
}
```

Implementations
- FallbackVmEngine: wraps the lightweight MIR interpreter (no plugins required). Scope: const/binop/compare/ret を中心に安全最小。
- FullVmEngine: placeholder（段階導入）。Scope: plugin/BoxCall/namespace を含む実運用 VM。

Factory
```rust
// NYASH_VM_ENGINE={auto|fallback|full}; default=fallback
pub fn vm_engine_from_env() -> Box<dyn VmEngine> { … }
```

Entry (single place)
- Runner calls `execute_vm_engine(file)` which performs Using/PreLex/MIR compile then `VmEngine::execute`.

## Why unify the entry?
- Previous code scattered VM switches across helpers (fallback vs full). Debugging issues across Runner→VM→BoxCall→Alias was hard.
- A single trait + factory isolates the differences behind one call site. The rest of Runner becomes backend‑agnostic.

## Phase plan
- Phase‑A (this change): introduce `VmEngine` + default=fallback, keep behavior.
- Phase‑B: add minimal BoxCall set (String/Array/Map) to fallback to keep quick green, then introduce a working `FullVmEngine` behind `NYASH_VM_ENGINE=full`.
- Phase‑C: promote `full` to default after smoke stability; keep fallback as debugging aid.

## Env knobs
- `NYASH_VM_ENGINE={fallback|full}`: select engine (default=fallback)
- `NYASH_USING=0|1` (default=1): enable/disable using system
- `NYASH_USING_STRATEGY={resolver|prelude}` (alias: `NYASH_USING_IMPL`) (default=resolver)
  - resolver: name resolution only（AST 統合なし）
  - prelude: AST prelude merge（dev/ci で既定ON、prod では明示）
- `NYASH_PLUGIN_POLICY={auto|off|force}` (default=auto)
  - auto: 現行のプラグイン自動ポリシー（プロファイル依存）
  - off: プラグイン読み込みを抑止（`NYASH_DISABLE_PLUGINS=1` 相当）
  - force: プラグイン経路を強制（`NYASH_PLUGIN_ONLY=1` 相当）
- `NYASH_SYNTAX_SUGAR_LEVEL={off|basic|full}`: parser sugar; PreLex runs for ON

Compatibility mapping
- `NYASH_ENABLE_USING` → `NYASH_USING`
- `NYASH_USING_AST` → `NYASH_USING_STRATEGY=prelude`
- `NYASH_DISABLE_PLUGINS`/`NYASH_PLUGIN_ONLY` → `NYASH_PLUGIN_POLICY=off|force`

Defaults (unset)
- NYASH_USING=1
- NYASH_USING_STRATEGY=resolver（dev/ci では prelude が既定で有効）
- NYASH_PLUGIN_POLICY=auto
- NYASH_DEV_FALLBACK=0（dev/quick ではプロファイルでON）

## Dev-only safety: unqualified helper normalization

In development profile (`--dev` or `NYASH_DEV=1`), the MIR builder applies a tiny, behavior-preserving fallback to improve robustness while working with desugaring and prelude merges:

- Context: inside a static box function, an internal helper may sometimes appear as an unqualified call (e.g., `_scan(x)` instead of `me._scan(x)`), due to experimental desugaring.
- Guard: when `NYASH_DEV=1`, the builder rewrites an unqualified, underscore-prefixed call name (e.g., `_scan`) to the current static box function form `Class._scan/arity`.
- Scope: DEV only. No change for CI/PROD. This keeps behavior stable without adding runtime shims in Ny code.

Rationale
- Keeps the codebase clean (no temporary top-level helpers) and localizes the guard to the compiler boundary.
- Makes selfhost flows resilient during bring-up while preserving the public specification.

## LocalSSA — Call-site materialization

LocalSSA is applied at call-sites to ensure receiver and arguments are materialized in the current block before emission.
- Scope: Method calls (receiver + args), and uniformly finalized via emit-guard/materialize helpers.
- Order within a block: PHI (at head) → Copy(materialize) → Call, enforced by BlockSchedule policies.
- Goal: eliminate cross-block transient values and prevent undefined uses across control-flow boundaries.

## Resolve DFS duplicate guard

The Using resolver DFS now avoids pushing the same real path into the prelude list more than once, even if canonicalization differs across entry points. This reduces redundant prelude processing and keeps logs tidier without changing semantics.
