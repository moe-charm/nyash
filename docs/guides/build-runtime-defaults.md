# Hakorune Default Build & Runtime Policy (Phase 15.7)

Status: adopted; Scope: local dev/selfhost/quick smokes

## Build Defaults
- Embed minimal Core Kernel always（GC/Handle/Extern Registry/Plugin Loader/traits）
- VM backend enabled（default）。LLVM harness/AOT はツール経由（`tools/build_llvm.sh`）
- Plugin loader is included, but no plugins are enabled by default

## Runtime Defaults
- Plugins: OFF（`HAKO_PLUGIN_POLICY=off`）
- Backend: VM（enable LLVM harness per‑run with `NYASH_LLVM_USE_HARNESS=1`）
- Using: OFF by default in CLI。selfhost/smokes でのみ ON（profile 指定）
- MapBox.get(missing): returns null（default; no ENV needed）

## Override Order
- User Box（Nyash） > Plugin Box（.so/.dll） > Kernel fallback（最小）

## Recommended Profiles
- dev‑fast（selfhost 用）
  - Plugins OFF, VM backend, Using ON（profile `dev`）
- plugin‑on（integration 用）
  - `HAKO_PLUGIN_POLICY=auto`、必要時に `NYASH_VM_PLUGIN_PREFER_*=1`
- release‑aot（配布/性能）
  - LLVM harness/AOT 経由で EXE 生成、静的リンク優先（nyrt + 静的プラグイン）

## Rationale
- 再現性と安定性を最優先（環境依存の差を除去）
- 段階導入（Loader は常同梱→必要時のみオプトイン）
- Selfhost 達成前に土台を固定（Fail‑Fast/最小主義）

## References
- Kernel/Plugin baseline: `docs/guides/kernel-plugin-baseline.md`
- Phase 15.7 plan: `docs/development/roadmap/phases/phase-15.7/README.md`
- ENV guide: `docs/config/env.md`
