# 🎯 Nyash ビルトインBox継承 3AI大会議記録

日時: 2025年8月10日
参加者: Claude（司会）、Gemini、ChatGPT
議題: ビルトインBox（P2PBox等）の継承・拡張システム設計

## 📋 背景・課題

現在のNyashではビルトインBox（P2PBox、StringBox等）は継承できず、コンポジション（内包）パターンで拡張する必要があるが、全メソッドをラップする必要があり記述が冗長。

```nyash
// 現在の冗長な書き方
box ChatNode {
    init { p2p, nodeId }
    
    constructor(nodeId, world) {
        me.p2p = new P2PBox(nodeId, world)
        me.nodeId = nodeId
    }
    
    // 全メソッドを手動でラップ...
    send(intent, data, target) {
        return me.p2p.send(intent, data, target)
    }
    broadcast(intent, data) {
        return me.p2p.broadcast(intent, data)
    }
    // ... 延々と続く
}
```

## 🌟 Gemini先生の提案

### 設計哲学
- ビルトインBoxは「選択的に開く」
- 低レベル・値系（StringBox等）は封印
- 高レベル・参照系（P2PBox等）は継承可能

### 4本柱アプローチ
1. **単一継承**: `extends`キーワード
2. **ミックスイン**: 軽量Trait的な水平合成
3. **拡張メソッド**: Refinement（スコープ付き）
4. **デリゲーション糖衣**: 内包の冗長さ解消

### 文法提案
```nyash
// デリゲーション糖衣
box ChatBox by P2PBox(p2p)  // 未解決メンバはp2pに委譲

// 選択的転送
box ChatBox wraps P2PBox by p2p delegate * except connect, send

// 継承
box ChatBox extends P2PBox with Retryable, Logger {
    override connect(addr) { 
        super.connect(addr)
        self.handshake() 
    }
}

// 拡張メソッド
extend P2PBox in NyaMeshExt { 
    def peer_count(self): Int { 
        self.peers().len 
    } 
}
using NyaMeshExt for P2PBox
```

### 実装優先順位
1. `by`/`delegate`糖衣（AST展開で実装容易）
2. `extend/using`（メソッド解決に拡張集合追加）
3. `open builtin`導入（vtable公開・検査）

## 💻 ChatGPT先生の提案

### 技術的アプローチ
- ビルトインBoxをtraitとして公開
- VTableチェーンによるメソッド解決
- Arc<Mutex>パターンとの統合

### コア設計
```rust
// Trait化
trait P2PApi: Send + Sync {
    fn send(&mut self, ...) -> Result<...>;
    fn broadcast(&mut self, ...) -> Result<...>;
}

// 派生Box（コンパイラ生成）
struct ChatNode {
    base: Arc<Mutex<dyn P2PApi>>,
    fields: ...,
    dispatch: MethodTable
}

// メソッドテーブル
struct MethodTable {
    fn_map: HashMap<MethodId, NativeFnPtr | BytecodeFnRef>,
    overridable: HashSet<MethodId>,
    final: HashSet<MethodId>,
    base: Option<TypeId>
}
```

### ロック戦略
- 派生ロックを保持したままsuperを呼ばない
- `with_super(|p2p| {...})`ヘルパー提供
- drop-before-callパターンをコード生成で強制

### 実装手順
1. ビルトインをtraitにリファクタ
2. TypeRegistryに基底リンケージ追加
3. `extends`パーサー・コード生成
4. invokeチェーン・ロック規律実装

## 🤝 3AI合意事項

### 基本方針
- ✅ **値型は封印、参照型は開放**
- ✅ **デリゲーション優先、継承は必要時のみ**
- ✅ **P2PBoxから段階的導入**

### 統一実装案

#### Phase 1: デリゲーション糖衣（最優先）
```nyash
box ChatNode delegates P2PBox {
    init { nodeId }
    
    new(nodeId, world) {
        super(nodeId, world)  // 基底インスタンス生成
        me.nodeId = nodeId
    }
    
    // 選択的オーバーライド
    override send(intent, data, target) {
        print("Sending: " + intent)
        super.send(intent, data, target)
    }
}
```

#### 実装方法
1. ASTに`delegates`キーワード追加
2. 内部的に`_base`フィールド自動生成
3. 未定義メソッドは`_base`へ自動転送
4. `super`を`me._base`にバインド

### 安全性合意
- **final by default**: 明示的overridable指定
- **ロック順序**: 派生→基底を強制
- **capability**: 危険操作に明示的権限

## 📊 比較表：継承 vs デリゲーション

| 観点 | 継承 | デリゲーション |
|------|------|----------------|
| is-a関係 | ✅ 子は親の一種 | ❌ 別の型として扱われる |
| メソッド解決 | 自動的に親を探索 | 明示的に転送 |
| 型の互換性 | 子を親として使える | 使えない（別の型） |
| 実装の柔軟性 | 親の実装に依存 | 任意の実装を委譲可能 |
| 多重継承 | 通常不可 | 複数オブジェクトに委譲可能 |

## 🎯 最終推奨

### 今すぐ実装
```nyash
// この構文で劇的に簡潔に！
box MeshNode delegates P2PBox {
    init { routing }
    
    new(id, world) {
        super(id, world)
        me.routing = new RoutingTable()
    }
    
    // 必要なものだけオーバーライド
    override send(intent, data, target) {
        me.routing.log(target)
        return super.send(intent, data, target)
    }
}
```

### 効果
- Before: 全メソッド手動ラップ（20行以上）
- After: delegatesで自動転送（5行程度）

## 📝 結論

**全員一致**：
1. delegates構文を最優先実装
2. 簡潔性と安全性の両立
3. NyaMesh開発が格段に容易に

---
記録者: Claude
承認: Gemini, ChatGPT