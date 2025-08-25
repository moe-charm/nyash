# Nyash統一Box設計の深い分析と今後の方向性
Status: Research
Created: 2025-08-25
Priority: High
Related: Everything is Box哲学の実装レベルでの完全実現

## 現状の統一Box設計

### 3種類のBoxの存在
1. **ビルトインBox** - Rustで実装（StringBox, IntegerBox, ConsoleBox等）
2. **プラグインBox** - 動的ライブラリで提供（FileBox等の置き換え可能）
3. **ユーザー定義Box** - Nyashコードで定義（box Person等）

### 現在の統一アーキテクチャ
```
UnifiedBoxRegistry（統一レジストリ）
├── BuiltinBoxFactory（優先度1）
├── UserDefinedBoxFactory（優先度2）
└── PluginBoxFactory（優先度3）
```

### 統一の美しさ

1. **透過的な置き換え**
   - 同じ名前のBoxをプラグインで上書き可能
   - 実行時の動的切り替えも可能

2. **統一インターフェース**
   ```rust
   pub trait BoxFactory: Send + Sync {
       fn create_box(&self, name: &str, args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError>;
       fn box_types(&self) -> Vec<&str>;
       fn supports_birth(&self) -> bool;
   }
   ```

3. **優先順位システム**
   - ビルトイン > ユーザー定義 > プラグイン
   - 予約語保護（StringBox等は上書き不可）

## InstanceBoxによる完全統一

### 統一実装の核心
```rust
pub struct InstanceBox {
    pub class_name: String,                       // "StringBox", "Person"等
    pub inner_content: Option<Box<dyn NyashBox>>, // 内包Box（統一！）
    pub fields_ng: Arc<Mutex<HashMap<String, NyashValue>>>,
    pub methods: Arc<HashMap<String, ASTNode>>,
}
```

### 3つの形態を統一的に扱う
```rust
// ビルトイン
InstanceBox::from_any_box("StringBox", Box::new(StringBox::new("hello")))

// プラグイン
InstanceBox::from_any_box("FileBox", plugin_loader.create_box("FileBox"))

// ユーザー定義
InstanceBox::from_declaration("Person", vec!["name", "age"], methods)
```

## 基本メソッドの統一提案

### 問題点
- `toString()` → `to_string_box()` （Rustの命名規則で異なる）
- `type()` → `type_name()` （微妙に異なる）
- 各Boxで個別実装されていて統一感がない

### 解決案：NyashBoxトレイトにデフォルト実装
```rust
pub trait NyashBox: BoxCore + Debug {
    // Nyash標準メソッド（デフォルト実装）
    fn toString(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(&self.to_string_box().value))
    }
    
    fn type(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(self.type_name()))
    }
    
    fn equals(&self, other: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        Box::new(BoolBox::new(self.equals_internal(other.as_ref())))
    }
    
    fn clone(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}
```

### VMでの統一的な呼び出し
```rust
pub(super) fn call_box_method(&self, box_value: Box<dyn NyashBox>, method: &str, args: Vec<Box<dyn NyashBox>>) -> Result<Box<dyn NyashBox>, VMError> {
    // 基本メソッドは全Boxで使える！
    match method {
        "toString" => Ok(box_value.toString()),
        "type" => Ok(box_value.type()),
        "equals" => Ok(box_value.equals(args[0].clone_or_share())),
        "clone" => Ok(box_value.clone()),
        _ => self.call_specific_method(box_value, method, args)
    }
}
```

## Gemini先生からの提案（部分）

### 1. NyashValue Enumの導入
```rust
pub enum NyashValue {
    // 即値（スタック上）
    Void,
    Bool(bool),
    Integer(i64),
    Float(f64),
    
    // ヒープ上の不変値
    String(Arc<String>),
    
    // 複雑なオブジェクト
    Object(Arc<NyashObject>),
}
```

**メリット**:
- 小さな値のヒープアロケーション回避
- パターンマッチによる高速ディスパッチ
- 型安全性の向上

### 2. トレイトの階層化
```rust
// 基本トレイト
pub trait NyashBox: Send + Sync + Debug {
    fn to_string(&self) -> String;
    fn type_name(&self) -> &'static str;
    fn equals(&self, other: &dyn NyashBox) -> bool;
}

// 拡張トレイト
pub trait Comparable: NyashBox {
    fn compare(&self, other: &dyn NyashBox) -> Option<Ordering>;
}

pub trait Arithmetic: NyashBox {
    fn add(&self, other: &dyn NyashBox) -> Result<Box<dyn NyashBox>, String>;
}
```

### 3. メタプログラミング機能
```rust
pub trait BoxMetadata {
    fn on_method_missing(&self, name: &str, args: &[NyashValue]) -> Option<NyashValue>;
    fn on_field_access(&self, name: &str) -> Option<NyashValue>;
}
```

## 統一継承の実現

### 現在の課題
- ビルトインBoxの継承ができない
- プラグインBoxの継承も未実装

### 理想的な統一継承
```nyash
// すべて可能に！
box MyString from StringBox { }      // ビルトイン継承
box MyFile from FileBox { }          // プラグイン継承  
box Employee from Person { }         // ユーザー定義継承
```

## さらなる美化への道

### 1. パイプライン演算子
```nyash
// 現在
local result = str.substring(0, 5).toUpperCase().trim()

// パイプライン版
local result = str
    |> substring(0, 5)
    |> toUpperCase()
    |> trim()
```

### 2. Box階層の整理
```
NyashBox (trait)
├── ValueBox (数値・文字列・真偽値)
│   ├── IntegerBox
│   ├── StringBox
│   └── BoolBox
├── ContainerBox (コレクション)
│   ├── ArrayBox
│   └── MapBox
├── IOBox (入出力)
│   ├── ConsoleBox
│   └── FileBox
└── ConcurrentBox (並行処理)
    ├── FutureBox
    └── ChannelBox
```

### 3. エフェクトシステムの美化
```rust
impl ArrayBox {
    #[effect(Pure)]
    fn length(&self) -> IntegerBox { }
    
    #[effect(State)]
    fn push(&mut self, item: Box<dyn NyashBox>) { }
}
```

## 実装優先順位（80/20ルール）

### 今すぐやるべき（80%）
1. 基本メソッド（toString, type, equals, clone）の統一実装
2. VMでの統一的なメソッドディスパッチ
3. InstanceBoxのinner_content活用の徹底

### 後でじっくり（20%）
1. NyashValue enum導入によるパフォーマンス最適化
2. トレイト階層化による整理
3. パイプライン演算子の実装
4. メタプログラミング機能
5. 完全な統一継承システム

## まとめ

現在のNyashの統一Box設計は、すでに相当美しく実装されている。特に：

1. **UnifiedBoxRegistry**による透過的な管理
2. **優先順位システム**による明確な解決
3. **InstanceBox**による統一的な扱い

これらは「Everything is Box」哲学を実装レベルで体現している。

ユーザー定義Boxをビルトイン/プラグインBoxと完全に同じレベルで扱うことは、強引ではなく、むしろ設計として自然で美しい。この統一により、言語の一貫性と拡張性が大幅に向上する。

今後は、基本メソッドの統一実装から始めて、段階的により洗練された設計へと進化させていくのが良いだろう。