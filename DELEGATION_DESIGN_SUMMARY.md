# 🎯 Nyash デリゲーション設計サマリー

作成日: 2025年8月10日
状態: 設計完了・実装待ち

## 🎉 決定事項

### **基本方針**
- ✅ **継承完全廃止** → デリゲーション全面移行
- ✅ **Everything is Box哲学維持**
- ✅ **明示性重視の文法**

### **最終採用文法**
```nyash
// 基本形式（80%のケース）
box MeshNode extends P2PBox {
    init routing = RoutingTable()
    
    constructor(nodeId, world) {
        super(nodeId, world)  // super解禁！
        me.routing = RoutingTable()
    }
    
    override send(intent, data, target) {
        me.routing.log(target)
        super.send(intent, data, target)  // 直感的
    }
}

// 複数デリゲーション（20%のケース）
box ComplexNode extends P2PBox {
    init cache = CacheBox()
    
    delegates cache  // 追加デリゲーション
    
    override send(intent, data, target) {
        me.cache.store(intent, data)
        super.send(intent, data, target)
    }
}
```

## 🌟 3AI大会議結果

### **参加者**
- 🤖 Claude（司会・バランス調整）
- 🌟 Gemini（ユーザー体験重視）
- 💻 ChatGPT（技術実装重視）

### **提案比較**
| 提案者 | 文法 | 特徴 |
|--------|------|------|
| Gemini | `delegates to self.pos` | シンプル・直感的 |
| ChatGPT | `delegate repo exposes API` | 細かい制御・柔軟性 |
| **採用案** | `extends` + `super` | 馴染みやすさ・学習コスト最小 |

## 🚀 実装すべき機能

### **Phase 1: 基本デリゲーション（最優先）**
```nyash
box SimpleWrapper extends SomeBox {
    constructor(args) {
        super(args)  // 基底初期化
    }
    
    override method() {
        super.method()  // 元実装呼び出し
    }
}
```

### **Phase 2: 複数デリゲーション（中期）**
```nyash
box ComplexWrapper extends PrimaryBox {
    init secondary = SecondaryBox()
    
    delegates secondary
    delegates tertiary only { save, load }  // 選択的
}
```

## 🛠️ 実装方針

### **内部実装**
- `extends`は実際にはデリゲーション
- `super`は内部フィールドへの参照
- 自動メソッド転送生成

### **ASTノード追加**
```rust
// ASTに追加すべき要素
BoxDeclaration {
    extends: Option<String>,    // extends PrimaryBox
    delegates: Vec<String>,     // delegates field1, field2
}

// superキーワード対応
SuperCall {
    method: String,
    arguments: Vec<Expression>,
}
```

## 📈 期待効果

### **NyaMeshライブラリでの改善**
```nyash
// Before: 20行以上の手動ラッピング
box MeshNode {
    // 全メソッド手動転送...
    send(...) { return me.p2p.send(...) }
    broadcast(...) { return me.p2p.broadcast(...) }
    // ...
}

// After: 5行程度
box MeshNode extends P2PBox {
    override send(intent, data, target) {
        me.routing.log(target)
        super.send(intent, data, target)
    }
}
```

**改善率**: 75%以上のコード削減

## 🎯 次のステップ

### **実装優先度**
1. 🔥 **`extends`構文追加**（パーサー・AST）
2. 🔥 **`super`キーワード解禁**（インタープリター）
3. 🔥 **自動メソッド転送**（メソッド解決）
4. 🟡 **複数delegates**（将来拡張）

### **技術的課題**
- [ ] ASTにextends/super追加
- [ ] superの型チェック・安全性
- [ ] メソッド転送の実装
- [ ] テストスイート作成

## 📝 設計哲学

> 「継承の表現力を、デリゲーションの安全性で実現する」

- **見た目は継承**：学習コストを最小化
- **実装はデリゲーション**：安全性を確保
- **Everything is Box**：一貫した設計哲学

---
**次回作業開始時の参照用サマリー完了** 🎉