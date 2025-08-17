# 🎯 現在のタスク (2025-08-17)

## 🚀 **現在進行中: Phase 9.75g-0 型定義ファースト BID-FFI実装**

**目的**: FFI ABI v0準拠のシンプルで動くプラグインシステム構築
**戦略**: 型定義は全部最初に、実装は段階的に（unimplemented!活用）
**期間**: 1週間（2025-08-17〜2025-08-24）
**詳細**: 
- [phase_9_75g_0_chatgpt_enhanced_final.md](../予定/native-plan/issues/phase_9_75g_0_chatgpt_enhanced_final.md) ← **ChatGPT最終案採用！**
- [ffi-abi-specification.md](../説明書/reference/box-design/ffi-abi-specification.md) ← **BID-1仕様に更新完了！**

### ✅ **Day 1 完了！** (2025-08-17) 
- ✅ ChatGPT先生の最終レビュー完了
- ✅ ffi-abi-specification.mdをBID-1 Enhanced Editionに更新
- ✅ Handle設計（type_id + instance_id）確定
- ✅ BID-1 TLVフォーマット仕様確定
- ✅ プラグインAPI（nyash_plugin_*）仕様確定
- ✅ **BID-1基盤実装完了！**
  - src/bid/モジュール構造作成
  - TLVエンコード/デコード実装
  - エラーコード定義（BidError）
  - 型システム（BidType, BidHandle）
  - **テスト4/4合格！** 🎉

### 🚀 **Day 2 開始！** (2025-08-17)
**目標**: メタデータAPI実装（ホスト統合・プラグイン情報管理）

**実装予定**:
- [ ] HostVtable: ホスト機能テーブル（alloc/free/wake/log）
- [ ] NyashPluginInfo: プラグイン情報構造体
- [ ] NyashMethodInfo: メソッド情報構造体
- [ ] C FFI関数シグネチャ定義
- [ ] プラグインライフサイクル管理

### 🎯 今週の実装計画（ChatGPT最終案準拠）
- **Day 1**: ✅ BID-1基盤実装（TLV仕様、Handle構造体、エンコード/デコード）
- **Day 2**: メタデータAPI実装（init/abi/shutdown、HostVtable、レジストリ）
- **Day 3**: 既存Box統合（StringBox/IntegerBox/FutureBoxブリッジ）
- **Day 4**: FileBoxプラグイン実装（open/read/write/close）
- **Day 5**: 統合テスト・最適化（メモリリーク検証、性能測定）
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