# nyash.toml v2.1 拡張仕様（最小）

目的: プラグインBoxのメソッド引数として、他のBoxを不透明参照（BoxRef）で安全に受け渡す。

## 変更点（v2 → v2.1）
- メソッド引数型に `kind = "box"` を追加（当面は `category = "plugin"` のみ）
- TLVに BoxRef（Handle）を追加（tag = 8）
  - payload: `type_id: u32 (LE)`, `instance_id: u32 (LE)`（合計8バイト）
- 既存タグは不変：1=Int64, 2=String(UTF-8), 3=Bool

## 例: libraries セクション
```toml
[libraries]
[libraries."libnyash_filebox_plugin.so"]
boxes = ["FileBox"]
path = "./target/release/libnyash_filebox_plugin.so"

[libraries."libnyash_filebox_plugin.so".FileBox]
type_id = 6
abi_version = 1

[libraries."libnyash_filebox_plugin.so".FileBox.methods]
# 既存
birth = { method_id = 0 }
open  = { method_id = 1 }
close = { method_id = 4 }

# 追加例: Box引数を1つ受け取る
copyFrom = { method_id = 7, args = [ { kind = "box", category = "plugin" } ] }
```

備考:
- `args` を省略した場合は引数なし（ゼロ引数）とみなす（v2互換）
- 複数引数は配列で列挙（例: 2引数なら2要素）
- ユーザー定義Boxや複雑なビルトインBoxは当面対象外（将来のvtable/retain-release設計で拡張）

## 呼び出し時のTLVエンコード
- 先頭ヘッダ `[ver:1, argc:1, rsv:2]` の後、各引数を `tag + payload` で列挙
- `tag=8 (Handle/BoxRef)`: payload = `type_id(4) + instance_id(4)` （LE）
- 未対応Box種別に対してはエラーを返す（開発時のみ toString フォールバックを許容可能）

## 戻り値（v2.1→v2.2）
- v2.1: Int/String/Bool（1/6/3）とVoid(9)
- v2.2: BoxRef(Handle, tag=8) の「返り値」対応を追加（同一/別Box型どちらも可）
  - payload: `type_id:u32` + `instance_id:u32`
  - Loaderは `type_id` から `lib_name/box_name` を逆引きし、`PluginBoxV2` を生成して返す

## 互換性
- `args` 宣言がない既存v2設定はそのまま利用可
- BoxRefを使わないメソッドは従来通り Int/String/Bool のみで動作

## 実装メモ（参考）
- Loader: invoke時の引数エンコードに `tag=4` を追加（`category=plugin` のみ）
- プラグイン側: 受領した `type_id` と期待型を照合し、不一致ならエラー
- 所有権: 呼び出し中の一時借用（保持は将来の retain/release で対応）
