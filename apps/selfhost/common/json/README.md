Mir JSON Builders (selfhost)

Overview
- `mir_builder_min.hako` (static API): 既存の最小 Builder（引数で状態 `st` を持ち回る）。
- `mir_builder2.hako` (instance API): 新規の回避 Builder（内部に状態 `me.st` を保持）。

Why two builders?
- 現在、言語ランタイム側の不具合により「static box メソッドの第1引数が消失する」ケースを確認しています。
- その影響で、`start_module(st)` のような「状態を第1引数に取る」API が失敗します。
- 当面は instance API の `MirJsonBuilder2` を使用すると安全に動きます（内部状態のみで動作）。

Migration
- emit 系の Box からは `MirJsonBuilder2` を呼び出してください。
- ランタイム側の修正が入り次第、`mir_builder_min.hako` へ段階的に戻す（あるいは一本化）予定です。

Notes
- どちらの Builder も「構造配列（blocks/instructions）→文字列」の `to_string_rebuild()` を既定とし、逐次出力は開発用に留めます。

