# 📚 Nyash Programming Language - 完全ガイド

**最終更新: 2025年8月12日 - Phase 1完了, P2P実装準備完了**

## 📖 概要

Nyashは「Everything is Box」哲学に基づく革新的なプログラミング言語です。
Rust製インタープリターによる高性能実行と、直感的な構文により、学習しやすく実用的な言語として完成しました。

## 🎯 核心哲学: "Everything is Box"

```nyash
# すべてのデータがBoxとして統一的に表現される
number = 42              # IntegerBox
text = "hello"           # StringBox  
flag = true              # BoolBox
array = new ArrayBox()   # ArrayBox
debug = new DebugBox()   # DebugBox
```

**重要な利点:**
- **統一性**: すべてのデータが共通インターフェース
- **メモリ安全性**: Arc<Mutex>パターンで完全スレッドセーフ
- **拡張性**: 新しいBox型を容易に追加可能

---

## 🔤 言語構文リファレンス

### 🏷️ **変数宣言・スコープ**

```nyash
// ローカル変数宣言
local x, y
local name = "Alice"

// 所有権移転変数（関数戻り値用）
outbox result = compute()

// グローバル変数
global CONFIG = "dev"
```

### 🧮 **演算子**

```nyash
// 算術演算子
a + b, a - b, a * b, a / b

// 比較演算子  
a == b, a != b, a < b, a > b, a <= b, a >= b

// 論理演算子
not condition, a and b, a or b

// Cross-type演算 (Phase 1で完全実装)
10 + 3.14              // → 13.14 (型変換)
"Value: " + 42         // → "Value: 42" (文字列連結)
```

### 🏗️ **Box定義・デリゲーション**

#### 基本Box定義
```nyash
box User {
    init { name, email }  // フィールド宣言
    
    pack(userName, userEmail) {  // 🎁 Box哲学の具現化！
        me.name = userName
        me.email = userEmail  
    }
    
    greet() {
        print("Hello, " + me.name)
    }
}
```

#### デリゲーション (2025-08 革命)
```nyash
box AdminUser from User {  // 🔥 from構文でデリゲーション
    init { permissions }
    
    pack(adminName, adminEmail, perms) {
        from User.pack(adminName, adminEmail)  // 親のpack呼び出し
        me.permissions = perms
    }
    
    override greet() {  // 明示的オーバーライド
        from User.greet()  // 親メソッド呼び出し
        print("Admin privileges: " + me.permissions)
    }
}
```

#### Static Box Main パターン
```nyash
static box Main {
    init { console, result }
    
    main() {
        me.console = new ConsoleBox()
        me.console.log("🎉 Everything is Box!")
        return "Success!"
    }
}
```

### 🔄 **制御構造**

```nyash
// 条件分岐
if condition {
    // 処理
} else {
    // 別処理
}

// ループ（唯一の正しい形式）
loop(condition) {
    // 処理
    if breakCondition {
        break
    }
}
```

---

## 📦 ビルトインBox型

### 基本型
- **StringBox**: 文字列 (`"hello"`)
- **IntegerBox**: 整数 (`42`) 
- **FloatBox**: 浮動小数点数 (`new FloatBox(3.14)`) ✅ Phase 1完了
- **BoolBox**: 真偽値 (`true`, `false`)
- **NullBox**: NULL値 (`null`)

### データ構造
- **ArrayBox**: 配列 (`new ArrayBox()`) ✅ Phase 1で sort/reverse/indexOf/slice 実装
- **MapBox**: 連想配列 (`new MapBox()`)

### ユーティリティ
- **ConsoleBox**: コンソール出力 (`new ConsoleBox()`)
- **DebugBox**: デバッグ機能 (`new DebugBox()`)
- **MathBox**: 数学関数 (`new MathBox()`)
- **TimeBox**: 時刻処理 (`new TimeBox()`)
- **DateTimeBox**: 日時処理 (`new DateTimeBox()`) ✅ Phase 1で完全動作

### 高度機能
- **RandomBox**: 乱数生成
- **BufferBox**: バッファ操作
- **RegexBox**: 正規表現  
- **JSONBox**: JSON処理
- **StreamBox**: ストリーム処理

### P2P通信 (Phase 2実装中)
- **IntentBox**: 構造化メッセージ (実装予定)
- **P2PBox**: P2P通信ノード (実装予定)

---

## 🚀 実用例

### データ処理例
```nyash
// 配列操作
local numbers = new ArrayBox()
numbers.push(3)
numbers.push(1) 
numbers.push(2)
numbers.sort()  // [1, 2, 3]

// 型変換・演算
local result = 10 + new FloatBox(3.14)  // 13.14
print("Result: " + result.toString())
```

### P2P通信例 (将来実装)
```nyash
// P2Pノード作成
local node_a = new P2PBox("alice", transport: "inprocess") 
local node_b = new P2PBox("bob", transport: "inprocess")

// メッセージ受信ハンドラ
node_b.on("chat.message", function(intent, from) {
    print("From " + from + ": " + intent.payload.text)
})

// 構造化メッセージ送信
local msg = new IntentBox("chat.message", { text: "Hello P2P!" })
node_a.send("bob", msg)  // → "From alice: Hello P2P!"
```

---

## ⚠️ 重要な注意点

### 必須のコンマ
```nyash
// ✅ 正しい
init { field1, field2 }

// ❌ 間違い（CPU暴走の原因）
init { field1 field2 }
```

### 変数宣言厳密化
```nyash
// ✅ 明示宣言必須
local x
x = 42

// ❌ 未宣言変数への代入はエラー
y = 42  // Runtime Error + 修正提案
```

注記: Nyash は `var`/`let` をサポートしません。必ず `local` で宣言してから使用します。

### ループ構文統一
```nyash
// ✅ 唯一の正しい形式
loop(condition) { }

// ❌ 削除済み構文
while condition { }  // 使用不可
```

---

## 📈 実装状況 (2025-08-12)

### ✅ Phase 1完了
- FloatBox toString() メソッド
- ArrayBox 改良 (sort/reverse/indexOf/slice)
- Cross-type演算子システム
- 包括的テストスイート (188行)

### 🚧 Phase 2実装中
- IntentBox (構造化メッセージ)
- MessageBus (プロセス内シングルトン)
- P2PBox (P2P通信ノード)

### 📋 将来予定
- WebSocket/WebRTC通信
- 非同期処理 (async/await)
- 追加のBox型拡張

---

**🎉 Nyashで「Everything is Box」の世界を体験しよう！**

📚 **関連ドキュメント:**
- [Getting Started](GETTING_STARTED.md) - 環境構築・最初の一歩
- [P2P Guide](P2P_GUIDE.md) - P2P通信システム完全ガイド
- [Built-in Boxes](reference/builtin-boxes.md) - ビルトインBox詳細リファレンス
