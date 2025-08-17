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

### ✅ **Day 2 完了！** (2025-08-17)
**目標**: メタデータAPI実装（ホスト統合・プラグイン情報管理）

**実装完了**:
- ✅ HostVtable: ホスト機能テーブル（alloc/free/wake/log）
- ✅ NyashPluginInfo: プラグイン情報構造体
- ✅ NyashMethodInfo: メソッド情報構造体  
- ✅ C FFI関数シグネチャ定義
- ✅ プラグインライフサイクル管理
- ✅ **テスト7/7合格！** 🎉

### ✅ **Day 3 完了！** (2025-08-17)
**目標**: 既存Box統合（StringBox/IntegerBox/FutureBoxブリッジ）

**実装完了** (100%達成！):
- ✅ BID Box Bridge設計: 既存Box型とBIDハンドルの相互変換インターフェース
- ✅ StringBox BIDブリッジ: Handle/TLV変換実装
- ✅ IntegerBox BIDブリッジ: Handle/TLV変換実装
- ✅ FutureBox BIDブリッジ: 非同期Box型の統合完了
- ✅ BoxRegistry: Box型とハンドルの管理システム
- ✅ 統合テスト: 全Box型ラウンドトリップテスト（4/4合格！）
- ✅ **Everything is Box理論の威力実証！** 🎉

### ✅ **Day 4 完了！** (2025-08-17)
**目標**: プラグインシステム基盤実装

**実装完了** (100%達成！):
- ✅ FileBoxプラグイン設計: open/read/write/close API設計
- ✅ FileBoxプラグイン実装: ハンドル管理・ファイル操作実装
- ✅ **プラグインシステム設計統合**: gemini先生とcodex先生の提案を統合
  - [Box プラグインシステム設計](../説明書/reference/box-design/plugin-system.md) 作成
  - YAML署名DSL仕様確定
  - nyash.tomlによる透過的置き換え設計
- ✅ nyash.tomlパーサー実装（シンプル版）
- ✅ PluginBoxプロキシ実装（最小版）
- ✅ BoxFactoryRegistry: 透過的ビルトイン↔プラグイン切り替え
- ✅ libloadingプラグイン動的ロード基盤
- ✅ **プラグインシステム統合テスト（14/14合格！）** 🎉

### 🎯 **Day 5 一時中断** (2025-08-18)
**目標**: 実際のプラグインライブラリ作成と統合

**実装戦略**:
- **段階的アプローチ**: ビルトインFileBox残して並行運用
- **透過的切り替え**: nyash.tomlで動的選択
- **完全実証**: BID-FFIシステムの実動作確認

**完了タスク**:
- ✅ FileBoxプラグイン用クレート作成（独立ライブラリ）
- ✅ C API実装とエクスポート（libnyash_filebox_plugin.so生成）
- ✅ Nyashインタープリターのプラグインロード統合
- ✅ 透過的切り替え実動作確認（PluginBox生成確認）

**中断理由**: 
- 🚨 **古いプラグインシステムのコードが混在していた**
- 🔧 ソースコードをcommit 3f7d71f（古いプラグイン実装前）に巻き戻し
- 📚 docsフォルダは最新状態を維持
- ✅ nyashバイナリの基本動作確認完了

**再開時の作業**:
- ⏳ BID-FFIシステムをクリーンに再実装
- ⏳ PluginBoxのtoString等メソッド実装
- ⏳ 実際のファイル操作メソッド（open/read/write）動作確認

### 🎯 今週の実装計画（段階的戦略に更新）
- **Day 1**: ✅ BID-1基盤実装（TLV仕様、Handle構造体、エンコード/デコード）
- **Day 2**: ✅ メタデータAPI実装（init/abi/shutdown、HostVtable、レジストリ）
- **Day 3**: ✅ 既存Box統合（StringBox/IntegerBox/FutureBoxブリッジ）**100%完了！**
- **Day 4**: ✅ プラグインシステム基盤（nyash.toml、PluginBox、BoxFactory）**100%完了！**
- **Day 5**: ⏳ 実際のプラグインライブラリ作成（.so/.dll、Nyash統合）**進行中！**
- **Day 6-7**: 実動作実証とドキュメント（透過的切り替え、開発ガイド）

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
- **Box型数**: 16種類（すべてRwLock統一）+ プラグインBox対応
- **MIR命令数**: 26（最適化済み）
- **ビルド時間**: 2分以上（改善中）
- **プラグインシステム**: BID-FFI 90%実装完了！

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
**最終更新**: 2025-08-18 09:00 JST  
**次回レビュー**: 2025-08-18（FileBoxプラグイン作成開始時）

## 🎯 **現在の状況** (2025-08-18)

### クリーンアップ完了
1. **古いプラグインシステム削除**: ソースコードをcommit 3f7d71fに巻き戻し ✅
2. **ドキュメント保持**: docs/は最新の状態を維持 ✅  
3. **基本動作確認**: nyashバイナリが正常動作 ✅
4. **ビルド成功**: `cargo build --release --bin nyash` 完了 ✅

### BID-FFI実装状況
- **仕様**: 完成済み（docs/説明書/reference/box-design/ffi-abi-specification.md）
- **設計**: 完成済み（docs/説明書/reference/box-design/plugin-system.md）
- **基盤コード**: src/bid/モジュールは削除済み（再実装必要）
- **プラグイン**: plugins/nyash-filebox-pluginも削除済み（再作成必要）

### 次のステップ（段階的アプローチ）

#### 📦 **Step 1: FileBoxプラグイン単体作成**
- plugins/nyash-filebox-plugin/Cargo.toml作成
- C FFI実装（birth/fini含む）
- 単体でビルド確認（.soファイル生成）

#### ⚙️ **Step 2: nyash.toml設定ファイル作成**
- シンプルな設定ファイル作成
- FileBox = "filebox"の設定のみ
- パーサーは後で実装

#### 🔌 **Step 3: プラグインテスター/ローダー作成**
- **独立したテストツール** `plugin-tester`作成
- プラグイン開発者向けの診断機能：
  - プラグインロードチェック
  - nyash_plugin_init呼び出し確認
  - メソッド一覧表示
  - birth/finiライフサイクルテスト
  - メモリリーク検出（valgrind連携）
  - TLVエンコード/デコード検証
- コマンド例：`./plugin-tester check libnyash_filebox_plugin.so`
- 成功部分を後でNyashに移植

#### 🎯 **Step 4: Nyashとの統合**
- src/bid/モジュール作成
- BoxFactoryRegistry実装
- PluginBoxプロキシ実装
- 実動作確認

### 仕様更新完了
- ✅ birth/finiライフサイクル管理を仕様書に追加
- ✅ メモリ所有権ルールを明確化
- ✅ プラグインが割り当てたメモリはプラグインが解放する原則

### 予定ディレクトリ構造
```
nyash-project/nyash/
├── plugins/
│   └── nyash-filebox-plugin/     # Step 1: プラグイン単体
│       ├── Cargo.toml
│       └── src/lib.rs
├── tools/
│   └── plugin-tester/            # Step 3: テストツール
│       ├── Cargo.toml
│       └── src/main.rs
├── nyash.toml                    # Step 2: 設定ファイル
└── src/
    └── bid/                      # Step 4: 統合時に作成
        ├── mod.rs
        ├── loader.rs
        └── registry.rs
```