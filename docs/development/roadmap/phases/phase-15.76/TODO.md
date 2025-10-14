# Phase 15.76 — TODO（extern_c / Self‑Host Bootstrap）

短期（Week 1） — extern_c MVP（VMのみ）
- [ ] Parser: `extern_c "name" (args)` 構文を AST に追加
- [ ] MIR Builder: `extern_call` で `interface="ffi.dynamic"` を発行
- [ ] VM: `call_dynamic_ffi()` 実装（0/1/2 引数・i64 返り）＋ ホワイトリスト
- [ ] スモーク（3本）: getpid()/strlen()/system() の最小動作

短期（Week 2） — ネイティブライブラリ
- [ ] `libs/llvm_backend/` 雛形＋ C API（`llvm_compile_mir_to_object`）
- [ ] Python llvmlite ハーネス呼び出し連携
- [ ] スモーク: MIR JSON → .o（戻り 0）

短期（Week 3） — LLVM AOT
- [ ] llvmlite ビルダーの `ffi.dynamic` 対応（declare/call）
- [ ] `.hako -> mir.json -> .o -> exe` を 1 ケース通す
- [ ] パリティ（VM/AOT）

短期（Week 4） — Self‑Host 最小統合
- [ ] apps/selfhost/compiler.hako（最小）: 1ファイル .o 化→clang link 実行
- [ ] スモーク: selfhost mini パイプライン（opt‑in）

参照
- 戦略（全体像）: ../phase-15.75/stage-4-chatgpt/EXTERN_C_SELFHOST_STRATEGY.md
- C‑ABI（最小）: ../phase-15.75/stage-4-chatgpt/C_ABI_MIN_SPEC.md
- 統合方針（Claude）: ../phase-15.75/stage-4-chatgpt/INTEGRATION_STRATEGY_CLAUDE.md
- Stage‑4（C‑ABIハーネス）: ../phase-15.75/stage-4-chatgpt/INDEX.md
