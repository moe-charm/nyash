# ⚡ From Boxes to JIT: How Abstraction Boundaries Enable Progressive Compiler Optimization

## 📑 論文概要

**タイトル**: From Boxes to JIT: How Abstraction Boundaries Enable Progressive Compiler Optimization

**対象会議**: PLDI 2026 / OOPSLA 2025

**著者**: [TBD]

**概要**: 箱による境界分離がJITコンパイラの段階的最適化を可能にする新しい設計パターンを提案。Nyashでの実装と性能評価。

## 🎯 研究の新規性

### 従来のJIT設計の問題
```
[Parser] → [AST] → [IR] → [Optimizer] → [CodeGen]
    ↑        ↑       ↑        ↑            ↑
   密結合により変更が全体に波及
```

### 箱理論によるJIT設計
```
[JIT箱] ⟷ [Handle] ⟷ [VM箱]
   ↑         ↑          ↑
独立進化   最小接点   フォールバック
```

## 🔬 技術的貢献

### 1. **Progressive Enhancement Pattern**
- VMフォールバックからの段階的最適化
- 箱単位での機能追加
- 部分的失敗の局所化

### 2. **Handle Registry Architecture**
- u64ハンドルによる疎結合
- スコープベースの生命管理
- GC非依存の設計

### 3. **Box-Based Testing**
- 箱単位での独立テスト
- モックハンドルによる分離テスト
- 性能回帰の早期検出

## 📊 評価計画

### ベンチマーク
- **Micro**: 個別命令の性能
- **Macro**: 実アプリケーション
- **Compile Time**: JIT遅延の測定

### 比較対象
- V8 (JavaScript)
- PyPy (Python)
- GraalVM (多言語)

### 測定項目
- スループット向上率
- メモリ使用量
- ウォームアップ時間
- フォールバック頻度

## 📝 論文構成案

```
1. Introduction
   - JIT複雑性の課題
   - 箱理論の着想

2. Background
   - 既存JIT設計
   - モジュラーコンパイラ

3. Box-Oriented JIT Design
   - アーキテクチャ概要
   - ハンドルレジストリ
   - フォールバック機構

4. Implementation
   - Nyash JITの実装
   - Cranelift統合
   - 最適化パス

5. Evaluation
   - 性能評価
   - 開発効率
   - エラー率分析

6. Case Studies
   - 分岐最適化
   - PHIノード処理
   - HostCall統合

7. Related Work
   - Meta-tracing JIT
   - Partial evaluation
   - Modular compilers

8. Conclusion
```

## 🏗️ 実装の詳細

### 現在の実装状況
- ✅ JitValue ABI（i64/f64/bool/handle）
- ✅ ハンドルレジストリ（u64 ↔ Arc<Box>）
- ✅ catch_unwindによる安全なフォールバック
- ✅ 基本的な分岐/PHI実装
- ⏳ HostCall最適化
- ⏳ 型特化パス

### コード例
```rust
// 箱境界での変換
impl JitAdapter {
    fn vm_to_jit(vm_val: VMValue) -> JitValue {
        match vm_val {
            VMValue::Integer(n) => JitValue::I64(n),
            VMValue::Box(b) => JitValue::Handle(
                registry.register(b)
            ),
            _ => panic!("Unsupported type")
        }
    }
}
```

## 🚀 進捗状況

- [x] アーキテクチャ設計
- [x] 基本実装（Phase 10.7）
- [ ] 性能最適化
- [ ] ベンチマーク実装
- [ ] 評価実験
- [ ] 論文執筆

## 📚 参考文献候補

- Würthinger, T., et al. (2017). Practical partial evaluation for high-performance dynamic language runtimes. PLDI
- Bolz, C. F., et al. (2009). Tracing the meta-level: PyPy's tracing JIT compiler. ICOOOLPS
- Lattner, C., & Adve, V. (2004). LLVM: A compilation framework for lifelong program analysis & transformation. CGO