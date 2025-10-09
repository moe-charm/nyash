# Hako ABI — Collections (String/Array/Map)

目的
- plugin‑on 環境で String/Array/Map を一貫した ABI（TypeBox v2）で扱うための最小ガイド。
- コード側のハードコーディングを避け、toml/spec/TypeBox による動的解決へ統一する。

共通方針
- size() 統一: 表面 API は size() を推奨。ABI/プラグイン側は length() を実装し、resolve で size→length を正規化する。
- birth 初期化: NewBox 後に birth/0（必要なら引数あり）で初期化。ホスト→プラグインの橋渡しで文字列を受ける場合は birth(s) を優先、無ければ fromUtf8(s)。
- Handle 受け渡し: プラグイン Box は TLV tag=8（type_id:u32 + instance_id:u32）で往復。ホスト側は type_id→box_type を逆引きし PluginBoxV2 を復元。

## StringBox（TypeBox v2 最小）
- length(0) -> i64（size の別名）
- isEmpty(0) -> bool
- substring(2) -> String
- indexOf(1..2) -> i64
- lastIndexOf(1..2) -> i64
- charAt(1) -> String
- fromUtf8(1) -> Handle(StringBox)（新規作成）

## ArrayBox（TypeBox v2 最小）
- length/len/size(0) -> i64
- get(1) -> i64 | Handle | null（範囲外は null）
- push(1) -> i64（新しい長さ）
- set(2) -> i64（末尾+1 で append 相当）

## MapBox（TypeBox v2 最小）
- size(0) -> i64
- has(1) -> bool（key: i64|string）
- get(1) -> i64 | Handle | null（key: i64|string）
- set(2) -> i64（value: i64|Handle）

## TLV エンコード（抜粋）
- 1=Bool, 2=I32, 3=I64, 6=String, 7=Bytes, 8=Handle(Plugin), 9=Handle(Host)
- 引数: PluginBoxV2 は tag=8 を使う（ハンドル優先）。数値なら i64、文字列は UTF‑8。
- 返り値: tag=8 を受け取ったら type_id から box_type を逆引きして PluginBoxV2 を構築。

## 実装メモ（今回の整備）
- VM→プラグイン引数: PluginBoxV2 は常に tag=8（src/runtime/plugin_ffi_common.rs）
- 返り値の型復元: tag=8 は type_id→box_type 逆引き（src/runtime/plugin_loader_v2/enabled/ffi_bridge.rs）
- String 橋渡し: 受けがホスト String のとき、birth(s)→fromUtf8(s) で一時 StringBox を生成して呼び出し（plugin_bridge.rs）
- Map 値のハンドル対応: set/get が Handle を保持・返却可能に（plugins/nyash-map-plugin）

## 運用上のヒント
- plugin‑on: HAKO_PLUGIN_POLICY=auto + NYASH_PLUGIN_CONFIG=hako.toml（tools/smokes/v2/configs/env/plugin-on.env 経由）
- plugins OFF: HAKO_PLUGIN_POLICY=off（Unknown Box を出さず Embedded 経路で最小動作）
- スモーク: identity（Map に Array を格納→get→両方から push で size が一致）を常設

