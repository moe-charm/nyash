# Phase 15.5: Core Box Unification - 3層→2層革命

## 📅 実施予定
2025年9月24日〜10月15日（3週間）

## 🎯 目標
コアBox（nyrt内蔵）を削除し、プラグインBox・ユーザーBoxの2層構造に統一

### 削減見込み
- **コード削減**: 約600行削除（nyrt実装）
- **特別扱い削除**: MIRビルダー/バックエンドから約100行
- **全体寄与**: Phase 15目標（80k→20k）の約1%

## 📊 構造変更の全体像

### 現状（3層構造）
```
1. コアBox（nyrt内蔵）← 完全削除
   - StringBox, IntegerBox, BoolBox
   - ArrayBox, MapBox
   - 特別な最適化パス・予約型保護

2. プラグインBox（.so/.dll）
   - FileBox, NetBox, ConsoleBox等
   - FFI TypeBox v2インターフェース

3. ユーザー定義Box（Nyashコード）
   - アプリケーション固有Box
```

### 最終形（2層構造）
```
1. プラグインBox（.so/.dll）← デフォルト動作
   - StringBox, IntegerBox（元コア）
   - FileBox, NetBox等（既存プラグイン）
   - 統一されたFFI TypeBox v2インターフェース

2. ユーザー定義Box（Nyashコード）
   - アプリケーション固有Box
   - 将来：StringBox等もNyashコード実装
```

## 🚀 実装計画

### Phase A: 予約型保護解除（1週目）

#### 1. 予約型保護の条件付き解除
```rust
// src/box_factory/mod.rs: is_reserved_type()修正
fn is_reserved_type(name: &str) -> bool {
    // 環境変数でプラグイン優先モード時は保護解除
    if std::env::var("NYASH_USE_PLUGIN_BUILTINS").is_ok() {
        if let Ok(types) = std::env::var("NYASH_PLUGIN_OVERRIDE_TYPES") {
            if types.split(',').any(|t| t.trim() == name) {
                return false;  // 予約型として扱わない
            }
        }
    }

    matches!(name, "StringBox" | "IntegerBox" | ...)
}
```

#### 2. プラグイン版テスト
```bash
# Step 1: 単体テスト
NYASH_USE_PLUGIN_CORE_BOXES=1 ./target/release/nyash test_integer.nyash
NYASH_USE_PLUGIN_CORE_BOXES=1 ./target/release/nyash test_string.nyash

# Step 2: スモークテスト
NYASH_USE_PLUGIN_CORE_BOXES=1 ./tools/jit_smoke.sh

# Step 3: 包括テスト
NYASH_USE_PLUGIN_CORE_BOXES=1 cargo test
```

#### 3. パフォーマンス測定
```bash
# ベンチマーク
./target/release/nyash --benchmark core_vs_plugin.nyash
```

### Phase B: MIRビルダー統一（2週目）

#### 削除対象ファイル・行
1. **src/mir/builder/utils.rs** (行134-156)
   - StringBox/IntegerBox等の特別な型推論削除
   - すべてのBoxを統一的に扱う

2. **src/mir/builder.rs** (行407-424)
   - build_new_expression の特別扱い削除

3. **src/mir/builder/builder_calls.rs**
   - parse_type_name_to_mir の修正

#### 実装変更
```rust
// Before（特別扱いあり）
match class.as_str() {
    "StringBox" => MirType::String,
    "IntegerBox" => MirType::Integer,
    // ...
}

// After（統一）
MirType::Box(class.to_string())
```

### Phase C: 完全統一（3週目）

#### 1. 予約型保護の完全削除
```rust
// src/box_factory/mod.rs: 予約型保護を完全削除
// この関数を削除またはコメントアウト
// fn is_reserved_type(name: &str) -> bool { ... }

// 登録処理から予約型チェックを削除
// if is_reserved_type(type_name) && !factory.is_builtin_factory() {
//     continue; // この部分を削除
// }
```

#### 2. nyrt実装削除
1. **crates/nyrt/src/lib.rs**
   - StringBox関連: 約150行
   - IntegerBox関連: 約50行
   - ArrayBox関連: 約50行
   - MapBox関連: 約50行

2. **crates/nyrt/src/plugin/**
   - array.rs: 143行（完全削除）
   - string.rs: 173行（完全削除）

3. **src/backend/llvm/compiler/codegen/instructions/newbox.rs**
   - コアBox最適化パス削除

#### 3. デフォルト動作の確立
```bash
# 環境変数なしでプラグインBox使用
./target/release/nyash test.nyash  # StringBox = プラグイン版
```

### Phase D: Nyashコード実装（将来）

```nyash
// apps/lib/core_boxes/string_box.nyash
box StringBox {
    data: InternalString  // 内部表現

    birth(init_val) {
        me.data = init_val or ""
    }

    length() {
        return me.data.len()
    }

    concat(other) {
        return me.data + other
    }

    substring(start, end) {
        return me.data.slice(start, end)
    }
}
```

```nyash
// apps/lib/core_boxes/integer_box.nyash
box IntegerBox {
    value: i64

    birth(init_val) {
        me.value = init_val or 0
    }

    get() {
        return me.value
    }

    set(new_val) {
        me.value = new_val
    }
}
```

## 📋 チェックリスト

### 準備段階
- [ ] プラグイン版の存在確認
  - [x] nyash-string-plugin
  - [x] nyash-integer-plugin
  - [x] nyash-array-plugin
  - [x] nyash-map-plugin
  - [ ] nyash-bool-plugin（作成必要？）

### Phase A
- [ ] 環境変数NYASH_USE_PLUGIN_CORE_BOXES実装
- [ ] フォールバック機構実装
- [ ] プラグイン版動作テスト
- [ ] パフォーマンス測定

### Phase B
- [ ] MIRビルダー特別扱い削除
- [ ] 型推論ロジック統一
- [ ] テスト通過確認

### Phase C
- [ ] nyrt StringBox実装削除
- [ ] nyrt IntegerBox実装削除
- [ ] nyrt ArrayBox実装削除
- [ ] nyrt MapBox実装削除
- [ ] LLVM最適化パス削除

### Phase D（将来）
- [ ] Nyashコード版StringBox実装
- [ ] Nyashコード版IntegerBox実装
- [ ] Nyashコード版ArrayBox実装
- [ ] Nyashコード版MapBox実装

## 🎯 成功基準

1. **機能的完全性**: すべてのテストがプラグイン版で通過
2. **パフォーマンス**: FFIオーバーヘッドが30%以内
3. **コード削減**: 700行以上の削減達成
4. **アーキテクチャ**: 3層→2層の簡潔化達成

## ⚠️ リスクと対策

| リスク | 影響 | 対策 |
|-------|------|------|
| FFIオーバーヘッド | 10-30%性能低下 | 静的リンク最適化 |
| プラグイン依存関係 | 配布複雑化 | 単一EXE生成（.a静的リンク） |
| デバッグ困難 | FFI境界問題 | 詳細ログ・トレース追加 |

## 📝 関連ドキュメント
- [MIR Call統一計画](mir-call-unification-master-plan.md)
- [Phase 15 全体計画](README.md)
- [セルフホスティング計画](self-hosting-plan.txt)

## 💡 技術的洞察

ChatGPTの「コアBox削除」提案は、以下の理由で革命的：

1. **全チェック地獄の解消**: 3層構造による複雑な型チェック削除
2. **統一ABI**: すべてのBoxが同じインターフェース
3. **静的リンク活用**: .so→.a変換で性能維持
4. **セルフホスティング加速**: Nyashコード化への道筋

この統一により、Phase 15の80k→20k行削減目標に向けた重要な一歩となる。