# MIR進化計画: MIR14 → MIR13 → MIR17

## 概要

LoopForm導入に向けた段階的なMIR進化戦略。PHI命令の責務をLLVM層に移管し、より高レベルな制御構造表現を導入する。

## 命令数の変遷

```
MIR14（現在）: 14命令（PHIあり）
    ↓ Phase 15終盤
MIR13: 13命令（PHI除去）
    ↓ Phase 16
MIR17: 17命令（LoopForm追加）
```

## MIR13（Phase 15終盤）

### 削除される命令
- **Phi**: SSA PHIノードの生成をLLVM層に移管

### 残る13命令
1. Const
2. Load / Store
3. UnaryOp / BinOp / Compare / TypeOp
4. Branch / Jump / Return
5. NewBox / BoxCall
6. ExternCall / Call
7. Throw / Catch（既存維持）

### 実装方針
- Bridge/BuilderはPHIを生成しない
- LLVM層（llvmlite/Resolver）がCFGパターンからPHIを合成
- VM/Interpreterは直線的実行を継続（PHI無視）

## MIR17（Phase 16）

### 追加される4命令（LoopForm）

1. **LoopHeader** `{ id, params[] }`
   - ループヘッダBBの宣言
   - params[]はループ搬送値のプレースホルダ

2. **LoopEnter** `{ header: bb, args[] }` (terminator)
   - Preheaderからヘッダへの遷移
   - args[]は搬送値の初期値

3. **LoopLatch** `{ header: bb, args[] }` (terminator)
   - 本体末尾からヘッダへのバックエッジ
   - args[]は更新後の搬送値

4. **LoopExit** `{ exit: bb }` (terminator)
   - ループからの脱出（break相当）

### なぜ+4命令か
- ループの構造を明示的に表現
- Preheader/Header/Body/Exitの役割を宣言的に
- LLVM層でのPHI生成が機械的に可能

## 合流点の扱い（将来検討）

### MergeForm（追加候補）
```rust
// If/Else等の合流用（将来のMIR18以降）
MergeHeader { id, params[] }
MergeEnter { header: bb, args[] }
```

## 移行のメリット

1. **責務分離の明確化**
   - MIR: 制御構造の宣言
   - LLVM: SSA表現の生成

2. **段階的移行**
   - Phase 15: PHI生成を止める（互換性維持）
   - Phase 16: LoopForm導入（新機能）

3. **将来拡張への道筋**
   - Generator/Async/Effectへの対応準備
   - LifeBox Model（LBM）への布石

## AI協働の記録（2025-09-16）

### Codex提案
- PHI合成をllvmlite側で実施
- LoopForm/MergeFormの構造化表現
- TryRegionによる例外構造化（将来）

### 人間の洞察
- 「MIR13に減らしてから+4でMIR17」
- 段階的移行の重要性
- 一気に変えないことの価値

### 合意形成
- Phase 15でまずPHI非生成化
- 安定後にLoopForm導入
- 責務分離を徹底

## 実装チェックリスト

### Phase 15終盤（MIR13化）
- [ ] Bridge: PHI生成を無効化するフラグ
- [ ] llvmlite: Resolver実装（PHI合成）
- [ ] Smoke: ループ搬送値のテスト
- [ ] VM: PHI無視の動作確認

### Phase 16（MIR17化）
- [ ] MIR: LoopForm命令の追加
- [ ] Bridge: Loop構造の認識とLoopForm生成
- [ ] llvmlite: LoopFormからPHIへの変換
- [ ] ドキュメント: MIR17仕様書

## 関連文献

- 論文A（MIR13/14）: 基本設計
- 論文D（SSA構築）: PHI生成の理論
- 論文E（本稿）: LoopForm理論
- CURRENT_TASK.md: 実装計画