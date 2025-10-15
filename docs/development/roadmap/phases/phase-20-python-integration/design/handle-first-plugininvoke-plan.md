# Phase 10.5c — Handle-First PluginInvoke 設計（最優先計画）

目的: Python専用の型伝搬を撤廃し、プラグイン呼び出しを「Everything is Handle」で統一。Lowererは箱名や戻り型に依存しない最小知識で `PluginInvoke` を実Emitし、型解決は実行時（TLV/Handle）に委譲する。

## 背景と問題
- 現状のLowererに「import/getattr/call の戻りを PyObjectBox とみなす」暫定コードが混入。これは Python 特化のハードコーディングで、将来のプラグイン拡張（File/Net/DB 等）にブレーキとなる。
- すでに ABI/VM は TLV tag=8（Handle）を標準化。戻り値を Handle として受け取り、タイプ情報（type_id）は実行時に判明する。

## 原則（Handle-First）
- すべてのプラグインメソッドの戻りは Handle（TLV: tag=8）またはプリミティブ（i64/f64/bool/string/bytes）。
- Lowerer は「戻り型が box かどうか」だけを気にすればよい。個々の箱名（PyObjectBox 等）を前提にしない。
- 型の詳細は `type_id` によって実行時に解決される。JIT/AOT は型非依存の汎用コード生成を行う。

## 設計
1) メタデータ駆動
   - `nyash_box.toml` の `methods.*.returns = { type = "box" | "i64" | "f64" | "string" | "void" | ... }` を単一の参照源に。
   - `PluginHost.resolve_method` に `returns.type` を含む情報を公開（Lowerer から参照）。

2) Lowerer の汎用化
   - `PluginInvoke` を常に `emit_plugin_invoke(type_id, method_id, argc, has_ret)` に落とす。
   - 「戻りが box のときに特定箱名を記録する」実装を撤去。必要なら「dst は Handle（box）」のヒントのみ保持。
   - 受け手箱名が未確定の場合に備え、by-name 経路（後述）を用意。

3) by-name シム（任意→推奨）
   - `nyrt`/builder に `nyash_plugin_invoke_by_name_{i64,f64}(box_type_name?, method_name, a0, a1, a2)` を追加。
   - 受け手の箱名が Lowerer 時点で特定できない場合、by-name シムを使用して実行時に `method_id` を解決。

4) 実行時の統一
   - 既存の `nyash_plugin_invoke3_{i64,f64}` と同様に、TLVで引数を構築。Handle/プリミティブ変換（StringBox/IntegerBoxの自動プリミティブ化）を継続。
   - 戻りTLVの tag を見て i64/f64 経由の値化（`NYASH_JIT_NATIVE_F64`）またはハンドル登録を行う。

## マイルストーン
- M1: Lowerer から Python特化の型伝搬を除去（dst=Handle ヒントのみ）。
- M2: `PluginHost.resolve_method` 拡張で `returns.type` を取得可能に。
- M3: by-name シムの追加と Lowerer 配線（箱名未確定時）。
- M4: AOT 最小ケース（import→getattr→call）を Handle-First で Green。
- M5: ドキュメントと CURRENT_TASK を刷新。

## 受け入れ条件（DoD）
- VM: `py.import("math"); (math.getattr("sqrt")).call(9)` が Green（autodecode=1 で 3）。
- AOT（strict）: 上記チェーン最小例で unsupported=0。Console 出力経路は PluginInvoke または extern 経由で表示。
- Lowerer に Python 固有の型分岐が存在しない（grepで検出不可）。

## 運用メモ
- 将来的な最適化（箱名が静的に分かる場面での特殊化）は、Handle-First を壊さない範囲で「分岐の1箇所」に限定して導入する。

