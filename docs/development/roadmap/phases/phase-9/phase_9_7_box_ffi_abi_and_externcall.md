# Phase 9.7: Box FFI/ABI + BID + MIR ExternCall + WASM RuntimeImports（最小実装）

本ドキュメントは、バックエンド横断の「Library-as-Box（あらゆるライブラリを箱に）」基盤を実装するために、Copilotへ依頼する具体タスクを日本語で定義します。

- Box FFI/ABI と BID（Box Interface Definition）の定義
- MIR への `ExternCall` 追加（NyIR Core 26命令の13番目として確立）
- `ExternCall` の各バックエンドへの写像（まずは WASM: RuntimeImports、VM はスタブ）
- 最小の E2E デモ（console/canvas）

## 目的（Goals）

- 外部ライブラリを安定した言語非依存 ABI で「Box」として呼び出せるようにする。
- MIR から `ExternCall` で外部呼び出しを一貫表現できるようにする。
- wasm-bindgen に頼らず、Nyash が直接出力する WASM から host API（console/canvas）を呼べるようにする。
- Everything is Box と MIR の effect モデルに整合させる。

前提（Prerequisites）
- Issue #62 による「WASM バックエンドの文字列定数＋StringBox 最小対応」が先行タスク（`print_str(ptr,len)` を含む）。
- ABI ドラフト/関連資料：
  - docs/予定/native-plan/box_ffi_abi.md
  - docs/nyir/spec.md（NyIR骨子；将来の公開IRとの整合を意識）

## 成果物（Deliverables）

1) ABI/BID 仕様（docs/box_ffi_abi.md）＋ BID サンプル（console, canvas）
2) MIR 命令：`ExternCall(dst, iface_name, method_name, args[])`
3) WASM RuntimeImports（最小）：`env.console.log/print_str`, `env.canvas.*`
4) WASM コード生成：`ExternCall` 呼び出し → import 経由、文字列は (ptr,len)
5) VM 側のスタブ経路（ログ/No-opで可）
6) E2E デモ：Nyash → MIR(ExternCall) → WASM → ブラウザ（console/canvas）

## ABI/BID（Box FFI/ABI）

- 型: i32/i64/f32/f64/bool/string(UTF-8 ptr,len)/boxref/array/null/void
- 名前付け: `namespace.box.method`（例: `env.console.log`, `env.canvas.fillRect`）
- 効果: pure/mut/io/control（MIR 最適化規則と整合）
- エラー: v0 はエラーコード or void を想定（例外は範囲外）
- 同期/非同期: v0 は同期。将来拡張の余地を設計。

Action items:
- `docs/box_ffi_abi.md` に上記を整備（特に文字列のエンコード/メモリ規約）。
- BID サンプル（YAML/JSON）：console/canvas の署名＋効果を明記。

## MIR: ExternCall

新命令：

```
ExternCall {
  dst: ValueId,               // void の場合は省略可
  iface_name: String,         // 例: "env.console"
  method_name: String,        // 例: "log"
  args: Vec<ValueId>,         // 引数列
}
```

Verifier:
- BID に定義された署名（型/引数個数/効果）に合致することを検証。
- 効果順序の保持：pure は再順序化可、mut/io は順序保持。

Lowering:
- 高位の外部 Box 呼び出し → 署名解決済みの `ExternCall` を生成。
- 既存の BoxCall/Field ops は変更しない。`ExternCall` は「外部/host API 専用」。

## WASM RuntimeImports

最小インポート：
- `(import "env" "print" (func $print (param i32)))`（既存）
- `(import "env" "print_str" (func $print_str (param i32 i32)))`
- `(import "env" "console_log" (func $console_log (param i32 i32)))`
- `(import "env" "canvas_fillRect" (func $canvas_fillRect (param i32 i32 i32 i32 i32 i32)))`
- `(import "env" "canvas_fillText" (func $canvas_fillText (param i32 i32 i32 i32 i32)))`

Notes:
- 文字列は (ptr,len) で線形メモリから読み出す（UTF-8）。
- canvas 系の第1引数 (ptr,len) は canvas 要素IDを表す。他は数値。

Host 側（ブラウザ/Node）：
- `print_str`, `console_log`, `canvas_*` を実装し、メモリから取り出した文字列を DOM/console へ反映。

## WASM コード生成（Mapping）

- `ExternCall` → 対応する import シンボルを `call $...` で呼び出す（引数整列）。
- 文字列：値を (ptr,len) で用意。定数は data segment、動的は実行時バッファ。
- v1 の対象：
  - `env.console.log(ptr,len)`
  - `env.canvas.fillRect(canvasIdPtr,len, x,y,w,h)`
  - `env.canvas.fillText(textPtr,len, x,y, fontPtr,len?)`（初版は簡略化可）

## VM 側（スタブ）

- VM ランナーに外部関数レジストリを用意。
- console 系：stdout 出力。canvas 系：No-op または引数ログ。

## E2E デモ

- Nyash から extern 経由で console/canvas を使うサンプルを用意（表面構文 or 最小注入）。
- ビルド → MIR に `ExternCall` を含む → WASM 生成 → ブラウザ側 importObject で対応。
- console ログと、既知の `<canvas>` id への矩形/文字描画を確認。

## 受け入れ基準（Acceptance Criteria）

- ABI 仕様と BID サンプルが存在し、verifier/codegen がそれを利用する。
- MIR に `ExternCall` が存在し検証される。
- WASM バイナリが指定 import を持ち、正しく呼び出す。
- ブラウザデモで canvas 描画/console 出力が外部呼び出し経由で行える。
- VM 経路はクラッシュしない（スタブで可）。

## 範囲外（Out of Scope）

- 非同期 extern の完全対応、エラーモデル統一、lifetimes/GC 連携。
- 豊富な canvas API 一式（v1 は fillRect/fillText に限定）。
- ブラウザ以外のターゲット（Node 以外）は今回対象外。

## リスクと回避策（Risks & Mitigation）

- 文字列/メモリ規約の曖昧さ → v1 は UTF-8 (ptr,len) を固定、ヘルパ提供。
- 効果モデルの不整合 → BID に効果を必須化、verifier で順序検証。
- wasm-bindgen 経路との乖離 → 旧経路は残しつつ整合テストを追加（段階的に ABI 経路へ集約）。

## 実施順（Phase 10 との関係）

- Phase 9.7 を Phase 10 の前に実施。外部 API 基盤は AOT/JIT/言語出力の前提。

## 関連/参照（References）

- ABI ドラフト: docs/予定/native-plan/box_ffi_abi.md
- NyIR 骨子: docs/nyir/spec.md
- 文字列定数の前提対応: docs/予定/native-plan/issues/issue_62_update_proposal.md
- LLVM 最小実装: docs/予定/native-plan/issues/phase_10_x_llvm_backend_skeleton.md
- Security（将来：権限/ケイパビリティ）: Phase 9.9
