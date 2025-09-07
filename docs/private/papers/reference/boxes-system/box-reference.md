# 📦 Nyash Box型完全リファレンス

Nyashで利用できる全ビルトインBox型の完全API仕様書です。

## 📋 Box型分類

### 🎯 基本型Box（Primitive Boxes）

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
pi.toString()        // 文字列変換
```

#### BoolBox
真偽値を扱うBox型。

```nyash
local flag = true           // 自動的にBoolBox
local explicit = new BoolBox(false)

// メソッド
flag.toString()      // 文字列変換
flag.not()           // 論理反転
```

#### NullBox
null値を表すBox型。

```nyash
local empty = null          // NullBox
local check = empty.isNull() // true
```

### 🔢 計算・データ処理系

#### MathBox
数学関数を提供するBox型。

```nyash
local math = new MathBox()

// メソッド
math.sin(pi/2)       // サイン関数
math.cos(0)          // コサイン関数  
math.sqrt(16)        // 平方根
math.pow(2, 8)       // べき乗
math.random()        // 乱数生成
```

#### ArrayBox
配列操作を行うBox型。

```nyash
local arr = new ArrayBox()

// メソッド
arr.push("item")     // 要素追加
arr.get(0)           // 要素取得
arr.set(0, "new")    // 要素設定
arr.length()         // 長さ取得
arr.clear()          // 全削除
```

#### MapBox
連想配列（辞書）操作を行うBox型。

```nyash
local map = new MapBox()

// メソッド
map.set("key", "value") // キー・値設定
map.get("key")          // 値取得
map.has("key")          // キー存在確認
map.keys()              // 全キー取得
map.clear()             // 全削除
```

### 🔗 通信・ネットワーク系

#### P2PBox
P2P通信を行うノードを表すBox。

```nyash
// コンストラクタ
local node = new P2PBox(node_id, world)
```

**パラメータ:**
- `node_id` (String): ノードの一意識別子
- `world` (IntentBox): 参加する通信世界

**メソッド:**

##### send(intent, data, target)
```nyash
local result = node.send("greeting", message_data, "target_node_id")
```
- `intent` (String): メッセージの種類
- `data` (Box): 送信するデータ
- `target` (String): 送信先ノードID
- **戻り値:** StringBox("sent")

##### on(intent, callback)
```nyash
node.on("chat", callback_function)
```
- `intent` (String): 監視するメッセージ種類
- `callback` (MethodBox): 受信時に呼ばれる関数

##### off(intent)
```nyash
node.off("chat")
```
- `intent` (String): 解除するメッセージ種類

#### SocketBox
TCP/IPソケット通信を行うBox型。

```nyash
local socket = new SocketBox()

// サーバーモード
socket.listen(8080)      // ポート8080でリッスン
socket.accept()          // 接続受け入れ

// クライアントモード  
socket.connect("localhost", 8080) // 接続
socket.send("Hello")     // データ送信
socket.receive()         // データ受信
socket.close()           // 接続終了
```

### 🖥️ I/O・GUI系

#### ConsoleBox
基本的なコンソールI/Oを行うBox型。

```nyash
local console = new ConsoleBox()

// メソッド
console.log("message")   // 標準出力
console.error("error")   // エラー出力
console.input()          // 標準入力
```

#### FileBox
ファイル操作を行うBox型（プラグイン対応）。

```nyash
local f = new FileBox("data.txt")

// メソッド
f.write("content")       // ファイル書き込み
f.read()                 // ファイル読み込み
f.exists()               // ファイル存在確認
f.close()                // ファイル閉じる
```

#### EguiBox
GUI開発を行うBox型。

```nyash
local app = new EguiBox()

// メソッド
app.setTitle("My App")   // タイトル設定
app.setSize(800, 600)    // サイズ設定
app.run()                // GUI実行
```

### 🎮 特殊・デバッグ系

#### DebugBox
デバッグ・イントロスペクション用Box型。

```nyash
local debug = new DebugBox()

// メソッド
debug.startTracking()    // メモリ追跡開始
debug.trackBox(obj, "desc") // オブジェクト追跡
debug.memoryReport()     // メモリレポート
```

#### RandomBox
乱数生成専用Box型。

```nyash
local rand = new RandomBox()

// メソッド
rand.next()              // 0-1の乱数
rand.nextInt(100)        // 0-99の整数乱数
rand.nextFloat(10.0)     // 0-10の浮動小数点乱数
```

#### TimeBox
時間・日付操作Box型。

```nyash
local time = new TimeBox()

// メソッド
time.now()               // 現在時刻取得
time.format("YYYY-MM-DD") // 時刻フォーマット
time.addDays(7)          // 日数加算
```

## 🔌 プラグインBox

Nyashはプラグインシステムにより、新しいBox型を動的に追加できます。

### プラグイン設定（nyash.toml）
```toml
[plugins]
FileBox = "nyash-filebox-plugin"
DatabaseBox = "nyash-db-plugin"
```

### 型情報管理
```toml
[plugins.FileBox.methods]
write = { args = [{ from = "string", to = "bytes" }] }
read = { args = [] }
```

**詳細**: [プラグインシステム](../plugin-system/)

---

**最終更新**: 2025年8月19日 - Box型リファレンス統合版
**関連ドキュメント**: [Everything is Box哲学](everything-is-box.md) | [プラグインシステム](../plugin-system/)