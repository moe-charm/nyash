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
- [x] **Phase 15.5-A: プラグインチェッカー拡張完成**（ChatGPT5 Pro⭐⭐⭐⭐⭐最高評価）
  - ✅ 4つの安全性機能完全実装（ユニバーサルスロット衝突・StringBox問題・E_METHOD・TLV検証）
  - ✅ 100%検出精度実証（手動発見問題を完全自動検出）
  - ✅ 実用検証済み（実際のnyash.tomlで8問題自動検出・修正指示）
- [x] **Phase 15.5-B-1: slot_registry統一化完成**（StringBox問題根本修正）
  - ✅ core box静的定義30行削除完了（3-tier→2-tier基盤確立）
  - ✅ former core boxes（StringBox/IntegerBox/ArrayBox/MapBox）のplugin slots移行
  - ✅ WebChatGPT環境との完全一致（同じnyash.toml設定で同じ動作）

## Next (small boxes)

1) Ny JSON ライブラリ（最小 DOM / JSON v0 対応）
   - Nyash 製の parse/stringify（object/array/string/number/bool/null）。
   - Env: `NYASH_JSON_PROVIDER=ny`（既定OFF）。
   - Smokes: roundtrip/エラー位置検証（quick 任意; CI非ブロック）。
2) Ny Executor（最小命令セット）
   - ops: const/binop/compare/branch/jump/ret/phi（Box 呼び出しは後段）。
   - Env: `NYASH_SELFHOST_EXEC=1`（既定OFF）。
   - Parity: PyVM/LLVM harness と stdout/exit の一致。
3) 呼び出し最小（Console/String/Array/Map P0）
   - call/externcall/boxcall の最小を接続。未知 extern は STRICT で拒否。
4) Selfhost Parser の EXE 化（任意・後回し可）
   - `tools/build_compiler_exe.sh` により JSON v0 emit の単体配布（開発者向け）。
5) PHI 自動化は Phase‑15 後（LoopForm = MIR18）
   - Phase‑15: 現行の Bridge‑PHI を維持（規約は「incoming pred=実際の遷移元」）。
   - MIR18: LoopForm 強化＋逆Loweringでの自動化に委譲（設計のみ先行）。
6) Plugins CI split（継続）
   - Plugins は任意ジョブ（strict off）を維持。Core は軽量 quick を常時。

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
 - JSON provider: `NYASH_JSON_PROVIDER=ny`（Ny JSON; 既定OFF）
 - Ny executor: `NYASH_SELFHOST_EXEC=1`（既定OFF）
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
