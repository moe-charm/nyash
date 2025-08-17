# 🎯 現在のタスク (2025-08-17)

## 🚀 **現在進行中: Phase 9.75g-0 型定義ファースト BID-FFI実装**

**目的**: FFI ABI v0準拠のシンプルで動くプラグインシステム構築
**戦略**: 型定義は全部最初に、実装は段階的に（unimplemented!活用）
**期間**: 1週間（2025-08-17〜2025-08-24）
**詳細**: 
- [phase_9_75g_0_revised_type_first_approach.md](../予定/native-plan/issues/phase_9_75g_0_revised_type_first_approach.md)
- [bid_ffi_ai_final_review_2025-08-17.md](../予定/native-plan/issues/bid_ffi_ai_final_review_2025-08-17.md)

### 🎯 今週の実装計画
- **Day 1**: 全型定義（BidType, Transport, Effect, Error）
- **Day 2**: プラグインローダー（dlopen/dlsym）
- **Day 3**: 文字列処理（UTF-8, 所有権）
- **Day 4**: FileBox最小実装
- **Day 5**: エラー処理とOption/Result
- **Day 6-7**: ドキュメント・CI・仕上げ

### 🔑 技術的決定事項
- ポインタ: `usize`（プラットフォーム依存）
- アライメント: 8バイト境界
- 単一エントリーポイント: `nyash_plugin_invoke`
- ターゲット: Linux x86-64限定

## ✅ **完了済み主要成果**

### **MIR 35→26命令削減** (2025-08-17)
- 実装期間: 1日（予定5週間の5%）
- 成果: 26命令体系確立、全バックエンド対応
- 詳細: [mir-26-specification.md](../説明書/reference/mir-26-specification.md)

### **Phase 9.75 RwLock変換** (完了)
- Arc<Mutex> → Arc<RwLock>全Box型変換
- 性能改善達成

### **Phase 9.75e using nyashstd** (完了)
- 標準ライブラリ統合
- リテラル自動変換実装

### **Phase 9.75j 警告削減** (完了)
- 106個→0個（100%削減）

## 🔮 **次期優先タスク**

1. **Phase 8.6: VM性能改善**（緊急）
   - 問題: VMがインタープリターより0.9倍遅い
   - 目標: 2倍以上高速化
   - 詳細: [phase_8_6_vm_performance_improvement.md](../予定/native-plan/issues/phase_8_6_vm_performance_improvement.md)

2. **Phase 9: JIT実装**
   - VM改善後の次ステップ

3. **Phase 10: LLVM Direct AOT**
   - 目標: 100-1000倍高速化
   - 期間: 4-6ヶ月

## 📊 **プロジェクト統計**

- **実行モード**: インタープリター/VM/WASM/AOT（開発中）
- **Box型数**: 16種類（すべてRwLock統一）
- **MIR命令数**: 26（最適化済み）
- **ビルド時間**: 2分以上（改善中）

## 🔧 **開発ガイドライン**

### クイックリファレンス
- [CLAUDE.md](../CLAUDE.md) - 開発者向けガイド
- [copilot_issues.txt](../予定/native-plan/copilot_issues.txt) - Phase順開発計画
- [syntax-cheatsheet.md](../quick-reference/syntax-cheatsheet.md) - 構文早見表

### テスト実行
```bash
# リリースビルド（推奨）
cargo build --release -j32

# 実行
./target/release/nyash program.nyash

# ベンチマーク
./target/release/nyash --benchmark --iterations 100
```

---
**最終更新**: 2025-08-17 21:30  
**次回レビュー**: 2025-08-20（Day 3完了時）