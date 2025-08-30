# Phase 11: LLVM AOT Backend（進行中）

## 🎯 概要

Phase 11は、LLVM を使用した Ahead-of-Time（AOT）コンパイル機能の研究・実装フェーズです。
Phase 10のCranelift JITで実用的な性能を達成した後、さらなる最適化をLLVM AOTで追求します。

## 📊 位置づけ

```
Phase 10: Cranelift JIT（実用的な高速化）← 完了
    ↓
Phase 11: LLVM AOT（最高性能への挑戦）← 進行中
```

## 📁 ドキュメント

### 🔬 研究・設計ドキュメント
- [phase10_aot_scaffolding.md](phase10_aot_scaffolding.md) - LLVM Direct AOT実装計画
  - MIR→LLVM IR直接変換
  - Everything is Box最適化（エスケープ解析）
  - LTO/PGO統合
  - 目標: 13,500倍高速化（対インタープリタ）

- [phase_10_x_llvm_backend_skeleton.md](phase_10_x_llvm_backend_skeleton.md) - LLVM Backend最小実装
  - 具体的な実装ステップ
  - ExternCall対応
  - オブジェクトファイル生成

## ⏰ タイムライン（短期スプリント）

- Status: In Progress（進行中）
- 前提条件: Phase 10（Cranelift JIT）の完了、Core‑15統一（VM/Verifierで運用）
- 想定期間: 4週間（各フェーズ1週間目安）
  - 11.1 基本変換: Const/Unary/Bin/Compare, Load/Store, Jump/Branch/Return/Phi
  - 11.2 Box統合: NewBox/BoxCall/ExternCall（安全パスはランタイム呼び出し）
  - 11.3 最適化: 注釈統合・型特化（get/setField・Array get/set のInline化＋バリア）
  - 11.4 高度化: 脱箱化・TBAA・PGO/ThinLTO

## 🎯 期待される成果

1. **最高性能**: インタープリタ比13,500倍の実行速度
2. **メモリ効率**: Box割当80%削減
3. **起動時間**: 1ms以下
4. **配布形式**: スタンドアロン実行ファイル

## ⚠️ 注意事項（運用方針）

- Core‑15 凍結（第三案）: { Const, UnaryOp, BinOp, Compare, TypeOp, Load, Store, Jump, Branch, Return, Phi, Call, NewBox, BoxCall, ExternCall }
- 統一ルール: ArrayGet/ArraySet, RefGet/RefSet, PluginInvoke はBoxCallに一本化（Optimizerで正規化、Verifierで禁止）
- バリア方針: 初期はランタイム関数側で安全に処理、型特化Lowering段でIRへ内挿（write barrier）

## 🔗 関連フェーズ

- [Phase 10](../phase-10/) - Cranelift JIT（前提）
- [Phase 9](../phase-9/) - 統一Box設計（基盤）
- [00_MASTER_ROADMAP.md](../00_MASTER_ROADMAP.md) - 全体計画
