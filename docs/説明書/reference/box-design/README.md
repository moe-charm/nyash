# 📦 Nyash Box設計ドキュメント

## 🎯 概要

Nyashの核心哲学「**Everything is Box**」に関する完全な設計ドキュメント集。
言語設計の根幹から実装詳細まで、Box設計のすべてを網羅しています。

## 📚 ドキュメント構成

### 🌟 設計思想

#### [everything-is-box.md](everything-is-box.md)
Nyashの核心哲学「Everything is Box」の完全解説。なぜすべてをBoxにするのか、その設計思想と利点を説明。

#### [box-types-catalog.md](box-types-catalog.md)  
Nyashで利用可能な全Box型のカタログ。基本型（StringBox, IntegerBox）から高度な型（P2PBox, EguiBox）まで。

### 🔄 システム設計

#### [delegation-system.md](delegation-system.md)
完全明示デリゲーションシステムの設計。`from`構文、`override`必須、`pack`構文の詳細仕様。

#### [memory-management.md](memory-management.md)
Arc<Mutex>一元管理、fini()システム、weak参照による循環参照回避の設計原則。

### 🌐 外部連携

#### [ffi-abi-specification.md](ffi-abi-specification.md)
Box FFI/ABI完全仕様。外部ライブラリを「箱に詰める」ための統一インターフェース。

#### FileBox マッピング
- [filebox-bid-mapping.md](filebox-bid-mapping.md) — Nyash APIとBID-FFIプラグインABIの対応表（メソッドID/TLV/戻り値）

### 🔧 実装ノート

#### [implementation-notes/](implementation-notes/)
開発者向けの実装詳細、既知の問題、進行中の設計変更などの技術情報。

- [current-issues.md](implementation-notes/current-issues.md) - 現在対応中の設計課題
- [socket-box-problem.md](implementation-notes/socket-box-problem.md) - Arc<Mutex>二重化問題の詳細分析
- [phase-9-75-redesign.md](implementation-notes/phase-9-75-redesign.md) - Box設計根本革命の実装計画

## 🎨 設計原則

### 1. **Everything is Box**
すべての値がBoxオブジェクト。プリミティブ型は存在しない。

### 2. **明示性重視**
暗黙的な動作を避け、すべてを明示的に記述。

### 3. **Arc<Mutex>一元管理**
Box内部でのロックを避け、インタープリターが一元管理。

### 4. **メモリ安全性**
fini()システムとweak参照による確実なメモリ管理。

## 🚀 クイックリファレンス

### Box作成
```nyash
// 基本型
local str = new StringBox("Hello")
local num = new IntegerBox(42)

// ユーザー定義Box
box User {
    init { name, email }
    
    birth(userName, userEmail) {
        me.name = userName
        me.email = userEmail
        print("🌟 User " + userName + " が誕生しました！")
    }
}
```

### デリゲーション
```nyash
box AdminUser from User {
    init { permissions }
    
    birth(adminName, adminEmail, perms) {
        from User.birth(adminName, adminEmail)
        me.permissions = perms
    }
    
    override toString() {
        return "Admin: " + from User.toString()
    }
}
```

### ビルトインBox継承（pack専用）
```nyash
// ビルトインBoxを継承する場合のみpackを使用
box EnhancedP2P from P2PBox {
    init { features }
    
    pack(nodeId, transport) {
        from P2PBox.pack(nodeId, transport)  // ビルトイン初期化
        me.features = new ArrayBox()
    }
    
    override send(intent, data, target) {
        me.features.push("send:" + intent)
        return from P2PBox.send(intent, data, target)
    }
}
```

### 外部ライブラリ統合（FFI/ABI）
```nyash
// ExternBoxで外部APIを統一的に利用
local console = new ExternBox("console")
console.call("log", "Hello from Nyash!")

local canvas = new ExternBox("canvas")
canvas.call("fillRect", 10, 10, 100, 50)
```

## 📖 関連ドキュメント

- [言語リファレンス](../language-reference.md)
- [ビルトインBox一覧](../builtin-boxes.md)
- [実装ガイド](../../../../CLAUDE.md)
- [開発計画](../../../../予定/native-plan/copilot_issues.txt)

## 🔄 更新履歴

- 2025-08-14: Box設計ドキュメント初版作成
- 2025-08-14: Phase 9.75（Arc<Mutex>責務一元化）対応開始

---

最終更新: 2025-08-14
