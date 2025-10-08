# Mini-VM Implementation Lessons (失敗記録)

**開始日**: 2025-10-09
**目的**: 失敗から学び、同じ間違いを繰り返さない

---

## 🚨 失敗報告の重要性（最優先！）

**プログラム開発では失敗報告が一番大事**

成功報告より失敗報告が重要な理由:
- ✅ 失敗は**次の改善の種**（成功は既に終わったこと）
- ✅ 失敗は**学習の最大の機会**（同じミスを繰り返さない）
- ✅ 失敗は**システムの脆弱性を教えてくれる**（本番障害を未然に防ぐ）
- ✅ 失敗は**見積もり精度を上げる**（楽観的予測を修正）

---

## 🎯 Phase 1: 基盤構築の失敗記録

### Phase 0 段階の学び（準備完了）

**学び**:
1. **ドキュメントファースト**: INSTRUCTION_SET.md/参考実装精読が重要
2. **既存実装の理解**: 既存 Mini-VM の問題点把握が戦略決定に必須
3. **新規 vs リファクタリング**: @match設計の一貫性のため新規実装を選択
4. **失敗記録準備**: 開始前にテンプレート準備が重要

**成功要因**:
- Choice A'' 戦略により @enum/@match が既に完備
- Phase 15.13-15.15 で既存コードへの @match 適用実績あり
- Task Teacher 活用で多角的調査完了

---

### Day 1: JSON MIRパーサー基盤（2025-10-09）

#### ❌ **失敗1: Using System設定ミス**

**問題**: `using "apps/selfhost/..."` がすべて "file paths are disallowed" エラー

**期待**: ファイルパスusing が動作する

**実際**:
- NYASH_USING_AST=1 でも失敗
- NYASH_USING_PROFILE=dev でも失敗
- NYASH_SKIP_TOML_ENV=1 でも失敗

**根本原因**: `hako.toml` の `[env]` セクションに `NYASH_USING="0"` が設定されており、using機能が**完全無効化**されていた

**解決**:
```toml
[env]
HAKO_USING = "1"
HAKO_USING_STRATEGY = "prelude"
HAKO_ALLOW_USING_FILE = "1"
HAKO_USING_PROFILE = "dev"
```

**時間損失**: 約1時間

**学び**:
1. **hako.toml の [env] は最強**: すべてのコマンドライン環境変数より優先される
2. **HAKO_* vs NYASH_***: 環境変数の名前空間が混在している（統一が必要）
3. **HAKO_ROOT 必須**: using file path は HAKO_ROOT からの相対パス解決が必要
4. **事前確認**: 新機能実装前に using system の設定状態を確認すべき

**再発防止策**:
- [x] **tools/dev_env.sh using 発見！** - ChatGPTアドバイスで判明
  ```bash
  # リポジトリルートで実行（または HAKO_ROOT 設定）
  source tools/dev_env.sh using
  ./target/release/hako apps/.../main.hako
  ```
  - 自動設定: HAKO_USING=1, HAKO_USING_STRATEGY=prelude, HAKO_ALLOW_USING_FILE=1
  - Day 1で手動設定した内容と完全一致！
- [ ] using system のクイックリファレンス作成
- [ ] 環境変数名の統一（NYASH_* → HAKO_*）を検討

**📚 ChatGPT推奨ハンドブック**:
1. **すぐ動かす手順**:
   - リポジトリルートにいること（または HAKO_ROOT 設定）
   - `source tools/dev_env.sh using`
   - `./target/release/hako apps/.../main.hako`

2. **トラブルシューティング**:
   - "Parse error: Expected identifier" → HAKO_USING, STRATEGY=prelude, CWD=ルート確認
   - "using: file paths are disallowed" → dev環境では HAKO_ALLOW_USING_FILE=1

---

#### ❌ **失敗2: @match 制約の理解不足（return文禁止）**

**問題**: @match arm 内で return 文を使うと "Builder emit after terminator forbidden" エラー

**コード**:
```hako
return match result {
  Ok(value) => value
  Err(error) => {
    print("[ERROR]")
    return -1  // ← これがエラー
  }
}
```

**期待**: match で分岐してそれぞれ return できる

**実際**: match arm 内の return がターミネーター扱いになり、その後の式評価ができなくなる

**原因**:
- @match は**式（expression）**として設計されている
- return は**文（statement）**であり**ターミネーター**
- match arm は式を返す必要があるが、return はその後の評価を不可能にする

**回避策**:
```hako
// パターン1: if-else に変更
if result.is_Ok() {
  return result.as_Ok()
} else {
  print("[ERROR]")
  return -1
}

// パターン2: match で値を取得してから return
local value = match result {
  Ok(v) => v
  Err(e) => {
    print("[ERROR]: " + e)
    -1
  }
}
return value
```

**影響箇所**:
1. `run()` 関数の Result 処理
2. `_handle_binop()` のエラーケース

**時間損失**: 約30分

**学び**:
1. **@match は純粋な式評価に最適**: 値を計算して返す用途
2. **制御フロー（early return）には if-else**: 複数の脱出点が必要な場合
3. **@match設計思想**: 関数型プログラミング的な式指向設計
4. **Phase 19で気づけなかった**: @match実装時にこの制約を文書化すべきだった

**再発防止策**:
- [ ] @match リファレンスに「return文禁止」を明記
- [ ] 良い例・悪い例をドキュメント化
- [ ] MIR Builder での terminator 後の emit チェックを維持

---

#### ❌ **失敗3: StringOps vs StringHelpers 混同**

**問題**: `StringHelpers.index_of_from` が "Unknown module function" エラー

**期待**: StringHelpers に index_of_from がある

**実際**: index_of_from は StringOps にある

**原因**: 2つの string 関連 Box の役割分担を忘れていた
- StringHelpers: 変換系（int_to_str, to_i64, read_digits, json_quote）
- StringOps: 検索系（index_of_from, substring_from）

**修正**:
```hako
using "apps/selfhost/common/string_ops.hako" as StringOps

// 4箇所修正
StringHelpers.index_of_from → StringOps.index_of_from
```

**時間損失**: 約10分

**学び**:
1. **共通ライブラリの役割分担**: 名前だけでなく役割も明確に
2. **クイックリファレンス必要**: 各 Box の関数一覧をすぐ参照できるように
3. **コード補完の重要性**: IDE support があれば防げた

**再発防止策**:
- [ ] StringHelpers/StringOps のクイックリファレンス作成
- [ ] 関数一覧を各ファイルの冒頭コメントに記載
- [ ] 統合を検討？（または命名を StringConvert/StringSearch に変更）

---

#### ✅ **成功要因**

1. **@match 命令ディスパッチの成功**:
   ```hako
   return match op {
     "const" => me._handle_const(inst_json, regs)
     "binop" => me._handle_binop(inst_json, regs)
     "ret" => me._handle_ret(inst_json, regs)
     "copy" => me._handle_copy(inst_json, regs)
     _ => Result.Err("unsupported instruction: " + op)
   }
   ```
   - 簡潔で読みやすい
   - 新しい命令の追加が容易
   - 未知の命令を明示的にエラー処理

2. **Result @enum によるエラー伝播**:
   - 各 handler が Result.Ok()/Err() を返す
   - エラーが呼び出し元に自動伝播
   - エラーメッセージが context を含む

3. **JsonCursorBox の活用**:
   - seek_obj_end/seek_array_end で JSON traversal が簡潔
   - 手動パース不要

4. **3テスト全成功**: const 42, 10+32, copy 42 すべて期待通り動作

---

#### 📊 **Day 1 統計**

- **見積もり**: 4時間
- **実績**: 約6時間
- **超過時間**: 2時間（50%超過）
- **超過理由**: using system問題（1時間）+ @match制約（0.5時間）+ その他（0.5時間）
- **コード行数**: 331行（hakorune_vm_core.hako: 288, test: 43）
- **実装命令数**: 4/16（Const, BinOp(Add), Ret, Copy）
- **テスト成功率**: 3/3 (100%)

---

### Day 2: BinOp全種・Compare全種実装（2025-10-09）

#### ❌ **失敗1: JSON parsing バグ（seek_obj_end inclusive問題）**

**問題**: すべてのinstruction JSONが閉じ括弧なしで切れる

**デバッグログ**:
```
[DEBUG inst] {"op":"const","dst":1,"value":{"type":"i64","value":50}
[DEBUG inst] {"op":"binop","op_kind":"Sub","dst":3,"lhs":1,"rhs":2
```

**期待**: 完全なJSON `{"op":"binop",...}`

**実際**: 閉じ括弧なし `{"op":"binop",...`

**原因**: `seek_obj_end()` は**inclusive**（閉じ括弧を含む位置を返す）
- 誤: `substring(pos, inst_end)` → 閉じ括弧の1つ前まで
- 正: `substring(pos, inst_end + 1)` → 閉じ括弧を含む

**修正箇所**:
- Line 90: `local inst_json = insts_json.substring(pos, inst_end + 1)`
- Line 110: `pos = inst_end + 1`

**時間損失**: 約1.5時間

**学び**:
1. **seek_* 系APIは inclusive/exclusive を必ず確認**
2. **JSON抽出は最初にprint確認**（構造が正しいか目視）
3. **JsonCursorBox のコメント不足**（inclusiveの明記なし）

**再発防止策**:
- [ ] JsonCursorBox.seek_obj_end() にdocコメント追加（inclusive明記）
- [ ] JSON抽出の単体テスト作成

---

#### ❌ **失敗2: Rust VM result_val変数消失バグ（重大）**

**問題**: else-ifブロック内で代入した変数が、ブロック外で0に変わる

**再現コード**:
```hako
local result_val = 0
if kind == "Add" {
  result_val = lhs_val + rhs_val
} else if kind == "Sub" {
  result_val = lhs_val - rhs_val  // Line 219: 42を代入
  print("Sub result: 42")         // ✅ 正常
} else if kind == "Mul" {
  result_val = lhs_val * rhs_val
}

// ここで result_val が 0 に戻っている！
regs.set(StringHelpers.int_to_str(dst), result_val)  // 0 が保存される
```

**詳細デバッグログ**:
```
[DEBUG binop] kind=[Sub] len=3 lhs=50 rhs=8
[DEBUG binop] Matched Sub
[DEBUG binop] Sub result: 50 - 8 = 42      ← ✅ 正常
[DEBUG binop] Before set: result_val=0     ← ❌ 異常！result_valが0に
[DEBUG binop] After set: result_val=0
[DEBUG binop] Stored v%3 = 0
```

**原因仮説**:
1. **Rust VM else-ifブロック内変数スコープバグ**
   - else-ifブロック内で代入した値が、ブロック外で失われる
   - ifブロック（kind=="Add"）は正常動作
   - else-ifブロック（kind=="Sub"以降）は異常

2. **MIR Builder の if-else lowering バグ**
   - 条件分岐のMIR変換時にスコープ処理ミス？

3. **Rust VM Interpreter のローカル変数実装バグ**
   - src/backend/mir_interpreter/exec.rs のローカル変数管理

**影響範囲**:
- ✅ Test 1-3 PASS: kind=="Add" は if で動作（問題なし）
- ❌ Test 4-10 FAIL: kind=="Sub"/Mul/Div/Mod はelse-ifで失敗（すべて0）
- ❌ Compare命令も同様に失敗する可能性大

**時間損失**: 約4時間（デバッグ調査）

**学び**:
1. **Rust VMにまだバグが潜んでいる**（production ready ではない）
2. **else-if連鎖は危険**（Hakoruneでは if のみ使うべき？）
3. **デバッグprint は変数の直前/直後に入れるべき**
4. **変数値が謎に変わる場合はVM層バグを疑う**

**回避策候補**:
1. **複数のif文に分解**:
   ```hako
   if kind == "Add" { result_val = lhs_val + rhs_val }
   if kind == "Sub" { result_val = lhs_val - rhs_val }
   if kind == "Mul" { result_val = lhs_val * rhs_val }
   // ... (else-if を使わない)
   ```

2. **@matchを使う**（Day 1で制約発見したが再検討）:
   ```hako
   result_val = match kind {
     "Add" => lhs_val + rhs_val
     "Sub" => lhs_val - rhs_val
     // return文を含まない純粋な式なら可能？
   }
   ```

3. **直接代入パターン**:
   ```hako
   if kind == "Sub" {
     regs.set(StringHelpers.int_to_str(dst), lhs_val - rhs_val)
     return Result.Ok(0)
   }
   // ... (result_val変数を使わない)
   ```

**再発防止策**:
- [ ] Rust VM exec.rs のif-else処理確認
- [ ] MIR Builder のif-else lowering確認
- [ ] Hakoruneコーディングガイドに「else-if危険」を明記
- [ ] issue作成: "Rust VM: else-ifブロック内変数代入が失われるバグ"

---

#### 📊 **Day 2 統計**

- **見積もり**: 4時間
- **実績**: 約8時間
- **超過時間**: 4時間（100%超過）
- **超過理由**: JSON parsing（1.5時間）+ Rust VMバグ調査（4時間）+ その他（2.5時間）
- **コード行数**: +80行（hakorune_vm_core: +10, test: +70）
- **実装命令数**: 9/16（Const, BinOp x5, Compare x6, Ret, Copy）
- **テスト成功率**: 3/10 (30%) ← Rust VMバグにより7テスト失敗
- **新規バグ発見**: 1件（Rust VM else-if変数消失バグ）
