# using系11件失敗パターン分類レポート

**作成日**: 2025-10-16
**調査対象**: Phase 1-3統合後のusing系テスト失敗11件
**目的**: 失敗パターン分類と共通原因特定

---

## 📊 エグゼクティブサマリー

### 🔴 重要発見: legacy-boxes除外は関係なし！

**結論**: 11件のusing系テスト失敗は**legacy-boxes除外とは無関係**。すべて**using/module resolution system自体の問題**。

### 主要エラーパターン（4分類）

| パターン | 件数 | 主な原因 |
|---------|------|---------|
| **A: Parser Error (invalid key)** | 5件 | module.hako をTOMLとしてパース試行 |
| **B: Type Error (Void/UnknownBox)** | 3件 | using解決失敗 → 型推論エラー連鎖 |
| **C: Static Singleton未具現化** | 1件 | static box メソッドルーティング失敗 |
| **D: Expected Failure誤検出** | 2件 | 本来失敗すべきテストが成功 |

---

## 📋 詳細分類表

### パターンA: Parser Error (invalid key) - 5件

**エラー**: `TOML parse error at line 1, column 1 ... invalid key`

| テスト名 | エラー詳細 | 原因 |
|---------|-----------|------|
| `using_missing_strict_vm` | "// module.hako" をTOMLとしてパース | module.hako（Hakorune構文）をTOML候補に含めている |
| `using_modules_alias_entry_selfhost_vm` | 同上 | 同上 |
| `using_auto_dir_namespace_vm` | 同上 + alias trace ログが予期しない出力 | 同上 + テスト期待値の不一致 |
| `using_workspace_cycle_strict_fail_vm` | 同上（後述パターンDと複合） | 同上 |
| `using_private_strict_vm` | 同上 | 同上 |

**根本原因**:
```rust
// src/frontend/using_resolver.rs (推測)
// module候補として .hako ファイルをTOML parserに渡している
candidates = [
  "hako_module.toml",
  "module.toml",
  "module.hako"  // ← これが問題！
]
```

**影響範囲**: 低（ログノイズのみ、実行には影響なし）
**修正方針**:
- `module.hako` を TOML parser に渡さない
- または TOML parse失敗時のエラーログを抑制

---

### パターンB: Type Error (Void/UnknownBox) - 3件

**エラー**: `Type error: unsupported binop/compare on Void` / `UnknownBox`

| テスト名 | エラー詳細 | 原因 |
|---------|-----------|------|
| `flow_using_alias_vm` | `unsupported binop Add on Integer(20) and Void` | FlowBox.stringify() 解決失敗 → Void返却 |
| `using_nested_alias_selfhost_common_vm` | 出力 "ng"（期待 "ok"） | StringScanBox.find_quote() 解決失敗 → 誤った値 |
| `using_modules_alias_selfhost_common_string_scan_vm` | `unsupported compare Lt on Void and Integer(0)` | StringScanBox 解決失敗 → Void返却 |

**根本原因**: using解決の連鎖失敗

1. **flow_using_alias_vm**:
   ```nyash
   using "flow_utils" as FU
   using FU.Flow as FlowBox
   ```
   - `flow_utils` モジュール解決失敗
   - → `FlowBox` が UnknownBox
   - → `FlowBox.stringify()` が Void
   - → `Integer + Void` で型エラー

2. **using_nested_alias_selfhost_common_vm**:
   ```nyash
   using selfhost.common as C
   using C.json.core.string_scan as StringScanBox
   ```
   - ネストされたalias解決失敗
   - → `StringScanBox` が不正な実装を参照
   - → `find_quote()` が誤動作

3. **using_modules_alias_selfhost_common_string_scan_vm**:
   - `selfhost.common.json.core.string_scan` 解決失敗
   - → メソッド呼び出しが Void返却
   - → 比較演算で型エラー

**影響範囲**: 高（実行失敗、ユーザーコード影響）
**修正方針**:
- workspace module resolution の修正（特にnested alias）
- fail-fast: 解決失敗時に UnknownBox ではなく即座にエラー

---

### パターンC: Static Singleton未具現化 - 1件

**エラー**: `Invalid instruction: Method router missing receiver (static singleton not materialized)`

| テスト名 | エラー詳細 | 原因 |
|---------|-----------|------|
| `namespace_module_first_json_utils_string_vm` | static box StringUtilsBox.size() 呼び出し失敗 | static singleton が MIR段階で具現化されていない |

**根本原因**:
```nyash
// apps/lib/json_native/string_utils.nyash
static box StringUtilsBox {
  size(s) { return s.length() }
}

// 呼び出し側
using json_utils.string as StringUtilsBox
StringUtilsBox.size("x")  // ← receiver missing
```

**MIR Issue**:
- static box は singleton として扱われるべき
- MIR Builder が static singleton の materialization を忘れている
- → method call の receiver が null

**影響範囲**: 中（static box全般に影響の可能性）
**修正方針**:
- MIR Builder: static box の implicit singleton allocation
- または Router: static method 用の special handling

---

### パターンD: Expected Failure誤検出 - 2件

**エラー**: 本来failすべきテストが成功してしまう

| テスト名 | エラー詳細 | 原因 |
|---------|-----------|------|
| `using_workspace_cycle_strict_fail_vm` | exit code 0（期待: non-zero） | 循環依存検出の失敗 |
| `using_modules_alias_hakorune_common_cursor_vm` | 出力にalias traceログ（期待: "ok"のみ） | デバッグログの漏出 |
| `using_modules_alias_timer_static_vm` | 出力にalias traceログ（期待: "ok"のみ） | デバッグログの漏出 |

**根本原因**:

1. **using_workspace_cycle_strict_fail_vm**:
   ```bash
   # テストの意図: 循環依存を検出して失敗すべき
   # 実際: 成功 (exit code 0)
   ```
   - 循環依存検出ロジックが働いていない
   - または strict mode が有効化されていない

2. **using_modules_alias_hakorune_common_cursor_vm / timer_static_vm**:
   ```bash
   # 期待出力: "ok"
   # 実際出力: "[using/alias] push pair alias=... \nok"
   ```
   - `[using/alias]` デバッグログが本番出力に混入
   - テスト環境の log filtering 不足

**影響範囲**:
- 循環依存検出: 高（セキュリティ/安定性）
- ログ漏出: 低（見た目のみ）

**修正方針**:
- 循環依存検出の再実装
- デバッグログの適切なフィルタリング（NYASH_USING_TRACE=1 等）

---

## 🔍 legacy-boxes除外との関連性

### 検証結果: **関連性なし**

**理由**:
1. **すべてのエラーが using/module resolution 層で発生**
   - Parser error: TOML/module.hako 混同
   - Type error: using解決失敗 → UnknownBox/Void
   - MIR error: static singleton未具現化
   - Test error: 期待値不一致

2. **kernel-embedded boxes (String/Integer/Array等) は正常動作**
   ```nyash
   // これらは問題なく動作
   local s = "aabb"  // StringBox
   local i = 42      // IntegerBox
   if (i < 0) { }    // BoolBox, 比較演算
   ```

3. **失敗しているのは workspace module resolution**
   ```nyash
   // 失敗例
   using selfhost.common as C              // ← module解決
   using C.json.core.string_scan as Box    // ← nested alias
   using hakorune.common.json.cursor as Box // ← workspace path
   ```

**結論**: legacy-boxes除外は無罪。問題は using/module system の実装不備。

---

## 🎯 修正優先度と推奨アクション

### 🔥 P0: 即座に修正すべき（機能ブロッカー）

1. **パターンB: Type Error (Void/UnknownBox)** - 3件
   - **影響**: ユーザーコードが実行不可
   - **修正**: workspace module resolution の修正
   - **ファイル**: `src/frontend/using_resolver.rs` (nested alias handling)

2. **パターンC: Static Singleton未具現化** - 1件
   - **影響**: static box が使用不可
   - **修正**: MIR Builder での singleton materialization
   - **ファイル**: `src/frontend/mir_builder.rs` (static box handling)

### ⚠️ P1: 高優先度（セキュリティ/安定性）

3. **パターンD: 循環依存検出失敗** - 1件
   - **影響**: 無限ループ/stack overflow の危険
   - **修正**: strict mode での循環依存チェック
   - **ファイル**: `src/frontend/using_resolver.rs` (cycle detection)

### 📝 P2: 低優先度（ログノイズ）

4. **パターンA: Parser Error (invalid key)** - 5件
   - **影響**: ログが見にくい（機能影響なし）
   - **修正**: module.hako を TOML parser に渡さない
   - **ファイル**: `src/frontend/using_resolver.rs` (module candidate selection)

5. **パターンD: ログ漏出** - 2件
   - **影響**: テスト出力が汚い
   - **修正**: デバッグログのフィルタリング
   - **ファイル**: `src/frontend/using_resolver.rs` (log macros)

---

## 📊 統計サマリー

| 項目 | 値 |
|------|-----|
| 総失敗件数 | 11件 |
| legacy-boxes関連 | **0件** |
| using/module関連 | **11件** (100%) |
| P0 (即座修正) | 4件 |
| P1 (高優先度) | 1件 |
| P2 (低優先度) | 6件 |

---

## 🚀 次のステップ

1. **P0修正**: workspace module resolution の修正（パターンB, C）
2. **回帰テスト**: 修正後、11件すべてを再実行
3. **ドキュメント更新**: using/module system の既知の制約を明記
4. **P1修正**: 循環依存検出の実装（パターンD）
5. **P2修正**: ログノイズの削減（パターンA, D）

---

## 📚 関連ドキュメント

- [using system reference](../../reference/language/using.md)
- [workspace module resolution](../../development/architecture/workspace-modules.md)
- [MIR static box handling](../../reference/mir/static-boxes.md)
- [Test debugging guide](../../guides/smoke-test-debugging.md)

---

**調査者**: Claude (Task Agent #4)
**調査日**: 2025-10-16
**バージョン**: Phase 1-3統合後
