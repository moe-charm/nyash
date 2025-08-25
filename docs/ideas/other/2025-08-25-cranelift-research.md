# Cranelift JIT 調査メモ
Status: Research
Created: 2025-08-25
Priority: High (Phase 10関連)
Related: Phase 10 - Cranelift JIT実装

## 調査内容

### Craneliftとは
- Wasmtime/Firefoxで使用されているコード生成器
- LLVMより軽量で高速なコンパイル
- Rustで書かれており、Nyashとの統合が容易

### 基本的な使用方法
```rust
use cranelift::prelude::*;
use cranelift_module::{Module, Linkage};

// 関数シグネチャ定義
let mut sig = module.make_signature();
sig.params.push(AbiParam::new(types::I64));
sig.returns.push(AbiParam::new(types::I64));

// 関数生成
let func_id = module.declare_function("add_one", Linkage::Export, &sig)?;
```

### MIR → Cranelift IR変換の検討
- MIR命令とCranelift命令の対応関係
- Box型の表現方法（ポインタ vs 値）
- GCとの統合方法

### パフォーマンス予測
- コンパイル時間: LLVM比 10x高速
- 実行速度: インタープリター比 10-100x高速
- メモリ使用量: 中程度

## 実装スケッチ
```rust
// MIR → Cranelift変換器
struct MirToCranelift {
    builder: FunctionBuilder<'static>,
    module: Module<SimpleJITBackend>,
}

impl MirToCranelift {
    fn translate_instruction(&mut self, inst: &MirInstruction) {
        match inst {
            MirInstruction::Const { dst, value } => {
                // Cranelift IRへの変換
            }
            // ...
        }
    }
}
```

## 課題と検討事項
1. **Box型の扱い**: ヒープ割り当てとGCの統合
2. **動的ディスパッチ**: BoxCallの効率的な実装
3. **例外処理**: Nyashのエラーハンドリングとの統合
4. **デバッグ情報**: ソース位置の保持

## 次のステップ
- [ ] 最小限のPoC実装（整数演算のみ）
- [ ] ベンチマーク環境の構築
- [ ] VM実装との性能比較

## 参考資料
- [Cranelift公式ドキュメント](https://github.com/bytecodealliance/wasmtime/tree/main/cranelift)
- [Cranelift IR Reference](https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/docs/ir.md)
- wasmtimeのJIT実装

## メモ
- Phase 10での実装が決定
- まずはホットパスの特定から
- 段階的な移行が重要（全部一度には無理）