# 🎯 Nyash デリゲーション文法設計 2AI大会議記録

日時: 2025年8月10日
参加者: Claude（司会）、Gemini、ChatGPT
議題: 継承廃止後のデリゲーション文法設計

## 📋 前提・制約

### 決定事項
- 継承は廃止、デリゲーションに全面移行
- Everything is Box哲学を維持

### Nyashの設計思想・制約
1. **明示性重視**: プログラマーが変数の宣言先を即座に特定可能
2. **superは現在禁止**: 混乱を避けるため
3. **コンストラクタオーバーロード不可**: 明示的初期化を強制
4. **init/local宣言**: 変数のスコープを明確化

## 🌟 Gemini先生の提案

### 核心アイデア: `delegates to`構文

```nyash
// デリゲートされる側 (振る舞いを定義したBox)
type Movable {
    init x = 0
    init y = 0

    fn move(dx, dy) {
        self.x = self.x + dx
        self.y = self.y + dy
        print("Moved to: ", self.x, ", ", self.y)
    }

    fn position() {
        return [self.x, self.y]
    }
}

// デリゲートする側
type Player {
    // 1. デリゲート先のインスタンスを、フィールドとして明示的に初期化
    init pos = Movable.new()
    
    // 2. `delegates to` キーワードで、どのフィールドにデリゲートするかを宣言
    delegates to self.pos

    // Player固有のフィールド
    init name = "Nyash"

    // Player固有のメソッド
    fn greet() {
        print("Hello, I'm ", self.name)
    }

    // 3. デリゲートされたメソッドをオーバーライド (任意)
    override fn move(dx, dy) {
        print(self.name, " is moving...")
        // 4. デリゲート先のオリジナルメソッドを明示的に呼び出す
        self.pos.move(dx, dy) 
    }
}
```

### 設計哲学
- **責任の所在が明確**: `init pos = Movable.new()`でデリゲート先の実体が明確
- **デリゲート関係が明確**: `delegates to self.pos`で委譲関係を明示宣言
- **暗黙の動作がない**: プログラマが意図して宣言する形式

### 質問への回答
1. **明示性哲学に最適**: `delegates to <field_name>`構文
2. **super不要**: `self.<field_name>.<method_name>()`で呼び出し
3. **初学者向け**: 3ステップが明確（部品作成→フィールド保持→委譲宣言）
4. **Everything is Box**: Boxのコンポジション（合成）そのもの

## 💻 ChatGPT先生の提案

### 核心アイデア: `delegate exposes`構文

```nyash
// 基本形
box Service {
    init repo: Repo
    
    delegate repo exposes RepoAPI  // インターフェース単位
    
    override save(item) {
        validate(item)
        repo.save(item)  // 明示的な呼び出し
    }
}

// 詳細制御
box ComplexService {
    init backend: Logger
    
    delegate backend exposes { find, save as saveRepo }  // 個別列挙
    delegate cache exposes RepoAPI prefix cache_         // プレフィックス付与
    
    override save(item) {
        validate(item)
        backend.save(item)
    }
}
```

### 設計要素
- **フィールド宣言**: `init repo: Repo`
- **デリゲート指定**: `delegate repo exposes RepoAPI`
- **細かい制御**: 個別列挙、プレフィックス、衝突回避
- **明示的呼び出し**: `delegateName.method(args...)`

### 実装アプローチ
- 名前解決: 自分 → 明示転送先（単一解決のみ）
- super不採用: 名前での明示呼び出し
- 衝突解決: `as`/`prefix`/`override`で明示

## 🤝 2AI合意事項

### 共通方針
1. ✅ **super禁止維持** - `self.fieldName.method()`で明示呼び出し
2. ✅ **init宣言との整合性** - デリゲート先もinitフィールドとして宣言
3. ✅ **overrideキーワード** - 上書きの意図を明確化
4. ✅ **Everything is Box** - デリゲート先はBoxフィールド

### 文法比較

| 観点 | Gemini提案 | ChatGPT提案 |
|------|------------|-------------|
| シンプルさ | 🟢 超シンプル | 🟡 やや複雑 |
| 制御力 | 🟡 基本のみ | 🟢 細かい制御 |
| 学習コスト | 🟢 低い | 🟡 中程度 |
| 明示性 | 🟢 明確 | 🟢 明確 |

## 🎯 Claude統合提案

### 段階的アプローチ

**Phase 1: シンプル形式（Gemini案ベース）**
```nyash
box ChatNode {
    init p2p = P2PBox("alice", world)
    init nodeId = "alice"
    
    delegates p2p  // シンプルに！
    
    override send(intent, data, target) {
        print("[" + me.nodeId + "] " + intent)
        me.p2p.send(intent, data, target)
    }
}
```

**Phase 2: 詳細制御（ChatGPT案要素）**
```nyash
box ComplexService {
    init primary = ServiceA.new()
    init secondary = ServiceB.new()
    
    delegates primary            // 全メソッド委譲
    delegates secondary only {   // 選択的委譲
        backup as doBackup,
        sync as syncData
    }
    
    override process(data) {
        me.primary.process(data)
        me.secondary.backup(data)
    }
}
```

### 最終推奨文法

**基本パターン（80%のケース）**：
```nyash
box MeshNode {
    init p2p = P2PBox(nodeId, world)
    init routing = RoutingTable()
    
    delegates p2p  // これだけ！
    
    override send(intent, data, target) {
        me.routing.log(target)
        me.p2p.send(intent, data, target)
    }
}
```

## 🎉 期待される効果

### Before（現在の冗長さ）
```nyash
box MeshNode {
    init { p2p, routing, nodeId }
    
    constructor(id, world) {
        me.p2p = new P2PBox(id, world)
        me.routing = new RoutingTable()
        me.nodeId = id
    }
    
    // 全メソッド手動転送...
    send(i, d, t) { return me.p2p.send(i, d, t) }
    broadcast(i, d) { return me.p2p.broadcast(i, d) }
    on(i, c) { return me.p2p.on(i, c) }
    off(i) { return me.p2p.off(i) }
    // ... 延々と続く
}
```

### After（提案後）
```nyash
box MeshNode {
    init p2p = P2PBox(nodeId, world)
    init routing = RoutingTable()
    
    delegates p2p  // たった1行で全メソッド使える！
    
    override send(intent, data, target) {
        me.routing.log(target)
        me.p2p.send(intent, data, target)
    }
}
```

**劇的な簡潔化**: 20行以上 → 5行程度

## 📝 実装計画

1. **ASTに`delegates`ノード追加**
2. **パーサーで`delegates fieldName`認識**
3. **未定義メソッドの自動転送生成**
4. **`me.fieldName.method()`でのアクセス確保**

## 📋 結論

**全員一致**：
1. シンプルさを最優先に`delegates fieldName`構文採用
2. 段階的学習可能な設計
3. NyaMeshライブラリが格段に簡潔に

---
記録者: Claude
承認: Gemini, ChatGPT