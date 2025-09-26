# Phase 15.5: JSON v0中心化・統一Call基盤革命

**セルフホスティング前の基盤アーキテクチャ大改革**

## 🎯 概要

Phase 15（セルフホスティング）の前段階として、Nyashの実行基盤を根本的に見直し、将来のRust離脱・多言語実装を見据えた堅牢な基盤を構築する。

### 🔥 核心目標 - 3つの統一革命
1. **🏗️ MIR命令生成統一** - Builder側でemit_unified_call()統一
2. **⚙️ Callee処理統一** - 全実行器で6種類Callee対応
3. **📋 JSON出力統一** - 統一Call形式での交換フォーマット確立

**最終目標**: 4実行器 × 3統一 = **完全統一Call基盤**でセルフホスティング成功

## 🚨 なぜPhase 15.5が必要か

### 現在の問題
- **言語依存**: Rust MIRに過度に依存
- **実行器分散**: 4つの実行器でCall処理がバラバラ
- **JSON混在**: 入力用v0と出力用が統一されていない
- **将来リスク**: Rust離脱時に全面書き直しが必要

### ChatGPT戦略提案の核心
> "MIR型＝正、JSON＝境界" の役割分担を保ちつつ、段階的にJSON v0中心化へ移行

## 📋 Phase構成

### [Phase A: JSON出力統一](./migration-phases.md#📋-phase-a-json出力統一今すぐ実装)
- `mir_json_emit`で統一Call対応
- `json_out:v1`スキーマ導入
- 後方互換性維持

### [Phase B: JSON中心化移行](./migration-phases.md#📋-phase-b-json中心化移行次段階)
- MIR ModuleをJSON v0ラッパー化
- HIR/名前解決情報のJSON化
- 型安全性維持

### [Phase C: 完全JSON化](./migration-phases.md#📋-phase-c-完全json化最終段階)
- Rust MIR廃止準備
- プリンター等もJSON経由
- 多言語実装基盤完成

## 🔗 関連ドキュメント

### 📊 現在の進捗
- **[実装状況追跡](./implementation-status.md)** - リアルタイム進捗管理
- **[段階移行計画](./migration-phases.md)** - 詳細実装ロードマップ

### 🛡️ 戦略・設計
- **[JSON v0中心化戦略](./json-v0-centralization.md)** - アーキテクチャ設計の核心
- **[リスク分析と対策](./risk-analysis.md)** - 潜在的リスクと軽減戦略

### 既存Phase連携
- [Phase 15: セルフホスティング](../phase-15/README.md)
- [MIR Call統一マスタープラン](../phase-15/mir-call-unification-master-plan.md)

## 📊 Phase 15との関係

```
Phase 15.5 (基盤革命) → Phase 15 (セルフホスティング)
        ↓                     ↓
   JSON v0基盤確立        Rust→Nyash移行
   統一Call完成           80k→20k行削減
```

Phase 15.5は**Phase 15成功の前提条件**である。

## 🎯 成功定義

### Phase A完了条件 - 修正版
- [ ] **MIR Builder**: emit_unified_call()で全Call生成 ✅
- [ ] **VM実行器**: Callee処理完全対応 ✅
- [ ] **Python LLVM**: llvmlite内部Callee統一 🔄 **要対応**
- [ ] **JSON出力**: 統一Call v1形式 ⏳
- [ ] **環境変数**: NYASH_MIR_UNIFIED_CALL=1で全統一 🔄

### Phase B完了条件
- [ ] JSON→MIRリーダーの薄化
- [ ] HIR情報のJSON化
- [ ] 型安全性とパフォーマンス維持

### Phase C完了条件
- [ ] MIR Module廃止準備完了
- [ ] 多言語実装の技術実証
- [ ] セルフホスティング基盤完成

## ⚠️ 重要注意事項

### 急がない原則
- **一気にJSON化しない** - 型喪失・性能劣化のリスク
- **既存機能を壊さない** - 段階移行で安全性確保
- **テスト駆動** - 各段階で完全な動作確認

### 撤退戦略
各Phaseで前段階に戻れる設計を維持する。

---

**更新**: 2025-09-24
**責任者**: Claude Code + ChatGPT戦略
**関連Issue**: Phase 15セルフホスティング前提条件