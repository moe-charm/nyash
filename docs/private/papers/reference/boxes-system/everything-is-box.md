# 🌟 Everything is Box - Nyashの核心哲学

## 📦 すべては箱である

Nyashでは、すべての値が「Box」と呼ばれるオブジェクトです。
数値も、文字列も、関数も、そしてBoxそのものも、すべてがBoxです。

```nyash
// これらはすべてBox
local number = 42              // IntegerBox
local text = "Hello"           // StringBox  
local flag = true              // BoolBox
local nothing = null           // NullBox
local container = new MapBox() // MapBox
```

## 🎯 なぜEverything is Boxなのか

### 1. **統一性**
プリミティブ型と参照型の区別がないため、すべてを同じ方法で扱えます。

```nyash
// すべて同じ方法でメソッドを呼べる
local strLen = "Hello".length()      // StringBoxのメソッド
local doubled = 42.multiply(2)       // IntegerBoxのメソッド
local formatted = true.toString()    // BoolBoxのメソッド
```

### 2. **拡張性**
すべてがオブジェクトなので、どんな型にもメソッドを追加できます。

```nyash
// ユーザー定義Boxで数値を拡張
box Money from IntegerBox {
    pack(amount) {
        from IntegerBox.pack(amount)
    }
    
    format() {
        return "$" + me.toString()
    }
}
```

### 3. **一貫性**
型チェック、メソッド呼び出し、デリゲーションがすべて統一的に動作します。

```nyash
// 型チェックも統一的
if value.isType("StringBox") {
    console.log("It's a string!")
}

// nullチェックも同様
if value.isType("NullBox") {
    console.log("It's null!")
}
```

## 🏗️ Box設計の基本原則

### 1. **Boxは不変の契約**
すべてのBoxは`NyashBox`トレイトを実装し、以下のメソッドを提供します：

- `type_name()` - Box型名を返す
- `clone_box()` - Boxの複製を作成
- `as_any()` - 動的型変換用
- `to_string_box()` - StringBox変換

### 2. **メモリ管理の統一**
すべてのBoxは`Arc<Mutex<dyn NyashBox>>`として管理され、自動的にメモリ安全です。

### 3. **明示的な操作**
暗黙的な型変換は行わず、すべての操作を明示的に行います。

```nyash
// ❌ 暗黙的な変換はない
local result = "Hello" + 42  // エラー！

// ✅ 明示的な変換
local result = "Hello" + 42.toString()  // OK: "Hello42"
```

## 📊 Box型の分類

### 基本Box型
- **StringBox** - 文字列
- **IntegerBox** - 整数
- **FloatBox** - 浮動小数点数
- **BoolBox** - 真偽値
- **NullBox** - null値

### コレクションBox型
- **ArrayBox** - 配列
- **MapBox** - 連想配列
- **SetBox** - 集合（予定）

### システムBox型
- **ConsoleBox** - コンソール入出力
- **FileBox** - ファイル操作
- **TimeBox** - 時刻操作
- **MathBox** - 数学関数

### ネットワークBox型
- **SocketBox** - TCP/UDPソケット
- **HTTPServerBox** - HTTPサーバー
- **P2PBox** - P2P通信

### GUI Box型
- **EguiBox** - GUIアプリケーション
- **CanvasBox** - 描画キャンバス

### 特殊Box型
- **FutureBox** - 非同期処理
- **WeakBox** - 弱参照
- **ExternBox** - 外部ライブラリ統合

## 🔄 Boxの生成と利用

### 基本的な生成
```nyash
// newによる明示的生成
local str = new StringBox("Hello")
local num = new IntegerBox(42)

// リテラルによる暗黙的生成
local str = "Hello"  // 自動的にStringBox
local num = 42       // 自動的にIntegerBox
```

### ユーザー定義Box
```nyash
box Point {
    init { x, y }
    
    pack(xVal, yVal) {
        me.x = xVal
        me.y = yVal
    }
    
    distance() {
        return (me.x * me.x + me.y * me.y).sqrt()
    }
}

local p = new Point(3, 4)
console.log(p.distance())  // 5
```

### デリゲーションによる拡張
```nyash
box Point3D from Point {
    init { z }
    
    pack(xVal, yVal, zVal) {
        from Point.pack(xVal, yVal)
        me.z = zVal
    }
    
    override distance() {
        local xy = from Point.distance()
        return (xy * xy + me.z * me.z).sqrt()
    }
}
```

## 🌐 外部世界との統合

Everything is Box哲学は、外部ライブラリにも適用されます。

```nyash
// ExternBoxで外部APIもBoxに
local fetch = new ExternBox("fetch")
local response = fetch.call("get", "https://api.example.com/data")

// JavaScript APIもBoxとして利用
local dom = new ExternBox("document")
local element = dom.call("getElementById", "myDiv")
```

## 🎉 まとめ

Everything is Box哲学により、Nyashは：

1. **シンプル** - すべてが同じルールに従う
2. **強力** - どんなものにもメソッドを追加できる
3. **安全** - 統一的なメモリ管理
4. **拡張可能** - 新しいBox型を簡単に追加
5. **統合的** - 外部ライブラリも同じ方法で利用

この哲学こそが、Nyashを特別な言語にしているのです。

---

関連ドキュメント：
- [Box型カタログ](box-types-catalog.md)
- [デリゲーションシステム](delegation-system.md)
- [メモリ管理](memory-management.md)