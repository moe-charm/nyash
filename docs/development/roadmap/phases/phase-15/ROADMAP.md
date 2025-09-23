# Phase 15 — Box Stacking Roadmap (Living)

This roadmap is a living checklist to advance Phase 15 with small, safe boxes. Update continuously as we progress.

## Now (ready/green)

- [x] v0 Ny parser (Ny→JSON IR v0) with wrappers (Unix/Windows)
- [x] Runner JSON v0 bridge (`--ny-parser-pipe`) → MIR → MIR-Interp
- [x] E2E + roundtrip practical recipes (Windows/Unix)
- [x] Docs path unify (phase-15 under roadmap tree)
- [x] Direct bridge (design + skeleton; feature-gated)
- [x] AOT P2 stubs (CraneliftAotBox/LinkerBox) + RUN smoke wiring
- [x] JIT‑only baseline stabilized (core smokes green; plugins optional)
- [x] Roundtrip (Case A/B) aligned; Case A re‑enabled via parser pipe
- [x] using/namespace (gated) + nyash.link minimal resolver
- [x] NyModules + ny_plugins regression suite (Windows path normalization/namespace derivation)
- [x] Standard Ny scripts scaffolds added (string/array/map P0) + examples + jit_smoke
- [x] Selfhost Parser accepts positional input file arg（EXE運用の前提）

## Next (small boxes)

1) EXE-first: Selfhost Parser → EXE（Phase 15.2）🚀
   - tools/build_compiler_exe.sh で EXE をビルド（同梱distパッケージ作成）
   - dist/nyash_compiler/{nyash_compiler,nyash.toml,plugins/...} で独立実行
   - 入力: Nyソース → 出力: JSON v0（stdout）
   - Smokes: sample.nyash→JSON 行生成（JSONのみ出力）
   - リスク: プラグイン解決（FileBox）をnyash.tomlで固定
2) LLVM Native EXE Generation（AOTパイプライン継続）
   - Python/llvmlite implementation as primary path (2400 lines, 10x faster development)
   - LLVM backend object → executable pipeline completion
   - Separate `nyash-llvm-compiler` crate (reduce main build weight)
   - Input: MIR (JSON/binary) → Output: native executable
   - Link with nyrt runtime (static/dynamic options)
   - Plugin all-direction build strategy (.so/.o/.a simultaneous generation)
   - Integration: `nyash --backend llvm --emit exe program.nyash -o program.exe`
3) Standard Ny std impl (P0→実体化)
   - Implement P0 methods for string/array/map in Nyash (keep NyRT primitives minimal)
   - Enable via `nyash.toml` `[ny_plugins]` (opt‑in); extend `tools/jit_smoke.sh`
4) Ny compiler MVP (Ny→MIR on JIT path) (Phase 15.3) 🎯
   - Ny tokenizer + recursive‑descent parser (current subset) in Ny; drive existing MIR builder
   - Target: 800 lines parser + 2500 lines MIR builder = 3300 lines total
   - No circular dependency: nyrt provides StringBox/ArrayBox via C ABI
   - Flag path: `NYASH_USE_NY_COMPILER=1` to switch rust→ny compiler; rust parser as fallback
   - Add apps/selfhost-compiler/ and minimal smokes
   - Stage‑1 checklist:
     - [ ] return/int/string/arithmetic/paren JSON v0 emit
     - [ ] Minimal ASI（newline separator + continuation tokens）
     - [ ] Smokes: `return 1+2*3` / grouping / string literal
   - Stage‑2 checklist:
     - [ ] local/if/loop/call/method/new/var/logical/compare
     - [ ] PHI 合流は Bridge に委譲（If/Loop）
     - [ ] Smokes: nested if / loop 累積 / and/or × if/loop
5) Phase 15.5: Core Box Unification（3層→2層革命）🎯
   - コアBox（nyrt内蔵）削除、プラグインBox/ユーザーBoxの2層に統一
   - 環境変数制御で段階的移行: `NYASH_USE_PLUGIN_CORE_BOXES=1`
   - 削減目標: 約700行（nyrt実装600行 + 特別扱い100行）
   - DLL動作確認→Nyashコード化の安全な移行戦略
   - 詳細: [phase-15.5-core-box-unification.md](phase-15.5-core-box-unification.md)
6) PHI 自動化は Phase‑15 後（LoopForm = MIR18）
   - Phase‑15: 現行の Bridge‑PHI を維持し、E2E 緑とパリティを最優先
   - MIR18 (LoopForm): LoopForm 強化＋逆Loweringで PHI を自動生成（合流点の定型化）
7) Bootstrap loop (c0→c1→c1')
   - Use existing trace/hash harness to compare parity; add optional CI gate
   - **This achieves self-hosting!** Nyash compiles Nyash
8) VM Layer in Nyash (Phase 15.4) ⚡
   - Implement MIR interpreter in Nyash (13 core instructions)
   - Dynamic dispatch via MapBox for instruction handlers
   - BoxCall/ExternCall bridge to existing infrastructure
   - Optional LLVM JIT acceleration for hot paths
   - Enable instant execution without compilation
   - Expected: 5000 lines for complete VM implementation
9) Plugins CI split (継続)
   - Core always‑on (JIT, plugins disabled); Plugins as optional job (strict off by default)

## Later (incremental)

- v1 Ny parser (let/if/call) behind `NYASH_JSON_IR_VERSION=1`
- JSON v1 bridge → MirBuilder (back-compat v0)
- 12.7 sugars normalized patterns in bridge (?. / ?? / range)
- E2E CI-lite matrix (no LLVM) for v0/v1/bridge roundtrip
- Ny script plugin examples under `apps/plugins-scripts/`
- Expand std Ny impl (String P1: trim/split/startsWith/endsWith; Array P1: map/each/filter; Map P1: values/entries/forEach)
- using/nyash.link E2E samples under `apps/` (small project template)
- Tighten Plugins job: migrate samples to Core‑13; re‑enable strict diagnostics

## Operational switches

- Parser path: `--parser {rust|ny}` or `NYASH_USE_NY_PARSER=1`
- JSON dump: `NYASH_DUMP_JSON_IR=1`
 - （予告）LoopForm: MIR18 で仕様化予定
 - Selfhost compiler: `NYASH_USE_NY_COMPILER=1`, child quiet: `NYASH_JSON_ONLY=1`
- EXE-first bundle: `tools/build_compiler_exe.sh` → `dist/nyash_compiler/`
- Load Ny plugins: `NYASH_LOAD_NY_PLUGINS=1` / `--load-ny-plugins`
- AOT smoke: `CLIF_SMOKE_RUN=1`

## Recipes / Smokes

- JSON v0 bridge: `tools/ny_parser_bridge_smoke.sh` / `tools/ny_parser_bridge_smoke.ps1`
- E2E roundtrip: `tools/ny_roundtrip_smoke.sh` / `tools/ny_roundtrip_smoke.ps1`
- EXE-first smoke: `tools/build_compiler_exe.sh && (cd dist/nyash_compiler && ./nyash_compiler tmp/sample.nyash > sample.json)`

## Implementation Dependencies

- Phase 15.2 (LLVM EXE) → Phase 15.3 (Nyash Compiler) → Phase 15.4 (VM in Nyash)
- Python llvmlite serves as rapid prototyping path while Rust/inkwell continues
- Plugin all-direction build enables static executable generation
- Total expected Nyash code: ~20,000 lines (75% reduction from 80k Rust)

## Stop criteria (Phase 15)

- v0 E2E green (parser pipe + direct bridge) including Ny compiler MVP switch
- v1 minimal samples pass via JSON bridge
- AOT P2: emit→link→run stable for constant/arith
 - Phase‑15 STOP には PHI 切替を含めない（PHI は LoopForm/MIR18 で扱う）
 - 15.3: Stage‑1 代表サンプル緑 + Bootstrap smoke（フォールバック許容）+ 文分離ポリシー公開
- Docs/recipes usable on Windows/Unix

## Notes

- JSON is a temporary, safe boundary. We will keep it for observability even after the in-proc bridge is default.
- Favor smallest viable steps; do not couple large refactors with new features.

## Ny Plugins → Namespace (Plan)

- Phase A (minimal): Add a shared `NyModules` registry (env.modules.{set,get}).
  - Map file path → namespace (project‑relative, separators → `.`, trim extension).
  - R5 hook: if a Ny plugin returns an exports map/static box, register it under the derived namespace.
  - Guard: reject reserved prefixes (e.g., `nyashstd.*`, `system.*`).
- Phase B (scope): Optionally run `[ny_plugins]` in a shared Interpreter to share static definitions.
  - Flag: `NYASH_NY_PLUGINS_SHARED=0` to keep isolated execution.
  - Logs: `[ny_plugins] <ns>: REGISTERED | FAIL(reason)`.
- Phase C (language bridge): Resolve `using foo.bar` via `NyModules`, then fallback to file/package resolver (nyash.link).
  - Keep IDE‑friendly fully qualified access; integrate with future `nyash_modules/`.
