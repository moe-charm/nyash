# 🌐 P2PBox完全実装 - AI大会議仕様準拠

## 📋 Issue概要

**目標**: NyaMeshP2Pライブラリ実現のためのP2P通信システムを、AI大会議で決定した最新仕様に従って完全実装する

**重要**: 既存の `src/boxes/intent_box.rs` と `src/boxes/p2p_box.rs` は**古い設計**のため、**完全に作り直し**が必要

## 🎯 AI大会議決定事項

### ✅ 採用仕様
- **構造化IntentBox**: `name` + `payload` 形式のメッセージBox
- **個別送信のみ**: `send(to, message)` 固定API
- **明示的デリゲーション**: `from Parent.method()` 統一構文

### ❌ 除外仕様  
- **ブロードキャスト**: 安全性のため完全除外（無限ループリスク回避）
- **関数オーバーロード**: `send(a)` vs `send(a,b)` 分岐不採用

## 🏗️ 新アーキテクチャ設計

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   P2PBox    │◄──►│ MessageBus   │◄──►│ Transport   │
│ (ユーザーAPI) │    │ (ローカル配送)  │    │ (送受信層)   │
└─────────────┘    └──────────────┘    └─────────────┘
       ▲                    ▲                   ▲
       │                    │                   │
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│ IntentBox   │    │ ハンドラ管理   │    │ InProcess   │
│ (構造化MSG)  │    │ ノード登録    │    │ WebSocket   │
└─────────────┘    └──────────────┘    │ WebRTC      │
                                       └─────────────┘
```

## 📦 段階的実装計画

### 🎯 **Phase 1: 基盤実装**

#### **Step 1: IntentBox（構造化メッセージ）**
**ファイル**: `src/boxes/intent_box.rs` (完全作り直し)

```rust
// 新しいIntentBox設計
pub struct IntentBoxData {
    pub name: String,           // "chat.message", "file.share"等
    pub payload: serde_json::Value,  // 任意のJSON data
}
pub type IntentBox = Arc<Mutex<IntentBoxData>>;
```

**実装要件**:
- Arc<Mutex>統一パターン準拠
- BoxCore + NyashBox実装
- serde_json::Value使用

**テストコード**:
```nyash
// tests/phase2/intent_box_test.nyash
local msg = new IntentBox("chat.message", { text: "Hello P2P!" })
local console = new ConsoleBox()
console.log("Name: " + msg.name)     // "chat.message"
console.log("Text: " + msg.payload.text)  // "Hello P2P!"
```

#### **Step 2: MessageBus（プロセス内シングルトン）**
**ファイル**: `src/messaging/message_bus.rs` (新規作成)

```rust
pub struct MessageBusData {
    nodes: HashMap<String, BusEndpoint>,           // ノード登録
    subscribers: HashMap<String, Vec<IntentHandler>>, // ハンドラー管理
}
pub type MessageBus = Arc<Mutex<MessageBusData>>;

impl MessageBusData {
    pub fn global() -> MessageBus  // シングルトンアクセス
    pub fn register_node(&mut self, id: String, endpoint: BusEndpoint)
    pub fn route(&self, to: &str, intent: IntentBox) -> Result<(), SendError>
}
```

#### **Step 3: Transport trait（送受信抽象化）**
**ファイル**: `src/transport/mod.rs` (新規作成)

```rust
pub trait Transport: Send + Sync {
    fn node_id(&self) -> &str;
    fn send(&self, to: &str, intent: IntentBox, opts: SendOpts) -> Result<(), SendError>;
    fn on_receive(&mut self, callback: Box<dyn Fn(IntentEnvelope) + Send + Sync>);
}
```

### 🎯 **Phase 2: InProcess実装**

#### **Step 4: InProcessTransport**
**ファイル**: `src/transport/inprocess.rs` (新規作成)

```rust
pub struct InProcessTransport {
    node_id: String,
    bus: MessageBus,  // MessageBus::global()を使用
}

impl Transport for InProcessTransport {
    // Bus経由の高速ローカル配送実装
}
```

### 🎯 **Phase 3: P2PBox統合**

#### **Step 5: P2PBox基本実装**
**ファイル**: `src/boxes/p2p_box.rs` (完全作り直し)

```rust
pub struct P2PBoxData {
    node_id: String,
    transport: Arc<dyn Transport>,
    bus: MessageBus,  // 全P2PBoxで共有
}
pub type P2PBox = Arc<Mutex<P2PBoxData>>;

impl P2PBoxData {
    pub fn new(node_id: String, kind: TransportKind) -> P2PBox
    pub fn on(&self, intent_name: &str, handler: IntentHandler) -> Result<(), P2PError>
    pub fn send(&self, to: &str, intent: IntentBox) -> Result<(), SendError>
    // ブロードキャストメソッドは実装しない
}
```

## 🧪 包括的テスト要件

### **基本動作テスト**
**ファイル**: `test_p2p_basic_new.nyash`

```nyash
// 2つのノード作成
local node_a = new P2PBox("alice", transport: "inprocess")
local node_b = new P2PBox("bob", transport: "inprocess")

// 受信ハンドラ設定
node_b.on("chat.message", function(intent, from) {
    local console = new ConsoleBox()
    console.log("From " + from + ": " + intent.payload.text)
})

// メッセージ送信
local msg = new IntentBox("chat.message", { text: "Hello P2P!" })
node_a.send("bob", msg)  // → "From alice: Hello P2P!"
```

### **エラーハンドリングテスト**
```nyash
// 存在しないノードへの送信
local result = node_a.send("nonexistent", msg)
// → SendError::NodeNotFound

// 不正なIntentBox
local invalid_msg = "not an IntentBox"
local result = node_a.send("bob", invalid_msg)
// → 型エラー
```

### **パフォーマンステスト**
```nyash
// 大量メッセージ送信テスト
local start_time = new TimeBox()
loop(i < 1000) {
    local msg = new IntentBox("test.performance", { id: i })
    node_a.send("bob", msg)
    i = i + 1
}
local end_time = new TimeBox()
// 実行時間計測
```

## 📁 必要なディレクトリ構成

```
src/
├── boxes/
│   ├── intent_box.rs          # 完全作り直し
│   └── p2p_box.rs            # 完全作り直し
├── messaging/                # 新規作成
│   └── message_bus.rs        # MessageBus実装
└── transport/                # 新規作成
    ├── mod.rs               # Transport trait
    └── inprocess.rs         # InProcessTransport
```

## 🔧 実装時の重要注意点

### **Arc<Mutex>統一パターン厳守**
```rust
// ✅ 正しい統一パターン
pub type IntentBox = Arc<Mutex<IntentBoxData>>;
pub type MessageBus = Arc<Mutex<MessageBusData>>;
pub type P2PBox = Arc<Mutex<P2PBoxData>>;

// ❌ 避けるべき
pub struct IntentBox { ... }  // Arcなし
```

### **BoxCore実装必須**
```rust
impl BoxCore for IntentBox {
    fn box_id(&self) -> u64 { self.lock().unwrap().base.id }
    fn parent_type_id(&self) -> Option<TypeId> { None }
    fn fmt_box(&self, f: &mut fmt::Formatter) -> fmt::Result { ... }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
```

### **エラーハンドリング設計**
```rust
#[derive(Debug, Clone)]
pub enum SendError {
    NodeNotFound(String),    // 宛先ノードが見つからない
    NetworkError(String),    // ネットワークエラー
    SerializationError(String), // JSON変換エラー
    BusError(String),        // MessageBusエラー
}
```

## 🎯 成功の定義

以下のテストが全て通過すること：

1. **基本通信**: ノード間でIntentBoxメッセージ送受信
2. **ハンドラ登録**: `on()` でイベントリスナー正常動作
3. **エラーハンドリング**: 不正な送信先・データで適切エラー
4. **パフォーマンス**: 1000メッセージ/秒以上の送信性能
5. **メモリ安全性**: valgrind等でメモリリーク検出なし

## 📚 参考ドキュメント

- **[P2P_GUIDE.md](docs/P2P_GUIDE.md)** - 設計詳細・使用例
- **[CURRENT_TASK.md](CURRENT_TASK.md)** - 実装状況・優先順位
- **[ai_conference_overload_decision.md](ai_conference_overload_decision.md)** - AI大会議決定事項
- **[docs/reference/override-delegation-syntax.md](docs/reference/override-delegation-syntax.md)** - デリゲーション構文仕様

## 🚀 実装開始

**Priority**: High  
**Assignee**: Copilot  
**Labels**: enhancement, p2p, breaking-change  
**Milestone**: P2P Phase 2 Complete

**最初に取り組むべき**: Step 1 IntentBox の完全作り直し

---

🎉 **この実装により、Nyashは本格的なP2P通信システムを持つ現代的プログラミング言語になります！**