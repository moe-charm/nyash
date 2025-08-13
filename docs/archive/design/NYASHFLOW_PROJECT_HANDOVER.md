# 🎨 NyashFlow プロジェクト引き継ぎドキュメント

## 📅 作成日: 2025-01-09
## 👤 作成者: Claude + ユーザー（にゃ〜）

---

# 🌟 プロジェクト概要

## 🎯 NyashFlowとは
**Nyashプログラミング言語のビジュアルプログラミング環境**
- 「Everything is Box」の哲学を視覚的に表現
- Boxをドラッグ&ドロップでつなげてプログラミング
- 教育的価値の高いツールを目指す

## 🚀 プロジェクトの経緯

### 1️⃣ **始まり：egui研究**
- NyashにGUI機能（EguiBox）を実装
- Windows版メモ帳、エクスプローラー風アプリを作成
- BMPアイコン表示まで成功

### 2️⃣ **ビジュアルプログラミングへの発展**
- eguiの可能性を探る中で、ノードベースUIの構想が生まれる
- 「Everything is Box」を視覚化するアイデア
- 教育現場での活用を想定

### 3️⃣ **CharmFlow v5からの学び**
- ユーザーが以前作成した大規模プロジェクト
- JavaScript + NyaMesh（P2P）で実装
- **失敗から学んだこと**：
  - カプセル化の欠如 → スパゲティコード化
  - 役割分担の不明確 → 保守困難
  - 過剰な機能 → 複雑化

### 4️⃣ **NyashFlowの方向性決定**
- Rust + WebAssemblyで実装
- Nyashとは別プロジェクトとして独立
- シンプルさを最優先

---

# 🏗️ 技術設計

## 📐 アーキテクチャ

### **基本構成**
```
nyashflow/
├── Cargo.toml              # プロジェクト設定
├── src/
│   ├── lib.rs              # ライブラリエントリ
│   ├── main.rs             # デスクトップ版エントリ
│   ├── visual/             # 🎨 ビジュアル表示層
│   │   ├── mod.rs
│   │   ├── node_renderer.rs      # ノード描画
│   │   ├── connection_renderer.rs # 接続線描画
│   │   └── canvas_manager.rs     # キャンバス管理
│   ├── execution/          # ⚡ 実行エンジン層
│   │   ├── mod.rs
│   │   ├── interpreter_bridge.rs  # Nyashインタープリタ連携
│   │   └── data_flow.rs          # データフロー管理
│   ├── interaction/        # 🖱️ ユーザー操作層
│   │   ├── mod.rs
│   │   ├── drag_drop.rs          # ドラッグ&ドロップ
│   │   ├── selection.rs          # 選択処理
│   │   └── context_menu.rs       # 右クリックメニュー
│   ├── model/              # 📦 データモデル層
│   │   ├── mod.rs
│   │   ├── visual_node.rs        # ノード定義
│   │   ├── connection.rs         # 接続定義
│   │   └── project.rs            # プロジェクト管理
│   └── wasm/               # 🌐 WebAssembly層
│       ├── mod.rs
│       └── bridge.rs             # JS連携
├── web/                    # 🌐 Web用リソース
│   ├── index.html
│   ├── style.css
│   └── pkg/                      # wasm-pack出力
└── examples/               # 📚 サンプル
    └── basic_flow.rs
```

### **設計原則**

#### 1. **徹底的なカプセル化**
```rust
pub struct VisualNode {
    // 🔒 すべてプライベート
    id: NodeId,
    node_type: BoxType,
    position: Pos2,
    #[serde(skip)]
    internal_state: NodeState,
}

impl VisualNode {
    // 🌍 公開APIは最小限
    pub fn get_id(&self) -> NodeId { self.id }
    pub fn get_type(&self) -> &BoxType { &self.node_type }
    pub fn set_position(&mut self, pos: Pos2) { 
        // バリデーション付き
        if self.validate_position(pos) {
            self.position = pos;
        }
    }
}
```

#### 2. **明確な責任分離**
```rust
// ❌ 悪い例（CharmFlowの失敗）
impl EverythingManager {
    fn handle_everything(&mut self, event: Event) {
        // 描画もイベントも実行も全部...
    }
}

// ✅ 良い例（単一責任）
impl NodeRenderer {
    pub fn render(&self, node: &VisualNode, ui: &mut Ui) {
        // 描画だけ！
    }
}

impl DragDropHandler {
    pub fn handle_drag(&mut self, event: DragEvent) {
        // ドラッグ処理だけ！
    }
}
```

#### 3. **コード品質の維持**
- 各ファイル100行以内を目標
- 関数は30行以内
- ネストは3階層まで
- 必ずテストを書く

---

# 💻 実装詳細

## 🎨 ビジュアルノードシステム

### **ノードの種類（初期実装）**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum BoxType {
    // 基本Box
    StringBox,
    IntegerBox,
    BoolBox,
    
    // 操作Box
    MathBox,
    ConsoleBox,
    
    // コンテナBox
    ArrayBox,
}

impl BoxType {
    pub fn color(&self) -> Color32 {
        match self {
            BoxType::StringBox => Color32::from_rgb(100, 149, 237),
            BoxType::IntegerBox => Color32::from_rgb(144, 238, 144),
            BoxType::MathBox => Color32::from_rgb(255, 182, 193),
            // ...
        }
    }
    
    pub fn icon(&self) -> &str {
        match self {
            BoxType::StringBox => "📝",
            BoxType::IntegerBox => "🔢",
            BoxType::MathBox => "🧮",
            // ...
        }
    }
}
```

### **接続システム**
```rust
pub struct Connection {
    id: ConnectionId,
    from_node: NodeId,
    from_port: PortId,
    to_node: NodeId,
    to_port: PortId,
}

pub struct Port {
    id: PortId,
    name: String,
    port_type: PortType,
    data_type: DataType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortType {
    Input,
    Output,
}
```

## ⚡ 実行エンジン

### **Nyashインタープリタ連携**
```rust
use nyash::interpreter::{NyashInterpreter, NyashValue};

pub struct ExecutionEngine {
    interpreter: NyashInterpreter,
    node_mapping: HashMap<NodeId, String>, // NodeId → Nyash変数名
}

impl ExecutionEngine {
    pub fn execute_flow(&mut self, nodes: &[VisualNode], connections: &[Connection]) -> Result<(), ExecutionError> {
        // 1. トポロジカルソート
        let sorted_nodes = self.topological_sort(nodes, connections)?;
        
        // 2. Nyashコード生成
        let nyash_code = self.generate_nyash_code(&sorted_nodes, connections);
        
        // 3. 実行
        self.interpreter.execute(&nyash_code)?;
        
        Ok(())
    }
}
```

## 🌐 WebAssembly統合

### **WASM Bridge**
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct NyashFlowApp {
    #[wasm_bindgen(skip)]
    nodes: Vec<VisualNode>,
    #[wasm_bindgen(skip)]
    connections: Vec<Connection>,
}

#[wasm_bindgen]
impl NyashFlowApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            nodes: vec![],
            connections: vec![],
        }
    }
    
    pub fn add_node(&mut self, node_type: &str, x: f32, y: f32) -> u32 {
        // ノード追加処理
    }
    
    pub fn connect_nodes(&mut self, from_id: u32, to_id: u32) -> Result<(), JsValue> {
        // 接続処理
    }
    
    pub fn execute(&self) -> Result<String, JsValue> {
        // 実行処理
    }
}
```

---

# 🚀 開発ロードマップ

## Phase 1: MVP（1-2週間）
- [ ] 基本的なノード表示
- [ ] 3種類のBox（String, Integer, Console）
- [ ] ドラッグでノード移動
- [ ] 接続線の表示
- [ ] 簡単な実行（ConsoleBoxでprint）

## Phase 2: 基本機能（2-3週間）
- [ ] 全基本Boxタイプ実装
- [ ] 接続の作成/削除
- [ ] 右クリックメニュー
- [ ] プロジェクト保存/読み込み（JSON）
- [ ] 実行結果の表示

## Phase 3: WebAssembly対応（2週間）
- [ ] wasm-pack設定
- [ ] Web用UI調整
- [ ] ブラウザでの動作確認
- [ ] GitHubPages公開

## Phase 4: 高度な機能（1ヶ月）
- [ ] カスタムBox作成
- [ ] デバッグ機能（ステップ実行）
- [ ] アニメーション（データフロー可視化）
- [ ] テンプレート機能

---

# 📝 実装上の注意点

## ⚠️ CharmFlowの失敗を避ける

### 1. **過剰な機能を避ける**
- P2P通信 → 不要
- プラグインシステム → Phase 4以降
- 複雑なIntent → 直接的なデータフロー

### 2. **コードレビューポイント**
```rust
// 毎回チェック
- [ ] ファイルが100行を超えていないか？
- [ ] 関数が30行を超えていないか？
- [ ] Private Fieldsを使っているか？
- [ ] 責任が単一か？
- [ ] テストを書いたか？
```

### 3. **定期的なリファクタリング**
- 週1回はコード全体を見直す
- 重複を見つけたら即座に統合
- 複雑になったら分割

## 🧪 テスト戦略

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_creation() {
        let node = VisualNode::new(BoxType::StringBox, Pos2::new(100.0, 100.0));
        assert_eq!(node.get_type(), &BoxType::StringBox);
    }
    
    #[test]
    fn test_connection_validation() {
        // StringBox → ConsoleBoxは接続可能
        assert!(Connection::can_connect(
            &BoxType::StringBox, 
            &PortType::Output,
            &BoxType::ConsoleBox, 
            &PortType::Input
        ));
    }
}
```

---

# 🎯 成功の指標

## 定量的指標
- コード行数：5,000行以内（CharmFlowの1/10）
- ファイル数：50個以内
- テストカバレッジ：80%以上
- 起動時間：1秒以内

## 定性的指標
- 小学生でも使える直感性
- Nyashの哲学が伝わる
- メンテナンスが苦にならない
- 拡張が容易

---

# 🔗 参考資料

## 技術資料
- [egui公式ドキュメント](https://docs.rs/egui)
- [wasm-bindgen book](https://rustwasm.github.io/wasm-bindgen/)
- [Nyashプロジェクト](../../../README.md)

## 設計思想
- CharmFlow v5の経験（反面教師）
- 「Everything is Box」哲学
- シンプル・イズ・ベスト

## 類似プロジェクト
- Scratch（教育的UI）
- Node-RED（フロープログラミング）
- Unreal Engine Blueprint（ゲーム向け）

---

# 💬 最後に

このプロジェクトは「プログラミングを視覚的に理解する」という夢を実現するものです。

CharmFlowの失敗から学び、Nyashの哲学を活かし、シンプルで美しいツールを作りましょう。

**「Everything is Box」が「Everything is Visible Box」になる瞬間を楽しみにしています！**

にゃ〜🎨✨

---

# 🔮 P2PBox/intentbox設計の活用（2025-01-09追記）

## 🎯 NyaMesh設計から学ぶこと

### **核心概念の抽出**

NyaMeshの`P2PBox`と`intentbox`から、NyashFlowに活用できる**本質的な設計思想**：

1. **intentbox = 通信世界の定義**
   - プロセス内、WebSocket、メモリ共有など
   - 通信の「場」を抽象化

2. **P2PBox = その世界に参加するノード**
   - どのintentboxに所属するかで通信相手が決まる
   - シンプルなsend/onインターフェース

### **NyashFlowへの応用（シンプル版）**

```rust
// ⚡ ローカル実行モード（Phase 1-2）
pub struct LocalExecutionContext {
    // ビジュアルノード間のデータフロー管理
    data_bus: DataFlowBus,
}

// 🌐 将来の拡張（Phase 4以降）
pub trait ExecutionContext {
    fn send_data(&mut self, from: NodeId, to: NodeId, data: NyashValue);
    fn on_data(&mut self, node: NodeId, callback: DataCallback);
}

// 異なる実行コンテキストの実装例
impl ExecutionContext for LocalExecutionContext { ... }
impl ExecutionContext for RemoteExecutionContext { ... }  // WebSocket経由
impl ExecutionContext for SharedMemoryContext { ... }     // 高速共有メモリ
```

### **段階的な導入計画**

#### Phase 1-2: シンプルなデータフロー
```rust
// 最初はシンプルに
pub struct DataFlowEngine {
    nodes: HashMap<NodeId, VisualNode>,
    connections: Vec<Connection>,
}

impl DataFlowEngine {
    pub fn execute(&mut self) {
        // 単純な同期実行
        for connection in &self.connections {
            let data = self.get_output_data(connection.from_node);
            self.set_input_data(connection.to_node, data);
        }
    }
}
```

#### Phase 3-4: 抽象化された実行コンテキスト
```rust
// P2PBox的な抽象化を導入
pub struct VisualNodeBox {
    id: NodeId,
    context: Box<dyn ExecutionContext>,  // どの「世界」で実行するか
}

impl VisualNodeBox {
    pub fn send(&self, data: NyashValue, to: NodeId) {
        self.context.send_data(self.id, to, data);
    }
    
    pub fn on_receive<F>(&mut self, callback: F) 
    where F: Fn(NyashValue) + 'static {
        self.context.on_data(self.id, Box::new(callback));
    }
}
```

### **実用的な応用例**

#### 1. **マルチスレッド実行（ローカル）**
```rust
// 重い処理を別スレッドで
let math_context = ThreadedExecutionContext::new();
let math_node = VisualNodeBox::new(BoxType::MathBox, math_context);
```

#### 2. **リアルタイムコラボレーション（将来）**
```rust
// WebSocketで他のユーザーと共有
let collab_context = WebSocketContext::new("wss://nyashflow.example.com");
let shared_node = VisualNodeBox::new(BoxType::SharedBox, collab_context);
```

#### 3. **デバッグモード**
```rust
// すべてのデータフローを記録
let debug_context = RecordingContext::new();
// 後でデータフローを再生・分析可能
```

### **設計上の重要な判断**

1. **最初はローカル実行のみ**
   - P2P機能は作らない（CharmFlowの教訓）
   - でも将来の拡張性は確保

2. **インターフェースの統一**
   - send/onのシンプルなAPIを維持
   - 実行コンテキストは隠蔽

3. **段階的な複雑性**
   - Phase 1-2: 同期的なローカル実行
   - Phase 3: 非同期実行対応
   - Phase 4: リモート実行（必要なら）

### **実装の指針**

```rust
// ❌ 避けるべき実装（CharmFlow的）
struct EverythingNode {
    p2p_manager: P2PManager,
    intent_bus: IntentBus,
    websocket: WebSocket,
    // ... 100個の機能
}

// ✅ 推奨される実装（NyashFlow的）
struct VisualNode {
    data: NodeData,
    // 実行コンテキストは外部から注入
}

struct ExecutionEngine {
    context: Box<dyn ExecutionContext>,
    // コンテキストを差し替え可能
}
```

### **まとめ：「いいとこ取り」の精神**

- **P2PBox/intentboxの優れた抽象化**を参考に
- **最初はシンプルに**実装
- **将来の拡張性**を設計に組み込む
- **過剰な機能は避ける**

これにより、NyashFlowは：
- 初期は単純なビジュアルプログラミング環境
- 必要に応じて高度な実行モデルに拡張可能
- CharmFlowの失敗を繰り返さない

---

## 📋 チェックリスト（開発開始時）

- [ ] このドキュメントを読み終えた
- [ ] Nyashプロジェクトをビルドできる
- [ ] eguiのサンプルを動かした
- [ ] プロジェクトフォルダを作成した
- [ ] 最初のコミットをした

頑張ってにゃ〜！🚀