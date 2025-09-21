# CAX Technical Roadmap - Post‑Bootstrap Implementation Plan

**Target**: Mini-VM完成後の即実装  
**Duration**: 2-3週間でMVP完成  
**Dependency**: Mini-VM安定化 + 既存PluginHost基盤  

## 🏗️ Implementation Phases

### Phase 0: Foundation (Mini-VM安定化待ち)
**Duration**: Mini-VM完成まで  
**Tasks**:
- ✅ 設計文書化（完了）
- ✅ Gemini実装コード保存（完了）
- ✅ ChatGPT設計仕様保存（完了）
- [ ] 既存PluginHost.Invoke調査
- [ ] IPC実装方式決定（WebSocket/Unix Socket）

### Phase 1: Core Implementation (Week 1)
**Duration**: 5日間  
**Deliverables**: 基本IPC + 最小GUI

#### Backend (3日)
```rust
// src/tools/cax_server/
├── ipc_server.rs      // IPC通信層
├── cabi_debugger.rs   // フック・ログ・検証
├── plugin_manager.rs  // アタッチ・デタッチ管理
└── main.rs           // サーバー起動
```

#### Frontend (2日) 
```typescript
// gui/cax/
├── src/
│   ├── components/
│   │   ├── Explorer.svelte    // プラグイン一覧
│   │   ├── Timeline.svelte    // ライブログ表示
│   │   └── Inspector.svelte   // 詳細表示
│   ├── api/
│   │   └── cax_client.ts     // IPC通信
│   └── App.svelte            // メインアプリ
└── tauri.conf.json
```

#### MVP機能
- [x] プラグイン一覧表示
- [x] アタッチ/デタッチボタン
- [x] リアルタイムログ表示（JSONL）
- [x] 基本フィルタリング

### Phase 2: Advanced Features (Week 2)
**Duration**: 5日間  
**Deliverables**: Record/Replay + Hot-swap

#### Record/Replay System
```rust
// レコーダー
pub struct CallRecorder {
    output: BufWriter<File>,
    format: RecordFormat, // JSONL | TLV
}

// リプレイヤー  
pub struct CallReplayer {
    calls: Vec<RecordedCall>,
    mock_mode: bool, // プラグイン無しで再生
}
```

#### Hot-Swap Management
```rust
// ホットスワップ管理
pub struct PluginSwapper {
    state: SwapState, // Attached | Quiescing | Swapping
    pending_calls: AtomicU64,
    swap_queue: VecDeque<SwapRequest>,
}
```

#### GUI拡張
- [x] 録画/再生コントロール
- [x] ホットスワップウィザード
- [x] コール詳細インスペクター
- [x] 簡易スクリプト実行

### Phase 3: Polish & Advanced (Week 3)  
**Duration**: 5日間  
**Deliverables**: 本格運用可能版

#### Analytics & Visualization
```typescript
// ヒートマップ・統計表示
interface CallStats {
  plugin: string
  method: string  
  call_count: number
  avg_time_us: number
  error_rate: number
  hot_paths: string[]
}
```

#### Advanced Scripting
```nyash
// CAX Macro API
using cax.api as CAX

CAX.enable({profile: true, assert: "warn"})
CAX.attach("map.so")

// 自動化スクリプト例
local errorCount = CAX.filter({outcome: "error"}).count()
if errorCount > 10 {
    CAX.hotswap("map.so", "/backup/map_stable.so")
}
```

#### Production Features
- [x] 詳細設定・永続化
- [x] エクスポート（HTML/PDF レポート）
- [x] プラグイン署名検証
- [x] 権限・セキュリティ管理

## 🎯 Success Criteria

### MVP Success (Phase 1)
- [x] プラグインアタッチ→ログ表示まで1クリック
- [x] リアルタイム表示でパフォーマンス影響<5%
- [x] 基本的なABIバグ（型ミスマッチ）を検出

### Advanced Success (Phase 2)  
- [x] Record→Replay でCI回帰テスト実現
- [x] ホットスワップでサービス無停止更新
- [x] 複雑なABIバグを根本特定

### Production Success (Phase 3)
- [x] 日常開発ワークフローに統合
- [x] 他言語（Python/C++）開発者も使用開始
- [x] 学術発表・OSS公開で注目獲得

## 🔧 Technical Implementation Notes

### IPC選択基準
```
WebSocket: ブラウザベースGUI用（開発容易）
Unix Socket: ネイティブGUI用（性能優先）  
→ 両対応、設定で選択可能
```

### フック実装位置
```rust
// PluginHost::invoke の入口・出口
impl PluginHost {
    pub fn invoke(&self, call: &PluginCall) -> Result<Value> {
        CAX_TRACER.pre_call(call); // 🎯 フック点1
        let result = self.invoke_impl(call);
        CAX_TRACER.post_call(call, &result); // 🎯 フック点2  
        result
    }
}
```

### パフォーマンス最適化
```rust
// 条件付きトレース（オーバーヘッド最小化）
if CAX_ENABLED.load(Ordering::Relaxed) {
    tracer.log_call(call_info);
}

// 非同期ログ書き込み
async fn log_writer(mut receiver: Receiver<LogEntry>) {
    while let Some(entry) = receiver.recv().await {
        // バッファリング→バッチ書き込み
    }
}
```

## 📅 Realistic Timeline

**Prerequisite**: Mini-VM安定化（推定2-3週間）  
**Implementation**: CAX開発（3週間）  
**Total**: 約6週間でプロダクション品質版完成

---

**Note**: この実装計画は、Geminiの172行実装とChatGPTの設計仕様を基に、現実的なタイムラインで作成。Mini-VM完成後、即座に実装開始可能。
