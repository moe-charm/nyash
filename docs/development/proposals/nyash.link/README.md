# nyash.linkシステム設計 - モジュール・依存関係管理革命

## 🎯 設計背景

### 📊 現状調査結果
- **include使用状況**: 主にexamples/text_adventureで10件程度、実用性は限定的
- **usingキーワード**: **未実装**（トークナイザーにも存在しない）
- **namespace設計**: Phase 9.75eで仕様完成、実装待ち

### 🌟 Gemini先生の推奨
> 「技術的に非常に妥当であり、現代的なプログラミング言語の設計として強く推奨される」

**結論**: includeほぼ未使用 + using未実装 = 完全に新設計で進められる！🎉

## 🚀 設計方針

### 💡 基本コンセプト
```
依存関係管理（nyash.link） + モジュールインポート（using） = 完璧な統合
```

### 🎯 他言語成功モデル
- **Rust**: `Cargo.toml + mod/use` - 厳格で分かりやすい
- **Node.js**: `package.json + import/export` - エコシステム成功  
- **Python**: `pyproject.toml + import` - 依存関係分離

## 📋 nyash.linkファイル仕様

### 基本フォーマット
```toml
# nyash.link (プロジェクトルート)
[project]
name = "my-nyash-project"
version = "0.1.0" 
description = "素晴らしいNyashプロジェクト"

[dependencies]
# 標準ライブラリ
nyashstd = { path = "./stdlib/nyashstd.nyash" }

# ユーザーライブラリ
mylib = { path = "./libs/mylib.nyash" }
utils = { path = "./src/utils.nyash" }

# 将来の外部パッケージ（例）
# http_client = { version = "1.0.0", registry = "nyash-pkg" }

[search_paths]
stdlib = "./stdlib/"
libs = "./libs/"
src = "./src/"

[build]
entry_point = "./src/main.nyash"
```

### 依存関係タイプ

#### 1. **ローカル依存**
```toml
[dependencies]
my_module = { path = "./src/my_module.nyash" }
```

#### 2. **標準ライブラリ**
```toml
[dependencies]
nyashstd = { stdlib = true }  # 特別扱い
```

#### 3. **将来の外部パッケージ**
```toml
[dependencies]
awesome_lib = { version = "^1.2.0", registry = "nyash-pkg" }
```

## 🔧 usingシステム設計

### 1. トークナイザー拡張
```rust
// src/tokenizer.rs に追加
pub enum TokenType {
    // 既存...
    USING,           // using (モジュールインポート)
    NAMESPACE,       // namespace (名前空間宣言)
}
```

### 2. パーサー拡張
```rust
// AST拡張
pub enum Statement {
    // 既存...
    UsingStatement {
        module_path: Vec<String>,  // ["nyashstd", "string"]
        alias: Option<String>,     // using nyashstd.string as str
    },
    NamespaceDeclaration {
        name: String,
        body: Vec<Statement>,
    },
}
```

### 3. 基本構文
```nyash
// ===== using構文パターン =====

// パターンA: 名前空間全体
using nyashstd
string.upper("hello")  // nyashstd.string.upper
math.sin(3.14)        // nyashstd.math.sin

// パターンB: 特定機能（将来拡張）
using nyashstd.string
upper("hello")        // string.upperを直接

// パターンC: エイリアス（将来拡張）
using nyashstd.string as str
str.upper("hello")

// パターンD: 完全修飾名（常時利用可能）
nyashstd.string.upper("hello")  // using不要
```

## 📁 推奨ディレクトリ構造

### 基本プロジェクト構造
```
my-nyash-project/
├── nyash.link              # 依存関係定義
├── src/
│   ├── main.nyash         # エントリーポイント
│   ├── utils.nyash        # ユーティリティモジュール
│   └── models/
│       └── user.nyash     # モデル定義
├── libs/                  # プロジェクト固有ライブラリ
│   └── mylib.nyash
├── stdlib/                # 標準ライブラリ（システム配布）
│   └── nyashstd.nyash
└── tests/                 # テストファイル
    └── test_main.nyash
```

### 標準ライブラリ構造
```
stdlib/
├── nyashstd.nyash         # メインエントリー
├── string/
│   └── mod.nyash         # string関連機能
├── math/
│   └── mod.nyash         # 数学関数
├── http/
│   └── mod.nyash         # HTTP関連
└── io/
    └── mod.nyash         # I/O関連
```

## 🔄 動作フロー

### 1. プロジェクト初期化
```bash
# 将来のCLI例
nyash init my-project      # nyash.linkテンプレート生成
cd my-project
```

### 2. 実行時解決
```
main.nyash実行
  ↓
nyash.link読み込み
  ↓
using nyashstd解析
  ↓
./stdlib/nyashstd.nyash読み込み
  ↓
namespace nyashstd解析・登録  
  ↓
string.upper()利用可能
```

### 3. 名前解決アルゴリズム
```
string.upper() 呼び出し
  ↓
1. ローカルスコープ検索
2. usingでインポートされた名前空間検索
3. 完全修飾名として解釈
4. エラー（未定義）
```

## 🧪 実装段階

### Phase 1: 最小実装
```nyash
// ✅ 実装目標
using mylib              // 単純パス解決
mylib.hello()           // 関数呼び出し

// nyash.link
[dependencies]  
mylib = { path = "./mylib.nyash" }
```

### Phase 2: 名前空間サポート
```nyash
// ✅ 実装目標
using nyashstd
string.upper("hello")

// nyashstd.nyash
namespace nyashstd {
    static box string {
        static upper(str) { ... }
    }
}
```

### Phase 3: 高度機能
- エイリアス（`using ... as ...`）
- 選択インポート（`using nyashstd.string`）
- 循環依存検出
- パッケージレジストリ連携

## ⚡ 実装優先順位

### 🚨 Critical（即時）
1. **UsingTokenizer実装** - Token::USINGを追加
2. **基本パーサー** - using文AST構築
3. **nyash.link解析** - TOML読み込み機能

### ⚡ High（今週）
4. **名前解決エンジン** - モジュール→ファイル解決
5. **基本テスト** - using mylib動作確認
6. **エラー処理** - 未定義モジュール等

### 📝 Medium（来週）
7. **namespace構文** - static box解析
8. **標準ライブラリ設計** - nyashstd.nyash作成
9. **完全修飾名** - nyashstd.string.upper()

### 🔮 Future（今後）
10. **IDE連携** - Language Server補完
11. **パッケージマネージャー** - 外部レジストリ
12. **循環依存検出** - 高度エラー処理

## 🎉 期待効果

### 📈 開発体験向上
- **IDE補完**: `ny`→全標準機能表示
- **探索可能性**: モジュール構造が明確
- **エラー削減**: 名前衝突・未定義の事前検出

### 🏗️ プロジェクト管理
- **依存関係明確化**: nyash.linkで一元管理
- **ビルド再現性**: 他環境での確実な動作
- **スケーラビリティ**: 大規模プロジェクト対応

### 🌍 エコシステム発展
- **ライブラリ共有**: 標準化されたモジュール形式
- **コミュニティ成長**: パッケージレジストリ基盤
- **言語成熟度**: モダンな言語仕様

---

**🐾 この設計でNyashが真にモダンなプログラミング言語になるにゃ！**