# Sync<T>デモアイデア - GCとスレッドセーフの統合実証

作成日: 2025-08-27

## 🎯 ChatGPT5さんへの回答案

### デモ案1: マルチスレッドWebサーバー
```nyash
static box WebServer {
    init { Sync<Map<str, str>> sessions }  // セッション管理
    init { Sync<int> request_count }       // アクセスカウンター
    
    handleRequest(request) {
        // スレッドセーフに自動でカウントアップ
        me.request_count.increment()
        
        // セッション管理も自動ロック
        me.sessions.put(request.id, request.data)
    }
}

// GCオン/オフ切り替えデモ
NYASH_GC=on ./server   // 開発時：メモリリーク検出
NYASH_GC=off ./server  // 本番時：高性能
```

### デモ案2: リアルタイムチャットサーバー
```nyash
static box ChatServer {
    init { Sync<Map<str, User>> users }
    init { Sync<Vec<Message>> messages }
    init { AtomicBox<int> connections }
    
    addMessage(user, text) {
        me.messages.push(new Message(user, text))  // 自動ロック！
        me.connections.increment()                  // 原子操作！
    }
}
```

## 💡 統合デモ：両方の特徴を一度に

「**GC切り替え**」と「**Sync<T>自動ロック**」を同時に実証：

1. **Phase 1**: GCオンで開発
   - メモリリーク検出
   - 「Memory leak detected: Message#1234 at line 15」

2. **Phase 2**: 修正後、GCオフで本番
   - 同じコードが13.5倍高速
   - スレッドセーフは維持

## 🏆 期待される効果

- **簡単さの実証**: `table.put()`だけでスレッドセーフ
- **性能の実証**: GCオフで高速動作
- **実用性の実証**: 実際のWebサーバーで動作

これ1つで「チート言語」の本質を完全に示せる！