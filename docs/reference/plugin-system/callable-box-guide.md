# CallableBox ガイド

このドキュメントは Phase 15.7 で導入された CallableBox の扱いと、VM 側で予約された疑似メソッドについてまとめます。

## methodRef 疑似メソッド

- 形式: `receiver.methodRef(name: String, arity: Integer) -> CallableBox`
- 実装: `src/runtime/method_router_box/method_ref.rs`
- VM ルーターは最初にこの疑似メソッドを判定し、型チェックと CallableBox 生成を一箇所で行う
- 失敗は Fail-Fast (`name` は String、`arity` は 0 以上の Integer)
- 返り値の CallableBox は `receiver.share_box()` を束縛しており、呼び出し時に receiver の同一性が保持される

## Map.call / Map.callAsync シュガー

- 形式: `map.call(key, argsArray)`, `map.callAsync(key, argsArray)`
- 実装: `src/runtime/method_router_box/map_callable.rs`
- VM が `map.get(key)` で CallableBox を取得 → `call`/`callAsync` を再帰的に `MethodRouterBox::route` へ委譲
- これによりプラグイン側は `get/set` 群のみ実装すればよく、`call` 系は VM 側で共通化

## TL;DR

- methodRef と Map.call/Async は VM 側の箱 (`MethodRefBox`, `MapCallableBox`) で処理する
- プラグインやユーザーボックスは疑似メソッドを実装する必要がない
- 仕様の詳細は `docs/reference/plugin-system/vm-plugin-integration.md` の Phase 15.7 セクションも参照
