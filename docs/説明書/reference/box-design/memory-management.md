# 🧠 Nyash メモリ管理設計

## 📋 概要

Nyashは「Everything is Box」哲学のもと、すべての値をBoxとして統一的に管理します。
メモリ安全性を保証しながら、シンプルで直感的なメモリ管理を実現しています。

## 🏗️ 基本アーキテクチャ

### Arc<Mutex>一元管理

```rust
// インタープリターレベルでの統一管理
type NyashObject = Arc<Mutex<dyn NyashBox>>;
```

すべてのBoxは、インタープリターレベルで`Arc<Mutex>`によって管理されます。
これにより：
- **スレッドセーフティ**: 自動的に保証
- **参照カウント**: 自動的なメモリ解放
- **統一的アクセス**: すべて同じ方法で操作

### ❌ アンチパターン（Phase 9.75で修正中）

```rust
// 現在の問題: Box内部での二重ロック
pub struct BadBox {
    data: Arc<Mutex<String>>,  // ❌ Box内部でロック管理
}

// 正しい設計
pub struct GoodBox {
    data: String,  // ✅ シンプルなフィールド
}
```

## 🔄 fini()システム

### 概要
Nyashは決定論的なリソース解放のために`fini()`システムを提供します。

```nyash
box FileHandler {
    init { file }
    
    fini() {
        // オブジェクト削除時に自動呼び出し
        if me.file != null {
            me.file.close()
            console.log("File closed automatically")
        }
    }
}
```

### fini()の特徴

1. **自動呼び出し**: オブジェクトの参照がゼロになると自動実行
2. **決定論的**: GCのタイミングに依存しない
3. **伝播**: 親オブジェクトのfini()が子オブジェクトに伝播

### 実装例

```nyash
box DatabaseConnection {
    init { connection, transactions }
    
    fini() {
        // トランザクションをすべてロールバック
        for tx in me.transactions {
            tx.rollback()
        }
        // 接続を閉じる
        me.connection.close()
    }
}

// 使用例
{
    local db = new DatabaseConnection()
    db.connect("localhost:5432")
    // ... データベース操作 ...
}  // スコープを抜けると自動的にfini()が呼ばれる
```

## 💭 weak参照システム

### 循環参照の問題と解決

```nyash
// 循環参照の例
box Parent {
    init { children }
    
    pack() {
        me.children = new ArrayBox()
    }
    
    addChild(child) {
        me.children.push(child)
        child.parent = me  // 循環参照！
    }
}

box Child {
    init { parent }
}
```

### weak参照による解決

```nyash
box Parent {
    init { children }
    
    pack() {
        me.children = new ArrayBox()
    }
    
    addChild(child) {
        me.children.push(child)
        child.parent = weak me  // weak参照で循環を防ぐ
    }
}

box Child {
    init { parent }  // weak参照として保持
    
    getParent() {
        // weak参照から通常参照を取得
        local p = strong me.parent
        if p == null {
            console.log("Parent has been deleted")
            return null
        }
        return p
    }
}
```

### weak参照の特徴

1. **自動null化**: 参照先が削除されるとnullになる
2. **メモリリーク防止**: 循環参照を断ち切る
3. **明示的変換**: `strong`で通常参照に変換

## 📊 メモリ管理パターン

### 1. 所有権パターン

```nyash
box Container {
    init { items }  // Containerがitemsを所有
    
    pack() {
        me.items = new ArrayBox()
    }
    
    fini() {
        // itemsも自動的に解放される
        console.log("Container and all items released")
    }
}
```

### 2. 共有参照パターン

```nyash
// 複数のオブジェクトで共有
local sharedData = new DataBox()

local viewer1 = new DataViewer(sharedData)
local viewer2 = new DataViewer(sharedData)

// sharedDataは両方のviewerから参照されている間は生存
```

### 3. 観察者パターン

```nyash
box Subject {
    init { observers }
    
    pack() {
        me.observers = new ArrayBox()
    }
    
    attach(observer) {
        // weak参照で観察者を保持
        me.observers.push(weak observer)
    }
    
    notify() {
        // weak参照をチェックしながら通知
        local aliveObservers = new ArrayBox()
        
        for weakObs in me.observers {
            local obs = strong weakObs
            if obs != null {
                obs.update(me)
                aliveObservers.push(weakObs)
            }
        }
        
        // 死んだ参照を削除
        me.observers = aliveObservers
    }
}
```

## 🛡️ メモリ安全性保証

### 1. 二重解放防止
Arc<Mutex>により、同じオブジェクトの二重解放は不可能。

### 2. Use-After-Free防止
参照カウントにより、使用中のオブジェクトは解放されない。

### 3. データ競合防止
Mutexにより、同時アクセスは自動的に同期される。

### 4. メモリリーク検出
```nyash
// デバッグモードでメモリリーク検出
DEBUG = new DebugBox()
DEBUG.startTracking()

// ... プログラム実行 ...

print(DEBUG.memoryReport())
// 出力: 未解放オブジェクト一覧
```

## 🚀 ベストプラクティス

### 1. fini()の正しい使い方
```nyash
box ResourceManager {
    init { resources }
    
    fini() {
        // 1. 子リソースから順に解放
        for resource in me.resources {
            resource.release()
        }
        
        // 2. 自身のリソースを解放
        me.cleanup()
        
        // 3. ログを残す（デバッグ用）
        console.log("ResourceManager cleaned up")
    }
}
```

### 2. weak参照の使い時
- **親子関係**: 子→親はweak参照
- **イベントリスナー**: Subject→Observerはweak参照
- **キャッシュ**: 一時的な参照はweak

### 3. メモリ効率的なコード
```nyash
// ❌ 非効率
loop(i < 1000000) {
    local temp = new StringBox("temp")
    // tempが毎回作られる
}

// ✅ 効率的
local temp = new StringBox("")
loop(i < 1000000) {
    temp.set("temp")
    // 既存オブジェクトを再利用
}
```

## 📈 パフォーマンス考慮事項

### 1. 参照カウントのオーバーヘッド
- 小さいが無視できない
- ホットパスでは最小限に

### 2. Mutexロックの競合
- Phase 9.75で一元化により改善予定
- 細粒度ロックを避ける

### 3. fini()の実行コスト
- 複雑なfini()は避ける
- 非同期処理は避ける

## 🔮 将来の拡張

### 1. 世代別GC
参照カウントと世代別GCのハイブリッド検討

### 2. メモリプール
頻繁に生成・破棄されるBoxのプール化

### 3. コンパクション
メモリ断片化対策

---

関連ドキュメント：
- [Everything is Box](everything-is-box.md)
- [fini/weak参照リファレンス](../finalization-system.md)
- [Phase 9.75実装計画](implementation-notes/phase-9-75-redesign.md)