# BuiltinBoxFactory段階削除戦略

> **Phase 15.5 "Everything is Plugin" への道**
> 作成日: 2025-09-24
> 戦略者: codex

## 🎯 目標

BuiltinBoxFactoryと`builtin_impls/`を安全に段階削除し、すべてをプラグインに移行する。

## 📊 現状分析

### 削除対象
- `src/box_factory/builtin.rs`: BuiltinBoxFactory実装
- `src/box_factory/builtin_impls/`: 8ファイル、32KB
  - string_box.rs
  - integer_box.rs
  - bool_box.rs
  - array_box.rs
  - map_box.rs
  - console_box.rs
  - null_box.rs
  - mod.rs

### 参照箇所
- `src/runtime/unified_registry.rs`: BuiltinBoxFactory登録（2箇所）
- `src/runtime/nyash_runtime.rs`: BuiltinBoxFactory登録（1箇所）

## 🚀 段階削除戦略

### Phase 1: 機能ゲート厳格化（現在）
```rust
// 現状：plugins-onlyでのみ無効化
#[cfg(not(feature = "plugins-only"))]
{
    registry.register(Arc::new(BuiltinBoxFactory::new()));
}
```

### Phase 2: Opt-in化（次回）
```rust
// 将来：builtin-coreでのみ有効化
#[cfg(feature = "builtin-core")]
{
    registry.register(Arc::new(BuiltinBoxFactory::new()));
}
```

### Phase 3: 個別Box削除順序
1. **StringBox**: プラグイン版が安定動作確認後
2. **IntegerBox**: 同上
3. **BoolBox**: シンプルなので早期削除可
4. **ArrayBox**: 依存関係確認後
5. **MapBox**: 最後に削除
6. **ConsoleBox**: print依存の最後
7. **NullBox**: 影響最小、いつでも削除可

## ✅ 検証項目（各段階）

### ビルド検証
```bash
cargo check --features llvm
cargo check --features plugins-only
```

### スモークテスト
```bash
# プラグインで基本動作確認
./target/release/nyash test_plugin_basic.nyash

# LLVM EXEでprint出力確認
NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm test.nyash
```

## 🛡️ ロールバック戦略

各削除は別ブランチで実施：
- `cleanup/builtin-string`
- `cleanup/builtin-integer`
- 等

失敗時は該当ブランチをrevertするだけで復帰可能。

## 🏁 完了条件

1. `builtin_impls/`ディレクトリ削除
2. `BuiltinBoxFactory`削除
3. すべてのBoxがプラグイン経由で動作
4. CI全緑確認

## 📅 タイムライン

- **Phase 1**: ✅ 完了（2025-09-24）
- **Phase 2**: 機能フラグ反転（次回作業）
- **Phase 3**: 個別Box削除（1週間程度）
- **完了予定**: Phase 15.5完了時