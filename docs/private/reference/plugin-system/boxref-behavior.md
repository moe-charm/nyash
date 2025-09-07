# BoxRef/Handle Behavior (v2.1–v2.2)

本書は、プラグインBoxの引数/返り値としてのBox参照（BoxRef/Handle）の扱いと、VM/インタプリタ/ローダ/プラグインにまたがる設計上の注意点をまとめます。

## 1. TLV仕様（BID-1）
- ヘッダ: `ver:u16=1, argc:u16`
- エントリ: `tag:u8, rsv:u8=0, size:u16, payload...`
- 主要タグ:
  - `6 = String(UTF-8)`, `7 = Bytes`
  - `8 = Handle(BoxRef)` → payload: `type_id:u32 + instance_id:u32`（計8バイト, LE）
  - `2 = I32`, `3 = I64`, `1 = Bool`, `9 = Void`

## 2. nyash.toml（v2.1〜）
- 引数宣言に `args=[{ kind="box", category="plugin" }]` を追加可能。
- 例:
  ```toml
  [libraries."libnyash_filebox_plugin.so".FileBox.methods]
  copyFrom = { method_id = 7, args = [ { kind = "box", category = "plugin" } ] }
  cloneSelf = { method_id = 8 }
  ```
- ローダは `args` の型宣言に基づいて実引数を検証。型不一致は `InvalidArgs`。

## 3. 返り値（v2.2）
- プラグインが `tag=8` を返した場合、ローダは `type_id` を `nyash.toml` で逆引きし、
  `PluginBoxV2 { box_type, type_id, invoke_fn, instance_id, fini_method_id }` を構築して返す。

## 4. VM/インタプリタの扱い
- メソッド呼び出しはローダ経由に統一（TLV/Handle処理もローダ側）。
- MIR Loweringは以下を厳守：
  - ユーザー定義Boxのみ関数化（Call）最適化可。
  - プラグイン/ビルトインは常に `BoxCall` を出す（VMでローダに委譲）。
- VMのBoxRefは共有ハンドルとして扱う：
  - `clone_box()` ではなく `share_box()` を使用（不意のbirth回避）。

## 5. プラグイン実装の注意
- `open` のモードによっては `read` ができない（例: "w"）。`copyFrom` は `file.read` が失敗したら `buffer` にフォールバックする。
- `write` 実装では、成功後に `buffer` を更新しておくと `copyFrom` のフォールバックで活きる。
- 典型メソッドID（例: FileBox）
  - `0=birth`, `1=open`, `2=read`, `3=write`, `4=close`, `0xFFFFFFFF=fini`, `7=copyFrom`, `8=cloneSelf`

## 6. トラブルシュート
- rc=-4 `Invalid arguments`
  - `args` 型宣言と実引数が不一致。ローダログの引数エンコードを確認（String化フォールバックが出ていないか）。
- rc=-5 `Plugin internal error`
  - プラグイン内部のread/write/lock失敗など。`copyFrom` のfile→bufferフォールバック不備を疑う。
- rc=-8 `Invalid handle`
  - 存在しない `instance_id` に対する呼び出し。VMで `clone_box` を使っていないか（`share_box` へ）。

## 7. 参考
- 仕様: `docs/reference/plugin-system/nyash-toml-v2_1-spec.md`
- 実装: `src/runtime/plugin_loader_v2.rs`（引数検証/Handle戻り値復元）
- 例: `docs/examples/plugin_boxref_return.nyash`

