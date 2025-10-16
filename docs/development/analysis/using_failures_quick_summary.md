# using系11件失敗 - クイックサマリー

**結論**: legacy-boxes除外は無関係。すべて using/module resolution の問題。

---

## 4つのエラーパターン

### 🟡 A: Parser Error (invalid key) - 5件 [P2: 低優先度]
- **症状**: `TOML parse error ... invalid key`
- **原因**: `module.hako`（Hakorune構文）をTOMLとしてパース試行
- **影響**: ログノイズのみ（機能影響なし）
- **修正**: `src/frontend/using_resolver.rs` - module.hako を TOML候補から除外

### 🔴 B: Type Error (Void/UnknownBox) - 3件 [P0: 即座修正]
- **症状**: `unsupported binop/compare on Void`
- **原因**: using解決失敗 → UnknownBox → Void返却 → 型エラー連鎖
- **影響**: ユーザーコード実行不可
- **修正**: `src/frontend/using_resolver.rs` - workspace/nested alias handling

**失敗テスト**:
- `flow_using_alias_vm`: FlowBox.stringify() 解決失敗
- `using_nested_alias_selfhost_common_vm`: nested alias 解決失敗
- `using_modules_alias_selfhost_common_string_scan_vm`: 同上

### 🔴 C: Static Singleton未具現化 - 1件 [P0: 即座修正]
- **症状**: `Method router missing receiver (static singleton not materialized)`
- **原因**: MIR Builder が static box の singleton を作成していない
- **影響**: static box メソッド呼び出し失敗
- **修正**: `src/frontend/mir_builder.rs` - static box implicit allocation

**失敗テスト**:
- `namespace_module_first_json_utils_string_vm`

### 🟠 D: Expected Failure誤検出 - 2件 [P1: 循環依存, P2: ログ]
- **症状**: 本来失敗すべきテストが成功 / デバッグログ漏出
- **原因**: 循環依存検出の失敗 / ログフィルタリング不足
- **影響**: セキュリティ（循環依存）/ ログノイズ（ログ漏出）
- **修正**: `src/frontend/using_resolver.rs` - cycle detection + log filtering

**失敗テスト**:
- `using_workspace_cycle_strict_fail_vm`: exit 0（期待: non-zero）
- `using_modules_alias_hakorune_common_cursor_vm`: alias trace ログ漏出
- `using_modules_alias_timer_static_vm`: alias trace ログ漏出

---

## 修正優先度

| 優先度 | パターン | 件数 | 理由 |
|--------|---------|------|------|
| **P0** | B: Type Error | 3件 | ユーザーコード実行不可 |
| **P0** | C: Static Singleton | 1件 | static box 使用不可 |
| **P1** | D: 循環依存検出 | 1件 | セキュリティ/安定性 |
| **P2** | A: Parser Error | 5件 | ログノイズのみ |
| **P2** | D: ログ漏出 | 2件 | 見た目のみ |

---

## 次のアクション

1. **P0修正**: workspace module resolution (パターンB)
2. **P0修正**: static box singleton materialization (パターンC)
3. **回帰テスト**: 11件すべてを再実行
4. **P1修正**: 循環依存検出 (パターンD)
5. **P2修正**: ログノイズ削減 (パターンA, D)

---

詳細: [using_failures_classification_report.md](using_failures_classification_report.md)
