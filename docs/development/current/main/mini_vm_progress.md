# Mini-VM Implementation Progress (Daily Log)

**開始日**: 2025-10-09
**戦略**: Choice A'' (Macro-Only) - Step 2: Mini-VM実装 with @match
**期間見積もり**: 10-15人日

---

## 📋 Phase 概要

- **Phase 1**: 基盤構築（3-5人日）- Const/Ret/基本演算
- **Phase 2**: 演算・比較（2-3人日）- BinOp/Compare/TypeOp
- **Phase 3**: 制御フロー（2-3人日）- Branch/Jump/Phi
- **Phase 4**: 呼び出し（2-3人日）- MirCall統一
- **Phase 5**: 残り命令（1-2人日）- Load/Store/GC

---

## 🎯 Phase 1: 基盤構築（Day 1-5）

### Day 0: 準備（2025-10-09）

**完了事項**:
- ✅ INSTRUCTION_SET.md 精読
- ✅ LLVM Python phi.py 精読（197行）
- ✅ Rust VM exec.rs PHI処理精読
- ✅ 既存 Mini-VM 構造調査（1,802行）
- ✅ 失敗記録テンプレート作成

**戦略決定**:
- ✅ 新規実装アプローチ採用（既存リファクタリングでなく）
- ✅ ディレクトリ: `apps/selfhost/hakorune-vm/`（Mini-VM v2 → Hakorune VM に命名変更）
- ✅ @match 最大限活用（@enum Result での error handling）

**次のステップ**:
- ✅ Phase 1 Day 1 完了: HakoruneVmCore実装＋テスト成功

---

### Day 1: JSON MIRパーサー基盤（2025-10-09 完了✅）

**目標**: JsonCursorBox活用で block/instructions 構造解析

**完了事項**:
- ✅ HakoruneVmCore 骨格作成（288行）
- ✅ JSON block/instructions パーサー実装（JsonCursorBox.seek_obj_end/seek_array_end活用）
- ✅ レジスタMap初期化（MapBox使用）
- ✅ テストケース作成: test_phase1_minimal.hako
- ✅ 4命令実装: Const/BinOp(Add)/Ret/Copy
- ✅ @match命令ディスパッチ実装
- ✅ Result @enum エラーハンドリング実装
- ✅ 3テスト全PASS: const 42, 10+32, copy 42

**実装ファイル**:
- `apps/selfhost/hakorune-vm/hakorune_vm_core.hako` (288行)
- `apps/selfhost/hakorune-vm/tests/test_phase1_minimal.hako` (43行)

**技術的成果**:
- @match でのinstruction dispatch成功（"const"/"binop"/"ret"/"copy" → handler関数）
- Result.Ok()/Result.Err() によるエラー伝播
- JsonCursorBox による JSON traversal（seek_obj_end/seek_array_end）
- StringHelpers/StringOps 統合活用（int_to_str, to_i64, index_of_from, read_digits）

**見積もり**: 4時間
**実績**: 約6時間（using system問題で+2時間）

**失敗記録**:
1. **Using system設定ミス**: hako.toml の NYASH_USING="0" により using が完全無効化されていた
   - 解決: HAKO_USING="1" + HAKO_ALLOW_USING_FILE="1" + HAKO_ROOT設定
   - 時間損失: 約1時間

2. **@match制約（return文禁止）**: match arm内でreturn文を使うとterminator errorになる
   - 原因: @matchは式評価、returnは文（terminator）
   - 回避策: early returnが必要な場合はif-elseを使う
   - 影響箇所: run() の Result処理、_handle_binop() のエラーケース
   - 時間損失: 約30分

3. **StringOps vs StringHelpers混同**: index_of_from を StringHelpers で探していた
   - 正: StringOps.index_of_from
   - 修正箇所: 4箇所
   - 時間損失: 約10分

**学び**:
- @match は純粋な式評価に最適、制御フロー（early return）には if-else が必要
- using system の環境変数は HAKO_* が優先（NYASH_* より）
- hako.toml の [env] 設定がすべてのフラグより強い

**次のステップ**:
- ✅ Day 2: BinOp全種・Compare全種実装完了（Rust VMバグ発見）

---

### Day 2: BinOp全種・Compare全種実装（2025-10-09 完了✅ - Rust VMバグ発見）

**目標**: BinOp全種（Sub/Mul/Div/Mod）+ Compare全種（Eq/Ne/Lt/Le/Gt/Ge）実装

**完了事項**:
- ✅ BinOp全種実装: Add（既存）, Sub, Mul, Div, Mod
- ✅ ゼロ除算エラーハンドリング（Div/Mod）
- ✅ Compare全種実装: Eq, Ne, Lt, Le, Gt, Ge
- ✅ 比較結果を0/1で返す（false/true）
- ✅ テストケース拡張: 10テスト作成（Test 1-10）
- ✅ @match命令ディスパッチに"compare"追加
- ✅ JSON parsing修正: `seek_obj_end()` はinclusive → `substring(pos, end+1)`必要
- ✅ instruction loop位置更新: `pos = inst_end + 1`

**実装ファイル**:
- `apps/selfhost/hakorune-vm/hakorune_vm_core.hako` (377行 → 387行, +10行)
- `apps/selfhost/hakorune-vm/tests/test_phase1_minimal.hako` (43行 → 113行, +70行)

**技術的成果**:
- BinOp 5種類実装完了（Add/Sub/Mul/Div/Mod）
- Compare 6種類実装完了（Eq/Ne/Lt/Le/Gt/Ge）
- JSON parsing修正（seek_obj_end inclusiveバグ修正）
- Instruction loop修正（position更新ミス修正）

**見積もり**: 4時間
**実績**: 約8時間（+100%超過）

**❌ 重大な問題発見: Rust VM result_val変数消失バグ**:

**現象**:
```hako
// Line 219: Sub計算実行
result_val = lhs_val - rhs_val  // 50 - 8 = 42

// 直後のprint（デバッグ時）: ✅ 正常
print("Sub result: 50 - 8 = 42")

// 次の行のprint: ❌ 異常
print("Before set: result_val=0")  // result_val が 0 に変わっている！
```

**詳細デバッグログ**:
```
[DEBUG binop] kind=[Sub] len=3 lhs=50 rhs=8
[DEBUG binop] Matched Sub
[DEBUG binop] Sub result: 50 - 8 = 42      ← ✅ 正常
[DEBUG binop] Before set: result_val=0     ← ❌ 異常！
[DEBUG binop] After set: result_val=0
[DEBUG binop] Stored v%3 = 0
```

**問題箇所**: `apps/selfhost/hakorune-vm/hakorune_vm_core.hako:219-237`

**影響範囲**:
- ✅ Test 1-3 PASS (Add/Copy): `kind == "Add"` は動作
- ❌ Test 4-10 FAIL (Sub/Mul/Div/Mod/Compare): すべて0を返す

**原因仮説**:
1. **Rust VM ローカル変数スコープバグ** - else-ifブロック内のローカル変数が正しく保持されない
2. **StringHelpers.int_to_str() 副作用** - print内の関数呼び出しがresult_valを破壊？
3. **レジスタMap.set() 副作用** - regs.set()呼び出しがスタックを破壊？

**回避策候補**:
- [ ] result_valを別変数に保存してからregs.set()
- [ ] if-elseの代わりに@matchを使う（Day 1で制約発見したが再検討）
- [ ] 計算結果を直接regs.set()に渡す

**再現手順**:
```bash
source tools/dev_env.sh using
NYASH_USING_AST=1 NYASH_DISABLE_PLUGINS=1 ./target/release/hako apps/selfhost/hakorune-vm/tests/test_phase1_minimal.hako
# Test 4-10 すべてFAIL（expected 42/1, got 0）
```

**次のステップ**:
- [x] Task Teacher + ultrathinkでRust VMバグ調査 → **根本原因発見！**
- [x] Rust VM if_phi.rs のextract_assigned_var修正
- [x] 修正検証完了（全10テストPASS）
- [ ] Rust VM issueとして報告（既に修正済み）

---

### 🎉 **Day 2 バグ修正完了！** (2025-10-09)

#### ✅ **根本原因特定**

**場所**: `src/mir/phi_core/if_phi.rs` Line 64-71

**関数**: `extract_assigned_var`

**問題**:
- else-if は nested IF として解析される
- nested IF の `then` branch のみに代入がある場合: `(Some("result"), None)`
- 既存コードは `_ => None` でマッチ → 親 IF が「else branch は変数代入なし」と誤判定
- PHI merge が pre-if 値（0）を使用してしまう

**修正内容**:
```rust
// Before:
match (tvar, evar) {
    (Some(tv), Some(ev)) if tv == ev => Some(tv),
    _ => None,  // ❌ (Some, None) を None 扱い
}

// After:
match (tvar, evar) {
    (Some(tv), Some(ev)) if tv == ev => Some(tv),
    (Some(tv), None) => Some(tv),  // ✅ Fix
    (None, Some(ev)) => Some(ev),  // ✅ Fix
    _ => None,
}
```

#### ✅ **修正検証結果**

**実行コマンド**:
```bash
source tools/dev_env.sh using
NYASH_USING_AST=1 NYASH_DISABLE_PLUGINS=1 ./target/release/hako apps/selfhost/hakorune-vm/tests/test_phase1_minimal.hako
```

**結果**: 全10テストPASS！ 🎉
```
Test 1: const 42 + ret → 42 ✅
Test 2: 10 + 32 → 42 ✅
Test 3: copy 42 → 42 ✅
Test 4: 50 - 8 → 42 ✅ (修正前: 0)
Test 5: 6 * 7 → 42 ✅ (修正前: 0)
Test 6: 84 / 2 → 42 ✅ (修正前: 0)
Test 7: 127 % 85 → 42 ✅ (修正前: 0)
Test 8: 42 == 42 → 1 ✅ (修正前: 0)
Test 9: 41 < 42 → 1 ✅ (修正前: 0)
Test 10: 42 >= 42 → 1 ✅ (修正前: 0)
[SUCCESS] All Phase 1 Day 2 tests passed! 🎉 (10/10)
```

#### 📊 **Day 2 最終統計**

- **見積もり**: 4時間
- **実績**: 約12時間（デバッグ含む）
- **超過時間**: 8時間（200%超過）
- **超過理由**: JSON parsing（1.5時間）+ Rust VMバグ調査・修正（6時間）+ その他（0.5時間）
- **コード行数**: hakorune_vm_core.hako: 389行 (+10行 from Day 1)
- **実装命令数**: 9/16（Const, BinOp x5, Compare x6, Ret, Copy）
- **テスト成功率**: 10/10 (100%) ✅
- **新規バグ発見**: 1件（Rust VM else-if PHI bug）
- **バグ修正**: 1件（src/mir/phi_core/if_phi.rs）

#### 🎯 **学び**

1. **else-if は nested IF**: パーサーの内部表現を理解することの重要性
2. **MIR Builder の最適化バグ**: PHI 処理は複雑で、edge case に注意が必要
3. **Task Teacher 有効性**: 複雑なバグ調査に Task Teacher が非常に有効
4. **最小再現コード**: バグ調査の基本は最小再現コード作成
5. **デバッグトレース**: print デバッグは変数の直前/直後に入れるべき

**次のステップ**:
- [x] Phase 1 Day 3: 制御フロー実装（Branch/Jump/Phi）→ 完了！

---

### Day 3: 制御フロー実装（Branch/Jump/Phi）+ 箱化モジュール化 (2025-10-09 完了✅)

**目標**: 複数ブロック実行・Branch/Jump/Phi 実装 + 箱化モジュール化

**完了事項**:
- ✅ **3つの新箱作成（箱化モジュール化戦略）**:
  1. **BlockMapperBox** (77行) - ブロックマップ作成
  2. **TerminatorHandlerBox** (208行) - Ret/Jump/Branch 処理
  3. **PhiHandlerBox** (223行) - PHI 命令処理（predecessor を使って値マージ）

- ✅ **HakoruneVmCore 拡張**:
  - `_execute_blocks()` - 複数ブロック実行ループ（新規）
  - `_execute_instructions_in_block()` - ブロック内命令実行（新規）
  - `_dispatch_instruction()` - PHIスキップ追加

- ✅ **テストケース作成**:
  - `test_phase1_day3.hako` (5テストケース)
    1. Simple Jump (block 0 → block 1)
    2. Branch (true path: 1 == 1 → 100)
    3. Branch (false path: 1 == 2 → 200)
    4. PHI merge (then path: 15 > 10 → 42)
    5. PHI merge (else path: 5 > 10 → 24)

- ✅ **全テストPASS**: 5/5 (100%) 🎉

**実装ファイル**:
- `apps/selfhost/hakorune-vm/block_mapper.hako` (77行)
- `apps/selfhost/hakorune-vm/terminator_handler.hako` (208行)
- `apps/selfhost/hakorune-vm/phi_handler.hako` (223行)
- `apps/selfhost/hakorune-vm/hakorune_vm_core.hako` (拡張: +100行)
- `apps/selfhost/hakorune-vm/tests/test_phase1_day3.hako` (103行)

**技術的成果**:
1. **箱化モジュール化**: 各機能を独立した箱に分離（読みやすさ・美しさ・保守性向上）
2. **Branch 命令実装**: condition に応じて then_bb/else_bb を選択
3. **Jump 命令実装**: 無条件に target ブロックへジャンプ
4. **Phi 命令実装**: predecessor を使って複数パスからの値をマージ
5. **複数ブロック実行**: ブロックマップ + ループで任意の制御フローを実行

**見積もり**: 4-6時間
**実績**: 約5時間（箱化により効率的）

**デバッグ結果**:
- 初回テスト時: コアダンプと誤認（実際はタイムアウト不足）
- 個別テスト: すべて成功（Jump/Branch/Phi 動作確認）
- 最終テスト: 全5テストPASS（タイムアウト30秒で実行）

**学び**:
1. **箱化モジュール化の威力**: 機能分離により実装・デバッグが容易
2. **最小テストケース**: 個別テストで各機能を確認してから統合
3. **タイムアウト設定**: 複雑なテストは十分なタイムアウトが必要
4. **PHI処理**: predecessor 記録が重要（どのブロックから来たか）

**次のステップ**:
- Phase 1 Day 4以降: 残り命令実装（TypeOp/Load/Store/ExternCall等）

---

### Day 3 リファクタリング: 箱化モジュール化強化（2025-10-09 完了✅）

**目標**: デッドコード削除 + 命令ハンドラーの箱化モジュール化

#### Option A: デッドコード削除

**削除内容**:
- `_execute_block0()` (35行) - Day 3で`_execute_blocks()`に置き換え済み
- 使用箇所0件確認済み

**成果**:
- 純削減: 35行
- hakorune_vm_core.hako: 488行 → 453行

#### Option C: 命令ハンドラー箱化モジュール化

**新規箱作成** (7箱):
1. **ValueManagerBox** (41行) - レジスタ操作の統一管理
   - `get(regs, reg_id)` - レジスタ値取得（デフォルト0）
   - `set(regs, reg_id, value)` - レジスタ値設定

2. **JsonFieldExtractorBox** (47行) - JSONフィールド抽出の統一
   - `extract_int(json, field_name)` - 整数フィールド抽出
   - `extract_string(json, field_name)` - 文字列フィールド抽出

3. **ConstHandlerBox** (39行) - Const命令専用ハンドラー
4. **BinOpHandlerBox** (70行) - BinOp命令専用ハンドラー
5. **CompareHandlerBox** (77行) - Compare命令専用ハンドラー
6. **CopyHandlerBox** (29行) - Copy命令専用ハンドラー
7. **InstructionDispatcherBox** (55行) - 命令ディスパッチャー

**削除内容**:
- hakorune_vm_core.hako から旧ハンドラー関数を削除: 272行
  - `_dispatch_instruction()`, `_handle_const()`, `_handle_binop()`, 
    `_handle_compare()`, `_handle_copy()`, `_handle_ret()`
  - `_extract_int_field()`, `_load_reg()`

**成果**:
- 純削減: 272行（358行新規作成 - 272行削除 = +86行、でも箱分離で読みやすさ向上）
- hakorune_vm_core.hako: 453行 → 181行 ✨

**設定ファイル更新**:
- `hako.toml`: `[modules.overrides]`に11エイリアス追加
- `nyash.toml`: `[modules]`に12エイリアス追加

**テスト結果**:
- ✅ Day 1+2 tests: 10/10 PASS
- ✅ Day 3 tests: 5/5 PASS
- ✅ **合計**: 15/15 PASS 🎉

**統計まとめ**:
- **合計削減**: 307行（Option A: 35行 + Option C: 272行）
- **新規箱**: 7箱（358行）
- **hakorune_vm_core.hako**: 488行 → 181行（-307行、-63%）
- **箱化後の平均ファイルサイズ**: 51行/箱（読みやすさ大幅向上）

**技術的成果**:
1. **Single Responsibility Principle完全実装**: 各箱が1つの責任のみ
2. **コード重複削減**: レジスタ操作とJSONパースロジックを統一
3. **テスト容易性向上**: 各ハンドラーを独立してテスト可能
4. **保守性向上**: 新規命令追加時の影響範囲を最小化

**学び**:
1. **箱化モジュール化の威力再確認**: 307行削減でも全テストPASS
2. **using system の注意点**: file path using 禁止プロファイル → hako.toml設定必須
3. **indexOf引数エラー**: StringBoxのindexOfは1引数のみ → StringOps.index_of_from使用
4. **result.hakoパス**: apps/lib/result.hako は存在しない → apps/selfhost/vm/boxes/result_box.hako使用

**次のステップ**:
- Phase 2: 残り命令実装（TypeOp/Load/Store/Call/BoxCall等）

---

## **Phase 2 Day 4: UnaryOp実装** (2025-10-09)

**目標**: 単項演算命令（UnaryOp）を実装し、Neg/Not/BitNot演算をサポート

### 📊 **開始状況**
- 実装済み命令: 12/16 (75%)
- 未実装命令: 8/16 (25%)
- Phase 1完了、Phase 2開始

### ✅ **実装完了: UnaryOp命令**

**実装内容**:
1. **UnaryOpHandlerBox作成** (63行)
   - 3種類の単項演算実装:
     - **Neg**: 算術否定 (`result = 0 - operand`)
     - **Not**: 論理否定 (`operand == 0 → 1, else → 0`)
     - **BitNot**: ビット否定 (`result = 0 - operand - 1`)
   - JsonFieldExtractorBox/ValueManagerBox使用
   - 統一Result型パターン

2. **InstructionDispatcherBox更新**
   - using追加: `UnaryOpHandlerBox`
   - @match case追加: `"unaryop" => UnaryOpHandlerBox.handle(...)`

3. **テストスイート作成** (test_phase2_day4.hako)
   - Test 1: Neg (-42 → -42)
   - Test 2: Neg positive (-(5) → -5)
   - Test 3: Not true (!1 → 0)
   - Test 4: Not false (!0 → 1)
   - Test 5: Not non-zero (!42 → 0)
   - Test 6: BitNot (~5 → -6)
   - Test 7: BitNot (~0 → -1)

**新規ファイル**:
- `apps/selfhost/hakorune-vm/unaryop_handler.hako` (63行)
- `apps/selfhost/hakorune-vm/tests/test_phase2_day4.hako` (テストスイート)

**更新ファイル**:
- `instruction_dispatcher.hako`: +1 using, +1 case (55→57行)
- `hako.toml`: +1 module override
- `nyash.toml`: +1 module

**テスト結果**:
- ✅ Phase 1 Day 1+2 tests: 10/10 PASS
- ✅ Phase 1 Day 3 tests: 5/5 PASS
- ✅ Phase 2 Day 4 tests: 7/7 PASS
- ✅ **合計**: 22/22 PASS 🎉

**統計まとめ**:
- **新規箱**: 1箱（63行）
- **実装済み命令**: 13/16 (81%)
- **残り命令**: 7/16 (19%)
- **総箱数**: 12箱
- **箱化後平均サイズ**: 54行/箱

**技術的成果**:
1. **Box-First設計の継続**: UnaryOpHandlerBoxも単一責任原則を遵守
2. **BitNot実装**: 2の補数表現 `~x = -x - 1` を利用
3. **統一テストパターン**: Phase 1と同じテスト構造を踏襲

**学び**:
1. **UnaryOp仕様確認の重要性**: Rust VM実装を参照して正確な動作を確認
2. **テストの網羅性**: 3演算 × 2-3ケース = 7テストで十分なカバレッジ
3. **段階的実装の有効性**: 1命令ずつ確実に実装することで品質維持

**次のステップ**:
- Phase 2 Day 5: TypeOp実装
- Phase 2 Day 6: Load/Store実装
- Phase 4: Call/BoxCall実装（最重要）
