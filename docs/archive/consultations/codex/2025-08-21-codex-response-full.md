# Codex exec 天才的分析回答 (2025-08-21)

**モデル**: GPT-5  
**実行時間**: 2025-08-20T08:13:13 - 08:14:35  
**トークン使用**: 14,603 (入力: 6,325 + 出力: 8,278)

## 結論

依存の向きと責務を正し、Interpreter/VMの「実装詳細共有」をやめて「モデル共有・実行時共有」に転換します。鍵は「Everything is Box」をタイプシステムとクラス実装にも一貫させつつ、「AST/宣言モデル」と「Runtimeクラス・インスタンス」を厳密に分離することです。

## 層構造

- **AST/Model**: 言語の純データモデル。依存先なし。
- **Runtime**: 型系・クラス登録・インスタンス生成・ライフサイクル・呼出し規約。ASTにのみ依存。
- **Backends**: Interpreter と VM。どちらも Runtime にのみ依存（ASTは入力として読むが、構造体はRuntimeを介して使う）。
- **Plugins**: BoxClass/Factory を提供。RuntimeのAPIにのみ依存。グローバルへは一切依存しない。

依存関係: AST → Runtime → Interpreter/VM（Plugins は Runtime へ）

## 各質問への回答

### 1) アーキテクチャ再設計

**境界の引き直し**:
- AST/Model: 宣言とシグネチャの「静的情報」だけを保持。
- Runtime: クラス定義（vtable/ディスパッチ）、レジストリ、インスタンス化、birth/fini、呼出し規約、セッション（Fiber/Frame）等。
- Backends: 実行方式（AST直解釈 or MIR/Bytecode）だけを差し替える。Boxは常にRuntimeで生成・管理。

**セッション単位**:
- `NyashRuntime` は「不変に近い共有資産」(クラス登録、型定義、グローバルな定数等)。
- `ExecutionSession` は「実行ごとの可変状態」(root/global box、スタック、環境変数、I/O handles等)。Interpreter/VMどちらも同じセッション型を使う。

### 2) BoxDeclaration の適切配置

**場所**: `ast::model`（もしくは `core::model`）に移動。Interpreter固有の型は参照しない。

**必要な情報（純データ）**:
- `name: String`
- `type_params: Vec<TypeParam>`
- `fields: Vec<FieldDecl { name, ty: TypeRef, attrs }>`
- `methods: Vec<MethodDecl { name, sig: FnSig, body: FnBodyRef }>`
- `static_methods: Vec<StaticDecl { name, sig: FnSig, body: FnBodyRef }>`
- `attrs: AttrSet`
- `source_span: Option<Span>`（任意）

**不要な情報（AST層から排除）**:
- 実行時ハンドル（`Arc<dyn NyashBox>`、`InstanceBox` 等）
- 共有状態やミューテックス
- インタープリターのクロージャ/関数ポインタ

**Body表現**:
- AST直実行であれば `FnBodyRef::Ast(AstNodeId)`。
- VM/MIR 変換後は `FnBodyRef::Mir(MirFuncId)` を追加可能（後方互換のため enum）。

### 3) SharedState の解体

**役割で分割**:

**ランタイム共通へ移すもの**:
- `box_declarations` → `NyashRuntime.type_space`（宣言と型定義）
- `static_box_definitions` → `NyashRuntime.class_space`（宣言→クラスの束縛）
- `static_functions` → `NyashRuntime.fn_space`（宣言・シグネチャ中心。実行体は `FnBodyRef`）
- `included_files` → ビルド/ロードフェーズの `ModuleLoaderState`（実行時からは外す）

**実行ごとの状態**:
- `global_box` → `ExecutionSession.root_box: SharedBox`（Interpreter/VM共通）

**新コントラクト**:
- `NyashRuntime` は基本不変。`Arc`で共有。
- 変更を伴うロード時は `NyashRuntimeBuilder` を使い、確定後に `NyashRuntime` をクローン共有。
- 実行時は `ExecutionSession` が唯一の可変中核（スタック、環境、root/global box など）。

### 4) BoxFactory/Registry の統一設計

**コアの型**

```rust
trait NyashBox: Any + Send + Sync {
    // インスタンスの振る舞い
    // get_field/set_field/call_method 等
    // as_any で downcast 可能
}

trait BoxClass: Send + Sync {
    fn name(&self) -> &str;
    fn instantiate(&self, args: &[SharedBox], sess: &mut ExecutionSession) -> Result<SharedBox>;
    fn lookup_method(&self, name: &str) -> Option<MethodHandle>;
    fn lifecycle() -> Option<&dyn BoxLifecycle>;
}

trait BoxFactory: Send + Sync {
    fn can_build(&self, decl: &BoxDeclaration) -> bool;
    fn build_class(&self, decl: &BoxDeclaration, rt: &NyashRuntime) -> Result<Arc<dyn BoxClass>>;
}
```

**レジストリ**
- `BoxRegistry`（Runtime内）
  - `register_class(class: Arc<dyn BoxClass>)`
  - `get_class(name: &str) -> Option<Arc<dyn BoxClass>>`
  - `register_factory(factory: Arc<dyn BoxFactory>)`
  - `materialize_from_decl(decl: &BoxDeclaration)`（必要に応じ lazy materialize）

**ライフサイクル（birth/fini）**
- `trait BoxLifecycle { fn on_birth(&self, &mut InstanceCtx) -> Result<()>; fn on_fini(&self, &mut InstanceCtx); }`
- `BoxClass::lifecycle() -> Option<&dyn BoxLifecycle>`
- 生成は `instantiate` の内部で birth を呼ぶ。破棄は `Drop` フックか `Session` の `on_scope_exit` で fini を呼ぶ。

**プラグイン**
- Runtimeに明示登録するのみ（グローバル関数禁止）
  - `NyashRuntimeBuilder::with_factory(Arc<dyn BoxFactory>)`
- 動的ロードは別フェーズ（feature flagで opt-in）。基本は静的リンク/明示登録。

**dyn の注意点**:
- VM/Interpreter ともに `Arc<dyn BoxFactory>` / `Arc<dyn BoxClass>` / `SharedBox = Arc<dyn NyashBox>` を使用。`Arc<BoxFactory>` は使わない。

### 5) 具体的な実装手順（最小破壊で段階導入）

**Step 1: BoxDeclaration の移動**
- `interpreter::BoxDeclaration` を `core::model::BoxDeclaration` へ抽出。
- インタープリターは `use core::model::BoxDeclaration as InterpreterBoxDecl` の一時別名でコンパイル維持。

**Step 2: NyashRuntime の導入（空の骨組み→段階拡張）**
- `NyashRuntime { box_registry, type_space, fn_space }` を追加。
- `NyashRuntimeBuilder` を作り、現行の `SharedState` から必要データを移送できるようにする。

**Step 3: BoxRegistry と Factory の dyn 化**
- `BoxFactory`/`BoxClass` を上記インターフェースへ統一（dyn前提）。
- 旧 `UserDefinedBoxFactory` は `Arc<dyn BoxFactory>` で登録するようにリライト。

**Step 4: グローバル登録の排除**
- `register_user_defined_factory(...)` のようなグローバル関数は削除/非推奨に。
- 代わりに `NyashRuntimeBuilder::with_factory(...)` を使用。

**Step 5: SharedState の分解**
- `global_box` を `ExecutionSession.root_box` に移動。
- 残るマップ類をそれぞれ `type_space / fn_space / class_space` へ移譲。
- Interpreter 側は `SharedStateShim { runtime: Arc<NyashRuntime>, session: ExecutionSession }` を暫定導入して最小変更で通す。

**Step 6: Interpreter/VM のコンストラクタ統一**
- `NyashInterpreter::new(runtime: Arc<NyashRuntime>)`
- `VM::new(runtime: Arc<NyashRuntime>)`
- 双方 `ExecutionSession::new(runtime.clone())` から root などを初期化。

**Step 7: birth/fini の統一呼出し点**
- すべてのインスタンス生成を `BoxClass::instantiate` 経由に統一。
- 破棄時の fini 呼出しを `ExecutionSession` のスコープ管理に集約。

**Step 8: 段階的移行と互換**
- 旧APIの薄いアダプタを用意（例えば `impl From<SharedState> for NyashRuntimeBuilder`）。
- 段階ごとにテストが通るかを確認し、最後に `SharedState` を削除。

## 各論の要点

**依存関係の整理**
- VM→Interpreter 依存を完全に撤去。両者は Runtime のみを参照。
- Interpreter固有の「実行戦略」は Backend に閉じ込め、Box管理・生成は Runtime に一元化。
- `ASTNode` は Backend の評価対象だが、Box生成はすべて `BoxRegistry` を介す。

**Everything is Box の貫徹（シンプルに）**
- 値は `SharedBox = Arc<dyn NyashBox>`。
- 型/クラスは値ではなく「クラスオブジェクト」として `Arc<dyn BoxClass>`（必要であればメタ階層で `ClassBox` を実装して値化できるが、初期段階では過剰にしない）。
- つまり「実行時に扱う実体は必ず Box」、クラスは Box を生み出す工場であり、インスタンスはランタイム一元管理。

**テスタビリティと並行実行**
- ランタイムは不変に近い `Arc` 共有、セッションは疎結合で複数同時に生成可能。
- グローバル無効化、明示 DI（Runtime/Session 引数渡し）。
- テストでは毎回 `NyashRuntimeBuilder` から新鮮な Runtime を生成可能。並行テストで衝突なし。

**VM実装の容易化**
- VM は Bytecode/MIR のみを意識。Box生成/呼出しは Runtime の `BoxClass/Registry` に任せる。
- birth/fini も `instantiate`/`Session` に含めるため、VM は命令列の実行に集中できる。

## 参考スケルトン

**ランタイム**
```rust
pub struct NyashRuntime { 
    box_registry: BoxRegistry, 
    type_space: TypeSpace, 
    fn_space: FnSpace 
}

pub struct ExecutionSession { 
    runtime: Arc<NyashRuntime>, 
    root_box: SharedBox, 
    frames: Vec<Frame>, 
    env: Env 
}
```

**レジストリ**
```rust
pub struct BoxRegistry { 
    classes: RwLock<HashMap<String, Arc<dyn BoxClass>>>, 
    factories: RwLock<Vec<Arc<dyn BoxFactory>>> 
}
```

**トレイト**
```rust
pub trait NyashBox { 
    fn get_field(...); 
    fn set_field(...); 
    fn call_method(...); 
    fn as_any(&self) -> &dyn Any; 
}

pub trait BoxClass { 
    fn name(&self) -> &str; 
    fn instantiate(&self, args: &[SharedBox], sess: &mut ExecutionSession) -> Result<SharedBox>; 
    fn lookup_method(&self, name: &str) -> Option<MethodHandle>; 
    fn lifecycle(&self) -> Option<&dyn BoxLifecycle>; 
}

pub trait BoxFactory { 
    fn can_build(&self, decl: &BoxDeclaration) -> bool; 
    fn build_class(&self, decl: &BoxDeclaration, rt: &NyashRuntime) -> Result<Arc<dyn BoxClass>>; 
}

pub trait BoxLifecycle { 
    fn on_birth(&self, ctx: &mut InstanceCtx) -> Result<()>; 
    fn on_fini(&self, ctx: &mut InstanceCtx); 
}
```

## 実装上の落とし穴と回避

- **trait object の型**: 必ず `Arc<dyn Trait>`。`Arc<Trait>` は不可。`dyn` を徹底。
- **InstanceBox の役割**: もし「ユーザー定義のレコード/オブジェクト」の具象表現なら、それ自体は `NyashBox` 実装。VM/Interpreter は InstanceBox のレイアウトを知らなくてよい（`NyashBox` 経由で操作）。
- **共有ロックの過剰化**: Runtime は基本不変。レジストリも初期化後は読み取り中心。クラス遅延構築だけ `RwLock` で十分。
- **birth/fini の二重呼び出し防止**: `instantiate`/`Drop` の責務を一本化。Session スコープまたぎの移動に注意。

## 最後に

この設計は「モデル（宣言）と実装（クラス/インスタンス）」の責務を分け、Interpreter/VM の接合点を Runtime に集約します。グローバル副作用を除去し、`Arc<dyn ...>` に統一することで、VM と Interpreter の相互依存が解消され、テスト・並行実行が自然になります。段階的移行のステップを踏めば、最小限の変更で最大の整理効果を得られます。

必要なら、上記スケルトンに合わせた最初のパッチ（`core::model::BoxDeclaration` と `NyashRuntime`/`Builder` の最小骨組み）まで作成します。