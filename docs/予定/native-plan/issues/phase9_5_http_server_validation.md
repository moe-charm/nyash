# 🌐 Phase 9.5: HTTPサーバー実用テスト（AOT検証）

## 📋 Summary
AOT実装完了後の複雑アプリケーション検証。並行処理・メモリ管理・実用性能測定を通じて、Nyashの実用レベル到達を実証する。

## 🎯 実装目標
```bash
# Phase 9完了後の目標
nyash --compile-native http_server.nyash -o http_server.exe  # AOTサーバー生成
./http_server.exe --port 8080                               # 高性能HTTPサーバー起動
curl http://localhost:8080/api/status                       # 実用API動作確認

# 検証内容
- 同時100接続でメモリリークなし
- fini()システム確実動作（I/Oハンドル解放）
- AOT環境での真の性能測定
```

## 🔧 技術的実装詳細

### 1. HTTPサーバー基本構造
```nyash
box HTTPServer {
    init { socket, clients, handlers, running }
    
    pack(port) {
        me.socket = new SocketBox()
        me.clients = new ArrayBox()
        me.handlers = new MapBox()
        me.running = true
        
        me.socket.bind("0.0.0.0", port)
        me.socket.listen(128)
    }
    
    start() {
        loop(me.running) {
            local client = me.socket.accept()
            me.clients.push(client)
            nowait me.handleClient(client)  // 非同期並行処理
        }
    }
    
    handleClient(client) {
        local request = client.readRequest()
        local response = me.processRequest(request)
        client.sendResponse(response)
        
        // 重要: 確実なリソース解放
        me.clients.remove(client)
        client.fini()
    }
    
    processRequest(request) {
        local path = request.getPath()
        local handler = me.handlers.get(path)
        
        if (handler != null) {
            return handler.handle(request)
        } else {
            return me.create404Response()
        }
    }
}
```

### 2. ルーティング・ハンドラーシステム
```nyash
box RouteHandler {
    init { pattern, callback }
    
    pack(pattern, callback) {
        me.pattern = pattern
        me.callback = callback
    }
    
    handle(request) {
        return me.callback.call(request)
    }
}

// 使用例
local server = new HTTPServer(8080)
server.route("/api/status", new StatusHandler())
server.route("/api/users/:id", new UserHandler())
server.start()
```

### 3. メモリ管理検証ポイント
```nyash
box ConnectionManager {
    init { connections, maxConnections }
    
    pack(maxConnections) {
        me.connections = new MapBox()
        me.maxConnections = maxConnections
    }
    
    addConnection(clientId, client) {
        if (me.connections.size() >= me.maxConnections) {
            // 古い接続をweak参照で自動解放
            me.cleanupOldConnections()
        }
        me.connections.set(clientId, client)
    }
    
    cleanupOldConnections() {
        // weak参照による自動null化テスト
        local toRemove = new ArrayBox()
        me.connections.forEach((id, conn) => {
            if (conn.isDisconnected()) {
                toRemove.push(id)
                conn.fini()  // 確実な解放
            }
        })
        
        toRemove.forEach((id) => {
            me.connections.remove(id)
        })
    }
}
```

## 📊 検証ポイント詳細

### 並行処理性能
```bash
# 負荷テストコマンド
ab -n 10000 -c 100 http://localhost:8080/api/test    # Apache Bench
wrk -t12 -c400 -d30s http://localhost:8080/         # Modern HTTP benchmarking
```

**検証項目**:
- **同時接続処理**: 100接続同時処理
- **スループット**: リクエスト/秒測定
- **レイテンシ**: 応答時間分布
- **リソース使用**: CPU・メモリ使用率

### メモリ管理検証
```nyash
// ストレステスト実装
box MemoryStressTest {
    runConnectionStress() {
        // 1000回接続・切断を繰り返し
        loop(1000) {
            local client = me.createClient()
            client.connect()
            client.sendRequest("/api/test")
            client.disconnect()
            client.fini()  // 明示的解放
        }
        
        // メモリリークチェック
        local memUsage = DEBUG.memoryReport()
        assert(memUsage.leaks == 0)
    }
}
```

### I/Oリソース管理
```nyash
box ResourceTracker {
    init { openSockets, openFiles }
    
    trackResource(resource) {
        me.openSockets.add(resource)
    }
    
    verifyCleanup() {
        // 全リソースが正しくfini()されているか確認
        assert(me.openSockets.size() == 0)
        assert(me.openFiles.size() == 0)
    }
}
```

## 🎯 実装ステップ（2週間）

### Week 1: HTTPサーバー基本実装
- [ ] SocketBox・HTTP基本プロトコル実装
- [ ] HTTPServer・RouteHandlerクラス実装
- [ ] 基本GET/POST対応
- [ ] 単一接続での動作確認

### Week 2: 並行処理・負荷テスト
- [ ] nowait/await非同期処理統合
- [ ] 同時接続管理システム
- [ ] メモリ管理・リソース解放検証
- [ ] 負荷テスト・ベンチマーク実装

## 📈 性能測定目標

| 指標 | 目標値 | 測定方法 |
|------|--------|----------|
| **同時接続数** | 100+ | Apache Bench |
| **スループット** | 1000+ req/s | wrk benchmark |
| **応答時間** | <10ms (P95) | レイテンシ分布 |
| **メモリ使用** | リークなし | 長時間実行テスト |
| **リソース解放** | 100%解放 | fini()追跡 |

## ✅ Acceptance Criteria

### 機能要件
- [ ] HTTPサーバーが安定動作
- [ ] REST API（GET/POST/PUT/DELETE）対応
- [ ] ルーティング・ミドルウェア機能
- [ ] 静的ファイル配信機能

### 性能要件  
- [ ] 同時100接続でクラッシュなし
- [ ] 1000 req/s以上のスループット
- [ ] レスポンス時間P95<10ms
- [ ] 24時間連続稼働でメモリリークなし

### 品質要件
- [ ] fini()システム100%動作
- [ ] weak参照自動null化確認
- [ ] I/Oリソース確実解放
- [ ] 例外経路でのリソース管理

## 🚀 期待される効果

### 実用性実証
- **配布可能サーバー**: `http_server.exe`として実用レベル
- **プロダクション検証**: 実際の負荷でのメモリ管理確認
- **AOT価値実証**: 真の高性能実行環境での検証

### 技術的価値
- **複雑メモリ管理**: Server→Clients→Requests階層構造
- **並行処理実証**: nowait/awaitの実用性能確認
- **Everything is Box**: 複雑アプリでのBox哲学実証

### デモ・広報価値
- **視覚的インパクト**: 動作するHTTPサーバーの強力デモ
- **実用性アピール**: 「おもちゃ言語」ではない実用性
- **性能実証**: 数値での性能証明

## 📖 References
- docs/予定/native-plan/copilot_issues.txt（Phase 9.5詳細）
- docs/予定/native-plan/issues/phase9_aot_wasm_implementation.md（Phase 9基盤）
- docs/予定/native-plan/issues/phase_8_7_real_world_memory_testing.md（kilo基盤）
- [HTTP/1.1 Specification](https://tools.ietf.org/html/rfc7230)
- [Apache Bench Documentation](https://httpd.apache.org/docs/2.4/programs/ab.html)

---

**💡 Tip**: kiloで確立したメモリ管理基盤を、より複雑な並行処理環境で実証する重要なマイルストーンです。

最終更新: 2025-08-14
作成者: Claude（実用優先戦略）