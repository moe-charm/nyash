# WASM実装ロードマップ（Phase 15.8完了まで）

**最終更新**: 2025-10-01
**現在**: Week 3 - Phase 3.1完了（PHI処理整理）
**残り期間**: 2025-10-01 ~ 10-22（21日間）

---

## 📊 **進捗サマリー**

### ✅ **完了済み** (Week 1-3前半)

#### Week 1完了 ✅ (2025-10-01 ~ 10-07)
- ✅ llvmlite WASM target対応（wasm32-unknown-wasi）
- ✅ WASMビルドパイプライン確立（llvmlite直接生成）
- ✅ 統合テストスクリプト作成

#### Week 2完了 ✅ (2025-10-08 ~ 10-14)
- ✅ Phase 2.1: 関数エクスポート解決（Python自己完結型）
- ✅ Phase 2.2: WASI fd_write実装
- ✅ Phase 2.3: 文字列constants処理（Hello World成功）
- ✅ Phase 2.4: binop全演算動作確認
- ✅ Phase 2.5: targets/箱理論分離設計
- ✅ Phase 2.6: compare/branch/jump実装
- ✅ Phase 2.7: スモークテスト整備（3テスト）

#### Week 3前半完了 ✅ (2025-10-15 ~ 10-01)
- ✅ Phase 3.1: PHI処理完全修正
  - PhiHandler箱化実装（197行）
  - デッドコード削除（instruction_lower.py）
  - フェイルファスト化（block_lower.py）
  - vmap二重登録の理由明確化
  - ドキュメント整備（phi_design.md）

---

## 🔧 **実装済みMIR命令** (18/18命令) 🎉🎉🎉

| 命令 | 状態 | Week | テスト |
|------|------|------|--------|
| **const** | ✅ 完了 | W2 | test_binop_*.json |
| **binop** | ✅ 完了 | W2 | test_arithmetic_smoke.json |
| **compare** | ✅ 完了 | W2 | test_compare_smoke.json |
| **branch** | ✅ 完了 | W2 | test_branch_if.json |
| **jump** | ✅ 完了 | W2 | test_jump_*.json |
| **ret** | ✅ 完了 | W2 | test_control_flow_smoke.json |
| **externcall** | ✅ 完了 | W2 | Hello World (print) |
| **call** | ✅ 完了 | - | 既存実装（mir_call.py） |
| **constants** | ✅ 完了 | W2 | グローバル文字列 |
| **phi** | ✅ 完了 | W3 | test_phi_if.json + **test_phi_loop.json** |
| **unop** | ✅ 完了 | W3 | test_unaryop_basic.json |
| **typeop** | ✅ 完了 | W3 | test_typeop_cast.json |
| **copy** | ✅ 完了 | W3 | test_copy_simple.json |
| **load** | ✅ 完了 | W3 | test_memory_basic.json |
| **store** | ✅ 完了 | W3 | test_memory_basic.json |
| **newbox** | ✅ 完了 | W3 | test_newbox_simple.json |
| **boxcall** | ✅ 完了 | W3 | test_boxcall_method.json |
| **safepoint** | ✅ 完了 | W3 | test_safepoint_nop.json |
| **barrier** | ✅ 完了 | W3 | test_barrier_nop.json |

**🎉🎉🎉 MIR18命令セット100%完全実装達成！**

---

## 📋 **未実装MIR命令** (0/18命令)

**すべて実装完了！** 🎊

### ✅ **Phase 3.2: 基本命令完成** (完了 2025-10-01)
**期間**: 2025-10-02 ~ 10-04（3日間） → **1日で完了！**

#### 実装内容
1. ✅ **unaryop** (単項演算)
   - LLVM IR: neg, not, bitwise_not
   - テスト: `test_unaryop_basic.json`

2. ✅ **typeop** (型変換)
   - LLVM IR: zext, sext, trunc, bitcast
   - テスト: `test_typeop_cast.json`

3. ✅ **copy/nop** (コピー・nop)
   - LLVM IR: local.get (copy), nop
   - テスト: `test_copy_simple.json`

#### 成果
- 🎉 基本演算完全対応
- ✅ スモークテスト3本追加

---

### ✅ **Phase 3.3: Box/GC命令実装** (完了 2025-10-01)
**期間**: 2025-10-05 ~ 10-09（5日間） → **1日で完了！**

#### 実装内容
1. ✅ **load/store** (メモリアクセス)
   - 新規: `src/llvm_py/instructions/memory.py`
   - LLVM IR: load, store
   - Memory Model: inttoptr/ptrtoint変換
   - テスト: `test_memory_basic.json`

2. ✅ **newbox** (Box生成)
   - LLVM IR: call @gc_alloc
   - テスト: `test_newbox_simple.json`

3. ✅ **boxcall** (Boxメソッド呼び出し)
   - LLVM IR: call_indirect
   - method_id経由呼び出し
   - テスト: `test_boxcall_method.json`

#### 成果
- 🎉 Memory Model実装完了
- ✅ Box生成・メソッド呼び出し対応
- ✅ スモークテスト3本追加

---

### ✅ **Phase 3.4: GC補助命令実装** (完了 2025-10-01)
**期間**: 2025-10-10 ~ 10-12（3日間） → **1日で完了！**

#### 実装内容
1. ✅ **safepoint** (GCセーフポイント)
   - LLVM IR: call @ny_safepoint
   - live値トラッキング
   - テスト: `test_safepoint_nop.json`

2. ✅ **barrier** (メモリバリア)
   - LLVM IR: fence seq_cst
   - Phase 15.8ではnop（将来WASM GC proposal対応）
   - テスト: `test_barrier_nop.json`

#### 成果
- 🎉 GC補助命令skeleton実装完了
- ✅ WASM GC proposal対応準備完了
- ✅ スモークテスト2本追加

---

### ✅ **Phase 3.5-A/B: ループPHI完全対応** (完了 2025-10-01) 🎉🎉🎉
**期間**: 2025-10-13 ~ 10-15（3日間） → **1日で完了！**

#### 実装内容
1. ✅ **PhiHandler拡張**
   - `incomplete_phis`トラッキング追加（前方参照対応）
   - `complete_incomplete_phis()`メソッド実装
   - 二段階PHI解決（即座+遅延）

2. ✅ **block_lower.py統合**
   - PHI完成呼び出し追加（body ops後）
   - `_current_vmap`優先参照

3. ✅ **llvmlite検証**
   - `test_phi_delayed.py`で`add_incoming`遅延追加確認

#### 成果
- 🎉🎉🎉 **ループPHI完全動作！**
- ✅ 前方参照（forward reference）解決
- ✅ テストファイル: `test_phi_loop.json`, `test_phi_delayed.py`

#### LLVM IR生成例
```llvm
bb1:
  %"phi_2" = phi i64 [0, %"bb0"], [%"add_4", %"bb1"]  ← 自己参照！
  %"add_4" = add i64 %"phi_2", 1
  %"cmp_6" = icmp slt i64 %"phi_2", 10
  br i1 %"cmp_6", label %"bb1", label %"bb2"
```

---

### 🚧 **Phase 3.5-C/D/E: 複雑制御フロー** (残り - 優先度A)
**期間**: 2025-10-01 ~ 10-03（3日間予定）

#### 残実装対象
1. **ネストif**
   - テスト: `test_controlflow_nested_if.json`
   - if内if、多段分岐

2. **ネストループ**
   - テスト: `test_controlflow_nested_loop.json`
   - ループ内ループ、break/continue

3. **switch文相当**
   - テスト: `test_controlflow_switch.json`
   - 多分岐パターン

#### 期待成果
- ✅ 複雑制御フローテスト完備
- ✅ PhiHandlerの堅牢性確認

---

### 🎉 **Phase 3.6: Parity確認 + 統合テスト** (優先度S)
**期間**: 2025-10-16 ~ 10-19（4日間）

#### 実装対象
1. **VM/LLVM/WASMパリティ確認**
   - 既存quickスモーク → WASM変換
   - 同一出力確認スクリプト作成
   - 工数: 2日

2. **統合スモークテスト**
   - `tools/smokes/v2/profiles/wasm/` 作成
   - 15本のWASMスモークテスト
   - 一括実行スクリプト
   - 工数: 2日

#### 成果物
- ✅ VM/LLVM/WASMパリティ確認レポート
- ✅ WASMスモークプロファイル完備
- ✅ 回帰テスト体制確立

---

### 📚 **Phase 3.7: ドキュメント整備** (優先度B)
**期間**: 2025-10-20 ~ 10-22（3日間）

#### 実装対象
1. **WASM実行ガイド**
   - `docs/guides/wasm-execution.md`
   - ビルド方法、実行方法、トラブルシューティング
   - 工数: 1日

2. **WASM命令マッピング表**
   - `src/llvm_py/docs/wasm_instruction_mapping.md`
   - MIR18命令 → LLVM IR → WASM対応表
   - 工数: 1日

3. **Phase 15.8完了レポート**
   - 成果サマリー、残課題、Phase 16提案
   - 工数: 1日

#### 成果物
- ✅ WASM実行ガイド完備
- ✅ 命令マッピング表完成
- ✅ Phase 15.8完了レポート

---

## 📊 **実装優先度マトリックス**

| Phase | 期間 | 優先度 | 依存関係 | 工数 |
|-------|------|--------|----------|------|
| **3.2: 基本命令** | 10/02-04 | A | なし | 1.5日 |
| **3.3: Box/GC** | 10/05-09 | B | 3.2完了 | 5日 |
| **3.4: GC補助** | 10/10-12 | C | 3.3完了 | 1日 |
| **3.5: ループPHI** | 10/13-15 | A+ | 3.1完了 | 3日 |
| **3.6: Parity** | 10/16-19 | S | 全完了 | 4日 |
| **3.7: ドキュメント** | 10/20-22 | B | 3.6完了 | 3日 |

**合計工数**: 17.5日（余裕含め21日）

---

## 🎯 **マイルストーン**

### M1: 基本命令完全対応 ✅ (10/04目標)
- ✅ unaryop, typeop, copy実装
- ✅ 基本演算テスト全PASS

### M2: Box/GC実装完了 (10/09目標)
- ✅ load/store, newbox, boxcall実装
- ✅ Memory Model確立

### M3: 全MIR18命令対応 (10/12目標)
- ✅ safepoint, barrier skeleton実装
- ✅ 全18命令WASM変換可能

### M4: 複雑制御フロー確認 (10/15目標)
- ✅ ループPHI動作確認
- ✅ ネスト・多段制御フロー対応

### M5: Parity確認完了 (10/19目標)
- ✅ VM/LLVM/WASM同一出力
- ✅ WASMスモークテスト全グリーン

### M6: Phase 15.8完了 (10/22目標)
- ✅ ドキュメント完備
- ✅ Phase 15.8完了レポート提出

---

## 🚨 **リスク管理**

### 高リスク項目
1. **Memory Model実装** (Phase 3.3)
   - linear memory管理の複雑さ
   - Stack/Heap境界の適切な設計
   - 対策: 簡素なモデルから開始、Phase 16で拡張

2. **call_indirect実装** (boxcall)
   - 型テーブル管理の複雑さ
   - 動的dispatch実装
   - 対策: 最小限の型テーブルで開始

3. **Parity確認** (Phase 3.6)
   - 浮動小数点精度差異
   - GC動作の違い
   - 対策: 許容誤差範囲設定、整数演算優先

### 中リスク項目
1. **ループPHI** (Phase 3.5)
   - 自己参照の正しい解決
   - 対策: PhiHandler既存実装で対応済み

2. **WASI連携** (継続課題)
   - fd_write以外のWASI関数
   - 対策: Phase 15.8は最小限、Phase 16で拡張

---

## 📝 **次の作業**

### 🔥 **今日・明日の優先タスク** (10/02 ~ 10/03)

#### Day 1 (10/02)
1. ✅ Phase 3.2開始: unaryop実装
   - `instructions/unop.py` 確認・WASM対応
   - テスト作成: `test_unaryop_basic.json`
   - 動作確認

2. ✅ typeop実装
   - `instructions/typeop.py` 確認・WASM対応
   - テスト作成: `test_typeop_cast.json`

#### Day 2 (10/03)
1. ✅ copy/nop実装
   - `instructions/copy.py` 確認・WASM対応
   - テスト作成: `test_copy_simple.json`

2. ✅ Phase 3.2完了
   - スモークテスト一括実行
   - WASM命令マッピング表更新
   - commit & push

---

## 🎉 **完了時の状態**

### Phase 15.8完了時点 (10/22)
- ✅ **MIR18命令完全対応**: すべてWASM変換可能
- ✅ **VM/LLVM/WASMパリティ**: 同一出力確認
- ✅ **スモークテスト完備**: 20+テスト全グリーン
- ✅ **ドキュメント完備**: 実行ガイド・マッピング表
- ✅ **箱理論実践**: targets/分離、PhiHandler完成

### Phase 16への準備
- 🔜 WASM GC proposal検討
- 🔜 最適化・バンドルサイズ削減
- 🔜 非同期runtime統合（Promise）
- 🔜 DOM API連携準備

---

**担当**: Claude Code + ユーザー協働
**レビュー**: ChatGPT Pro 分析反映
**更新頻度**: 毎Phase完了時
