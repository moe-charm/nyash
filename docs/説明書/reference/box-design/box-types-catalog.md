# 📦 Nyash Box型カタログ

## 📋 概要

Nyashで利用可能なすべてのBox型の完全カタログです。
各Box型の用途、API、使用例を網羅しています。

## 🏗️ Box型の分類

### 📊 基本型Box（Primitive Boxes）

#### StringBox
文字列を扱う基本Box型。

```nyash
local str = "Hello, Nyash!"  // 自動的にStringBox
local explicit = new StringBox("Explicit creation")

// メソッド
str.length()         // 文字数を取得
str.toUpperCase()    // 大文字変換
str.split(",")       // 文字列分割
str.contains("Nya")  // 部分文字列検索
```

#### IntegerBox
整数を扱う基本Box型。

```nyash
local num = 42              // 自動的にIntegerBox  
local big = new IntegerBox(1000000)

// メソッド
num.add(10)          // 加算
num.multiply(2)      // 乗算
num.toString()       // 文字列変換
num.isEven()         // 偶数判定
```

#### FloatBox
浮動小数点数を扱うBox型。

```nyash
local pi = 3.14159          // 自動的にFloatBox
local precise = new FloatBox(2.718281828)

// メソッド
pi.round(2)          // 小数点以下2桁に丸める
pi.ceil()            // 切り上げ
pi.floor()           // 切り下げ
```

#### BoolBox
真偽値を扱うBox型。

```nyash
local flag = true           // 自動的にBoolBox
local condition = new BoolBox(false)

// メソッド
flag.not()           // 論理反転
flag.and(condition)  // 論理AND
flag.or(condition)   // 論理OR
```

#### NullBox
null値を表すBox型。

```nyash
local nothing = null        // 自動的にNullBox

// メソッド
nothing.isNull()     // 常にtrue
nothing.toString()   // "null"
```

### 📚 コレクション型Box（Collection Boxes）

#### ArrayBox
動的配列を扱うBox型。

```nyash
local arr = new ArrayBox()
arr.push(1)
arr.push(2)
arr.push(3)

// メソッド
arr.length()         // 要素数
arr.get(0)           // インデックスアクセス
arr.set(1, 42)      // 要素設定
arr.pop()            // 末尾削除
arr.slice(1, 3)     // 部分配列
arr.forEach(callback) // 反復処理
```

#### MapBox
キー・値ペアを扱う連想配列Box型。

```nyash
local map = new MapBox()
map.set("name", "Nyash")
map.set("version", "1.0")

// メソッド
map.get("name")      // 値取得
map.has("version")   // キー存在確認
map.keys()           // 全キー取得
map.values()         // 全値取得
map.forEach(callback) // 反復処理
```

### 🖥️ システムBox（System Boxes）

#### ConsoleBox
コンソール入出力を扱うBox型。

```nyash
local console = new ConsoleBox()

// メソッド
console.log("Hello!")        // 標準出力
console.error("Error!")      // エラー出力
console.read()               // 標準入力
console.clear()              // 画面クリア
```

#### FileBox
ファイル操作を扱うBox型。

```nyash
local file = new FileBox("data.txt")

// メソッド
file.read()                  // ファイル読み込み
file.write("content")        // ファイル書き込み
file.append("more")          // 追記
file.exists()                // 存在確認
file.delete()                // 削除
```

#### TimeBox
時刻・日付を扱うBox型。

```nyash
local time = new TimeBox()

// メソッド
time.now()                   // 現在時刻
time.format("YYYY-MM-DD")    // フォーマット
time.add(1, "day")           // 日付計算
time.diff(otherTime)         // 時間差
```

#### MathBox
数学関数を提供するBox型。

```nyash
local math = new MathBox()

// メソッド
math.sqrt(16)                // 平方根
math.pow(2, 10)              // 累乗
math.sin(math.PI / 2)        // 三角関数
math.random()                // 乱数
```

### 🌐 ネットワークBox（Network Boxes）

#### SocketBox
TCP/UDPソケット通信を扱うBox型。

```nyash
// サーバー
local server = new SocketBox()
server.bind("0.0.0.0", 8080)
server.listen(10)
local client = server.accept()

// クライアント
local client = new SocketBox()
client.connect("localhost", 8080)
client.write("Hello")
```

#### HTTPServerBox
HTTPサーバー機能を提供するBox型。

```nyash
local server = new HTTPServerBox()
server.bind("0.0.0.0", 3000)
server.route("/", handler)
server.start()
```

#### P2PBox
P2P通信を実現するBox型。

```nyash
local node = new P2PBox("node1", "testnet")
node.connect()
node.send("broadcast", data)
node.onReceive(handler)
```

### 🎨 GUI Box（GUI Boxes）

#### EguiBox
GUIアプリケーション作成用Box型。

```nyash
local app = new EguiBox()
app.setTitle("My App")
app.setSize(800, 600)
app.addButton("Click me", callback)
// app.run()  // メインスレッド制約
```

### 🔌 特殊Box（Special Boxes）

#### FutureBox
非同期処理を扱うBox型。

```nyash
local future = new FutureBox(asyncTask)
future.then(onSuccess)
future.catch(onError)
future.await()  // 同期的に待機
```

#### WeakBox
弱参照を提供するBox型。

```nyash
local weak = new WeakBox(target)
local strong = weak.upgrade()  // 通常参照に変換
if strong != null {
    // targetはまだ生きている
}
```

#### ExternBox
外部ライブラリ統合用Box型。

```nyash
local console = new ExternBox("console")
console.call("log", "External API call")

local dom = new ExternBox("document")
dom.call("getElementById", "myDiv")
```

#### DebugBox
デバッグ支援機能を提供するBox型。

```nyash
local debug = new DebugBox()
debug.startTracking()
debug.trackBox(myObject, "My Object")
print(debug.memoryReport())
```

## 🚀 Box型の選び方

### 用途別ガイド
- **テキスト処理**: StringBox
- **数値計算**: IntegerBox, FloatBox, MathBox
- **データ構造**: ArrayBox, MapBox
- **I/O操作**: ConsoleBox, FileBox
- **ネットワーク**: SocketBox, HTTPServerBox, P2PBox
- **GUI**: EguiBox
- **非同期**: FutureBox
- **メモリ管理**: WeakBox, DebugBox

### パフォーマンス考慮
- 基本型は軽量で高速
- コレクション型は要素数に注意
- ネットワーク型は非同期推奨
- GUI型はメインスレッド制約

---

関連ドキュメント：
- [Everything is Box](everything-is-box.md)
- [ビルトインBox詳細API](../builtin-boxes.md)
- [Box実装ガイド](memory-management.md)