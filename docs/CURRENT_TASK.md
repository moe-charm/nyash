# 🎯 現在のタスク (2025-08-18 更新)

## 🆕 今取り組むタスク（最優先）
- plugin-tester: open/read/write のTLVテスト追加（E2E強化）✅ 完了
- FileBoxプラグイン: invokeに open/read/write/close 実装（BID-1 TLV準拠）✅ 完了
- Nyash本体: `new FileBox(...)` をプラグイン優先で生成（暫定フック）✅ **実装済み（codex）**
- PluginBox: メソッド転送（TLV encode/decode）最小実装 ⚠️ **実装済み（codex）、TLV修正要**

### 本日の成果（2025-08-18 午後）
- plugin-tester `io` サブコマンド追加（open→write→close→open→read 一連動作）
- プラグイン側 `nyash_plugin_invoke` に open/read/write/close 実装＋2段階応答のプリフライト時は副作用なしで必須サイズ返却に修正
- 説明書を追加: `docs/説明書/reference/plugin-tester.md`（使い方・TLV・エラーコード・トラブルシュート）
- FileBox API対応表: `docs/説明書/reference/box-design/filebox-bid-mapping.md` 追加（Nyash API ↔ BID-FFI マッピング）

### 🎉 **ローカル実行テスト結果（2025-08-18 実測）**

#### ✅ **plugin-tester**: 完全動作確認
```bash
$ plugin-tester check libnyash_filebox_plugin.so
✓: Plugin loaded successfully
✓: ABI version: 1
✓: Plugin initialized
Plugin Information: FileBox (ID: 6), Methods: 6
✓: Plugin shutdown completed

$ plugin-tester io libnyash_filebox_plugin.so
✓: birth → instance_id=1
✓: open(w), close, open(r)
⚠️: read rc=-8 (デコードエラー、TLV修正要)
```

#### ✅ **Nyash統合**: 部分的成功（プラグインロード確認）
```bash
$ ./target/debug/nyash local_tests/test_plugin_filebox.nyash
🔌 BID plugin loaded: FileBox (instance_id=1)  ← 成功！
✅ Parse successful!
✅ new FileBox(...) まで到達
⚠️ Segmentation fault (ファイル操作部分、TLV処理改善要)
```

#### 🎯 **codex実装成果（1時間で達成）**
- ✅ **プラグインシステム基盤**: 完全動作
- ✅ **plugin-tester診断ツール**: 汎用設計で完璧動作
- ✅ **Nyash統合**: プラグインロード・Box生成まで成功
- ⚠️ **残り課題**: TLVエンコード/デコード最適化

#### 簡易実行テスト状況（過去ログ参考）
- `nyash` 本体実行（引数なし/単純スクリプト）: ✅ 実行OK
- `plugin-tester io` による FileBox E2E: ✅ open→write→close→open→read でOK
- `nyash` からプラグイン FileBox を new して利用: ⚠️ サンドボックス制約により実行中にSIGKILL（dlopen系の制約）
  - ローカル実行（手元環境）では `cargo build --bin nyash` → `./target/debug/nyash local_tests/test_plugin_filebox.nyash` で動作見込み
  - 期待出力: `READ=Hello from Nyash via plugin!`

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

### ✅ **Day 5 完了！** (2025-08-18)
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
- ✅ **プラグインメソッド呼び出し実装** - execute_plugin_file_method
- ✅ **完全動作確認** - `f.write()`, `f.read()` 完全成功

**重要な発見と対応**:
- 🔍 **セグフォルト調査**: 実際はセグフォルトではなく型エラーが原因
- 🎯 **真の問題**: PluginFileBoxのメソッド呼び出し処理が未実装
- ✅ **解決**: calls.rsにPluginFileBox処理追加、io_methods.rsに実装

**Day 5 最終テスト結果**:
```bash
$ ./target/release/nyash local_tests/test_plugin_filebox.nyash
READ=Hello from Nyash via plugin!
✅ Execution completed successfully!
```

**🚨 発見した設計課題（Day 6対応予定）**:
- **問題**: メソッド名がハードコード（read/write/exists/close）
- **課題**: プラグインからメソッド情報を動的取得すべき
- **目標**: 汎用的なプラグインメソッド呼び出しシステム実装

### 🎯 今週の実装計画（段階的戦略に更新）
- **Day 1**: ✅ BID-1基盤実装（TLV仕様、Handle構造体、エンコード/デコード）
- **Day 2**: ✅ メタデータAPI実装（init/abi/shutdown、HostVtable、レジストリ）
- **Day 3**: ✅ 既存Box統合（StringBox/IntegerBox/FutureBoxブリッジ）**100%完了！**
- **Day 4**: ✅ プラグインシステム基盤（nyash.toml、PluginBox、BoxFactory）**100%完了！**
- **Day 5**: ✅ 実際のプラグインライブラリ作成（.so/.dll、Nyash統合）**完了！**
- **Day 6**: 🎯 動的メソッド呼び出しシステム実装（メソッド名脱ハードコード）
- **Day 7**: 実動作実証とドキュメント（透過的切り替え、開発ガイド）

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
**最終更新**: 2025-08-18 10:00 JST  
**次回レビュー**: 2025-08-18（Nyash統合開始時）

## 🎯 **現在の状況** (2025-08-18)

### ✅ Step 1-3 完了！BID-FFI基盤実装成功

#### 達成した重要な設計原則
1. **Box名非決め打ち**: プラグインが「私はFileBoxです」と宣言
2. **汎用的設計**: plugin-testerは任意のプラグインに対応
3. **メモリ管理明確化**: birth/finiライフサイクル実装

#### 実装完了項目
- ✅ **FileBoxプラグイン**: 293KB .soファイル生成、6メソッド実装
- ✅ **nyash.toml**: プラグイン設定ファイル（FileBox = "nyash-filebox-plugin"）
- ✅ **plugin-tester**: 診断ツール完成、メソッド一覧表示機能付き

### テスト結果
```
Plugin Information:
  Box Type: FileBox (ID: 6)  ← プラグインから取得！
  Methods: 6
  - birth [ID: 0, Sig: 0xBEEFCAFE] (constructor)
  - open [ID: 1, Sig: 0x12345678]
  - read [ID: 2, Sig: 0x87654321]
  - write [ID: 3, Sig: 0x11223344]
  - close [ID: 4, Sig: 0xABCDEF00]
  - fini [ID: 4294967295, Sig: 0xDEADBEEF] (destructor)
```

### 実装完了ステップ

#### ✅ **Step 1: FileBoxプラグイン単体作成** 
- ✅ plugins/nyash-filebox-plugin/Cargo.toml作成
- ✅ C FFI実装（birth/fini含む）
- ✅ 単体でビルド確認（293KB .soファイル生成）

#### ✅ **Step 2: nyash.toml設定ファイル作成**
- ✅ シンプルな設定ファイル作成
- ✅ FileBox = "nyash-filebox-plugin"の設定
- ✅ プラグイン検索パス定義

#### ✅ **Step 3: プラグインテスター/ローダー作成**
- ✅ **独立したテストツール** `plugin-tester`作成
- ✅ Box名を決め打ちしない汎用設計
- ✅ 実装済み機能：
  - プラグインロードチェック ✅
  - nyash_plugin_init呼び出し確認 ✅
  - メソッド一覧表示 ✅
  - ABI version確認 ✅
- ⏳ 今後の拡張予定：
  - birth/finiライフサイクルテスト
  - メモリリーク検出（valgrind連携）
  - TLVエンコード/デコード検証

### 🎯 **次のステップ: Step 4 - Nyashとの統合**

#### 実装計画
1. **src/bid/モジュール作成**
   - TLVエンコード/デコード実装 ✅ `src/bid/tlv.rs`
   - BidHandle構造体定義 ✅ `src/bid/types.rs`
   - エラーコード定義 ✅ `src/bid/error.rs`

2. **プラグインローダー実装**
   - nyash.tomlパーサー（簡易版）✅ `src/bid/registry.rs`
   - libloadingによる動的ロード ✅ `src/bid/loader.rs`
   - プラグイン初期化・シャットダウン管理 ✅ `src/bid/loader.rs`

3. **BoxFactoryRegistry実装**
   - ビルトインBox vs プラグインBoxの透過的切り替え
   - Box名 → プラグイン名マッピング
   - new FileBox()時の動的ディスパッチ

4. **PluginBoxプロキシ実装**
   - NyashBoxトレイト実装（準備段階、最小のインスタンス管理）
   - メソッド呼び出しをFFI経由で転送（未）
   - birth/finiライフサイクル管理（Dropトレイト）✅ `src/bid/plugin_box.rs`

5. **統合テスト**
   - FileBoxのビルトイン版とプラグイン版の動作比較
   - nyash.tomlありなしでの切り替え確認
   - メモリリークチェック

### 仕様更新完了
- ✅ birth/finiライフサイクル管理を仕様書に追加
- ✅ メモリ所有権ルールを明確化
- ✅ プラグインが割り当てたメモリはプラグインが解放する原則

### 現在のディレクトリ構造
```
nyash-project/nyash/
├── plugins/
│   └── nyash-filebox-plugin/     # ✅ 実装済み
│       ├── Cargo.toml
│       ├── src/lib.rs            # birth/fini含む6メソッド
│       └── .gitignore
├── tools/
│   └── plugin-tester/            # ✅ 実装済み
│       ├── Cargo.toml
│       ├── src/main.rs           # 汎用プラグインチェッカー
│       └── .gitignore
├── nyash.toml                    # ✅ 実装済み
└── src/
    └── bid/                      # ✅ Step 4の基盤作成済み
        ├── mod.rs                # モジュール公開
        ├── loader.rs             # プラグインローダー（libloading, init, ABI検証）
        ├── registry.rs           # 簡易nyash.toml読取＋ロード
        └── plugin_box.rs         # PluginBoxインスタンス（birth/fini）

## ✅ 直近の進捗（2025-08-18 午前）

- plugin-tester: `lifecycle` サブコマンド実装（birth→finiまでE2E確認）
- FileBoxプラグイン: `nyash_plugin_invoke` をBID-1の2段階応答（ShortBuffer=-1）に準拠、birth/fini実装
- Nyash側: `loader/registry/plugin_box` 追加、ビルド通過

### 実行結果（抜粋）
```
$ plugin-tester check libnyash_filebox_plugin.so
✓: ABI version: 1
✓: Plugin initialized
Plugin Information: FileBox(ID:6), Methods: 6

$ plugin-tester lifecycle libnyash_filebox_plugin.so
✓: birth → instance_id=1
✓: fini  → instance 1 cleaned
```

## 🎯 **次アクション（Day 6: 動的メソッド呼び出し革命）**

### 🚨 **緊急課題**: メソッド名脱ハードコード化
現在の実装は `read/write/exists/close` がソースコードに決め打ちされており、BID-FFI理念に反している。

### 🎯 **Day 6 実装計画**
1. **プラグインメタデータからメソッド情報取得**
   - プラグインが持つメソッド一覧を動的に取得
   - メソッドID・シグネチャ・引数情報の活用

2. **汎用プラグインメソッド呼び出しシステム**
   - `execute_plugin_file_method` → `execute_plugin_method_generic`
   - Box型特化処理の廃止
   - TLVエンコード/デコードの汎用化

3. **完全動的システム実現**
   - 新しいプラグインBox追加時のソースコード修正不要
   - nyash.tomlでの設定のみで新Box型対応

### 🔧 **実装順序**
1. プラグインメタデータ取得API強化
2. 汎用メソッド呼び出し処理実装
3. 既存execute_plugin_file_method置き換え
4. テスト・動作確認
```

### 重要な技術的決定
1. **プラグイン識別**: プラグインが自らBox名を宣言（type_name）
2. **メソッドID**: 0=birth, MAX=fini、他は任意
3. **メモリ管理**: プラグインが割り当てたメモリはプラグインが解放
4. **エラーコード**: -1〜-5の標準エラーコード定義済み
