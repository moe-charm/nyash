# Phase 20.6 - VM Core Complete (Phase B+C 完全達成)

**期間**: 2026-03-01 - 2026-05-24 (12週間)
**状態**: Planning
**前提**: Phase 20.5完了 (VM Foundations PoC + op_eq Migration)

---

## 🎯 このフェーズで実現すること

**"Hakorune VM Core 完全実装 + Dispatch統一"**

### フェーズの位置づけ

Phase 20.6は**Pure Hakorune Roadmap**における2つの重要フェーズを完了します:

1. **Phase B Complete (Week 1-6)**: VM Core in Hakorune
   - 全16個のMIR命令を実装
   - 制御フロー（分岐、PHI、ループ）完全対応
   - Golden Tests: Rust-VM vs Hako-VM 100%一致

2. **Phase C (Week 7-12)**: Dispatch Unification
   - Resolver統合（すべてのメソッド呼び出しを統一）
   - CallableBoxリファクタリング
   - Universal Route Minimization（特殊ケース排除）

---

## 💡 背景と動機

### Phase 20.5での達成内容

Phase 20.5で以下を完了:
- ✅ HostBridge API (C-ABI境界)
- ✅ op_eq Migration (Hakoruneで実装)
- ✅ VM Foundations PoC (5命令: const, binop, compare, jump, ret)

### Phase 20.6での拡張

Phase 20.6では:
1. **VM Core Complete**: 5命令 → 16命令へ拡張
2. **制御フロー強化**: branch, phi, loopform対応
3. **メモリ操作**: load, store, copy対応
4. **メソッド呼び出し**: call, boxcall, externcall対応
5. **Dispatch統一**: Resolverによる単一解決経路

---

## 🏗️ アーキテクチャ概要

### Phase B: VM Core Complete (Week 1-6)

```
【16個のMIR命令】
┌─────────────────────────────────────┐
│ 基本値・制御 (5命令)                │
│ ✅ const, ret, jump, branch, phi    │  ← Phase 20.5完了
├─────────────────────────────────────┤
│ 演算 (6命令)                        │
│ ✅ binop (Add/Sub/Mul/Div/Mod)      │  ← Phase 20.5完了
│ ✅ compare (Eq/Ne/Lt/Le/Gt/Ge)      │  ← Phase 20.5完了
│ ⬜ unaryop                          │  ← Phase 20.6追加
├─────────────────────────────────────┤
│ メソッド呼び出し (3命令)            │
│ ⬜ call (関数呼び出し)              │  ← Phase 20.6追加
│ ⬜ boxcall (Boxメソッド呼び出し)    │  ← Phase 20.6追加
│ ⬜ externcall (外部関数呼び出し)    │  ← Phase 20.6追加
├─────────────────────────────────────┤
│ メモリ・型 (5命令)                  │
│ ⬜ load, store, copy               │  ← Phase 20.6追加
│ ⬜ typeop, newbox                  │  ← Phase 20.6追加
├─────────────────────────────────────┤
│ 制御最適化 (3命令)                  │
│ ⬜ barrier, safepoint, loopform    │  ← Phase 20.6追加
└─────────────────────────────────────┘
```

### Phase C: Dispatch Unification (Week 7-12)

```
【現状: 複数の解決経路】
┌────────────────────────────────────┐
│ Method Call Dispatch               │
│  ├─ Global Function (特殊ケース)   │
│  ├─ Box Method (特殊ケース)        │
│  ├─ Closure (特殊ケース)           │
│  ├─ Constructor (特殊ケース)       │
│  └─ Module Function (特殊ケース)   │
└────────────────────────────────────┘
          ↓ Phase C統一
┌────────────────────────────────────┐
│ Unified Resolver Path              │
│                                    │
│  Resolver.lookup(type_id, method, arity)
│           ↓                        │
│      MethodHandle                  │
│           ↓                        │
│  ExecBox.call_by_handle(           │
│      handle, args, NoOperatorGuard)│
└────────────────────────────────────┘
```

**重要な設計原則**:
- **単一解決経路**: すべてのメソッド呼び出しがResolver経由
- **Fail-Fast**: 未知メソッド → 即座にRuntimeError
- **特殊ケース排除**: 疑似メソッド実装を削除

---

## 📊 週次計画 (Week 1-12)

### Phase B Complete: VM Core (Week 1-6)

#### Week 1-2: メモリ操作命令 (2026-03-01 - 03-14)

**目標**: load, store, copy実装

**実装対象**:
```hakorune
// load: メモリから値をロード
// store: メモリに値を保存
// copy: レジスタ間のコピー
```

**成果物**:
- [ ] `load_handler.hako` 実装
- [ ] `store_handler.hako` 実装
- [ ] `copy_handler.hako` 実装
- [ ] テスト: 10ケース (メモリ操作基本)

#### Week 3-4: メソッド呼び出し命令 (2026-03-15 - 03-28)

**目標**: call, boxcall, externcall実装

**実装対象**:
```hakorune
// call: 関数呼び出し (Global/Module/Closure)
// boxcall: Boxメソッド呼び出し
// externcall: 外部関数呼び出し (nyrt.time等)
```

**成果物**:
- [ ] `call_handler.hako` 実装 (Unified MirCall)
- [ ] `boxcall_handler.hako` 実装
- [ ] `externcall_handler.hako` 実装
- [ ] テスト: 15ケース (呼び出し基本)

#### Week 5: 型操作命令 (2026-03-29 - 04-04)

**目標**: typeop, newbox実装

**実装対象**:
```hakorune
// typeop: 型変換・型チェック
// newbox: Box生成
```

**成果物**:
- [ ] `typeop_handler.hako` 実装
- [ ] `newbox_handler.hako` 実装
- [ ] テスト: 10ケース (型操作基本)

#### Week 6: 制御最適化命令 + Golden Tests (2026-04-05 - 04-11)

**目標**: 残り命令 + Golden Testing完了

**実装対象**:
```hakorune
// barrier: GC barrier
// safepoint: GC safepoint
// loopform: ループ検出ヒント
// unaryop: 単項演算
```

**Golden Tests**:
- [ ] 算術演算: 10ケース
- [ ] 制御フロー: 10ケース
- [ ] コレクション操作: 10ケース
- [ ] 再帰: 5ケース
- [ ] クロージャ: 5ケース
- [ ] 文字列操作: 10ケース
- [ ] 型操作: 10ケース
- [ ] メモリ操作: 10ケース
- [ ] 複合ケース: 30ケース

**合計**: 100個のGolden Tests

**成果物**:
- [ ] 残り命令実装完了
- [ ] Golden Test Suite完成 (100+ tests)
- [ ] パフォーマンス測定: Hako-VM ≥ 50% of Rust-VM

---

### Phase C: Dispatch Unification (Week 7-12)

#### Week 7-8: Resolver統合 (2026-04-12 - 04-25)

**目標**: Resolver.lookup実装 + 統合

**実装内容**:
```hakorune
// Resolver: すべてのメソッド呼び出しを統一
static box ResolverBox {
    // type_id, method, arity → MethodHandle
    lookup(type_id: IntegerBox,
           method: StringBox,
           arity: IntegerBox) {
        // 1. 型情報から適切なハンドラーを検索
        // 2. メソッドシグネチャ検証
        // 3. MethodHandle返却
        // 4. 見つからない場合 → RuntimeError (Fail-Fast)
    }
}
```

**成果物**:
- [ ] `resolver_box.hako` 実装
- [ ] `method_handle_box.hako` 実装
- [ ] 統合テスト: 20ケース

#### Week 9-10: CallableBox Refactoring (2026-04-26 - 05-09)

**目標**: 単一呼び出しエントリーポイント実装

**実装内容**:
```hakorune
// CallableBox: すべての呼び出しを統一
static box ExecBox {
    call_by_handle(handle: MethodHandleBox,
                   args: ArrayBox,
                   guard: NoOperatorGuard) {
        // 1. ハンドルから実装取得
        // 2. 引数検証
        // 3. NoOperatorGuardで再帰防止
        // 4. 実行
    }
}
```

**マクロデシュガー**:
```hakorune
// Before: arr.push(value)
// After:  Callable.ref_method(arr, :push, 1).call([value])
```

**成果物**:
- [ ] `exec_box.hako` 実装
- [ ] `no_operator_guard_box.hako` 実装
- [ ] マクロデシュガー実装
- [ ] テスト: 25ケース

#### Week 11: Universal Route Minimization (2026-05-10 - 05-16)

**目標**: 疑似メソッド実装削除

**削除対象**:
- ❌ 特殊ケースdispatch (Global/Box/Closure/Constructor/Module)
- ❌ ハードコードされたメソッドテーブル
- ❌ フォールバック実装

**実装内容**:
- ✅ すべてResolver経由
- ✅ 未知メソッド → RuntimeError (Fail-Fast)
- ✅ 単一コードパス

**成果物**:
- [ ] 特殊ケース削除完了
- [ ] コードベース整理
- [ ] テスト: 30ケース (エッジケース)

#### Week 12: 統合テスト + ドキュメント (2026-05-17 - 05-24)

**目標**: Phase 20.6完了報告

**統合テスト**:
- [ ] Golden Tests再実行 (100+ tests ALL PASS)
- [ ] パフォーマンステスト (≥ 50% of Rust-VM)
- [ ] ストレステスト (大規模プログラム)
- [ ] リグレッションテスト

**ドキュメント**:
- [ ] VM Core完全リファレンス
- [ ] Dispatch Unification設計書
- [ ] パフォーマンスレポート
- [ ] Phase 20.6完了報告書
- [ ] Phase 20.7計画書

---

## 🏆 成功基準 (DoD)

### 技術的基準

1. **VM Core Complete**:
   - [ ] 全16個のMIR命令が動作
   - [ ] 制御フロー完全対応 (branch, phi, loopform)
   - [ ] メモリ操作完全対応 (load, store, copy)

2. **Golden Tests**:
   - [ ] 100個以上のテストケースすべてPASS
   - [ ] Rust-VM vs Hako-VM: 100%出力一致
   - [ ] エッジケース網羅

3. **Dispatch Unified**:
   - [ ] すべてのメソッド呼び出しがResolver経由
   - [ ] 特殊ケースdispatch完全削除
   - [ ] Fail-Fast動作確認 (未知メソッド→即エラー)

### パフォーマンス基準

- [ ] **Hako-VM ≥ 50% of Rust-VM** (実行速度)
- [ ] **メモリ使用量**: < 200MB (通常プログラム)
- [ ] **コンパイル時間**: < 10s (小規模プログラム)

### 品質基準

- [ ] **テストカバレッジ**: すべての命令・制御パスをカバー
- [ ] **ドキュメント**: アーキテクチャ・設計・移行ガイド完備
- [ ] **コードレビュー**: ChatGPT + Claude承認済み

---

## ⚠️ リスク & 対策

### リスク1: 実装複雑性

**問題**: 16命令すべての実装は複雑

**影響**: HIGH

**対策**:
- Rust VMを参照実装として利用
- 週次で段階的実装 (2-4命令/週)
- 各命令に専用テストスイート
- Golden Testsで早期バグ検出

### リスク2: Dispatch統一の難易度

**問題**: 既存の特殊ケース削除は影響範囲が大きい

**影響**: MEDIUM

**対策**:
- 段階的移行: 新コード追加 → テスト → 旧コード削除
- Resolverを先に実装・テスト
- フィーチャーフラグで切り替え可能に
- リグレッションテスト強化

### リスク3: パフォーマンス劣化

**問題**: Hako-VMがRust-VMより遅い可能性

**影響**: MEDIUM

**対策**:
- 初期目標: 50% (許容範囲)
- 週次でベンチマーク測定
- ボトルネック早期特定
- 最適化はPhase 20.7以降

### リスク4: Golden Tests不一致

**問題**: Rust-VMとHako-VMで出力が異なる可能性

**影響**: HIGH

**対策**:
- 決定性保証 (JSON正規化、ソート済みキー)
- 浮動小数点演算は整数で代替
- ランダム性排除 (タイムスタンプ、PID等)
- 差分詳細ログ

---

## 📚 関連リソース

### Phase 20.6ドキュメント

- **[INDEX.md](INDEX.md)** - ドキュメント構造
- **[PLAN.md](PLAN.md)** - 実行計画 (English)
- **[CHECKLIST.md](CHECKLIST.md)** - 週次チェックリスト

### 前後のフェーズ

- **前**: [Phase 20.5 - VM Foundations](../phase-20.5/)
- **次**: [Phase 20.7 - Collections in Hakorune](../phase-20.7/) (予定)

### 参照ドキュメント

- [Pure Hakorune Roadmap](../phase-20.5/PURE_HAKORUNE_ROADMAP.md) - 全体計画
- [MIR Instruction Set](../../../../reference/mir/INSTRUCTION_SET.md) - MIR命令仕様
- [Hakorune VM Discovery](../phase-20.5/HAKORUNE_VM_DISCOVERY.md) - VM発見レポート

---

## 🎉 成功後の世界

### Phase 20.6完了後（22週間累計、2026-05-24）:

1. **VM Core Complete**: 全16個のMIR命令が動作
2. **Golden Tests PASS**: 100%一致保証
3. **Dispatch Unified**: 単一解決経路確立
4. **パフォーマンス達成**: ≥ 50% of Rust-VM
5. **ドキュメント完備**: 完全なアーキテクチャドキュメント

### 次のステップ (Phase 20.7):

- **Collections in Hakorune**: MapBox/ArrayBoxのHakorune実装
- **Deterministic Behavior**: 決定性保証（イテレーション順序等）
- **Performance Optimization**: 70%目標

---

**作成日**: 2025-10-14
**Phase開始予定**: 2026-03-01
**Phase終了予定**: 2026-05-24
**想定期間**: 12週間
**前提フェーズ**: Phase 20.5 (VM Foundations PoC)
**成果**: VM Core Complete + Dispatch Unification ✅
