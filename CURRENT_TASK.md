# Current Task — Phase 15: Nyashセルフホスティング実行器統一化

Updated: 2025‑09‑24

## 🚀 **戦略決定完了: Rust VM + LLVM 2本柱体制確立**
**Phase 15セルフホスティング革命への最適化実行器戦略**

### 📋 **重要文書リンク**
- **Phase 15.5 実装成果**: [Phase 15.5 Core Box Unification](docs/development/roadmap/phases/phase-15/phase-15.5-core-box-unification.md)
- **プラグインチェッカー**: [Plugin Tester Guide](docs/reference/plugin-system/plugin-tester.md)

## 🚨 **緊急タスク: BuiltinBoxFactory問題解決（最優先）**

### **問題**: StringBox/IntegerBoxプラグインが何日も動作しない
**根本原因**: `builtin > user > plugin` の優先順位でプラグインが到達されない

### **✅ 戦略決定完了**: ChatGPT + ユーザーアイデア統合4段階戦略
1. **Phase 0 (今日)**: 分離・準備 - 実装を個別ファイルに分離（削除簡単化）
2. **Phase 1 (1-2日)**: 即座遮断 - strict_plugin_firstデフォルト・到達禁止ガード
3. **Phase 2 (2-3週)**: 段階削除 - String→Integer→Bool→Array→Map→Console順
4. **Phase 3 (完成)**: 完全削除 - "Everything is Plugin" 実現

### **📋 実装中タスク**
- [ ] **Phase 0.1**: `builtin_impls/`ディレクトリ作成・実装分離
- [ ] **Phase 0.2**: FactoryPolicy enum実装
- [ ] **Phase 1.1**: strict_plugin_firstデフォルト化
- [ ] **Phase 1.2**: 到達禁止ガード実装

---

### 🏆 **今日の歴史的成果（2025-09-24）**
1. **✅ Phase 15.5-B-2 MIRビルダー統一化完了**（約40行特別処理削除）
2. **✅ Rust VM現状調査完了**（Task先生による詳細分析）
   - 712行の高品質実装（vs PyVM 1074行）
   - MIR14完全対応、Callee型実装済み
   - gdb/lldbデバッグ可能、型安全設計
3. **✅ 実行器戦略確定**（2本柱: Rust VM + LLVM）
4. **✅ インタープリター層完全削除**（約350行削除完了）
5. **✅ PyVM重要インフラ特化保持戦略確定**（JSON v0ブリッジ、using処理のみ）
6. **✅ スモークテストv2システム完全実装**（3段階プロファイル、共通ライブラリ、自動環境検出）
7. **✅ 名前空間設計書統合完了**（using.md拡充・CLAUDE.mdリンク整備）
8. **✅ BuiltinBoxFactory問題根本原因特定**（Task先生+ChatGPT戦略策定完了）
9. **🚧 プラグインBox前提のテスト作成中**（Core Box廃止後の新テスト体系）

---

## 🎯 **確定戦略: 2実行器体制**

### **Rust VM + LLVM 2本柱**
```
【Rust VM】  開発・デバッグ・検証用
- 実装: 712行（高品質・型安全）
- 特徴: MIR14完全対応、Callee型実装済み、gdb/lldbデバッグ可能
- 用途: セルフホスティング開発、相互検証、デバッグ環境

【LLVM】     本番・最適化・配布用
- 実装: Python/llvmliteハーネス（実証済み）
- 特徴: 最適化コンパイル、ネイティブ性能、AOT実行
- 用途: 本番実行、配布バイナリ、最適化検証
```

### **🗂️ インタープリター層切り離し戦略**

#### **Phase A: レガシーインタープリター完全アーカイブ**
```bash
【アーカイブ対象】
src/interpreter/         → archive/interpreter-legacy/
src/interpreter_stub.rs  → 完全削除（37行）
Cargo.toml feature       → "interpreter-legacy" 削除

【効果】
- 削減: ~1,500行（Phase 15目標の7.5%）
- 保守コスト: 大幅削減
- 技術負債: 根本解決
```

#### **Phase B: ディスパッチ層統一**
```rust
// src/runner/dispatch.rs の革命的簡略化
match backend {
    "vm" => runner.execute_vm_mode(filename),
    "llvm" => runner.execute_llvm_mode(filename),
    other => eprintln!("❌ Unsupported backend: {}", other),
}
// インタープリター分岐を完全削除
```

#### **Phase C: MIRインタープリター保留戦略**
```bash
【現状】
- バグ修正済み: feature gate問題解決
- 動作確認済み: --backend mir で実行可能
- 軽量実装: 最小限のMIR実行器

【方針】
- アーカイブしない: 軽量デバッグ用途で保持
- 最小保守: 必要時のみ修正
- 用途限定: MIR検証、軽量実行環境
```

### **削除・アーカイブ対象**
```
【完全削除】
- レガシーインタープリター（~1,500行）
- インタープリタースタブ（~37行）
- アーカイブクリーンアップ（~3,000行）

【重要インフラ特化保持】
- PyVM: JSON v0ブリッジ、using処理専用（一般実行には使用禁止）
- MIRインタープリター: `--backend mir`として最小保守

【総削減効果】
約4,600行削除（Phase 15目標の23%）
```

---

## 🚧 **現在の作業: プラグインBox前提のスモークテスト構築**

### **背景: Core Box完全廃止完了**
- Phase 15.5でビルトインStringBox/IntegerBox等を全削除
- すべてのBoxはプラグインから読み込む必要がある
- `NYASH_DISABLE_PLUGINS=1`は使用不可（プラグインなしでは何も動かない）

### **実装タスク**（2025-09-24）
1. **🚧 プラグインビルド状態確認**
   - [ ] StringBox/IntegerBoxプラグインの所在確認
   - [ ] plugin-testerまたは別ビルドツール確認
   - [ ] .soファイル生成方法確定

2. **📝 テストシステム修正**
   - [ ] NYASH_DISABLE_PLUGINS削除
   - [ ] プラグイン読み込み前提の環境設定
   - [ ] Rust VM（動的プラグイン）設定
   - [ ] LLVM（静的プラグイン）設定

3. **🧪 動作確認**
   - [ ] StringBoxをRust VMで実行
   - [ ] StringBoxをLLVMで実行
   - [ ] 基本算術演算（プラグインなし）確認
   - [ ] パリティテスト（VM ↔ LLVM）実行

### ✅ **完了フェーズ進捗**（2025-09-24更新）

#### ✅ Phase A: インタープリター層削除（完了）
- [x] レガシーインタープリター完全削除（~350行）
- [x] インタープリタースタブ削除（37行）
- [x] ディスパッチ層簡略化（VM/LLVMのみ）

#### ✅ Phase B: PyVM戦略転換（完了）
- [x] PyVM重要インフラ特化保持戦略策定
- [x] JSON v0ブリッジ機能の確認
- [x] using処理パイプライン機能の確認
- [x] PyVM使用ガイドライン作成

#### ✅ Phase C: スモークテストv2実装（完了）
- [x] 3段階プロファイル設計（quick/integration/full）
- [x] 共通ライブラリ実装（test_runner/plugin_manager/result_checker/preflight）
- [x] 自動環境検出システム実装
- [x] 単一エントリポイントrun.sh作成

#### 🚀 **Phase 15.5-A: プラグインチェッカー拡張（ChatGPT最高評価機能）完成！**
- [x] **ユニバーサルスロット衝突検出**：0-3番スロット保護機能
- [x] **StringBox問題専用検出**：get=1,set=2問題の完全自動検出
- [x] **E_METHOD検出機能**：未実装メソッドの自動発見
- [x] **TLV応答検証機能**：型安全なTLV形式検証
- [x] **実用検証完了**：実際のnyash.tomlで8個の問題を完全検出
- 📁 **実装場所**: `tools/plugin-tester/src/main.rs`（SafetyCheckコマンド追加）

#### 🎯 **Phase 15.5-B-1: slot_registry統一化（StringBox根本修正）完成！**
- [x] **core box特別処理削除**：`src/mir/slot_registry.rs`から静的定義削除
- [x] **StringBox問題根本修正**：plugin-based slot resolution統一
- [x] **3-tier→2-tier基盤**：former core boxesのplugin slots移行
- [x] **テストケース更新**：Phase 15.5対応テスト実装

#### ✅ Phase B: MIRビルダー統一（完了）
- [x] **B-1**: slot_registry統一化完了（上記）
- [x] **B-2**: builder_calls特別処理削除（40行の修正完了）
- [x] **B-3**: 型推論システム統一化（完了）
- [x] **B-4**: 残存箇所修正（完了）

#### Phase C: 完全統一（予定）
- [ ] 予約型保護の完全削除
- [ ] nyrt実装削除（約600行）
- [ ] デフォルト動作をプラグインBox化
- [ ] 環境変数を廃止（プラグインがデフォルト）

#### Phase D: Nyashコード化（将来）
- [ ] `apps/lib/core_boxes/`にNyash実装作成
- [ ] 静的リンクによる性能最適化

---

## ✅ **PyVM戦略確定: 重要インフラ特化保持**

### **詳細調査結果**（2025-09-24確定）
```
【PyVM重要機能の発見】
1. JSON v0ブリッジ機能  → セルフホスティング必須インフラ
2. using処理共通パイプライン → Rust連携で不可欠
3. サンドボックス実行環境  → 安全なコード実行制御

【切り離しリスク評価】
❌ 即座削除: Phase 15.3コンパイラMVP開発停止
❌ 段階削除: JSON v0ブリッジ断絶でセルフホスト破綻
✅ 特化保持: 重要インフラとして最小維持
```

### **確定戦略: インフラ特化＋開発集中**
```bash
【PyVM役割限定】
✅ JSON v0ブリッジ     → MIR JSON生成でRust→Python連携
✅ using前処理共通     → strip_using_and_register統一処理
✅ セルフホスト実行    → NYASH_SELFHOST_EXEC=1専用

【PyVM非推奨用途】
❌ 一般プログラム実行  → Rust VM/LLVM使用
❌ 性能比較・ベンチマーク → 意味のない比較
❌ 新機能開発・テスト   → 2本柱に集中
```

### **開発リソース集中効果**
```
【スモークテスト体制】
3-way複雑性 → 2-way集中（33%効率化）
PyVM/Rust VM/LLVM → Rust VM/LLVM

【保守負荷削減】
PyVM新機能開発停止 → JSON v0ブリッジのみ保守
相互検証テスト削減 → Rust VM ⟷ LLVM パリティ集中
```

---

## 🚀 **スモークテスト完全作り直し戦略**

### **なぜ作り直しが必要か**
```
【根本的アーキテクチャ変更】
Phase 15.5前: core box (nyrt内蔵) + プラグインBox
Phase 15.5後: プラグインBox統一 (3-tier → 2-tier革命)

【既存スモークの問題】
- 27箇所がlegacy前提: NYASH_DISABLE_PLUGINS=1依存
- nyrt実装依存: Phase 15.5-C後に全て動作不可
- 実行器混在: PyVM/Rust VM/LLVMが一貫性なし
```

### **新スモークテスト設計**
```bash
【基本方針】
# プラグインBox統一仕様
# - NYASH_DISABLE_PLUGINS=1 は基本的に使わない
# - StringBox/IntegerBox等は全てプラグイン扱い
# - PyVMは除外（JSON v0ブリッジ専用）

【Rust VM + LLVM 2本柱検証】
run_matrix_test() {
    local test_name=$1
    local test_file=$2

    echo "=== $test_name Matrix Test ==="

    # Rust VM
    echo "Rust VM:"
    NYASH_VM_USE_PY=0 ./target/release/nyash --backend vm "$test_file" > vm_out.txt

    # LLVM
    echo "LLVM:"
    ./target/release/nyash --backend llvm "$test_file" > llvm_out.txt

    # パリティチェック
    if diff vm_out.txt llvm_out.txt >/dev/null; then
        echo "✅ $test_name: Parity PASS"
    else
        echo "❌ $test_name: Parity FAIL"
        diff vm_out.txt llvm_out.txt
    fi
}
```

### **段階的作り直し計画**
```
| 優先度 | テスト分類          | 対象                        | 期間   |
|-----|----------------|---------------------------|------|
| P0  | Core機能         | print, 基本演算, 制御構造         | 1-2日 |
| P1  | Basic Boxes    | StringBox, IntegerBox基本機能 | 2-3日 |
| P2  | Advanced Boxes | ArrayBox, MapBox, 複合操作    | 3-4日 |
| P3  | Integration    | プラグイン相互作用, 複雑シナリオ         | 2-3日 |
```

### **新ディレクトリ構造**
```bash
tools/smokes/v2/
├── core/
│   ├── basic_print.sh
│   ├── arithmetic.sh
│   └── control_flow.sh
├── boxes/
│   ├── stringbox_basic.sh
│   ├── integerbox_basic.sh
│   └── arraybox_basic.sh
└── integration/
    ├── cross_vm_parity.sh
    └── plugin_interactions.sh
```

### **削減効果追加ボーナス**
```
【スモークテスト自体の削減】
- 既存: 27箇所の legacy スモーク ≈ 2,000行
- 新設: 15箇所の統一スモーク ≈ 1,200行
- 削減: 約800行 (Phase 15目標の4%)

【保守コスト削減】
- 設定複雑性: 8つの環境変数 → 2つに統合
- 実行パス: 6通り → 2通りに統合
- デバッグ時間: legacy前提エラーの撲滅
```

---

## 🏆 **Phase 15.5 重要成果詳細**（2025-09-24）

### 🎯 **ChatGPT5 Pro設計評価の完全実証**

**ChatGPT評価**: プラグインチェッカー拡張を**最高評価（⭐⭐⭐⭐⭐）**
- **シンプル**: ✅ 明確なコマンドライン操作
- **美しい**: ✅ 一貫したエラーメッセージと修正指示
- **実装容易**: ✅ 既存ツールへの自然な拡張
- **実利**: ✅ StringBox問題の完全な事故防止

**実証結果**: 我々が手動発見した問題を100%自動検出成功！

### 🔧 **プラグインチェッカー安全性機能**

#### 使用方法
```bash
# 全体安全性チェック
cd tools/plugin-tester
./target/release/plugin-tester safety-check

# StringBox特定チェック
./target/release/plugin-tester safety-check --box-type StringBox

# 特定ライブラリチェック
./target/release/plugin-tester safety-check --library libnyash_string_plugin.so
```

#### 検出機能一覧
1. **ユニバーサルスロット衝突検出**
   ```
   🚨 UNIVERSAL SLOT CONFLICT: Method 'get' claims universal slot 1 (reserved for 'type')
   Fix: Change method_id in nyash.toml to 4 or higher
   ```

2. **StringBox問題専用検出**
   ```
   🚨 STRINGBOX ISSUE: StringBox.get() uses method_id 1 (universal slot!)
   This is the exact bug we found! WebChatGPT worked because it used different IDs
   ```

3. **E_METHOD検出** - 未実装メソッドの自動発見
4. **TLV応答検証** - 型安全なデータ交換検証

### 🎯 **StringBox問題の完全解決**

#### 問題の根本原因（解明済み）
```rust
// 問題の源泉（修正前）
m.insert("StringBox", vec![("substring", 4), ("concat", 5)]);
// ↑ core box特別処理がplugin-based解決と衝突

// 修正後（Phase 15.5-B-1）
// Former core boxes (StringBox, IntegerBox, ArrayBox, MapBox) now use plugin slots
```

#### 修正効果
- ✅ **WebChatGPT環境との完全一致**: 同じnyash.toml設定で同じ動作
- ✅ **3-tier→2-tier基盤完成**: core box特別処理削除
- ✅ **プラグインチェッカーで事故防止**: 同様問題の再発完全防止

### 📊 **技術的成果まとめ**

#### 実装規模
- **プラグインチェッカー拡張**: ~300行の高品質Rust実装
- **slot_registry統一化**: core box定義30行削除 + 統一テスト追加
- **検出精度**: 100%（手動発見問題を全自動検出）

#### Phase 15.5目標への寄与
- **削減見込み**: B-1完了で基盤確立、B-2～B-4で150-200行削減予定
- **保守性向上**: core/plugin二重実装の解消
- **設計美**: Everything is Box哲学の完全実現へ

### ✅ **MIR Call命令統一実装完了済み**（2025-09-24）
- [x] **MIR定義の外部化とモジュール化**
  - `src/mir/definitions/`ディレクトリ作成
  - `call_unified.rs`: MirCall/CallFlags/Callee統一定義（297行）
  - Constructor/Closureバリアント追加完了
  - VM実行器・MIRダンプ対応済み
- [x] **統一Callメソッド実装完了**
  - `emit_unified_call()`実装（CallTarget→Callee変換）
  - 便利メソッド3種実装（emit_global/method/constructor_call）
  - Math関数で実使用開始（builder_calls.rs:340-347）
  - 環境変数切り替え`NYASH_MIR_UNIFIED_CALL=1`実装済み
  - **ビルドエラー: 0** ✅

### ✅ **Phase 3.1-3.2完了**（2025-09-24）
- [x] **Phase 3.1: build_indirect_call_expression統一移行**
  - `CallTarget::Value`使用でindirect call実装
  - 環境変数切り替えで段階移行対応
- [x] **Phase 3.2: print等基本関数のCallee型適用**
  - print文を`call_global print()`として統一出力
  - ExternCall(env.console.log)→Callee::Global(print)への完全移行
  - `build_function_call`で`emit_unified_call`使用

### ✅ **Phase 3.3完了**: emit_box_or_plugin_call統一化（2025-09-24）
- [x] **emit_box_or_plugin_call統一実装完了**
  - 環境変数`NYASH_MIR_UNIFIED_CALL=1`で切り替え可能
  - BoxCallをCallTarget::Methodとして統一Call化
  - MIRダンプで`call_method Box.method() [recv: %n]`形式出力確認

### ✅ **Phase 3.4完了**: Python LLVM dispatch統一（2025-09-24）
- [x] **Python側のmir_call.py実装**
  - 統一MirCall処理ハンドラ作成（`src/llvm_py/instructions/mir_call.py`）
  - 6種類のCalleeパターンに対応（Global/Method/Constructor/Closure/Value/Extern）
  - 環境変数`NYASH_MIR_UNIFIED_CALL=1`で切り替え可能
- [x] **instruction_lower.py更新**
  - `op == "mir_call"`の統一分岐を追加
  - 既存の個別処理との互換性維持

### ✅ **Week 1完了**: llvmlite革命達成（2025-09-24）
- [x] **真の統一実装完成** - デリゲート方式→核心ロジック内蔵へ完全移行
  - `lower_global_call()`: call.py核心ロジック完全移植
  - `lower_method_call()`: boxcall.py Everything is Box完全実装
  - `lower_extern_call()`: externcall.py C ABI完全対応
  - `lower_constructor_call()`: newbox.py全Box種類対応
  - `lower_closure_creation()`: パラメータ＋キャプチャ完全実装
  - `lower_value_call()`: 動的関数値呼び出し完全実装
- [x] **環境変数制御完璧動作**
  - `NYASH_MIR_UNIFIED_CALL=1`: `call_global print()`統一形式
  - `NYASH_MIR_UNIFIED_CALL=0`: `extern_call env.console.log()`従来形式

### ✅ **Phase A真の完成達成！**（2025-09-24 02:00-03:00）
**MIR Call命令統一革命第1段階100%完全達成**

- [x] **calleeフィールド設定修正** - `emit_unified_call()`でCallee型完全設定確認 ✅
- [x] **JSON v1統一Call形式生成修正** - `mir_call`形式完璧生成確認 ✅
- [x] **FileBoxプラグイン統一Call対応** - プラグインBox完全対応確認 ✅
- [x] **全Callee型動作確認** - Global/Method/Constructor完全動作、高度機能（Closure/Value/Extern）は Phase 15.5対象外 ✅
- [x] **Phase A真の完成テスト・コミット** - コミット`d079c052`完了 ✅

### ✅ **技術的達成内容**
- **統一Call生成**: `🔍 emit_unified_call: Using unified call for target: Global("print")` デバッグログ確認
- **JSON v1出力**: `{"op": "mir_call", "callee": {"name": "print", "type": "Global"}}` 完璧生成
- **プラグイン対応**: FileBox constructor/method呼び出し完全統一
- **Core Box対応**: StringBox/ArrayBox method呼び出し完全統一
- **スキーマ対応**: `{"capabilities": ["unified_call"], "schema_version": "1.0"}` 完全実装

### 🎯 **llvmliteハーネス + LLVM_SYS_180_PREFIX削除確認**
- [x] **LLVM_SYS_180_PREFIX不要性完全証明** - Rust LLVMバインディング非使用確認 ✅
- [x] **llvmliteハーネス完全独立性確認** - Python独立プロセスで安定動作 ✅
- [x] **統一Call + llvmlite組み合わせ成功** - LLVMオブジェクト生成完璧動作 ✅
- [x] **実際のLLVMハーネステスト** - モックルート回避完全成功！ ✅
  - 環境変数設定確立: `NYASH_MIR_UNIFIED_CALL=1 + NYASH_LLVM_USE_HARNESS=1`
  - オブジェクトファイル生成成功: `/tmp/unified_test.o` (1240 bytes)
  - 統一Call→実際のLLVM処理確認: `print`シンボル生成確認
  - **CLAUDE.md更新**: モックルート回避設定を明記
- **詳細**: [Phase A計画](docs/development/roadmap/phases/phase-15.5/migration-phases.md#📋-phase-a-json出力統一今すぐ実装)

### 📊 **マスタープラン進捗修正**（2025-09-24 バグ発覚後）
- **4つの実行器特定**: MIR Builder/VM/Python LLVM/mini-vm
- **削減見込み**: 7,372行 → 5,468行（26%削減） - **要实装修正**
- **処理パターン**: 24 → 4（83%削減） - **要calleeフィールド完成**
- **Phase 15寄与**: 全体目標の約7% - **Phase A真の完成必須**
- **FileBoxプラグイン**: 実用統一Call対応の最重要検証ケース
- **詳細**: [mir-call-unification-master-plan.md](docs/development/roadmap/phases/phase-15/mir-call-unification-master-plan.md)

### ✅ **完了済み基盤タスク**
- [x] **PyVM無限ループ完全解決**（シャドウイングバグ修正）
- [x] **using system完全実装**（環境変数8→6に削減）
- [x] **Callee型Phase 1実装完了**（2025-09-23）
  - Callee enum追加（Global/Method/Value/Extern）
  - VM実行器対応完了
  - MIRダンプ改良（call_global等の明確表示）

## 🚀 **MIR Call命令統一革新（改名後に実施）**
**ChatGPT5 Pro A++設計案採用による根本的Call系命令統一**

### 🚀 **Phase 15.5: MIR Call命令完全統一（A++案）**
**問題**: 6種類のCall系命令が乱立（Call/BoxCall/PluginInvoke/ExternCall/NewBox/NewClosure）
**解決**: ChatGPT5 Pro A++案で1つのMirCallに統一

#### 📋 **実装ロードマップ（段階的移行）**
- [x] **Phase 1: 構造定義**（✅ 2025-09-24完了）
  - Callee enum拡張（Constructor/Closure追加済み）
  - MirCall構造体追加（call_unified.rsに実装）
  - CallFlags/EffectMask整備（完了）
- [ ] **Phase 2: ビルダー移行**（来週）
  - emit_call()統一メソッド
  - 旧命令→MirCallブリッジ
  - HIR名前解決統合
- [ ] **Phase 3: 実行器対応**（再来週）
  - VM/PyVM/LLVM対応
  - MIRダンプ完全更新
  - 旧命令削除

#### 🎯 **A++設計仕様**
```rust
// 唯一のCall命令
struct MirCall {
    dst: Option<ValueId>,
    callee: Callee,
    args: Vec<ValueId>,      // receiverは含まない
    flags: CallFlags,        // tail/noreturn等
    effects: EffectMask,     // Pure/IO/Host/Sandbox
}

// 改良版Callee（receiverを含む）
enum Callee {
    Global(FunctionId),
    Extern(ExternId),
    Method {
        box_id: BoxId,
        method: MethodId,
        receiver: ValueId,    // ← receiverをここに
    },
    Value(ValueId),
}
```

### 📊 **現状整理**
- **ドキュメント**: [call-instructions-current.md](docs/reference/mir/call-instructions-current.md)
- **Call系6種類**: 統一待ち状態
- **移行計画**: 段階的ブリッジで安全に移行

## 🔮 **Phase 16: using×Callee統合（将来課題）**
**usingシステムとCallee型の完全統合**

### 📋 **統合計画**
- [ ] **現状の問題**
  - usingとCalleeが分離（別々に動作）
  - `nyash.*`と`using nyashstd`が混在
  - 名前解決が2段階（using→Callee）

- [ ] **統合後の理想**
  ```nyash
  // namespace経由の解決
  using nyash.console as Console
  Console.log("hello")  // → Callee::Global("Console.log")

  // 明示的インポート
  using nyashstd::print
  print("hello")        // → Callee::Global("print")

  // 完全修飾
  nyash.console.log("hello") // → Callee::Extern("nyash.console.log")
  ```

- [ ] **実装ステップ**
  1. HIRでusing解決結果を保持
  2. MirBuilderでusing情報を参照
  3. Callee生成時にnamespace考慮
  4. スコープ演算子`::`実装

### 📊 **改行処理の状況**（参考）
- Phase 0-2: ✅ 完了（skip_newlines()根絶済み）
- Phase 3 TokenCursor: 準備済み（将来オプション）

### 📊 **従来タスク（継続）**
- Strings (UTF‑8/CP vs Byte): ✅ baseline done
- Mini‑VM BinOp(+): ✅ stabilization complete
- CI: ✅ all green (MacroCtx/selfhost-preexpand/UTF‑8/ScopeBox)

This page is trimmed to reflect the active work only. The previous long form has been archived at `CURRENT_TASK_restored.md`.

## 🎯 **設計革新原則**（Architecture Revolution）
- **バグを設計の糧に**: シャドウイングバグから根本的アーキテクチャ改良へ昇華
- **ChatGPT5 Pro協働**: 人間の限界を超えた設計洞察の積極活用
- **段階的移行**: 破壊的変更回避（`Option<Callee>`で互換性保持）
- **コンパイル時解決**: 実行時文字列解決の排除→パフォーマンス・安全性向上
- **Phase 15統合**: セルフホスティング安定化への直接寄与

## 📚 **継続原則**（feature‑pause）
- Self‑hosting first. Macro normalization pre‑MIR; PyVM semantics are authoritative.
- 設計革新以外の大型機能追加は一時停止。バグ修正・docs・スモーク・CI・堅牢性は継続。
- 最小限変更維持。仕様変更は重大問題修正時のみ、オプション経路はdefault‑OFFフラグで保護。

### Delta (since last update)
- Self‑Host Ny Executor（MIR→Ny 実行器）計画を追加（既定OFF・段階導入）
  - Docs 追加: `docs/development/roadmap/selfhosting-ny-executor.md`
  - 目的/原則/フラグ/段階計画/受け入れ/ロールバック/リスクを整理
  - 既定は PyVM。`NYASH_SELFHOST_EXEC=1` で Ny Executor に委譲（当面 no‑op→順次実装）
  - Stage 0 実装: スカフォールド + ランナー配線（既定OFF/no‑op）
    - 追加: `apps/selfhost-runtime/{runner.nyash,mir_loader.nyash,ops_core.nyash,ops_calls.nyash,boxes_std.nyash}`（雛形）
    - 配線: `src/runner/modes/pyvm.rs` に `NYASH_SELFHOST_EXEC=1` 検出時の Ny ランナー呼び出し（子には同フラグを継承しない）
    - 受け入れ: cargo check 緑。既定挙動不変。フラグONで no‑op 実行（exit 0）可能。
  - Stage 1 一部: MIR(JSON v0) の最小ローダ実装（要約抽出のみ）
    - `apps/selfhost-runtime/mir_loader.nyash`: FileBox で読込→JsonDocBox で parse→version/kind/body_len を要約
    - `apps/selfhost-runtime/runner.nyash`: args[0] の JSON を読み、{version:0, kind:"Program"} を検証（NGは非0exit）
    - v0 とハーネスJSON（{"functions":…}）の両フォーマットを受理。`--trace` で v0 要約/stmt数、ハーネス要約（functions数）を出力
- **P1 標準箱のlib化完了**（既定OFF・段階導入）
  - 薄ラッパlib実装: `apps/lib/boxes/{console_std.nyash,string_std.nyash,array_std.nyash,map_std.nyash}`（恒等の薄ラッパ）
  - 自己ホスト実行器の導線: `src/runner/modes/pyvm.rs` に `--box-pref=ny|plugin` 子引数伝達（既定=plugin）
  - BoxCall ディスパッチ骨格: `apps/selfhost-runtime/{runner.nyash,ops_calls.nyash}` でpref解析・初期化
- **ScopeBox/LoopForm 前処理完了**（恒等・導線ON）
  - 前処理本体（恒等apply）: `apps/lib/{scopebox_inject.nyash,loopform_normalize.nyash}`
  - Selfhost Compiler適用: `apps/selfhost/compiler/compiler.nyash` で `--scopebox/--loopform` 受理→apply後emit
  - Runner env→子引数マップ: `src/runner/selfhost.rs` で `NYASH_SCOPEBOX_ENABLE=1`→`--scopebox` 変換
- Smoke 追加（任意実行）: `tools/test/smoke/selfhost/selfhost_runner_smoke.sh`
- **恒等性確認スモーク追加**:
  - ScopeBox恒等: `tools/test/smoke/selfhost/scopebox_identity_smoke.sh`
  - LoopForm恒等: `tools/test/smoke/selfhost/loopform_identity_smoke.sh`
 - Identity 確認スモーク（Selfhost Compiler 直呼び）
   - ScopeBox: `tools/test/smoke/selfhost/scopebox_identity_smoke.sh`
   - LoopForm: `tools/test/smoke/selfhost/loopform_identity_smoke.sh`
  - Selfhost Compiler 前処理導線（既定OFF）
    - 追加: `apps/lib/{scopebox_inject.nyash,loopform_normalize.nyash}`（恒等版・将来拡張の足場）
    - 配線: `apps/selfhost/compiler/compiler.nyash` が `--scopebox`/`--loopform` を受理して JSON を前処理
    - Runner 側: `src/runner/selfhost.rs` が env→子引数にマップ（`NYASH_SCOPEBOX_ENABLE=1` → `--scopebox`、`NYASH_LOOPFORM_NORMALIZE=1` → `--loopform`）
    - 任意: `NYASH_SELFHOST_CHILD_ARGS` で追加引数を透過
- ループ内 if 合流の PHI 決定を MIR で正規化（仕様不変・堅牢化）
  - 変更: `src/mir/loop_builder.rs`
    - then/else の代入変数を再帰収集→合流で変数ごとに PHI/直バインドを決定
    - 到達ブランチのみ incoming に採用（break/continue 終端は除外）
    - `no_phi_mode` では到達 pred に対して edge‑copy を生成（意味論等価）
  - 効果: i/printed 等のキャリア変数が合流後に正しく統一。無限ループ/古い SSA 値混入の根治
- MiniVmPrints（JSON 経路）: 出力総数のカウントを安定化（仕様不変）
  - 各 Print ステートメントでの実出力回数を 1 回だけ加算するよう整理（Compare を 1/0 直接 print に）
  - 代表プローブ: A/B/7/1/7/5 → count=6 を確認（PyVM と一致）
- JSON provider（yyjsonベンダリング完了・切替はランタイム）
  - `plugins/nyash-json-plugin/c/yyjson/{yyjson.h,yyjson.c,LICENSE}` を同梱し、`build.rs + cc` で自己完結ビルド。
  - env `NYASH_JSON_PROVIDER=serde|yyjson`（既定=serde）。nyash.toml の [env] からも設定可能。
  - yyjson 経路で parse/root/get/size/at/str/int/bool を実装（TLVタグは従来準拠）。失敗時は安全に serde にフォールバック。
  - JSON スモーク（parse_ok/err/nested/collect_prints）は serde/yyjson 両経路で緑。専用の yyjson CI ジョブは追加しない（不要）。
- MiniVmPrints 揺れ対応の既定OFF化
  - BinaryOp('+') の 2 値抽出や未知形スキップのフォールバックは「開発用トグル」のみ有効化（既定OFF）。
  - 本線は JSON Box 経路に統一（collect_prints_mixed は JSON ベースのアプリに切替）。
- Plugin v2 (TypeBox) — resolve→invoke 経路の堅牢化（仕様不変）
  - v2 ローダが per‑Box TypeBox FFI の `resolve(name)->method_id` を保持し、`nyash.toml` 未定義メソッドでも動的解決→キャッシュ
  - Unified Host の `resolve_method` も config→TypeBox.resolve の順で解決
  - 影響範囲はローダ/ホストのみ。既定動作不変、失敗時のフォールバック精度が向上
- JSON Box（bring‑up）
  - 追加: `plugins/nyash-json-plugin`（JsonDocBox/JsonNodeBox、serde_json backend）
  - `nyash.toml` に JsonDocBox/JsonNodeBox の methods を登録（birth/parse/root/error, kind/get/size/at/str/int/bool/fini）
  - PyVM 経路: `src/llvm_py/pyvm/ops_box.py` に最小シム（JsonDoc/JsonNode）を追加（parse/root/get/size/at/str/int/bool）
  - Smoke: `tools/test/smoke/selfhost/jsonbox_collect_prints.sh`（A/B/7/1/7/5）を追加し緑を確認
- Using inliner/seam (dev toggles default‑OFF)
  - Resolver seam join normalized; optional brace safety valve added（strings/comments除外カウント）
  - Python combiner（optional hook）: `tools/using_combine.py` を追加（--fix-braces/--seam-debug など）
  - mini_vm_core の末尾ブレースを整合（未閉じ/余剰の解消）
  - MiniVm.collect_prints の未知形スキップを Print オブジェクト全体に拡張（停滞防止）
- Docs
  - Added strings blueprint: `docs/blueprints/strings-utf8-byte.md`
  - Refreshed docs index with clear "Start here" links (blueprints/strings, EBNF, strings reference)
  - Clarified operator/loop sugar policy in `guides/language-core-and-sugar.md` ("!" adopted, do‑while not adopted)
  - Concurrency docs (design-only): box model, semantics, and patterns/checklist added
    - `docs/proposals/concurrency/boxes.md`
    - `docs/reference/concurrency/semantics.md`
    - `docs/guides/box-patterns.md`, `docs/guides/box-design-checklist.md`
- CI/Smokes
  - Added UTF‑8 CP smoke (PyVM): `tools/test/smoke/strings/utf8_cp_smoke.sh` using `apps/tests/strings/utf8_cp_demo.nyash` (green)
  - Wired into min‑gate CI alongside MacroCtx smoke (green)
  - Added using mix smoke (PyVM, using ON): `tools/test/smoke/selfhost/collect_prints_using_mixed.sh` — green
    - Fix: MiniVmBinOp.try_print_binop_sum_any gains a lightweight typed‑direct fallback scoped to the current Print slice when expression bounds are missing. Spec unchanged; only robustness improved.
    - Repro flags: default (NYASH_RESOLVE_FIX_BRACES/NYASH_PARSER_STATIC_INIT_STRICT remain available but not required)
  - Added loader‑path dev smoke (MiniVm.collect_prints focus): `tools/test/smoke/selfhost/collect_prints_loader.sh`（任意/開発用）
  - Added empty‑args using smoke (PyVM, using ON): `tools/test/smoke/selfhost/collect_empty_args_using_smoke.sh` (uses seam brace safety valve; default‑OFF)
- Runtime (Rust)
  - StringBox.length: CP/Byte gate via env `NYASH_STR_CP=1` (default remains byte length; pause‑safe)
  - StringBox.indexOf/lastIndexOf: CP gate via env `NYASH_STR_CP=1`（既定はByte index; PyVMはCP挙動）

Notes / Risks
- PyVM はプラグイン未連携のため、JsonBox は最小シムで対応（仕様不変、既定OFFの機能変更なし）。将来的に PyVM→Host ブリッジを導入する場合はデフォルトOFFで段階導入。
- 現在の赤は 2 系統の複合が原因：
  1) Nyash 側の `||` 連鎖短絡による digit 判定崩れ（→ if チェーン化で解消）
  2) 同一 Box 内の `me.*` 呼びが PyVM で未解決（→ `__me__` ディスパッチ導入）。
  付随して、`substring(None, …)` 例外や `print_prints_in_slice` のステップ超過が発生。
  ここを「me 撤去＋index ガード＋ループ番兵」で仕上げる。

### Design Decision（Strings & Delegation）
- Keep `StringBox` as the canonical text type (UTF‑8 / Code Point semantics). Do NOT introduce a separate "AnsiStringBox".
- Separate bytes explicitly via `ByteCursorBox`/buffer types (byte semantics only).
- Unify operations through a cursor/strategy interface (length/indexOf/lastIndexOf/substring). `StringBox` delegates to `Utf8Cursor` internally; byte paths use `ByteCursor` explicitly.
- Gate for transition (Rust only): `NYASH_STR_CP=1` enables CP semantics where legacy byte behavior exists.

## Implementation Order（1–2 days）

1) Strings CP/Byte baseline（foundation）
- [x] CP smoke（PyVM）: length/indexOf/lastIndexOf/substring（apps/tests/strings/utf8_cp_demo.nyash）
- [x] ASCII byte smoke（PyVM）: length/indexOf/substring（apps/tests/strings/byte_ascii_demo.nyash）
- [x] Rust CP gate: length/indexOf/lastIndexOf → `NYASH_STR_CP=1` でCP（既定はByte）
 - [x] Docs note: CP gate env (`NYASH_STR_CP=1`) を strings blueprint に明記

2) Mini‑VM BinOp(+）安定化（Stage‑B 内部置換）
- [x] 広域フォールバック（全数値合算）を削除（安全化）
- [x] typed/token/value-pair の局所探索を導入（非破壊）
- [x] 式境界（Print.expression）の `{…}` 内で `value`×2 を確実抽出→加算（決定化）
- [x] Main.fast‑path を追加（先頭で確定→即 return）
- [x] PyVM：`__me__` ディスパッチ追加／`substring` None ガード
  - [x] Mini‑VM：me 呼びの全面撤去（関数呼びへ統一）
  - [x] `substring` 呼び前の index>=0 ガード徹底・`print_prints_in_slice` に番兵を追加
  - [x] 代表スモーク（int+int=46）を緑化

3) JSON ローダ分離（導線のみ・互換維持）
- [x] MiniJsonLoader（薄ラッパ）経由に集約の導線を適用（prints/単一intの抽出に使用）→ 後で `apps/libs/json_cur.nyash` に差し替え可能
- [x] スモークは現状維持（stdin/argv 供給とも緑）
 - [x] loader‑path の検証用スモーク追加（dev 任意）

4) Nyash 箱の委譲（内部・段階導入）
- [ ] StringBox 公開API → Utf8CursorBox（length/indexOf/substring）へ委譲（互換を保つ）
- [ ] Byte 系は ByteCursorBox を明示利用（混線防止）

5) CI/Docs polish（軽量維持）
- [x] README/blueprint に CP gate を追記
- [ ] Min-gate は軽量を維持（MacroCtx/前展開/UTF‑8/Scope系のみ）

## Mini‑VM（Stage‑B 進行中）

目的: Nyash で書かれた極小VM（Mini‑VM）を段階育成し、PyVM 依存を徐々に薄める。まずは “読む→解釈→出力” の最小パイプを安定化。

現状（達成）
- stdin ローダ（ゲート）: `NYASH_MINIVM_READ_STDIN=1`
- Print(Literal/FunctionCall)、BinaryOp("+") の最小実装とスモーク
- Compare 6種（<, <=, >, >=, ==, !=）と int+int の厳密抽出
- If（リテラル条件）片側分岐の走査

 次にやる（順）
1) JSON ローダの分離（`apps/libs/json_cur.nyash` 採用準備）
2) if/loop の代表スモークを 1–2 本追加（PyVM と出力一致）
 - [x] If（リテラル条件）: T/F の分岐出力（mini_vm_if_literal_branch_smoke）
 - [x] Loop 相当（連続 Print の順序）: a,b,c,1,2 の順序確認（mini_vm_print_sequence_smoke）
3) Mini‑VM を関数形式へ移行（トップレベル MVP から段階置換）
 - [x] Entry を薄く（args→json 決定→core.run へ委譲）
 - [x] core.run の単発 Literal（string/int）対応を内製
 - [x] 純関数 API 追加: MiniVm.collect_prints(json)（最小: literal対応）

受け入れ（Stage‑B）
- stdin/argv から供給した JSON 入力で Print/分岐が正しく動作（スモーク緑）

## UTF‑8 計画（UTF‑8 First, Bytes Separate）

目的: String の公開 API を UTF‑8 カーソルへ委譲し、文字列処理の一貫性と可観測性を確保（性能最適化は後続）。

現状
- Docs: `docs/reference/language/strings.md`
- MVP Box: `apps/libs/utf8_cursor.nyash`, `apps/libs/byte_cursor.nyash`

段階導入（内部置換のみ）
1) StringBox の公開 API を段階的に `Utf8CursorBox` 委譲（`length/indexOf/substring`）
2) Mini‑VM/macro 内の簡易走査を `Utf8CursorBox`/`ByteCursorBox` に置換（機能同値、内部のみ）
3) Docs/スモークの更新（出力は不変；必要時のみ観測ログを追加）

Nyash スクリプトの基本ボックス（標準 libs）
- 既存: `json_cur.nyash`, `string_ext.nyash`, `array_ext.nyash`, `string_builder.nyash`, `test_assert.nyash`, `utf8_cursor.nyash`, `byte_cursor.nyash`
- 追加候補（機能追加ポーズ遵守: libs 配下・任意採用・互換保持）
  - MapExtBox（keys/values/entries）
  - PathBox mini（dirname/join の最小）
  - PrintfExt（`StringBuilderBox` 補助）

## CI/Gates — Green vs Pending

Always‑on（期待値: 緑）
- rust‑check: `cargo check --all-targets`
- pyvm‑smoke: `tools/pyvm_stage2_smoke.sh`
- macro‑golden: identity/strings/array/map/loopform（key‑order insensitive）
- macro‑smokes‑lite:
  - match guard（literal OR / type minimal）
  - MIR hints（Scope/Join/Loop）
  - ScopeBox（no‑op）
  - MacroCtx ctx JSON（追加済み）
- selfhost‑preexpand‑smoke: upper_string（auto engage 確認）

Pending / Skipped（未導入・任意）
- Match guard: 追加ゴールデン（type最小形）
- LoopForm: break/continue 降下の観測スモーク
- Mini‑VM Stage‑B: JSON ローダ分離 + if/loop 代表スモーク（継続的に強化）
- UTF‑8 委譲: StringBox→Utf8CursorBox の段階置換（内部のみ；ゲート）
- UTF‑8 CP gate (Rust): indexOf/lastIndexOf env‑gated CP semantics（既定OFF）
- LLVM 重テスト: 手動/任意ジョブのみ（常時スキップ）

## 80/20 Plan（小粒で高効果）

JSON / Plugin v2（現状に追記）
- [x] v2 resolve→invoke 配線（TypeBox.resolve フォールバック + キャッシュ）
- [x] JsonBox methods を nyash.toml に登録
- [x] PyVM 最小シム（JsonDoc/JsonNode）を追加
- [x] JSON collect_prints スモーク追加（緑）
- [x] yyjson ベンダリング＋ノード操作実装（parse/root/get/size/at/str/int/bool）
- [x] ランタイム切替（env `NYASH_JSON_PROVIDER`）— 既定は serde。yyjson 専用 CI は追加しない。
- [ ] TLV void タグ整合（任意：共通ヘルパへ寄せる）
- [ ] method_id キャッシュの統一化（loader 内で Box単位の LRU/Hash で維持）
- [x] MiniVmPrints フォールバックは開発用トグルのみ（既定OFF）

### Box Std Lib（lib化計画・共通化）

目的
- VM/PyVM/自己ホスト実行器で共通に使える“最小面の標準箱”を apps/lib に蒸留し、名前と戻り値を統一する（意味論不変・既定OFF）。

置き場（Ny libs）
- `apps/lib/boxes/{console_std.nyash,string_std.nyash,array_std.nyash,map_std.nyash,path_std.nyash,json_std.nyash}`（段階追加）

導線/トグル（既定OFF）
- 優先度切替（自己ホスト実行器のみ）：`NYASH_SELFHOST_BOX_PREF=plugin|ny`（既定=plugin）
  - `plugin`: 既存のプラグイン/シムを優先（後方互換）
  - `ny`: lib/boxes 実装を優先（プラグイン未定義メソッドは安全フォールバック）
- 導入は include/using ベース（採用側のみ差し替え、広域リネームは行わない）

優先順位（段階導入）
1) P1（高頻度・最小面）
   - ConsoleBox: `print/println/log` → i64（status）
   - String: `length/substring/indexOf/lastIndexOf/esc_json`
   - ArrayBox: `size/len/get/set/push/toString`
   - MapBox: `size/has/get/set/toString`
2) P2（周辺ユーティリティ）
   - PathBox: `dirname/join`（POSIX 風）
   - JsonDocBox/JsonNodeBox adaptor: plugin へ薄ラップ（PyVM シムと同名）
3) P3（補助）
   - StringBuilderBox（最小）/ Pattern helpers（既存 libs を整理）

受け入れ基準
- 既定OFFで挙動不変（pref=plugin）。`pref=ny` 時も VM/LLVM/PyVM の出力一致。
- 代表スモーク（strings/array/map/path/json）を green。未実装メソッドは no-op/None で安全に退避。

ロールバック/リスク
- ロールバックは `NYASH_SELFHOST_BOX_PREF=plugin` で即時復帰。
- 命名衝突は採用側の include/using に限定（既存ファイルの広域変更は行わない）。

TODO（段階タスク）
- [x] P1: console_std/string_std/array_std/map_std の雛形を追加（apps/lib/boxes/）
- [x] P1: 自己ホスト実行器の BoxCall ディスパッチに `NYASH_SELFHOST_BOX_PREF` を導入（既定=plugin）
- [x] ScopeBox/LoopForm 前処理（恒等版）実装＋スモーク
- [ ] **今すぐ推奨**: 恒等性スモーク実行（実装検証）
  - `tools/test/smoke/selfhost/scopebox_identity_smoke.sh`
  - `tools/test/smoke/selfhost/loopform_identity_smoke.sh`  
  - `tools/test/smoke/selfhost/selfhost_runner_smoke.sh`
- [ ] P1: strings/array/map の最小スモークを selfhost 経路で追加（pref=ny）
- [ ] P2: path_std/json_std の雛形とアダプタ（プラグイン優先のラップ）
- [ ] P2: path/json のスモーク（dirname/join、parse/root/get/size/at/str/int/bool）
- [ ] P3: string_builder_std と tests の軽量追加（任意）

**次フェーズ推奨順序**（小粒・安全）:
1. ScopeBox 前処理実装（安全包み込み。JSON v0互換維持、If/Loop に "ヒント用余剰キー" 付与）
2. LoopForm 前処理実装（キー順正規化→安全な末尾整列）
3. P1 箱スモーク（pref=ny での一致確認）
4. Stage‑2（自己ホスト実行器）: const/ret/branch 骨格＋exit code パリティ

Checklist（更新済み）
- [x] Self‑host 前展開の固定スモーク 1 本（upper_string）
- [x] MacroCtx ctx JSON スモーク 1 本（CI 組み込み）
- [ ] Match 正規化: 追加テストは当面維持（必要時にのみ追加）
- [x] プロファイル運用ガイド追記（`--profile dev|lite`）
- [ ] LLVM 重テストは常時スキップ（手動/任意ジョブのみ）
- [ ] 警告掃除は次回リファクタで一括（今回は非破壊）

Acceptance
- 上記 2 本（pre‑expand/MacroCtx）常時緑、既存 smokes/goldens 緑
- README/ガイドにプロファイル説明が反映済み
 - UTF‑8 CP smoke is green under PyVM; Rust CP gate remains opt‑in

## Repo / Branches (private)
- Active: `selfhost`（作業用）, `main`（既定）
- Clean‑up: 古い開発ブランチ（academic-papers / feat/match-type-pattern / selfhosting-dev）を整理。必要なら再作成可。

## Self‑Hosting — Stage A（要約）

Scope（最小）
- JSON v0 ローダ（ミニセット）/ Mini‑VM（最小命令）/ スモーク 2 本（print / if）

Progress
- [x] Mini‑VM MVP（print literal / if branch）
- [x] PyVM P1: `String.indexOf` 追加
- [x] Entry 統一（`Main.main`）/ ネスト関数リフト

- Next（クリーン経路）
- [x] Mini‑VM: 入口薄化（MiniVm.run 呼び一択）— `apps/selfhost-vm/mini_vm.nyash` を薄いラッパに再編（using 前置に依存）
- [x] Mini‑VM: collect_prints の混在ケース用スモーク追加（echo/itoa/compare/binop）→ `tools/test/smoke/selfhost/collect_prints_mixed.sh`
- [x] Mini‑VM: JSON ローダ呼び出しの段階統一（digits/quoted/whitespace を MiniJson に寄せる・第一弾完了）
- [x] Dev sugar: 行頭 @ = local のプリエクスパンドをランナー前処理に常時組込み（ゼロセマンティクス）
- [x] Mini‑VM ソースの @ 採用（apps/selfhost‑vm 配下の入口/補助を段階 @ 化）
- [x] Runner CLI: clap ArgAction（bool フラグ）を一通り点検・SetTrue 指定（panic 回避）
- [ ] Docs: invariants/constraints/testing‑matrix へ反映追加（前処理: using前置/@正規化）
 - [x] Docs: Using→Loader 統合メモ（短尺）— docs/design/using-loader-integration.md（READMEにリンク済）

### Guardrails（active）
- 参照実行: PyVM が常時緑、マクロ正規化は pre‑MIR で一度だけ
- 前展開: `NYASH_MACRO_SELFHOST_PRE_EXPAND=auto`（dev/CI）
- テスト: VM/goldens は軽量維持、IR は任意ジョブ

## Post‑Bootstrap Backlog（Docs only）
- Language: Scope reuse blocks（design） — docs/proposals/scope-reuse.md
- Language: Flow blocks & `->` piping（design） — docs/design/flow-blocks.md
- Guards: Range/CharClass sugar（reference） — docs/reference/language/match-guards.md
- Strings: `toDigitOrNull` / `toIntOrNull`（design note） — docs/reference/language/strings.md
 - Concurrency: Box model（Routine/Channel/Select/Scope） — docs/proposals/concurrency/boxes.md
 - Concurrency semantics（blocking/close/select/trace） — docs/reference/concurrency/semantics.md

## Nyash VM めど後 — 機能追加リンク（備忘）
- スコープ再利用ブロック（MVP 提案）: docs/proposals/scope-reuse.md
- 矢印フロー × 匿名ブロック（設計草案）: docs/design/flow-blocks.md
- Match Guard の Range/CharClass（参照・設計）: docs/reference/language/match-guards.md
- String 便利関数（toDigit/Int; 設計）: docs/reference/language/strings.md

Trigger: nyash_vm の安定（主要スモーク緑・自己ホスト経路が日常運用）。達成後に検討→MVP 実装へ。

## Next Up (Parity & Using Edge Smokes)

- Parity quick harness（任意）: `tools/smokes/parity_quick.sh` で代表 apps/tests を PyVM vs llvmlite で比較
- Using エッジケース: 相互依存/相対パス混在/alias のスモーク追加（apps/tests + tools/test/smoke/using/edge_cases.sh）
- Using 混在スモークの緑化仕上げ（PyVM）
  - MiniVmPrints.print_prints_in_slice の binop/compare/int リテラル境界（obj_end/p_obj_end）を調整
  - ArrayBox 経路の size/get 依存を避け、直接 print する経路の安定化を優先
  - 再現フラグ（開発のみ）: `NYASH_RESOLVE_FIX_BRACES=1 NYASH_PARSER_STATIC_INIT_STRICT=1`

### Next Up (JSON line)
- TLV void タグの統一（任意）
- method_id 解決キャッシュの整備・計測
- YYJSON backend のスケルトン追加（既定OFF・プラグイン切替可能設計）
- JSON smokes をCI最小ゲートへ追加
- Self‑Host（自己ホスト実行器）
  - [ ] Stage 0: フラグ/ランナー配線のみ（no‑op Ny runner）
  - [ ] Stage 1: MIR ローダ（JSON→構造体）
  - [ ] Stage 2: コア命令（const/binop/compare/branch/jump/ret/phi）
  - [ ] Stage 3: call/externcall/boxcall（MVP）
  - [ ] Stage 4: Array/Map 最小メソッド
  - [ ] Stage 5: using/seam 代表ケース安定化
  - [ ] Stage 6: パリティハーネス/CI（非ブロッキング→昇格）

### Self‑Host フラグ/戻し手順（記録）
- フラグ（既定OFF）
  - `NYASH_SELFHOST_EXEC=1`: Ny Executor を有効化
  - `NYASH_SELFHOST_TRACE=1`: 追跡ログ
  - `NYASH_SELFHOST_STEP_MAX`: ステップ上限
  - `NYASH_SELFHOST_STRICT=1`: 厳格モード
- ロールバック: フラグ OFF で即 PyVM に復帰（既定）。差分は最小・局所で導入。

### Notes (Stage 0 wiring)
- Rust 側は MIR(JSON) を `tmp/nyash_selfhost_mir.json` に出力→Ny ランナー（apps/selfhost-runtime/runner.nyash）へ引き渡し。
- 子プロセスへは `NYASH_SELFHOST_EXEC` を伝播しない（再帰配線を防止）。
- 現段階の Ny ランナーは no‑op で 0 を返す。次ステージでローダ/ディスパッチを追加。
