# Phase 7: Async model in MIR (nowait/await)

## Summary
- nowait/await を MIR に薄く導入（Future 表現）。スレッドベース実装と整合。
- 既存のFutureBox実装を活用し、MIR/VMレイヤーで非同期処理を表現。

## Background
- Nyashでは既にFutureBoxが実装済み（`src/boxes/future/mod.rs`）
- nowait/awaitはトークン・ASTノードとして定義済み
- 現在のインタープリターではthread::spawnベースの実装

## Scope
### MIR命令の追加
- `FutureNew { dst, value }` - 新しいFuture作成（初期値付き）
- `FutureSet { future, value }` - Futureに値を設定
- `Await { dst, future }` - Futureの完了を待って値を取得

### Lowering実装
- `ASTNode::Nowait { variable, expression }` → 
  1. expressionを評価
  2. FutureNew命令でFuture作成
  3. 別スレッドでの実行をスケジュール
- `ASTNode::AwaitExpression { expression }` →
  1. expressionを評価（Future値を期待）
  2. Await命令で値取得

### VM実装
- FutureNew: 新しいVMValue::Future作成
- FutureSet: Future値の更新（is_readyフラグも設定）
- Await: Future完了まで待機してから値を返す

## Tasks
- [ ] Phase 7.1: MIR命令定義
  - [ ] `src/mir/instruction.rs`にFutureNew/FutureSet/Await追加
  - [ ] Effect maskの設定（FutureNewはPURE、AwaitはREAD）
  - [ ] printer/verificationサポート
- [ ] Phase 7.2: AST→MIR lowering
  - [ ] `src/mir/builder.rs`にnowait/awaitの処理追加
  - [ ] 適切なbasic block分割（awaitは制御フローに影響）
- [ ] Phase 7.3: VM実装
  - [ ] `src/backend/vm.rs`にVMValue::Future追加
  - [ ] 各命令の実行ロジック実装
  - [ ] FutureBoxとの統合
- [ ] Phase 7.4: テスト・検証
  - [ ] 基本的なnowait/awaitのテストケース
  - [ ] 複数のnowait実行順序テスト
  - [ ] エラーケース（Future未完了時の扱い等）

## Test Cases
```nyash
// 基本的なnowait/await
static box Main {
  main() {
    nowait f1 = compute(10)
    nowait f2 = compute(20)
    local result1 = await f1
    local result2 = await f2
    print(result1 + result2)
  }
}

// ネストしたnowait
static box Main {
  main() {
    nowait outer = {
      nowait inner = compute(5)
      await inner * 2
    }
    print(await outer)
  }
}
```

## Acceptance Criteria
- 上記テストケースがMIRダンプで正しい命令列を生成
- VM実行で期待通りの結果（並行実行→正しい順序で結果取得）
- 既存のFutureBox実装との整合性維持
- verifierがFuture関連の不正を検出

## Implementation Notes
- 初期実装ではシンプルにthread::spawnベース継続
- Futureの型情報は当面VMValue内で管理（型システムは後続フェーズ）
- エラー処理は最小限（Future未完了時のawaitはブロック）

## Out of Scope (Phase 7)
- async/await構文（Rustライク）
- Promise chain / then構文
- 取り消し可能なFuture
- 複雑なスケジューリング戦略
- Future型の静的型チェック

## References
- `docs/nyash_core_concepts.md`（nowait/await + FutureBox）
- `src/boxes/future/mod.rs`（既存FutureBox実装）
- `src/interpreter/async_methods.rs`（現在のnowait/await実装）

