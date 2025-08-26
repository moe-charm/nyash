# Phase 10.4: GC Switchable Runtime - 世界初の柔軟なメモリ管理

Status: Planned
Owner: core-runtime
Target: After Cranelift JIT (Phase 10.0)
Last Updated: 2025-08-26
Dependencies: Phase 10.0 (Cranelift JIT), Phase 9.79b (Unified Box)

## 🎯 概要

Nyashを**世界初のGC切り替え可能プログラミング言語**にする革新的機能。開発時はGCで快適に、本番ではGCなしで最高性能を実現。

## 📊 技術的背景

### 現状のメモリ管理
- Everything is Box哲学（すべてのデータがBoxオブジェクト）
- 明示的メモリ管理（スコープベースfini）
- Arc<Mutex>によるスレッドセーフ設計

### 提案する2つのモード
1. **Explicit Mode（現在のデフォルト）**
   - スコープを抜けたら即座にfini()呼び出し
   - 予測可能な性能（リアルタイムアプリ向け）
   - メモリ使用量が最小

2. **Reference Counting Mode（新規）**
   - 参照カウントが0になったらfini()呼び出し
   - 循環参照はweak参照で解決
   - 開発効率重視（一般アプリ向け）

## 🏗️ アーキテクチャ設計

### MIR層：所有権イベントの抽象化
```rust
// GCモードに依存しない所有権表現
enum MirOwnership {
    Move(temp_id),      // 所有権移動
    Copy(temp_id),      // 複製
    Drop(temp_id),      // 破棄
    StorageLive(id),    // 生存開始
    StorageDead(id),    // 生存終了
    Escape(target),     // エスケープ解析
}
```

### ランタイム層：モード別実装
```rust
// 統一APIでモード切り替え
trait MemoryManager {
    fn retain_ref(&self, ptr: *const BoxHeader);
    fn release_ref(&self, ptr: *const BoxHeader);
    fn destroy(&self, ptr: *const BoxHeader);
}

struct ExplicitManager;    // 即座に破棄
struct RefCountManager;    // 参照カウント管理
```

### JIT層：関数マルチバージョン化
```
関数テーブル:
┌─────────────┬──────────────┬──────────────┐
│  Function   │ Explicit Ver │ RefCount Ver │
├─────────────┼──────────────┼──────────────┤
│ array_push  │ 0x1000_0000  │ 0x2000_0000  │
│ map_get     │ 0x1000_1000  │ 0x2000_1000  │
└─────────────┴──────────────┴──────────────┘

トランポリン → 現在のモードに応じてジャンプ
```

## 📋 実装計画

### Phase 10.4.1: 基盤構築（2週間）
- [ ] BoxHeaderに参照カウントフィールド追加
- [ ] MemoryManagerトレイト定義
- [ ] インタープリターでの基本実装

### Phase 10.4.2: MIR対応（1ヶ月）
- [ ] 所有権イベント（Move/Copy/Drop等）の導入
- [ ] retain_ref/release_ref命令の追加
- [ ] エスケープ解析の基礎実装

### Phase 10.4.3: 最適化（3週間）
- [ ] 近接ペア消去（retain直後のrelease削除）
- [ ] ループ不変式の移動
- [ ] φノードでの差分管理

### Phase 10.4.4: JIT統合（1ヶ月）
- [ ] 関数マルチバージョン生成
- [ ] トランポリン機構実装
- [ ] fast path/slow path分離

### Phase 10.4.5: 実戦投入（2週間）
- [ ] モード切り替えCLI実装
- [ ] メモリリーク検出ツール
- [ ] ベンチマーク・性能評価

## 🎯 使用例

### 開発フロー
```bash
# 1. 開発中：GCオンで快適に開発
nyash --gc-mode=ref-counting --detect-leaks dev.nyash

# 2. テスト：メモリリークがないことを確認
nyash --gc-mode=ref-counting --memory-report test.nyash
# => No memory leaks detected!

# 3. 本番：GCオフで最高性能
nyash --gc-mode=explicit --optimize prod.nyash
```

### コード例
```nyash
// 同じコードが両モードで動作
box DataProcessor {
    init { buffer, cache }
    
    process(data) {
        me.buffer = data.transform()  // GCありなら参照カウント管理
        me.cache.put(data.id, data)   // GCなしなら即座に古い値を破棄
        return me.buffer
    }
    
    fini() {
        print("Cleanup!")  // タイミングはモード次第
    }
}
```

## ⚠️ 技術的課題と解決策

### 1. Arc<Mutex>の重さ
**課題**: 現在すべてのBoxがArc<Mutex>で重い
**解決**: 必要な場所のみ同期、基本型は非同期に

### 2. 実行時オーバーヘッド
**課題**: モードチェックのコスト
**解決**: JITでの関数マルチバージョン化（間接ジャンプ1回のみ）

### 3. 循環参照
**課題**: RefCountingモードでの循環参照
**解決**: 既存のWeakBox活用＋明示的切断

### 4. セマンティクスの違い
**課題**: デストラクタ呼び出しタイミングの差
**解決**: ドキュメント化＋移行ガイド作成

## 📊 期待される効果

1. **開発効率**: 30%向上（メモリ管理の負担軽減）
2. **実行性能**: GCオフ時は現状維持、GCオン時は5-10%低下
3. **メモリ効率**: モード次第で最適化可能
4. **教育価値**: メモリ管理の学習に最適なツール

## 🔗 関連ドキュメント
- [Phase 10.0: Cranelift JIT](phase_10_cranelift_jit_backend.md)
- [Phase 9.79b: Unified Box Design](../phase-9/phase_9_79b_1_unified_registry_ids_and_builder_slotting.md)
- [GC Switchable Language Idea](../../../ideas/other/2025-08-26-gc-switchable-language.md)

## ✅ 受け入れ基準
- [ ] インタープリター/VM/JITすべてで両モード動作
- [ ] モード切り替えが実行時に可能（再コンパイル不要）
- [ ] 既存コードが無修正で動作（後方互換性）
- [ ] パフォーマンス劣化が許容範囲（GCオン時10%以内）
- [ ] メモリリーク検出ツールの提供

## 🚀 将来の拡張
- Mark & Sweep GCモードの追加
- 世代別GC
- リージョンベースメモリ管理
- プロファイルベース自動モード選択