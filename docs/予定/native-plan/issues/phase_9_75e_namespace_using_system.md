# Phase 9.75e: namespace & using システム実装

## 🎯 背景・目的

IDE補完機能との相性を最優先にした、現代的な名前空間・インポートシステムの実装。

### 問題意識
- プレリュード方式：IDE補完が効かない、探索可能性が低い
- 全機能明示：冗長、タイプ数が多い
- 理想：`ny` と打つだけで全標準機能が補完される

### 目標
```nyash
# IDE補完完璧
nyashstd.string.upper("hello")  # ny → 全候補表示

# using文で簡潔
using nyashstd
string.upper("hello")  # 短い＆明確
math.sin(3.14)        # 探索可能性維持
```

## 📋 要求仕様

### 1. namespace構文
```nyash
# ファイル：nyashstd.nyash
namespace nyashstd {
    static box string {
        static upper(str) {
            return StringBox.upper(str)  # 既存実装活用
        }
        static lower(str) { ... }
        static split(str, sep) { ... }
    }
    
    static box math {
        static sin(x) { ... }
        static cos(x) { ... }
    }
}
```

### 2. using構文（2パターン）
```nyash
# パターンA: 完全インポート
using nyashstd
string.upper("hello")
math.sin(3.14)

# パターンB: 選択インポート（将来拡張）
using nyashstd.string
using nyashstd.math
string.upper("hello")
math.sin(3.14)
```

### 3. 完全修飾名（常時利用可能）
```nyash
# using なしでも常に使える
nyashstd.string.upper("hello")
nyashstd.math.sin(3.14)
```

## 🔧 技術的課題

### A. パーサー拡張
1. **namespace宣言解析**
   - `namespace identifier { ... }` 構文
   - ネストしたstatic box解析
   - スコープ管理

2. **using文解析**
   - `using namespace_path` 構文
   - ファイル先頭での使用制限
   - 重複インポート検出

3. **修飾名解析**
   - `identifier.identifier.identifier` 構文
   - 名前解決の段階的処理

### B. インタープリター/VM拡張
1. **名前空間レジストリ**
   - グローバル名前空間管理
   - 階層的名前解決
   - キャッシュ機能

2. **using解決**
   - インポートされた名前の局所化
   - 名前衝突検出・エラー処理
   - スコープ境界管理

### C. ファイル間依存関係システム
```nyash
# ファイル: main.nyash
using nyashstd          # ← nyashstd.nyash の読み込みが必要
string.upper("hello")

# ファイル: nyashstd.nyash
namespace nyashstd { ... }
```

**課題:**
- ファイル読み込み順序の決定
- 循環依存の検出・防止
- 依存関係解決アルゴリズム
- パフォーマンス（キャッシュ・遅延読み込み）

## 🚀 実装方針

### Step 1: パーサー拡張
```rust
// AST拡張
pub enum Statement {
    // 既存...
    NamespaceDeclaration {
        name: String,
        body: Vec<Statement>,
    },
    UsingStatement {
        namespace_path: Vec<String>,  // ["nyashstd", "string"]
    },
}

// 修飾名アクセス
pub enum Expression {
    // 既存...
    QualifiedAccess {
        path: Vec<String>,  // ["nyashstd", "string", "upper"]
        args: Vec<Expression>,
    },
}
```

### Step 2: 名前空間レジストリ
```rust
// グローバル名前空間管理
pub struct NamespaceRegistry {
    namespaces: HashMap<String, NamespaceDefinition>,
    using_imports: HashMap<String, Vec<String>>,  // ファイル別インポート
}

pub struct NamespaceDefinition {
    static_boxes: HashMap<String, StaticBoxDefinition>,
}

pub struct StaticBoxDefinition {
    static_methods: HashMap<String, MethodDefinition>,
}
```

### Step 3: 依存関係解決
```rust
// ファイル依存関係グラフ
pub struct DependencyResolver {
    file_dependencies: HashMap<PathBuf, Vec<PathBuf>>,
    load_order: Vec<PathBuf>,
}

impl DependencyResolver {
    // 循環依存検出
    pub fn detect_cycles(&self) -> Result<(), Vec<PathBuf>>;
    
    // 読み込み順序決定
    pub fn resolve_load_order(&self) -> Result<Vec<PathBuf>, DependencyError>;
}
```

## 🧪 必須テストケース

### 1. 基本動作テスト
```nyash
# test_namespace_basic.nyash
namespace test_ns {
    static box example {
        static hello() {
            return "Hello from namespace!"
        }
    }
}

local result = test_ns.example.hello()
assert(result == "Hello from namespace!")
```

### 2. using文テスト
```nyash
# test_using_basic.nyash
using nyashstd

local upper = string.upper("hello")
assert(upper == "HELLO")

local result = math.sin(0)
assert(result == 0)
```

### 3. 名前衝突テスト
```nyash
# test_name_collision.nyash
using nyashstd

# ❌ これはエラーになるべき
static box string {
    static custom() { return "custom" }
}
# Error: 'string' already imported from nyashstd
```

### 4. 依存関係テスト
```nyash
# File: dependency_test_main.nyash
using dependency_test_lib
local result = helper.process("data")

# File: dependency_test_lib.nyash  
namespace dependency_test_lib {
    static box helper {
        static process(data) { return "processed: " + data }
    }
}
```

### 5. 循環依存エラーテスト
```nyash
# File: circular_a.nyash
using circular_b
# ...

# File: circular_b.nyash
using circular_a  # ← Error: Circular dependency detected
# ...
```

## ✅ 完了条件

### パーサー
- [ ] namespace宣言の正常解析
- [ ] using文の正常解析  
- [ ] 修飾名アクセスの正常解析
- [ ] 構文エラーの適切な報告

### インタープリター/VM
- [ ] 名前空間レジストリ動作
- [ ] using解決機能
- [ ] 名前衝突検出・エラー処理
- [ ] パフォーマンス許容範囲（既存の90%以上）

### 依存関係システム
- [ ] ファイル間依存解決
- [ ] 循環依存検出・エラー報告
- [ ] 適切な読み込み順序決定
- [ ] キャッシュ機能（同一ファイル重複読み込み防止）

### テスト
- [ ] 全テストケース通過
- [ ] エラーケース適切処理
- [ ] IDE補完対応確認（Language Server連携）

## 🔗 関連Phase
- Phase 8.9: birth()統一システム（完了）
- Phase 9: AOT WASM実装（完了）
- Phase 10: 高度メモリ管理（完了）
- **Phase 11**: FFI/外部ライブラリ統合（予定）

## 📝 実装ノート

### 優先順位
1. **High**: パーサー拡張（namespace, using）
2. **High**: 基本名前解決機能
3. **Medium**: 依存関係システム
4. **Low**: パフォーマンス最適化

### 既存コードとの互換性
- 既存のStringBox等は変更なし
- static box string は既存Boxのラッパーとして実装
- 段階的移行可能な設計

---

**🐾 Copilot様、この詳細仕様で namespace & using システムの実装をお願いします！**