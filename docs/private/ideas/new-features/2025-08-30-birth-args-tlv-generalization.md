# Birth 引数の一般化メモ（可変長TLV / 例外ハンドリング）

目的（10.5c）: birth/by-name を汎用化し、String/Integer はプリミティブ、他は Handle で TLV 化。将来的な可変長と例外連携を見据えた最小要件を定義。

## ゴール
- 可変長引数の birth（例: `new FileBox(path, mode)` / `new MapBox(initial_pairs...)`）。
- by-name birth: `type_name.birth(args...)` を TLV 経由で統一。
- 失敗は Result/エラー文字列で伝搬し、パニック/例外の越境は行わない。

## TLV レイアウト（案）
- Header: `u16 entry_count`
- Entries: `[tag:u16][len:u16][payload..] * entry_count`
- 許容型: bool(1), i64(3), f64(5), string(6), bytes(7), handle(8)

## 変換規約
- StringBox → TLV string / IntegerBox → TLV i64（プリミティブ化）
- その他の Box → TLV handle (type_id:u32 + instance_id:u32)
- 即値（i64/f64/bool）はそのまま TLV 化

## 例
- `new StringBox()` → 引数0
- `new IntegerBox(42)` → [i64:42]
- `new FileBox("/tmp/x", "w")` → [string:"/tmp/x"], [string:"w"]
- `new ArrayBox(1, 2, 3)` → [i64:1], [i64:2], [i64:3]

## 例外/エラーの扱い
- birth 失敗時は、
  - 可能なら `returns_result=true` とし、Ok(Handle)/Err(String) の TLV 形状を返す（VMは Result で包む）。
  - それが未設定の場合は、TLV string（エラーメッセージ）を返し、呼び手が `try_birth` 等で明示的に扱う（将来）。
- パニック/例外の越境は禁止（C-ABI上はコード戻り or TLV に限定）。

## by-name birth
- `nyash_plugin_birth_by_name(box_type_name, args_tlv, out_tlv)`（将来/任意）。
- Lowerer は型名が静的に分かるときは従来の `type_id` 経路、未確定時は by-name を使用可。

## 今後の拡張
- 可変長 >2 の aN 受け: 既存 `*_invoke3_*` シムを拡張（`invokeN`/スタック経由/一時領域）。
- Named 引数: TLV に name ハッシュ or 別エントリで拡張（後段）。
- 既定値: Plugin 側の既定値解決と TLV 省略の整合性を設計。

