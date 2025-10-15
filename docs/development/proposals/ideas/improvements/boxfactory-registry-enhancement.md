# BoxFactory Registry機能強化計画

## 📋 概要

**発見日**: 2025-09-30
**優先度**: 🟡 中（Core Box統一化に関連）
**影響範囲**: BoxFactory・Box生成システム

## 🎯 問題

BoxFactoryに2つの未実装TODO：

### 該当箇所

#### 1. `src/box_factory/plugin.rs:47` - プロバイダーリスト取得
```rust
// TODO: Get list from BoxFactoryRegistry
```

**問題**: 現在はハードコードされたリスト使用

#### 2. `src/box_factory/plugin.rs:55` - プロバイダー存在確認
```rust
// TODO: Add method to check if registry has any providers
```

**問題**: プロバイダーの有無を動的に確認できない

## 💡 解決策案

### 背景: BoxFactory構造

現在の実装：
```rust
pub fn get_plugin_providers() -> Vec<String> {
    // TODO: Get list from BoxFactoryRegistry
    vec![
        "StringBox".to_string(),
        "IntegerBox".to_string(),
        "BoolBox".to_string(),
        // ... ハードコード
    ]
}

pub fn has_plugin_providers() -> bool {
    // TODO: Add method to check if registry has any providers
    !get_plugin_providers().is_empty()
}
```

**問題点**:
- 新しいBoxを追加するたびにコード修正必要
- プラグインBoxが自動認識されない
- 実行時のBox一覧取得が不可能

### Option A: BoxFactoryRegistry実装（推奨）

```rust
use std::sync::RwLock;
use std::collections::HashMap;

/// Box生成ファクトリーのレジストリ
pub struct BoxFactoryRegistry {
    // box_name -> factory_function
    factories: RwLock<HashMap<String, BoxFactoryFn>>,
}

type BoxFactoryFn = Box<dyn Fn(&[Value]) -> Result<SharedNyashBox, String> + Send + Sync>;

lazy_static! {
    pub static ref GLOBAL_BOX_FACTORY: BoxFactoryRegistry = BoxFactoryRegistry::new();
}

impl BoxFactoryRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            factories: RwLock::new(HashMap::new()),
        };

        // ビルトインBox登録
        registry.register_builtin();
        registry
    }

    /// ビルトインBox一括登録
    fn register_builtin(&mut self) {
        self.register("StringBox", Box::new(|args| {
            Ok(Arc::new(StringBox::new(/* ... */)))
        }));

        self.register("IntegerBox", Box::new(|args| {
            Ok(Arc::new(IntegerBox::new(/* ... */)))
        }));

        // ... 他のビルトインBoxも同様
    }

    /// Box factory登録
    pub fn register(&self, box_name: &str, factory: BoxFactoryFn) {
        self.factories.write().unwrap()
            .insert(box_name.to_string(), factory);
    }

    /// Box生成
    pub fn create(&self, box_name: &str, args: &[Value]) -> Result<SharedNyashBox, String> {
        let factories = self.factories.read().unwrap();
        let factory = factories.get(box_name)
            .ok_or_else(|| format!("Box '{}' not found", box_name))?;

        factory(args)
    }

    /// 登録済みBox一覧取得
    pub fn get_registered_boxes(&self) -> Vec<String> {
        self.factories.read().unwrap()
            .keys()
            .cloned()
            .collect()
    }

    /// プロバイダー存在確認
    pub fn has_provider(&self, box_name: &str) -> bool {
        self.factories.read().unwrap()
            .contains_key(box_name)
    }

    /// プロバイダー有無確認
    pub fn has_any_providers(&self) -> bool {
        !self.factories.read().unwrap().is_empty()
    }
}
```

**利点**:
- 動的にBox登録可能
- プラグインBoxも自動認識
- スレッドセーフ（RwLock）
- 実行時Box一覧取得

**実装時間**: 4-5時間

### Option B: トレイトベースFactory

```rust
/// Box Factoryトレイト
pub trait BoxFactory: Send + Sync {
    fn box_name(&self) -> &str;
    fn create(&self, args: &[Value]) -> Result<SharedNyashBox, String>;
}

/// レジストリ
pub struct BoxFactoryRegistry {
    factories: RwLock<HashMap<String, Box<dyn BoxFactory>>>,
}

impl BoxFactoryRegistry {
    pub fn register<F: BoxFactory + 'static>(&self, factory: F) {
        self.factories.write().unwrap()
            .insert(factory.box_name().to_string(), Box::new(factory));
    }
}

// 使用例
struct StringBoxFactory;
impl BoxFactory for StringBoxFactory {
    fn box_name(&self) -> &str { "StringBox" }
    fn create(&self, args: &[Value]) -> Result<SharedNyashBox, String> {
        Ok(Arc::new(StringBox::new(/* ... */)))
    }
}

GLOBAL_BOX_FACTORY.register(StringBoxFactory);
```

**利点**:
- トレイト境界で型安全
- Factory自体が構造体（テスト容易）

**欠点**:
- ボイラープレート多い
- 既存コード大幅修正必要

**実装時間**: 6-8時間

### Option C: マクロによる自動登録

```rust
/// Box Factory自動登録マクロ
#[macro_export]
macro_rules! register_box {
    ($box_type:ty, $box_name:expr) => {
        inventory::submit! {
            BoxFactoryEntry {
                name: $box_name,
                factory: |args| Ok(Arc::new(<$box_type>::new(args))),
            }
        }
    };
}

// 使用例
register_box!(StringBox, "StringBox");
register_box!(IntegerBox, "IntegerBox");
// ... プラグインでも使用可能

// 自動収集
pub fn collect_factories() -> Vec<BoxFactoryEntry> {
    inventory::iter::<BoxFactoryEntry>().cloned().collect()
}
```

**利点**:
- 宣言的（1行で登録）
- プラグインでも同様に使用可能
- コンパイル時収集（inventoryクレート）

**欠点**:
- inventory依存追加
- デバッグ困難（マクロ展開）

**実装時間**: 3-4時間

## 🚀 実装ステップ（推奨: Option A）

### Step 1: BoxFactoryRegistry実装 - 4時間
1. Registry構造体実装
2. register/create/get_registered_boxes実装
3. GLOBAL_BOX_FACTORY グローバル変数

### Step 2: ビルトインBox登録 - 2時間
1. register_builtin()実装
2. 全ビルトインBox登録
3. エラーハンドリング

### Step 3: プラグイン統合 - 3時間
1. プラグインローダーで自動登録
2. plugin.nyashbox.tomlから読み込み
3. 動的Box一覧取得

## 📊 影響範囲

### 修正必要ファイル
- `src/box_factory/mod.rs` - Registry実装
- `src/box_factory/plugin.rs` - TODO解消
- `src/box_factory/builtin_impls/` - 各Box登録
- `src/runtime/plugin_loader_v2/` - プラグインBox自動登録
- `src/backend/mir_interpreter/handlers/boxes.rs` - Registry使用

### 依存追加（Option Cの場合）
- `Cargo.toml`: `inventory = "0.3"` （マクロ自動収集）

### テスト追加
- `tests/boxfactory_registry_basic.rs` - 基本動作
- `tests/boxfactory_dynamic_register.rs` - 動的登録
- `tests/boxfactory_plugin_integration.rs` - プラグイン統合
- スモークテスト: 全Box生成確認

## 🎯 成功基準

- ✅ 新しいBox追加時にコード修正不要
- ✅ プラグインBoxが自動認識
- ✅ 実行時Box一覧取得可能
- ✅ has_provider()/has_any_providers()実装
- ✅ スレッドセーフ動作確認
- ✅ 既存のすべてのスモークテストがPASS

## 🔗 関連資料

- [Phase 15.5 Core Box統一計画](../../../../development/roadmap/phases/phase-15.5/)
- [Box Factory設計](../../../../reference/architecture/box-factory-design.md)
- [プラグインシステム](../../../../reference/plugin-system/)

## 📝 補足

**優先度判断**:
- **Phase 15.5 Core Box統一化**と密接に関連
- Box生成システムの一元化に必須
- builtin vs plugin問題の根本解決に寄与

**実装タイミング**: Phase 15.5完了直後、Phase 15.6で実装推奨

**メリット**:
- Box追加時のコード修正箇所削減
- プラグインBox自動認識（開発体験向上）
- 実行時Box一覧取得（REPL等で有用）
- Core Box統一化の基盤強化

**Phase 15.5との関係**:
- Core Box統一で「builtin優先、plugin代替」が確定
- BoxFactoryRegistryで優先度管理可能：
  ```rust
  pub enum BoxSource {
      Builtin,  // 最優先
      Plugin,   // 代替
  }
  ```
- Phase 15.5完了後に実装すると、統一化の恩恵を最大化