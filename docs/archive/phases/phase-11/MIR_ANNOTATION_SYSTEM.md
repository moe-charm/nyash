# MIR Annotation System - 15命令を保ちながら最適化ヒントを追加

Status: Proposed
Created: 2025-08-31
Phase: 11 (LLVM Backend)

## 🌟 革命的アイデア：MIR15 + アノテーション

MIR命令数を15個に保ちながら、最適化に必要なすべての情報をアノテーションとして付与する。

## 📋 概要

### 基本コンセプト
- **命令**: 15個のまま（変更なし）
- **アノテーション**: オプショナルな最適化ヒント
- **互換性**: アノテーションを無視しても正しく動作

### 設計
```rust
pub struct MirInstruction {
    // 既存の命令（15種類のまま）
    pub kind: MirInstructionKind,
    
    // NEW: 最適化アノテーション（オプション）
    pub annotations: Option<OptimizationHints>,
}

#[derive(Debug, Clone, Default)]
pub struct OptimizationHints {
    // インライン展開ヒント
    pub inline: Option<InlineHint>,
    
    // エイリアス情報
    pub alias: Option<AliasHint>,
    
    // 実行頻度ヒント
    pub frequency: Option<FrequencyHint>,
    
    // ベクトル化ヒント
    pub vectorize: Option<VectorizeHint>,
    
    // GC関連ヒント
    pub gc: Option<GcHint>,
    
    // 純粋性ヒント
    pub purity: Option<PurityHint>,
}
```

## 🎯 具体的な活用例

### 1. Call命令へのインラインヒント
```rust
Call {
    dst: %result,
    func: %add,
    args: [%a, %b],
    annotations: Some(OptimizationHints {
        inline: Some(InlineHint::Always), // 常にインライン展開
        purity: Some(PurityHint::Pure),   // 副作用なし
        ..Default::default()
    })
}
```

### 2. フィールド書き込み（setField）へのGCヒント
```rust
BoxCall {
    box_val: %obj,
    method: "setField",
    args: [ Const("data"), %new_val ],
    annotations: Some(OptimizationHints { gc: Some(GcHint::YoungGen), ..Default::default() })
}
```

### 3. Branch命令への分岐予測ヒント
```rust
Branch {
    condition: %cond,
    then_bb: bb1,
    else_bb: bb2,
    annotations: Some(OptimizationHints {
        frequency: Some(FrequencyHint::Hot(0.95)), // 95%の確率でthen
        ..Default::default()
    })
}
```

## 🚀 メリット

1. **命令数維持**: MIR15命令の純粋性を保持
2. **段階的最適化**: VM/JIT/AOTが必要に応じてヒントを活用
3. **互換性**: ヒントを無視しても正しく動作
4. **拡張性**: 新しい最適化ヒントを追加可能
5. **JIT/AOTフレンドリー**: 邪魔にならない（無視可能）

## 🔧 実装方法

### VM（ヒント無視）
```rust
match instruction.kind {
    MirInstructionKind::Call { .. } => execute_call(...),
    // アノテーションは完全に無視
}
```

### JIT/AOT（ヒント活用）
```rust
match instruction.kind {
    MirInstructionKind::Call { .. } => {
        if let Some(hints) = &instruction.annotations {
            if hints.inline == Some(InlineHint::Always) {
                emit_inlined_call(...);
            } else {
                emit_normal_call(...);
            }
        }
    }
}
```

## 📊 Codex先生の指摘との整合性

> **AA/最適化ヒント**: Box用アドレス空間分離、TBAA階層、`nonnull`/`dereferenceable`、`noalias`（エスケープしない一時Box）、`musttail`/`tail`の活用

これらすべてをアノテーションで表現可能！

```rust
pub enum AliasHint {
    NoAlias,                // エイリアスなし
    Dereferenceable(usize), // 参照可能サイズ
    NonNull,                // NULL不可
    Unique,                 // 唯一の参照
    AddressSpace(u32),      // アドレス空間
}

pub enum InlineHint {
    Never,      // インライン禁止
    Default,    // コンパイラ判断
    Always,     // 必ずインライン
    Hint(u32),  // ヒント強度（0-100）
}

pub enum GcHint {
    YoungGen,      // 若い世代
    OldGen,        // 古い世代
    NoBarrier,     // バリア不要
    CardMarking,   // カードマーキング必要
}
```

## 🎨 LLVM IRへの変換

```rust
// RefSetの例
BoxCall { box_val, method: "setField", args: [name, value], annotations } => {
    // 型特化Lowering時:
    let ptr = builder.build_gep(...); // name→オフセット解決
    
    // アノテーションからLLVMメタデータを生成
    if let Some(hints) = annotations {
        if let Some(alias) = hints.alias {
            match alias {
                AliasHint::NoAlias => builder.add_attribute("noalias"),
                AliasHint::NonNull => builder.add_attribute("nonnull"),
                // ...
            }
        }
        
        if let Some(gc) = hints.gc {
            match gc {
                GcHint::YoungGen => emit_young_gen_barrier(),
                GcHint::NoBarrier => { /* バリアスキップ */ },
                // ...
            }
        }
    }
    
    builder.build_store(value, ptr);
}
```

## 📈 段階的実装計画

### Phase 11.1: 基盤実装
- [ ] OptimizationHints構造体の定義
- [ ] MirInstructionへの統合
- [ ] パーサー/プリンターの更新

### Phase 11.2: 基本ヒント
- [ ] InlineHint実装
- [ ] PurityHint実装
- [ ] FrequencyHint実装

### Phase 11.3: 高度なヒント
- [ ] AliasHint実装
- [ ] GcHint実装
- [ ] VectorizeHint実装

### Phase 11.4: LLVM統合
- [ ] ヒント→LLVMメタデータ変換
- [ ] 最適化パスでの活用
- [ ] ベンチマーク検証

## 🎉 結論

**MIR15命令 + アノテーション = 最強の最適化基盤！**

- 命令セットの純粋性を保ちながら
- 最適化に必要な情報をすべて付与
- VM/JIT/AOTすべてで最適な実行
- 論文の「15命令で十分」主張を強化

## 関連文書
- [AI_CONFERENCE_CODEX_ANALYSIS.md](AI_CONFERENCE_CODEX_ANALYSIS.md)
- [AI_CONFERENCE_SUMMARY.md](AI_CONFERENCE_SUMMARY.md)
- [../../reference/mir/INSTRUCTION_SET.md](../../reference/mir/INSTRUCTION_SET.md)
