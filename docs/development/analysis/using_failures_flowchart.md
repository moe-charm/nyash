# using系失敗フローチャート

## エラー分類フローチャート

```
                    using系テスト失敗 (11件)
                            |
                            v
              ┌─────────────┴─────────────┐
              |                           |
        エラーログを確認                実行結果を確認
              |                           |
    ┌─────────┴─────────┐         ┌───────┴───────┐
    |                   |         |               |
"TOML parse error"   その他    exit code      出力内容
    |                   |         |               |
    v                   v         v               v
┌────────┐      ┌──────────┐  ┌──────┐     ┌──────────┐
│パターンA│      │パターンB│  │パターンD│     │パターンB/C│
│5件     │      │3件      │  │1件     │     │4件      │
│P2      │      │P0       │  │P1      │     │P0       │
└────────┘      └──────────┘  └────────┘     └──────────┘
```

## パターン別エラー連鎖

### パターンA: Parser Error (P2 - 低優先度)

```
module候補収集
    ↓
["hako_module.toml", "module.toml", "module.hako"]
    ↓
module.hako を TOML parser に渡す
    ↓
"// module.hako" がコメントとして認識されない
    ↓
❌ TOML parse error: invalid key
    ↓
(処理は継続、ログにノイズ)
```

**影響**: 低（ログのみ、機能影響なし）

---

### パターンB: Type Error (P0 - 即座修正)

```
using selfhost.common as C
    ↓
workspace module resolution
    ↓
❌ nested alias 解決失敗
    ↓
C.json.core.string_scan → UnknownBox
    ↓
StringScanBox.find_quote() → Void
    ↓
Void と Integer を比較
    ↓
❌ Type error: unsupported compare Lt on Void and Integer(0)
```

**影響**: 高（ユーザーコード実行不可）

---

### パターンC: Static Singleton未具現化 (P0 - 即座修正)

```
static box StringUtilsBox { size(s) {...} }
    ↓
MIR Builder: static box 処理
    ↓
❌ singleton materialization を忘れる
    ↓
MIR: method_call without receiver
    ↓
VM実行: receiver = null
    ↓
❌ Invalid instruction: Method router missing receiver
```

**影響**: 中（static box 全般に影響の可能性）

---

### パターンD: Expected Failure誤検出 (P1/P2 - 混在)

#### D-1: 循環依存検出失敗 (P1 - 高優先度)

```
using "a.foo" as Foo
    ↓
a.foo → b.bar → a.foo (循環依存)
    ↓
❌ 循環依存検出ロジックが動作していない
    ↓
本来 error すべきだが success
    ↓
❌ exit code 0 (期待: non-zero)
```

**影響**: 高（セキュリティ/安定性）

#### D-2: ログ漏出 (P2 - 低優先度)

```
using hakorune.common.json.cursor as Box
    ↓
alias resolution 成功
    ↓
デバッグログ: "[using/alias] push pair alias=..."
    ↓
❌ ログが stdout に漏出
    ↓
テスト期待値と不一致
```

**影響**: 低（見た目のみ）

---

## 修正の依存関係

```
                 P0修正 (最優先)
                      |
        ┌─────────────┴─────────────┐
        |                           |
  パターンB修正                パターンC修正
  (workspace module)          (static singleton)
        |                           |
        └─────────────┬─────────────┘
                      |
                回帰テスト (11件)
                      |
                      v
                 P1修正
                      |
              パターンD-1修正
              (循環依存検出)
                      |
                      v
                 P2修正
                      |
        ┌─────────────┴─────────────┐
        |                           |
  パターンA修正                パターンD-2修正
  (TOML parse)               (ログ漏出)
        |                           |
        └─────────────┬─────────────┘
                      |
                最終テスト
```

---

## 修正の影響範囲

### パターンB修正の波及効果

```
src/frontend/using_resolver.rs
    ↓ 修正
workspace module resolution
nested alias handling
    ↓ 影響
3件のテストが PASS
    ↓ 副作用の可能性
- 他の workspace 使用箇所の動作変化
- alias resolution のパフォーマンス変化
- エラーメッセージの変化
```

### パターンC修正の波及効果

```
src/frontend/mir_builder.rs
    ↓ 修正
static box implicit singleton allocation
    ↓ 影響
1件のテストが PASS
static box 全般が使用可能に
    ↓ 副作用の可能性
- MIR サイズの増加 (singleton allocation分)
- 既存の static box の動作変化
- JIT/LLVM backend への影響
```

---

## テスト優先度マップ

```
P0 (即座修正) ━━━━━━━━━━━━━━━━━━━━━━ 4件
│
├─ flow_using_alias_vm (B)
├─ using_nested_alias_selfhost_common_vm (B)
├─ using_modules_alias_selfhost_common_string_scan_vm (B)
└─ namespace_module_first_json_utils_string_vm (C)

P1 (高優先度) ━━━━━━━━━━━━━━━━━━━━━━ 1件
│
└─ using_workspace_cycle_strict_fail_vm (D-1)

P2 (低優先度) ━━━━━━━━━━━━━━━━━━━━━━ 6件
│
├─ using_missing_strict_vm (A)
├─ using_modules_alias_entry_selfhost_vm (A)
├─ using_auto_dir_namespace_vm (A)
├─ using_private_strict_vm (A)
├─ using_modules_alias_hakorune_common_cursor_vm (D-2)
└─ using_modules_alias_timer_static_vm (D-2)
```

---

## 検証チェックリスト

### P0修正後のチェック
- [ ] 4件のP0テストが PASS
- [ ] 既存の passing tests が break していない (170件)
- [ ] workspace module resolution の他の使用箇所が正常動作
- [ ] static box の他の使用箇所が正常動作

### P1修正後のチェック
- [ ] 循環依存が正しく検出される
- [ ] 非循環依存は正常に解決される
- [ ] エラーメッセージが明確

### P2修正後のチェック
- [ ] TOML parse error ログが抑制される
- [ ] デバッグログが本番出力に混入しない
- [ ] 6件のP2テストが PASS

---

**作成日**: 2025-10-16
**関連**: [using_failures_classification_report.md](using_failures_classification_report.md)
