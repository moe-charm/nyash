# Nyash開発ガイド for Claude

Nyashプログラミング言語開発に必要な情報をまとめたクイックリファレンス。

## 🚀 クイックスタート

### 🐧 Linux/WSL版
```bash
# ビルドと実行
cargo build --release
./target/release/nyash program.nyash
```

### 🪟 Windows版 (NEW!)
```bash
# クロスコンパイルでWindows実行ファイル生成
cargo install cargo-xwin
cargo xwin build --target x86_64-pc-windows-msvc --release

# 生成された実行ファイル (916KB)
target/x86_64-pc-windows-msvc/release/nyash.exe
```

### 🌐 WebAssembly版
```bash
# ブラウザープレイグラウンド
cd projects/nyash-wasm
./build.sh
# nyash_playground.html をブラウザーで開く
```

## 📚 ドキュメント構造

### 🎯 よく使う情報
- **[構文早見表](docs/quick-reference/syntax-cheatsheet.md)** - 基本構文・よくある間違い
- **[演算子一覧](docs/quick-reference/operators-summary.md)** - 実装済み演算子
- **[開発コマンド](docs/quick-reference/development-commands.md)** - build/test/AI相談

### 📊 最新開発状況
- **[実装状況](docs/status/current-implementation.md)** - 完全な機能実装状況
- **[最新成果](docs/status/recent-achievements.md)** - 2025-08-08更新
- **[既知の問題](docs/status/known-issues.md)** - 制限事項・回避策

### 📖 詳細リファレンス
- **[完全リファレンス](docs/reference/)** - 言語仕様詳細
  - [予約語一覧](docs/reference/keywords.md)
  - [演算子リファレンス](docs/reference/operators.md)
  - [ビルトイン型](docs/reference/built-in-boxes.md)
  - [MethodBox（invoke）](docs/reference/method-box-reference.md)
  - [ジェネリクス](docs/reference/generics-reference.md)
- **[学習ガイド](docs/language-guide/)** - 体系的学習用

### 🎮 実用例・アプリ
- **[実用例](docs/examples/)** - サンプルコード・パターン集
- **実装済みアプリ**: サイコロRPG・統計計算・LISPインタープリター

## ⚡ 重要な設計原則

### 🏗️ Everything is Box
- すべての値がBox（StringBox, IntegerBox, BoolBox等）
- ユーザー定義Box: `box ClassName { init { field1, field2 } }`

### 🌟 完全明示デリゲーション（2025-08-11革命）
```nyash
// デリゲーション構文
box Child from Parent {  // from構文でデリゲーション
    init(args) {  // コンストラクタは「init」に統一
        from Parent.init(args)  // 親の初期化
    }
    
    override method() {  // 明示的オーバーライド必須
        from Parent.method()  // 親メソッド呼び出し
    }
}
```

### 🔄 統一ループ構文
```nyash
// ✅ 唯一の正しい形式
loop(condition) { }

// ❌ 削除済み構文
while condition { }  // 使用不可
loop() { }          // 使用不可
```

### 🎁 pack構文 - Box哲学の具現化（2025-08-11実装）
```nyash
// 🎁 「箱に詰める」直感的コンストラクタ
box User {
    init { name, email }
    
    pack(userName, userEmail) {  // ← Box哲学を体現！
        me.name = userName
        me.email = userEmail
    }
}

// 🔄 デリゲーションでのpack
box AdminUser from User {
    init { permissions }
    
    pack(adminName, adminEmail, perms) {
        from User.pack(adminName, adminEmail)  // 親のpackを呼び出し
        me.permissions = perms
    }
}

// ✅ 優先順位: pack > init > Box名形式
local user = new User("Alice", "alice@example.com")  // packが使われる
```

### 🎯 正統派Nyashスタイル（2025-08-09実装）
```nyash
// 🚀 Static Box Main パターン - エントリーポイントの統一スタイル
static box Main {
    init { console, result }  // フィールド宣言
    
    main() {
        // ここから始まる！他の言語と同じエントリーポイント
        me.console = new ConsoleBox()
        me.console.log("🎉 Everything is Box!")
        
        // local変数も使用可能
        local temp
        temp = 42
        me.result = temp
        
        return "Revolution completed!"
    }
}
```

### 📝 変数宣言厳密化システム（2025-08-09実装）
```nyash
// 🔥 すべての変数は明示宣言必須！（メモリ安全性・非同期安全性保証）

// ✅ static box内のフィールド
static box Calculator {
    init { result, memory }  // 明示宣言
    
    calculate() {
        me.result = 42  // ✅ フィールドアクセス
        
        local temp     // ✅ local変数宣言
        temp = me.result * 2
    }
}

// ✅ static関数内の所有権移転
static function Factory.create() {
    outbox product  // 呼び出し側に所有権移転
    product = new Item()
    return product
}

// ❌ 未宣言変数への代入はエラー
x = 42  // Runtime Error: 未宣言変数 + 修正提案
```

### ⚡ 実装済み演算子（Production Ready）
```nyash
// 論理演算子（完全実装）
not condition    // NOT演算子
a and b         // AND演算子  
a or b          // OR演算子

// 算術演算子
a / b           // 除算（ゼロ除算エラー対応済み）
a + b, a - b, a * b  // 加算・減算・乗算
```

### ⚠️ 重要な注意点
```nyash
// ✅ 正しい書き方
init { field1, field2 }  // カンマ必須（CPU暴走防止）

// ❌ 間違い
init { field1 field2 }   // カンマなし→CPU暴走
```

## 🎨 GUI開発（NEW!）

### EguiBox - GUIアプリケーション開発
```nyash
// EguiBoxでGUIアプリ作成
local app
app = new EguiBox()
app.setTitle("Nyash GUI App") 
app.setSize(800, 600)

// 注意: 現在メインスレッド制約により
// app.run() は特別な実行コンテキストが必要
```

**実装状況**: 基本実装完了、GUI実行コンテキスト対応中

## 🔧 開発サポート

### 🤖 AI相談
```bash
# Gemini CLIで相談
gemini -p "Nyashの実装で困っています..."
```

### 🧪 テスト実行
```bash
# 基本機能テスト
cargo test

# 演算子統合テスト
./target/debug/nyash test_comprehensive_operators.nyash

# 実用アプリテスト
./target/debug/nyash app_dice_rpg.nyash
```

### 🐛 デバッグ

#### パーサー無限ループ対策（NEW! 2025-08-09）
```bash
# 🔥 デバッグ燃料でパーサー制御
./target/release/nyash --debug-fuel 1000 program.nyash      # 1000回制限
./target/release/nyash --debug-fuel unlimited program.nyash  # 無制限
./target/release/nyash program.nyash                        # デフォルト10万回

# パーサー無限ループが検出されると自動停止＋詳細情報表示
🚨 PARSER INFINITE LOOP DETECTED at method call argument parsing
🔍 Current token: IDENTIFIER("from") at line 17
🔍 Parser position: 45/128
```

**対応状況**: must_advance!マクロでパーサー制御完全実装済み✅  
**効果**: 予約語"from"など問題のあるトークンも安全にエラー検出

#### アプリケーション デバッグ
```nyash
// DebugBox活用
DEBUG = new DebugBox()
DEBUG.startTracking()
DEBUG.trackBox(myObject, "説明")
print(DEBUG.memoryReport())
```

## 📚 ドキュメント再編成戦略

### 🎯 現在の課題
- **CLAUDE.md肥大化** (500行) - 必要情報の検索困難
- **情報分散** - 実装状況がCLAUDE.md/current_task/docsに分散
- **参照関係不明確** - ファイル間の相互リンク不足

### 🚀 新構造プラン
```
docs/
├── quick-reference/          # よく使う情報（簡潔）
│   ├── syntax-cheatsheet.md     # 構文早見表
│   ├── operators-summary.md     # 演算子一覧
│   └── development-commands.md  # 開発コマンド集
├── status/                   # 最新開発状況
│   ├── current-implementation.md  # 実装状況詳細
│   ├── recent-achievements.md     # 最新成果
│   └── known-issues.md            # 既知の問題
├── reference/                # 完全リファレンス（現存活用）
└── examples/                 # 実用例（現存拡充）
```

### ⚡ 実装優先順位
1. **Phase 1**: CLAUDE.md簡潔化（500行→150行ハブ）
2. **Phase 2**: 基本構造作成・情報移行
3. **Phase 3**: 相互リンク整備・拡充

### 🎉 期待効果
- **検索性**: 必要情報への高速アクセス
- **メンテナンス性**: 責任分離・局所的更新
- **拡張性**: 新機能追加が容易

**📋 詳細**: [DOCUMENTATION_REORGANIZATION_STRATEGY.md](DOCUMENTATION_REORGANIZATION_STRATEGY.md)

---

最終更新: 2025年8月11日 - **🎁 `pack`構文革命完全達成！**
- **Everything is Packed**: Gemini・ChatGPT両先生一致推薦の`pack`コンストラクタ採用
- **Box哲学具現化**: 「箱に詰める」直感的メタファーでコードを書くたび哲学体験
- **完全実装**: `pack()`、`from Parent.pack()`、`pack`>`init`>Box名順優先選択
- **革命的UX**: 他言語の`new`/`init`を超越、Nyash独自アイデンティティ確立
- **デリゲーション完成**: `box Child from Parent`+`pack`+`override`+`from`統合完了