# ビルトインBox型 API リファレンス

Nyashで利用できる全ビルトインBox型のAPI仕様書です。

## 📡 P2PBox - 通信ノードBox

P2P通信を行うノードを表すBox。通信世界（IntentBox）に参加してメッセージを送受信できます。

### コンストラクタ
```nyash
// 通信ノードを作成
local node = new P2PBox(node_id, world)
```

**パラメータ:**
- `node_id` (String): ノードの一意識別子
- `world` (IntentBox): 参加する通信世界

### メソッド

#### send(intent, data, target)
特定のノードにメッセージを送信します。
```nyash
local result = node.send("greeting", message_data, "target_node_id")
```

**パラメータ:**
- `intent` (String): メッセージの種類
- `data` (Box): 送信するデータ
- `target` (String): 送信先ノードID

**戻り値:** StringBox("sent")

#### on(intent, callback)
指定したintentのメッセージを受信した際のリスナーを登録します。
```nyash
node.on("chat", callback_function)
```

**パラメータ:**
- `intent` (String): 監視するメッセージ種類
- `callback` (MethodBox): 受信時に呼ばれる関数

**戻り値:** StringBox("listener added")

#### off(intent)
指定したintentのリスナーを解除します。
```nyash
node.off("chat")
```

**パラメータ:**
- `intent` (String): 解除するメッセージ種類

**戻り値:** StringBox("listener removed" / "no listener found")

#### get_node_id()
このノードのIDを取得します。
```nyash
local id = node.get_node_id()
```

**戻り値:** StringBox(ノードID)

### 使用例
```nyash
// 通信世界を作成
local world = new IntentBox()

// 2つのノードを作成
local alice = new P2PBox("alice", world)
local bob = new P2PBox("bob", world)

// Bobがgreetingを受信するリスナー設定
bob.on("greeting", greeting_handler)

// AliceからBobにメッセージ送信
local message = new MapBox()
message.set("text", "Hello Bob!")
alice.send("greeting", message, "bob")
```

---

## 📨 IntentBox - 通信世界Box

P2PBoxが通信を行うための世界（ネットワーク）を表すBox。複数のノードが同一のIntentBoxを共有して通信します。

### コンストラクタ
```nyash
// 通信世界を作成
local world = new IntentBox()
```

**パラメータ:** なし

### 特徴
- ローカル通信: 同一プロセス内のP2PBox間でメッセージをやり取り
- スレッドセーフ: Arc<Mutex>により並行アクセス対応
- 将来拡張: WebSocket版や分散版への拡張予定

### 使用例
```nyash
// 1つの通信世界に複数ノードが参加
local world = new IntentBox()
local node1 = new P2PBox("server", world)
local node2 = new P2PBox("client", world)

// 同一世界内での通信が可能
node1.send("data", payload, "client")
```

---

## 📝 StringBox - 文字列Box

文字列データを格納・操作するBox。

### コンストラクタ
```nyash
local text = new StringBox("Hello")
```

### 基本メソッド
- `toString()`: 文字列表現を取得
- `length()`: 文字列長を取得
- `concat(other)`: 文字列結合
- `substring(start, end)`: 部分文字列取得

---

## 🔢 IntegerBox - 整数Box

整数データを格納・操作するBox。

### コンストラクタ
```nyash
local num = new IntegerBox(42)
```

### 基本メソッド
- `toString()`: 文字列表現を取得
- `add(other)`: 加算
- `subtract(other)`: 減算
- `multiply(other)`: 乗算
- `divide(other)`: 除算

---

## 📺 ConsoleBox - コンソール出力Box

コンソールへの出力を行うBox。

### コンストラクタ
```nyash
local console = new ConsoleBox()
```

### メソッド
- `log(message)`: メッセージをログ出力
- `error(message)`: エラーメッセージを出力

---

## 🗂️ MapBox - 連想配列Box

キー・バリューペアでデータを格納するBox。

### コンストラクタ
```nyash
local map = new MapBox()
```

### メソッド
- `set(key, value)`: キー・バリューを設定
- `get(key)`: 値を取得
- `has(key)`: キーが存在するかチェック
- `remove(key)`: キー・バリューを削除

## 📊 BufferBox - バイナリデータ処理Box

バイナリデータの読み書きを効率的に処理するBox。ファイル操作、ネットワーク通信、画像処理で使用。

### コンストラクタ
```nyash
// 空のバッファを作成
local buffer = new BufferBox()
```

### 基本メソッド
- `write(data)`: バイトデータ書き込み (ArrayBox[integers])
- `read(count)`: 指定バイト数読み取り → ArrayBox
- `readAll()`: 全データ読み取り → ArrayBox
- `clear()`: バッファクリア → StringBox("ok")
- `length()`: データサイズ取得 → IntegerBox
- `append(buffer)`: 他BufferBoxを追加 → IntegerBox(新サイズ)
- `slice(start, end)`: 部分データ取得 → BufferBox

### ⭐ Phase 10: 高度なメモリ管理API

#### ゼロコピー検出API
```nyash
// ゼロコピー共有の検出
local buffer1 = new BufferBox()
local shared_buffer = buffer1.share_reference(null)

// 共有検出
local is_shared = buffer1.is_shared_with(shared_buffer)  // → BoolBox(true)
```

- `is_shared_with(other)`: 他BufferBoxとのメモリ共有を検出 → BoolBox
- `share_reference(data)`: Arc参照を共有した新BufferBoxを作成 → BufferBox  
- `memory_footprint()`: 現在のメモリ使用量を取得 → IntegerBox(bytes)

#### 実装詳細
- **Arc::ptr_eq()**: 真のポインタ共有検出でゼロコピーを保証
- **共有状態**: `share_reference()`で作成されたBufferは元のデータを共有
- **独立性**: `clone_box()`は完全に独立したコピーを作成

### 使用例
```nyash
// HTTP転送でのゼロコピー検証
static box ProxyServer {
    relay_data(client_data) {
        if (me.upstream_buffer.is_shared_with(client_data)) {
            print("✅ Zero-copy achieved!")
        }
        return me.upstream_buffer.share_reference(client_data)
    }
}
```

---

最終更新: 2025年8月15日 (Phase 10: BufferBox高度メモリ管理API追加)