# Phase 10.8: 統一デバッグシステム - DeepInspectorBoxとグローバルデバッグ整理

作成日: 2025-08-27
発見者: ニャー
参照元: docs/ideas/other/2025-08-25-unified-box-design-deep-analysis.md

## 🚨 現在の問題

### 1. デバッグ環境変数の乱立
現在20個以上の環境変数が散在：
- `NYASH_VM_STATS=1`
- `NYASH_VM_DEBUG_BOXCALL=1`
- `NYASH_DEBUG_PLUGIN=1`
- `NYASH_NET_LOG=1`
- `NYASH_JIT_THRESHOLD=1`
- ... など多数

### 2. 統一されていないデバッグ体験
- VM、プラグイン、JIT、ネットワークなど各コンポーネントが独自のデバッグフラグ
- 複数の環境変数を組み合わせる必要がある
- 何をONにすればいいか分かりにくい

## 🌟 提案: 統一デバッグシステム

### 1. 環境変数の整理統合

```bash
# Before (現在)
NYASH_VM_STATS=1 NYASH_VM_DEBUG_BOXCALL=1 NYASH_NET_LOG=1 ./nyash

# After (提案)
NYASH_DEBUG=vm,boxcall,net ./nyash
```

### 2. デバッグレベル制御

```bash
# シンプルなレベル制御
NYASH_DEBUG_LEVEL=0  # OFF
NYASH_DEBUG_LEVEL=1  # 基本情報のみ
NYASH_DEBUG_LEVEL=2  # 詳細情報
NYASH_DEBUG_LEVEL=3  # すべて

# カテゴリ別レベル
NYASH_DEBUG=vm:2,net:1,jit:3
```

### 3. プリセット（よく使う組み合わせ）

```bash
# プリセット
NYASH_DEBUG_PRESET=basic     # 基本的なデバッグ情報
NYASH_DEBUG_PRESET=perf       # パフォーマンス分析用
NYASH_DEBUG_PRESET=network    # ネットワーク問題調査用
NYASH_DEBUG_PRESET=memory     # メモリリーク調査用
NYASH_DEBUG_PRESET=all        # すべて有効
```

## 🔍 DeepInspectorBox - Everything is Debugの実現

### グローバルシングルトンデバッガー

```nyash
// グローバルに1つだけ存在する統一デバッガー
static box DeepInspectorBox {
    public { enabled }
    private { 
        boxCreations, methodCalls, fieldAccess,
        memorySnapshots, referenceGraph, performanceMetrics
    }
    
    // === 簡単な有効化 ===
    enable(categories) {
        // "vm,net,memory" のようなカテゴリ文字列を解析
        me.parseAndEnableCategories(categories)
    }
    
    // === プリセット対応 ===
    usePreset(presetName) {
        match presetName {
            "basic" => me.enable("vm,error")
            "perf" => me.enable("vm,boxcall,stats")
            "network" => me.enable("net,plugin,tlv")
            "memory" => me.enable("alloc,gc,leak")
            "all" => me.enable("*")
        }
    }
    
    // === 統合ログ出力 ===
    log(category, level, message) {
        if me.shouldLog(category, level) {
            local formatted = me.formatLog(category, level, message)
            me.output(formatted)
        }
    }
}
```

### MIRレベルでの統一実装

```rust
// MIR生成時にデバッグフックを埋め込み
impl MirBuilder {
    fn build_new_box(&mut self, type_id: TypeId) -> ValueId {
        let result = self.push(NewBox { type_id });
        
        // デバッグモード時のみ
        if self.debug_enabled {
            self.push(DebugHook {
                event: DebugEvent::BoxCreated,
                type_id,
                value: result,
            });
        }
        
        result
    }
}
```

## 📊 実装計画

### Phase 10.8a: 環境変数統合（3日）
- [ ] 統一パーサー実装（`NYASH_DEBUG`解析）
- [ ] レベル制御システム
- [ ] プリセット定義
- [ ] 既存環境変数との互換性層

### Phase 10.8b: DeepInspectorBox基礎（1週間）
- [ ] グローバルシングルトン実装
- [ ] カテゴリ管理システム
- [ ] 基本的なログ収集
- [ ] 出力フォーマッター

### Phase 10.8c: MIR統合（1週間）
- [ ] DebugHook命令追加
- [ ] MirBuilderへのフック埋め込み
- [ ] VMでのDebugHook実行
- [ ] JITでのデバッグ情報保持

### Phase 10.8d: 高度な機能（2週間）
- [ ] メモリリーク検出
- [ ] 参照グラフ構築
- [ ] P2P非同期フロー追跡
- [ ] パフォーマンスプロファイリング

## 🎯 期待される効果

### 1. 使いやすさ向上
- 1つの環境変数で制御
- 分かりやすいプリセット
- 統一されたログフォーマット

### 2. デバッグ効率向上
- 必要な情報だけを表示
- カテゴリ別フィルタリング
- レベル別詳細度制御

### 3. 保守性向上
- 新しいデバッグ機能の追加が容易
- 統一されたインターフェース
- MIRレベルでの一元管理

## ✅ 成功基準

1. **環境変数の数**: 20個以上 → 3個以下
2. **デバッグ有効化**: 複雑なコマンド → `NYASH_DEBUG=basic`
3. **統一体験**: すべてのコンポーネントで同じデバッグインターフェース

---

*Everything is Box, Everything is Debug - 統一されたデバッグ体験へ*