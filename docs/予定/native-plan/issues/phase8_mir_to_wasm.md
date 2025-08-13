# Phase 8: MIR→WASM codegen (browser/wasmtime; sandboxed; Rust runtime free)

## Summary
- MIR から素の WebAssembly を生成し、ブラウザ/wasmtime（WASI）でサンドボックス実行する。
- Rust は「コンパイラ本体」のみ。実行は純WASM＋ホストimport（env.print 等）。

## Scope
- ABI/Imports/Exports（最小）
  - exports: `main`, `memory`
  - imports: `env.print(i32)`（デバッグ用に整数のみ。将来文字列ABIを定義）
- メモリ/ヒープ
  - 線形メモリに簡易ヒープ（bump or フリーリスト）
  - Box の固定レイアウト（フィールド→オフセット表; 型名→レイアウトは暫定固定）
- 命令カバレッジ（段階導入）
  - PoC1: 算術/比較/分岐/loop/return/print
  - PoC2: RefNew/RefSet/RefGet（Phase 6 と整合）で `print(o.x)`
  - PoC3: Weak/Barrier の下地（WeakLoad は当面 Some 相当、Barrier は no-op）
- CLI 統合（任意）
  - `--backend wasm` で生成・実行（wasmtime 呼び出し）。未実装の場合は分かりやすいエラーで誘導。

## Tasks
- [ ] WASMバックエンド新規モジュールの足場作成（`src/backend/wasm/`）
- [ ] PoC1: MIR → WAT/WASM 変換（算術/比較/分岐/loop/return/print）
- [ ] PoC2: RefNew/RefSet/RefGet の線形メモリ上実装（簡易ヒープ + 固定レイアウト）
- [ ] PoC3: WeakLoad/Barrier のダミー実装（将来GC対応のためのフック）
- [ ] 実行ラッパ（wasmtime CLI）とブラウザローダ（JS importObject）のサンプル
- [ ] （任意）`--backend wasm` CLI のプレースホルダ/実装

## Acceptance Criteria
- PoC1: 生成WASMを wasmtime で実行し、戻り値・print 出力が期待通り
- PoC2: `o = new Obj(); o.x = 1; print(o.x)` 相当が Ref 系命令で動作
- PoC3: Weak/Barrier のダミー命令を含むWASMが生成・実行（実質 no-op で可）
- CLI 連携を行う場合、未対応箇所は明瞭なエラー/誘導メッセージ

## Tests
- tests/wasm_poc1_basic.sh（または Rust integration）
  - MIR → WASM の出力を wasmtime で実行し、終了コード/標準出力を検証
- tests/wasm_poc2_ref_ops.sh
  - RefNew/RefSet/RefGet 経由の `print(o.x)` を確認
- Node/ブラウザ用：ヘッドレス実行で `env.print` の呼び出しを検証する最小スクリプト

## Out of Scope
- 本格GC/Weak無効化、fini/Pin/Unpin、JIT/AOT の高度最適化、複雑な文字列ABI

## References
- docs/予定/native-plan/README.md（Phase 8）
- docs/説明書/wasm/*
