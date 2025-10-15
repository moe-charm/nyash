# Collection Boxes — Unified Semantics (Phase 15.7)

Everything is Box の方針に合わせ、ArrayBox / MapBox の最小仕様をまとめ直したリファレンス。VM 内蔵・プラグイン・AOT runtime はこの表を共有仕様として扱う。

## Quick Reference Table

| Box      | Method            | Return            | Notes |
|----------|-------------------|-------------------|-------|
| ArrayBox | `size()`          | IntegerBox        | `length()` は alias（互換） |
| ArrayBox | `isEmpty()`       | BoolBox           | `size() == 0` を即時判定 |
| ArrayBox | `get(index)`      | Box / NullBox     | 範囲外は `null`。プラグイン/ユーザーボックスは `share_box()` で同一性維持 |
| ArrayBox | `set(index,val)`  | NullBox           | 置換/末尾 append。範囲外はエラーメッセージ（Fail-Fast ガイドライン） |
| ArrayBox | `push(val)`       | NullBox           | 末尾追加。Stage 2 で sort/reverse も Null 返却へ統一 |
| ArrayBox | `remove(index)`   | Box / NullBox     | 取得して削除。OOB は `null` |
| ArrayBox | `indexOf(val)`    | IntegerBox        | 未一致は `-1` |
| ArrayBox | `clear()`         | NullBox           | 全要素削除 |
| MapBox   | `size()`          | IntegerBox        | `isEmpty()` も BoolBox で提供 |
| MapBox   | `isEmpty()`       | BoolBox           | `size() == 0` |
| MapBox   | `get(key)`        | Box / NullBox     | 不在は `null`（Stage 15.7 fix） |
| MapBox   | `set(key,val)`    | NullBox           | 差分書き込み。値は Box/Plugin/HostHandle を透過 |
| MapBox   | `has(key)`        | BoolBox           | 文字列キー/整数キーの両対応 |
| MapBox   | `remove(key)`     | Box / NullBox     | 削除された値を返却（無い場合は `null`） |
| MapBox   | `clear()`         | NullBox           | 全エントリ削除 |
| MapBox   | `keys()`          | ArrayBox          | Stage-2 HostHandle で ArrayBox を共有。Stage-1 `keysS()` は改行文字列 |
| MapBox   | `values()`        | ArrayBox          | `keys()` と同じく Stage-2 で ArrayBox を共有。Stage-1 `valuesS()` は改行文字列 |

## SetBox — Map ベースの集合（Map<Key, Unit>）

Everything is Box の原則に従い、Set は Map の仕組み（Eq/Hash 正規化・決定モード）を再利用して実装する。

### API（最小）

| Method         | Return      | Semantics |
|----------------|-------------|-----------|
| `add(v)`       | NullBox     | 既に存在していても No‑op（常に Null） |
| `remove(v)`    | NullBox     | 不在なら No‑op（常に Null） |
| `has(v)`       | BoolBox     | 値の存在判定 |
| `size()`       | IntegerBox  | 要素数 |
| `isEmpty()`    | BoolBox     | `size()==0` の糖衣 |
| `clear()`      | NullBox     | 全要素削除 |
| `toArray()`    | ArrayBox    | 順序は Map.keys() と同一（決定モードで安定） |

補足
- Set の内部は Map<Key, Unit> と等価。Unit は観測不可（Null/内部Unit）で外部挙動には影響しない。
- 破壊系（add/remove/clear）の戻り値は NullBox に統一（Map と一貫）。
- Eq/Hash/決定モードは Map のポリシーをそのまま継承（非Hashableキーは Fail‑Fast）。
 - 提供形態: プラグイン `plugins/nyash-set-plugin`（`SetBox`）。`hako.toml`/`nyash.toml` に `libnyash_set_plugin.so` をマップする（type_id=15 推奨）。


## Shared Behaviour

- **Null-first**: 存在しない要素は `NullBox` を返す（文字列による「Key not found」等は撤退済み）。
- **同一性保持**: PluginBox / InstanceBox / ArrayBox は `share_box()` で返し、Map/Array 経由の更新が共有される。
- **Void 撤退**: 破壊的メソッド（push/set/clear/remove など）は `NullBox` を返す。旧来の "ok"/size/i64 戻り値は docs でも非推奨。
- **isEmpty()**: すべてのコレクションで提供し、`size()==0` の糖衣として利用。

## Stage-2 Keys/Values

`NYASH_PLUGIN_MAP_ARRAY_HANDLE=1` を有効にすると、プラグイン版 MapBox の `keys()/values()` が HostHandle(ArrayBox) を返す。ArrayBox 側の `size()/isEmpty()/get()` はそのまま利用でき、VM 便宜ハンドラなしで恒等性が維持される。plugins プロファイルでは既定で ON（`tools/smokes/v2/configs/env/plugins.env`）。

Stage-1 互換として文字列版 `keysS()/valuesS()` も残しているが、将来的に削除予定。新規コードは Stage-2 Array 経路を利用すること。

Note（HostHandleRouter 運用）
- plugins プロファイルでは、HostHandleRouter 経路を既定で優先（Map.size/has/get/set、Array.size/get/set、String.size/len を slot へ強制）。
- quick プロファイルでは既定OFF（開発時に `NYASH_MAP_FORCE_HOST=1`, `NYASH_ARRAY_FORCE_HOST=1`, `NYASH_STRING_SIZE_FORCE_HOST=1` を使用）。

Length 系の統一（実装メモ）
- String/Array の `size/len/length` は、Builder 正規化で Extern に降格（String:`nyrt.string.length` / Array:`nyrt.array.size`）。
- 受けの materialize は EmitGuard で一度だけ実施し、正規化では新しい ValueId を発行しない（再materialize禁止）。
- VM extern は HostHandle/Builtin を吸収し、受け未定義や Method 経路の揺れに影響されない。

## Identity Smokes

Phase 15.7 では下記の恒等性テストを常時回している：

- `map_array_identity_vm.sh` — Map に格納した ArrayBox を共有し、push 後に両方の `size()` が更新されるか検証。
- `map_callable_identity_vm.sh` — `methodRef` で得た CallableBox を Map に格納し、`call()` でレシーバを共有。
- `map_filebox_identity_vm.sh` — FileBox (HostHandle) を Map に格納し、書き込み後に元インスタンスの `read()` で内容が観測できるか確認。

## Error Policy

- インデックスやキーの型が不正な場合は開発ビルドで Fail-Fast（明示的なエラーメッセージ）する。リリースビルドでは `null` を返す互換を維持。
- 域外アクセス（Array.set の OOB など）はエラーメッセージを返す。構造的に防げる場合は先に境界を分離すること。

## Plugin Alignment

- プラグイン実装（`plugins/nyash-array-plugin`, `plugins/nyash-map-plugin`）は同じ仕様に追従する。Stage-2 モードで HostHandle を返し、`set/remove` の戻り値は NullBox を返す予定。
- `nyash_box.toml` では `returns = { type = "box" }` を指定し、HostHandle/PluginHandle を含む一般値として扱う。
