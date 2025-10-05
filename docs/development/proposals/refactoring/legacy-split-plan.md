# Legacy.rs Split Plan (Phase 15.10)

## 目的
handlers/calls/legacy.rs (617行) と handlers/boxes/legacy.rs (515行) の巨大ファイルを責任ごとに分割し、保守性・テスト容易性を向上。

## calls/legacy.rs (617行) 構造分析

### 現状の関数構成
```
Line   6-25 :  handle_call (19行) - エントリーポイント
Line  26-250:  execute_callee_call (224行)
  ├─ Method処理: 175行（巨大！）
  │   ├─ unborn guard: 20行
  │   ├─ receiver resolution: 100行（超複雑！）
  │   └─ builtin bridges + 実行: 55行
  ├─ Global/ModuleFunction/Extern: 各1行（delegateのみ）
  └─ Constructor/Closure/Value: 未実装
Line 252-588:  execute_legacy_call (336行)
  ├─ 名前解決: 100行（複雑なフォールバック）
  ├─ トレース処理: 100行
  └─ 実行: 136行
Line 589-617:  execute_extern_function (28行)
```

### 分割方針: legacy/ サブディレクトリ作成

```
handlers/calls/
├── function.rs (既存、変更なし)
├── adapter.rs (既存、変更なし)
└── legacy/
    ├── mod.rs (~30行)
    │   - handle_call (エントリーポイント)
    │   - pub use re-exports
    ├── callee_dispatcher.rs (~50行)
    │   - execute_callee_call (dispatcher only)
    │   - Callee variant routing
    ├── method_handler.rs (~180行)
    │   - Method処理全体
    │   - receiver resolution
    │   - builtin bridges
    ├── legacy_resolver.rs (~200行)
    │   - execute_legacy_call (名前解決)
    │   - FunctionIndex使用
    │   - トレース処理
    └── extern_handler.rs (~30行)
        - execute_extern_function
```

### 利点
- ✅ 各ファイル200行以下に収まる
- ✅ 責任が明確（Method/LegacyResolver/Extern）
- ✅ テストしやすい（モジュール単位）
- ✅ 段階的移行可能
- ✅ 既存コード（function.rs）に影響なし

## boxes/legacy.rs (515行) 構造分析

（TODO: 次のステップで分析）

## 実装順序

1. **Phase 15.10-A**: calls/legacy.rs分割
   - legacy/mod.rs作成
   - method_handler.rs抽出
   - legacy_resolver.rs抽出
   - extern_handler.rs抽出
   - callee_dispatcher.rs抽出
   - テスト（smoke test）

2. **Phase 15.10-B**: boxes/legacy.rs分割
   - 構造分析
   - 分割実行
   - テスト

3. **Commit**: 各フェーズごとにコミット

## テスト戦略
- json_lint_vm smoke test（既存）
- cargo check（コンパイル確認）
- 動作変更なし（完全後方互換）

---

**作成日**: 2025-10-05
**ステータス**: 分析完了、実装待ち
