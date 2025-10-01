# [Proposal] BID Registry簡略化計画

Status: Backlog (Phase A: cache-only done)

Note: Phase Aとして、nyash.toml のパース結果キャッシュを PluginLoaderV2 に導入（挙動不変・I/O削減）。本ドキュメントの本体リファクタ（レジストリ一本化・責務再配置）は後続フェーズで検討する。

## 📋 概要

**発見日**: 2025-09-30
**優先度**: 🟡 中（プラグインシステム改善）
**影響範囲**: BIDプラグインローダー・LoadedPlugin構造体

## 🎯 問題

BID（Box Interface Definition）レジストリに2つの未実装TODO：

### 該当箇所

#### 1. `src/bid/registry.rs:70` - invoke-onlyプラグイン対応
```rust
// TODO: Update LoadedPlugin to work with invoke-only plugins
```

**問題**: 現在のLoadedPluginはinit/ABIフル機能前提

#### 2. `src/bid/registry.rs:97` - 簡略化LoadedPlugin
```rust
// TODO: Create simplified LoadedPlugin without init/abi
```

**問題**: invoke専用プラグインに複雑な構造が不要

## 💡 解決策案

### 背景: プラグインシステムの複雑性

現在のLoadedPlugin構造：
```rust
pub struct LoadedPlugin {
    pub lib: Library,               // 動的ライブラリハンドル
    pub abi: Box<dyn PluginAbi>,   // ABI関数群
    pub invoke: InvokeFunc,         // invoke関数
    // ... 他多数のフィールド
}
```

**問題点**:
- invoke専用プラグインでもinit/ABIフル実装必須
- 実装負担大（新規プラグイン開発が困難）
- 不要な初期化処理（パフォーマンス低下）

### Option A: LoadedPluginバリアント（推奨）

```rust
pub enum LoadedPlugin {
    /// フル機能プラグイン（init/ABI/invoke）
    Full {
        lib: Library,
        abi: Box<dyn PluginAbi>,
        invoke: InvokeFunc,
        init_func: InitFunc,
        metadata: PluginMetadata,
    },

    /// Invoke専用プラグイン（最小実装）
    InvokeOnly {
        lib: Library,
        invoke: InvokeFunc,
        metadata: PluginMetadata,
    },
}

impl LoadedPlugin {
    /// Invoke実行（共通インターフェース）
    pub fn invoke(&self, box_name: &str, method: &str, args: &[Value]) -> Result<Value, String> {
        match self {
            LoadedPlugin::Full { invoke, .. } => invoke(box_name, method, args),
            LoadedPlugin::InvokeOnly { invoke, .. } => invoke(box_name, method, args),
        }
    }

    /// 初期化（Fullのみ）
    pub fn init(&mut self) -> Result<(), String> {
        match self {
            LoadedPlugin::Full { init_func, .. } => init_func(),
            LoadedPlugin::InvokeOnly { .. } => Ok(()), // 何もしない
        }
    }
}
```

**利点**:
- プラグイン開発者が選択可能
- 既存のフルプラグインと互換
- invoke専用プラグインは最小実装で済む

**実装時間**: 3-4時間

### Option B: トレイト分離

```rust
/// Invoke専用プラグイントレイト
pub trait InvokePlugin {
    fn invoke(&self, box_name: &str, method: &str, args: &[Value]) -> Result<Value, String>;
    fn metadata(&self) -> PluginMetadata;
}

/// フル機能プラグイントレイト
pub trait FullPlugin: InvokePlugin {
    fn init(&mut self) -> Result<(), String>;
    fn abi(&self) -> &dyn PluginAbi;
}

/// レジストリは両方を扱う
pub enum PluginHandle {
    Invoke(Box<dyn InvokePlugin>),
    Full(Box<dyn FullPlugin>),
}
```

**利点**:
- トレイト境界で明確な区別
- 型安全性向上

**欠点**:
- 既存プラグイン全修正必要
- 動的ライブラリローディング複雑化

**実装時間**: 6-8時間

### Option C: ビルダーパターン

```rust
pub struct LoadedPluginBuilder {
    lib: Library,
    invoke: InvokeFunc,
    abi: Option<Box<dyn PluginAbi>>,
    init_func: Option<InitFunc>,
    metadata: PluginMetadata,
}

impl LoadedPluginBuilder {
    pub fn new(lib: Library, invoke: InvokeFunc, metadata: PluginMetadata) -> Self {
        Self {
            lib,
            invoke,
            abi: None,
            init_func: None,
            metadata,
        }
    }

    pub fn with_abi(mut self, abi: Box<dyn PluginAbi>) -> Self {
        self.abi = Some(abi);
        self
    }

    pub fn with_init(mut self, init_func: InitFunc) -> Self {
        self.init_func = Some(init_func);
        self
    }

    pub fn build(self) -> LoadedPlugin {
        LoadedPlugin {
            lib: self.lib,
            invoke: self.invoke,
            abi: self.abi,
            init_func: self.init_func,
            metadata: self.metadata,
        }
    }
}

// 使用例
let plugin = LoadedPluginBuilder::new(lib, invoke, metadata)
    .build(); // invoke専用

let full_plugin = LoadedPluginBuilder::new(lib, invoke, metadata)
    .with_abi(abi)
    .with_init(init_func)
    .build(); // フル機能
```

**利点**:
- 段階的構築可能
- 既存コードへの影響小

**欠点**:
- Optionフィールド多用（null参照リスク）

**実装時間**: 2-3時間

## 🚀 実装ステップ（推奨: Option A）

### Step 1: LoadedPluginバリアント実装 - 3時間
1. enum LoadedPlugin定義
2. Full/InvokeOnlyバリアント実装
3. 共通メソッド（invoke等）実装

### Step 2: ローダー修正 - 2時間
1. プラグインローダーでバリアント判定
2. 自動分岐（init関数存在確認）
3. エラーハンドリング

### Step 3: 既存プラグイン検証 - 2時間
1. 全プラグインで動作確認
2. invoke専用プラグイン作成例
3. ドキュメント更新

## 📊 影響範囲

### 修正必要ファイル
- `src/bid/registry.rs` - LoadedPluginバリアント実装
- `src/runtime/plugin_loader_v2/enabled/loader.rs` - ローダー修正
- `plugins/*/` - 既存プラグイン検証（修正不要）
- `docs/reference/plugin-system/` - ドキュメント更新

### テスト追加
- `tests/plugin_invoke_only.rs` - invoke専用プラグイン
- `tests/plugin_full_feature.rs` - フル機能プラグイン
- スモークテスト: 両方のプラグインタイプ

## 🎯 成功基準

- ✅ invoke専用プラグインが最小実装で動作
- ✅ 既存のフルプラグインが動作継続
- ✅ プラグインローダーが自動判別
- ✅ 既存のすべてのスモークテストがPASS
- ✅ ドキュメント完備（invoke専用プラグイン作成ガイド）

## 🔗 関連資料

- [プラグインシステム設計](../../../../reference/plugin-system/plugin-design.md)
- [BID仕様](../../../../reference/plugin-system/bid-specification.md)
- Phase 15.5 Core Box統一化との連携

## 📝 補足

**優先度判断**:
- 新規プラグイン開発の障壁低減
- Phase 15.5でCore Box統一化が進む中、プラグインシステム簡略化は重要
- **Phase 15.6で実装推奨**

**実装タイミング**: Phase 15.5完了後、Core Box統一が安定してから

**メリット**:
- プラグイン開発者の負担軽減（init/ABI不要）
- シンプルなプラグイン（FileBox等）の実装容易化
- パフォーマンス向上（不要な初期化削減）

**デメリット**:
- LoadedPlugin構造体の複雑化（enum導入）
- 既存コードの修正箇所増加（ただし互換性維持）
