# MIR Annotation System - 15命令を保ちながら最適化ヒントを追加

Status: Pending
Created: 2025-08-31
Priority: High
Related-Code: src/mir/instruction.rs

## 概要

MIR15命令はそのまま維持しつつ、最適化メタデータをアノテーションとして追加する革新的アプローチ。

## 背景

- MIR命令数を15個に保つことが「Everything is Box × MIR15」論文の核心
- しかしJIT/AOT最適化には追加情報が必要
- 命令追加なしで最適化ヒントを付与する方法が必要

## 提案設計

```rust
pub struct MirInstruction {
    // 既存の命令（15種類のまま）
    pub kind: MirInstructionKind,
    
    // NEW: 最適化アノテーション（オプション）
    pub annotations: Option<OptimizationHints>,
}

pub struct OptimizationHints {
    pub inline: Option<InlineHint>,      // インライン展開
    pub alias: Option<AliasHint>,        // エイリアス情報
    pub frequency: Option<FrequencyHint>, // 実行頻度
    pub vectorize: Option<VectorizeHint>, // ベクトル化
    pub gc: Option<GcHint>,              // GC最適化
    pub purity: Option<PurityHint>,      // 純粋性
}
```

## 利点

1. **命令数維持**: MIR15命令の純粋性を保持
2. **段階的最適化**: VM/JIT/AOTが必要に応じて活用
3. **後方互換性**: アノテーションを無視しても正しく動作
4. **拡張可能**: 新しい最適化ヒントを追加可能
5. **JIT/AOTフレンドリー**: 必要な最適化情報を完備

## 実装例

### Call命令へのインラインヒント
```rust
Call {
    dst: %result,
    func: %add,
    args: [%a, %b],
    annotations: Some(OptimizationHints {
        inline: Some(InlineHint::Always),
        purity: Some(PurityHint::Pure),
        ..Default::default()
    })
}
```

### RefSet命令へのGCヒント
```rust
RefSet {
    reference: %obj,
    field: "data",
    value: %new_val,
    annotations: Some(OptimizationHints {
        gc: Some(GcHint::YoungGen), // 世代別GC最適化
        ..Default::default()
    })
}
```

## Codex先生の指摘との整合性

> AA/最適化ヒント: Box用アドレス空間分離、TBAA階層、`nonnull`/`dereferenceable`、`noalias`

これらすべてをアノテーションで表現可能。

## 実装タイミング

- [ ] Phase 11.1（LLVM最適化フェーズ）で導入
- [ ] まずはインラインヒントから開始
- [ ] 段階的に他のヒントを追加

## 関連ドキュメント

- [AI_CONFERENCE_CODEX_ANALYSIS.md](../../development/roadmap/phases/phase-11/AI_CONFERENCE_CODEX_ANALYSIS.md)
- [MIR INSTRUCTION_SET.md](../../reference/mir/INSTRUCTION_SET.md)