# 🌟 Nyash アーキテクチャ再設計提案

*by Codex exec (2025-08-21)*

## 🎯 核心的洞察

**「実装詳細共有」から「モデル共有・実行時共有」への転換**

現在の問題の本質は、InterpreterとVMが「実装詳細」を共有しようとしていること。正しいアプローチは「モデル（宣言）」と「ランタイム（実行環境）」を共有し、実行戦略だけを分離すること。

## 🏗️ 新アーキテクチャ層構造

```
┌─────────────┐
│  AST/Model  │ ← 純粋なデータモデル（依存なし）
└──────┬──────┘
       │
┌──────▼──────┐
│   Runtime   │ ← 型システム・クラス管理・インスタンス生成
└──────┬──────┘
       │
┌──────┴──────┬──────────┬────────────┐
│ Interpreter │    VM    │  Plugins   │
└─────────────┴──────────┴────────────┘
```

### 各層の責務

**AST/Model層**
- 言語の純データモデル
- BoxDeclaration、ASTNode、型シグネチャ
- 実行時情報を含まない

**Runtime層**
- BoxClass/BoxFactoryによる型システム
- インスタンス生成とライフサイクル管理
- メソッドディスパッチと呼び出し規約

**Backend層**
- Interpreter: AST直接実行
- VM: MIR/Bytecode実行
- 両者ともRuntimeを通じてBoxを操作

## 🔧 具体的な設計

### 1. BoxDeclarationの移動

```rust
// core::model::box_declaration.rs
pub struct BoxDeclaration {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub fields: Vec<FieldDecl>,
    pub methods: Vec<MethodDecl>,
    pub static_methods: Vec<StaticDecl>,
    pub attrs: AttrSet,
    pub source_span: Option<Span>,
}

pub struct FieldDecl {
    pub name: String,
    pub ty: TypeRef,
    pub attrs: AttrSet,
}

pub struct MethodDecl {
    pub name: String,
    pub sig: FnSig,
    pub body: FnBodyRef,  // AST or MIR reference
}
```

### 2. NyashRuntimeの導入

```rust
// runtime::mod.rs
pub struct NyashRuntime {
    box_registry: BoxRegistry,
    type_space: TypeSpace,
    fn_space: FnSpace,
}

pub struct ExecutionSession {
    runtime: Arc<NyashRuntime>,
    root_box: SharedBox,
    frames: Vec<Frame>,
    env: Environment,
}

// SharedBox = Arc<dyn NyashBox>
pub type SharedBox = Arc<dyn NyashBox>;
```

### 3. BoxClass/Factoryシステム

```rust
// runtime::box_class.rs
pub trait BoxClass: Send + Sync {
    fn name(&self) -> &str;
    fn instantiate(
        &self, 
        args: &[SharedBox], 
        sess: &mut ExecutionSession
    ) -> Result<SharedBox>;
    fn lookup_method(&self, name: &str) -> Option<MethodHandle>;
    fn lifecycle(&self) -> Option<&dyn BoxLifecycle>;
}

pub trait BoxFactory: Send + Sync {
    fn can_build(&self, decl: &BoxDeclaration) -> bool;
    fn build_class(
        &self, 
        decl: &BoxDeclaration, 
        rt: &NyashRuntime
    ) -> Result<Arc<dyn BoxClass>>;
}

pub trait BoxLifecycle {
    fn on_birth(&self, ctx: &mut InstanceCtx) -> Result<()>;
    fn on_fini(&self, ctx: &mut InstanceCtx);
}
```

### 4. 統一されたBox管理

```rust
// runtime::registry.rs
pub struct BoxRegistry {
    classes: RwLock<HashMap<String, Arc<dyn BoxClass>>>,
    factories: RwLock<Vec<Arc<dyn BoxFactory>>>,
}

impl BoxRegistry {
    pub fn register_class(&self, class: Arc<dyn BoxClass>) {
        // 登録処理
    }
    
    pub fn get_class(&self, name: &str) -> Option<Arc<dyn BoxClass>> {
        // クラス取得
    }
    
    pub fn create_instance(
        &self,
        class_name: &str,
        args: &[SharedBox],
        sess: &mut ExecutionSession
    ) -> Result<SharedBox> {
        let class = self.get_class(class_name)?;
        class.instantiate(args, sess)
    }
}
```

## 📋 実装手順（最小破壊的移行）

### Step 1: BoxDeclarationの移動
```rust
// 1. core::model モジュールを作成
// 2. BoxDeclarationを移動
// 3. インタープリターで一時的に別名を使用
use core::model::BoxDeclaration as InterpreterBoxDecl;
```

### Step 2: NyashRuntimeの骨組み作成
```rust
// 最初は空の実装から始める
pub struct NyashRuntime {
    // 段階的に追加
}

pub struct NyashRuntimeBuilder {
    // SharedStateからの移行を支援
}
```

### Step 3: BoxFactoryのdyn化
```rust
// 現在の trait BoxFactory を使用
// すべて Arc<dyn BoxFactory> として扱う
```

### Step 4: グローバル登録の排除
```rust
// 削除: register_user_defined_factory(...)
// 追加: NyashRuntimeBuilder::with_factory(...)
```

### Step 5: SharedStateの段階的分解
```rust
// 一時的なシム
pub struct SharedStateShim {
    runtime: Arc<NyashRuntime>,
    session: ExecutionSession,
}

// 互換性のためのFrom実装
impl From<SharedState> for SharedStateShim {
    // 移行ロジック
}
```

### Step 6-8: 統一と最適化
- Interpreter/VMのコンストラクタ統一
- birth/finiライフサイクルの一元化
- 最終的なSharedState削除

## 🎯 得られる利点

1. **依存関係の明確化**
   - VM→Interpreter依存が完全に解消
   - 両者はRuntimeのみに依存

2. **テスタビリティ向上**
   - グローバル状態なし
   - 並行テスト可能

3. **保守性向上**
   - 責務が明確に分離
   - 新しいBackend追加が容易

4. **Everything is Box哲学の貫徹**
   - 統一的なBox管理
   - birth/finiライフサイクルの一元化

## ⚠️ 実装上の注意点

1. **trait objectは必ず`Arc<dyn Trait>`**
   - `Arc<Trait>`は使わない
   - dynキーワードを忘れない

2. **段階的移行**
   - 各ステップでテストが通ることを確認
   - 互換性レイヤーを活用

3. **ロックの最小化**
   - Runtimeは基本的に不変
   - 必要最小限のRwLock使用

---

この設計により、Nyashはよりシンプルでエレガントなアーキテクチャとなり、InterpreterとVMの統合が自然に実現されます。