# 🔥 Nyash finiシステム - 論理的解放フック

**最終更新: 2025年8月13日 - ChatGPT5協議による革命的設計完了**

## 🎯 概要

Nyashの`fini()`システムは、物理的メモリ破棄ではなく**論理的使用終了**を宣言する革新的なリソース管理システムです。Everything is Box哲学と完全に統合され、予測可能で安全なリソース管理を実現します。

## 🌟 核心コンセプト

### 📝 論理的解放フック
```nyash
box MyResource {
    init { name, file }
    
    fini() {
        print("Resource " + me.name + " is being finalized")
        // ファイルクローズなどのクリーンアップ処理
        // 物理的メモリは共有参照が残っていても論理的には「終了」
    }
}
```

**重要**: `fini()`は「このオブジェクトをもう使わない」という宣言であり、物理的な即時破棄ではありません。

## 🔄 実行順序（最終仕様）

### 自動カスケード解放
```nyash
box Pipeline {
    init { r1, r2, r3, weak monitor }
    
    fini() {
        // 1) ユーザー定義処理（柔軟な順序制御可能）
        me.r3.fini()  // 依存関係でr3→r2の順
        me.r2.fini()
        
        // 2) 自動カスケード: 残りのr1がinit宣言順で自動解放
        // 3) weakフィールドは対象外（lazy nil化）
    }
}
```

### 決定的な解放順序
1. **finalized チェック** - 既に解放済みなら何もしない（idempotent）
2. **再入防止** - `in_finalization`フラグで再帰呼び出し防止
3. **ユーザー定義fini()実行** - カスタムクリーンアップ処理
4. **自動カスケード** - `init`宣言順で未処理フィールドを解放
5. **フィールドクリア** - 全フィールドを無効化
6. **finalized設定** - 以後の使用を禁止

## ⚠️ 厳格な禁止事項

### weak フィールドへのfini呼び出し禁止
```nyash
box Parent {
    init { weak child }
    
    fini() {
        // ❌ 絶対にダメ！ビルドエラーまたは実行時エラー
        // me.child.fini()  
        
        // ✅ 正しい方法
        me.child = null    // 参照解除
        // または自動nil化に任せる
    }
}
```

**理由**: weak参照は所有権を持たない非所有参照のため、fini()を呼ぶ権利がありません。

### finalized後の使用禁止
```nyash
box Example { }

local x = new Example()
x.fini()

// ❌ 以下は全てエラー
x.someMethod()      // → "Instance was finalized; further use is prohibited"
x.field = value     // → 同上
local val = x.field // → 同上
```

## 🏗️ 実装アーキテクチャ

### InstanceBox拡張
```rust
pub struct InstanceBox {
    // 既存フィールド...
    
    init_field_order: Vec<String>,                    // 決定的カスケード順序
    weak_fields_union: HashSet<String>,               // weak判定高速化
    in_finalization: bool,                            // 再入防止
    finalized: bool,                                  // 使用禁止フラグ
}
```

### 実行時ガード
- **メソッド呼び出し**: `finalized`チェック → エラー
- **フィールドアクセス**: `finalized`チェック → エラー  
- **フィールド代入**: `finalized`チェック → エラー

## 💡 使用例とパターン

### 基本的な使用例
```nyash
box FileHandler {
    init { filename, handle }
    
    pack(name) {
        me.filename = name
        me.handle = openFile(name)
    }
    
    fini() {
        if (me.handle != null) {
            closeFile(me.handle)
            print("File " + me.filename + " closed")
        }
    }
}

// 使用
local handler = new FileHandler("data.txt")
// ... ファイル操作 ...
handler.fini()  // 明示的クリーンアップ
```

### 再代入時の自動解放
```nyash
box Holder { init { resource } }
box Resource { fini() { print("Resource cleaned up") } }

local h = new Holder()
h.resource = new Resource()     // 新しいリソース設定
h.resource = new Resource()     // → 前のリソースが自動的にfini()される
```

### カスタム解放順序
```nyash
box DatabaseConnection {
    init { transaction, connection, logger }
    
    fini() {
        // 依存関係に基づく手動順序制御
        if (me.transaction != null) {
            me.transaction.rollback()
            me.transaction.fini()
        }
        
        if (me.connection != null) {
            me.connection.close()
            me.connection.fini()
        }
        
        // loggerは自動カスケードに任せる
    }
}
```

### 循環参照の安全な解決
```nyash
box Node {
    init { data, weak parent, children }
    
    pack(value) {
        me.data = value
        me.children = new ArrayBox()
    }
    
    addChild(child) {
        me.children.push(child)
        child.setParent(me)  // 子→親はweak参照
    }
    
    fini() {
        // 子ノードを先に解放
        loop (me.children.length() > 0) {
            local child = me.children.pop()
            child.fini()
        }
        // 親への参照は自動的にnil化される
    }
}
```

## 🧪 テストパターン

### 基本動作テスト
```nyash
box Counter {
    init { value }
    pack() { me.value = 0 }
    increment() { me.value = me.value + 1 }
    fini() { print("Counter finalized with value: " + me.value.toString()) }
}

local c = new Counter()
c.increment()
c.increment()
c.fini()
// c.increment()  // → エラー: finalized後の使用禁止
```

### 循環参照テスト
```nyash
box Parent {
    init { child }
    pack() {
        me.child = new Child()
        me.child.setParent(me)
    }
    fini() { print("Parent finalized") }
}

box Child {
    init { weak parent }
    setParent(p) { me.parent = p }
    fini() { print("Child finalized") }
}

local p = new Parent()
p.fini()  // Parent → Child の順で解放、リークなし
```

## 🎯 期待される効果

### メモリ安全性
- **循環参照リーク完全防止**: weak参照とfiniの組み合わせ
- **二重解放防止**: idempotentな設計
- **使用禁止ガード**: finalized後の誤用防止

### 予測可能性
- **決定的順序**: init宣言順による自動カスケード
- **明示的制御**: ユーザー定義fini()での柔軟な順序指定
- **エラーメッセージ**: 明確で修正提案付きのエラー

### 開発体験
- **直感的**: リソースの「終了宣言」として理解しやすい
- **デバッグ容易**: 解放タイミングが明確
- **保守性**: 依存関係の変更に強い設計

## 📚 関連ドキュメント

- [weak参照設計](weak-reference-design.md) - 循環参照解決との統合
- [Everything is Box](design-philosophy.md) - 基本設計思想
- [言語リファレンス](language-reference.md) - 構文詳細

---

**Everything is Box, Everything is Finalized!** 🔥